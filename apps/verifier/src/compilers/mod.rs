use std::{collections::BTreeMap, path::PathBuf, process::ExitStatus};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    process::Command,
    time::{self, Duration},
};

use crate::{config::Config, source_storage::SourceMapData};

const MAX_WORKER_STDOUT_BYTES: usize = 16 * 1024 * 1024;
const MAX_WORKER_STDERR_BYTES: usize = 64 * 1024;

#[async_trait]
pub trait CompilerService: Send + Sync + 'static {
    async fn compile(&self, request: CompileRequest) -> Result<CompileOutput, CompilerError>;
}

pub struct NodeCompilerService {
    node_bin: String,
    worker_path: PathBuf,
    timeout: Duration,
}

impl NodeCompilerService {
    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        Self {
            node_bin: config.compiler_node_bin().to_owned(),
            worker_path: config.compiler_worker_path().to_path_buf(),
            timeout: config.compiler_timeout(),
        }
    }
}

#[async_trait]
impl CompilerService for NodeCompilerService {
    async fn compile(&self, request: CompileRequest) -> Result<CompileOutput, CompilerError> {
        let input = serde_json::to_vec(&request).map_err(CompilerError::SerializeInput)?;
        let worker_path = dunce::canonicalize(&self.worker_path).map_err(|source| {
            CompilerError::ResolveWorkerPath {
                path: self.worker_path.clone(),
                source,
            }
        })?;
        let worker_directory = worker_path
            .parent()
            .ok_or_else(|| CompilerError::MissingWorkerDirectory(worker_path.clone()))?;
        let mut child = isolated_command(&self.node_bin)
            .arg("--permission")
            .arg("--disallow-code-generation-from-strings")
            .arg("--disable-proto=throw")
            .arg("--no-experimental-sqlite")
            .arg(format!("--allow-fs-read={}", worker_directory.display()))
            .arg(&worker_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(CompilerError::Spawn)?;

        let mut stdin = child.stdin.take().ok_or(CompilerError::MissingStdin)?;
        let stdout = child.stdout.take().ok_or(CompilerError::MissingOutput)?;
        let stderr = child.stderr.take().ok_or(CompilerError::MissingOutput)?;

        // Drain both pipes while sending sources: a worker can fill its output pipe
        // before reading stdin. The deadline covers this exchange as well as compilation.
        let execution = time::timeout(self.timeout, async {
            tokio::try_join!(
                async {
                    stdin
                        .write_all(&input)
                        .await
                        .map_err(CompilerError::WriteStdin)?;
                    drop(stdin);
                    Ok(())
                },
                read_worker_output(stdout, MAX_WORKER_STDOUT_BYTES, "stdout"),
                read_worker_output(stderr, MAX_WORKER_STDERR_BYTES, "stderr"),
                async { child.wait().await.map_err(CompilerError::Wait) },
            )
        })
        .await
        .map_err(|_| CompilerError::Timeout {
            timeout_ms: self.timeout.as_millis(),
        })
        .and_then(std::convert::identity);

        if execution.is_err() {
            // Reap the child before releasing this request's resources.
            let _ = child.kill().await;
        }
        let ((), stdout, stderr, status) = execution?;

        if !status.success() {
            return Err(CompilerError::WorkerFailed {
                status,
                stderr: String::from_utf8_lossy(&stderr).into_owned(),
            });
        }

        let output = serde_json::from_slice::<WorkerOutput>(&stdout)
            .map_err(CompilerError::DeserializeOutput)?;

        match output {
            WorkerOutput::Ok {
                code_hash,
                used_source_paths,
                generated_sources,
                source_map,
            } => Ok(CompileOutput {
                code_hash,
                used_source_paths,
                generated_sources,
                source_map,
            }),
            WorkerOutput::CompileError { error } => Err(CompilerError::CompileFailed(error)),
        }
    }
}

// Compiler diagnostics and generated metadata are untrusted in size, even for
// a small source upload. Stop collecting as soon as either pipe exceeds its budget.
async fn read_worker_output(
    reader: impl AsyncRead + Unpin,
    limit: usize,
    stream: &'static str,
) -> Result<Vec<u8>, CompilerError> {
    let mut bytes = Vec::new();
    reader
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .await
        .map_err(CompilerError::Wait)?;
    if bytes.len() > limit {
        return Err(CompilerError::OutputTooLarge { stream, limit });
    }
    Ok(bytes)
}

fn isolated_command(program: &str) -> Command {
    let program = PathBuf::from(program);
    let executable = std::env::var_os("PATH")
        .filter(|_| program.components().count() == 1)
        .and_then(|path| {
            std::env::split_paths(&path)
                .map(|directory| directory.join(&program))
                .find(|candidate| candidate.is_file())
        })
        .unwrap_or(program);
    let mut command = Command::new(executable);
    command.env_clear();
    command
}

#[derive(Debug, Serialize)]
pub struct CompileRequest {
    pub language: String,
    pub compiler_version: String,
    pub entrypoint: String,
    pub import_mappings: BTreeMap<String, String>,
    pub compile_params: Value,
    pub sources: Vec<CompileSource>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CompileSource {
    pub path: String,
    pub content: String,
    pub is_entrypoint: bool,
    pub include_in_command: Option<bool>,
    pub is_stdlib: Option<bool>,
    pub has_include_directives: Option<bool>,
}

pub struct CompileOutput {
    pub code_hash: String,
    pub used_source_paths: Option<Vec<String>>,
    pub generated_sources: Vec<CompileGeneratedSource>,
    pub source_map: Option<SourceMapData>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CompileGeneratedSource {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum WorkerOutput {
    Ok {
        code_hash: String,
        #[serde(default)]
        used_source_paths: Option<Vec<String>>,
        #[serde(default)]
        generated_sources: Vec<CompileGeneratedSource>,
        source_map: Option<SourceMapData>,
    },
    CompileError {
        error: String,
    },
}

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("failed to serialize compiler input: {0}")]
    SerializeInput(serde_json::Error),
    #[error("compiler concurrency limiter is closed")]
    ConcurrencyLimiterClosed,
    #[error("failed to resolve compiler worker path {path}: {source}")]
    ResolveWorkerPath {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("compiler worker path has no parent directory: {0}")]
    MissingWorkerDirectory(PathBuf),
    #[error("failed to spawn compiler worker: {0}")]
    Spawn(std::io::Error),
    #[error("compiler worker stdin was not available")]
    MissingStdin,
    #[error("compiler worker output pipe was not available")]
    MissingOutput,
    #[error("compiler worker {stream} exceeded {limit} bytes")]
    OutputTooLarge { stream: &'static str, limit: usize },
    #[error("failed to write compiler worker stdin: {0}")]
    WriteStdin(std::io::Error),
    #[error("compiler worker timed out after {timeout_ms} ms")]
    Timeout { timeout_ms: u128 },
    #[error("failed to wait for compiler worker: {0}")]
    Wait(std::io::Error),
    #[error("compiler worker failed with status {status}: {stderr}")]
    WorkerFailed { status: ExitStatus, stderr: String },
    #[error("failed to parse compiler worker output: {0}")]
    DeserializeOutput(serde_json::Error),
    #[error("invalid compiler worker output: {0}")]
    InvalidOutput(String),
    #[error("compile error: {0}")]
    CompileFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_compile_request() -> CompileRequest {
        CompileRequest {
            language: "tolk".to_owned(),
            compiler_version: "1.4.2".to_owned(),
            entrypoint: "main.tolk".to_owned(),
            import_mappings: BTreeMap::new(),
            compile_params: Value::Null,
            sources: vec![CompileSource {
                path: "main.tolk".to_owned(),
                content: "x".repeat(1024 * 1024),
                is_entrypoint: true,
                include_in_command: None,
                is_stdlib: None,
                has_include_directives: None,
            }],
        }
    }

    // Exercise the real process boundary: mocks cannot reproduce a full stdin
    // pipe or a worker that writes output before it reads the request.
    async fn run_worker_script(
        script: &str,
        timeout: Duration,
    ) -> Result<CompileOutput, CompilerError> {
        let directory = tempfile::tempdir().expect("worker directory");
        let worker_path = directory.path().join("worker.mjs");
        std::fs::write(&worker_path, script).expect("worker script");
        let service = NodeCompilerService {
            node_bin: "node".to_owned(),
            worker_path,
            timeout,
        };
        time::timeout(
            Duration::from_secs(5),
            service.compile(test_compile_request()),
        )
        .await
        .expect("compiler must enforce its own deadline")
    }

    #[tokio::test]
    async fn worker_deadline_covers_blocked_stdin() {
        let result =
            run_worker_script("setInterval(() => {}, 1000)", Duration::from_millis(100)).await;
        assert!(matches!(result, Err(CompilerError::Timeout { .. })));
    }

    #[tokio::test]
    async fn worker_output_limits_abort_both_streams() {
        for stream in ["stdout", "stderr"] {
            let script = format!(
                "process.{stream}.write('x'.repeat(17 * 1024 * 1024)); setInterval(() => {{}}, 1000)"
            );
            let result = run_worker_script(&script, Duration::from_secs(3)).await;
            assert!(
                matches!(result, Err(CompilerError::OutputTooLarge { stream: actual, .. }) if actual == stream)
            );
        }
    }

    #[tokio::test]
    async fn worker_drains_output_while_sending_sources() {
        let result = run_worker_script(
            r"
            process.stdout.write(' '.repeat(1024 * 1024), () => {
                let input = '';
                process.stdin.on('data', chunk => input += chunk);
                process.stdin.on('end', () => {
                    const request = JSON.parse(input);
                    process.stdout.write(JSON.stringify({
                        status: 'ok',
                        code_hash: String(request.sources[0].content.length),
                        used_source_paths: ['main.tolk']
                    }));
                });
            });
            ",
            Duration::from_secs(3),
        )
        .await
        .expect("full duplex exchange must complete");
        assert_eq!(result.code_hash, "1048576");
        assert_eq!(result.used_source_paths, Some(vec!["main.tolk".to_owned()]));
    }

    #[tokio::test]
    async fn node_compiler_service_enforces_runtime_restrictions() {
        let worker_directory = tempfile::tempdir().expect("worker directory should be created");
        let denied_directory = tempfile::tempdir().expect("denied directory should be created");
        let worker_path = worker_directory.path().join("worker.mjs");
        let denied_file = denied_directory.path().join("secret.txt");
        let output_file = worker_directory.path().join("output.txt");
        std::fs::write(&denied_file, "secret").expect("denied fixture should be written");
        std::fs::write(
            &worker_path,
            r#"
                import fs from "node:fs";
                import { execFileSync } from "node:child_process";
                import { WASI } from "node:wasi";
                import { Worker } from "node:worker_threads";
    
                let input = "";
                for await (const chunk of process.stdin) input += chunk;
                const request = JSON.parse(input);
                const { deniedPath, outputPath } = request.compile_params;
                const restrictions = {};
    
                try {
                  fs.readFileSync(deniedPath, "utf8");
                  restrictions.fsRead = "allowed";
                } catch (error) {
                  restrictions.fsRead = error.code;
                }
    
                try {
                  fs.writeFileSync(outputPath, "output");
                  restrictions.fsWrite = "allowed";
                } catch (error) {
                  restrictions.fsWrite = error.code;
                }
    
                try {
                  eval("1");
                  restrictions.eval = "allowed";
                } catch (error) {
                  restrictions.eval = error.name;
                }
    
                try {
                  execFileSync(process.execPath, ["--version"]);
                  restrictions.childProcess = "allowed";
                } catch (error) {
                  restrictions.childProcess = error.code;
                }
    
                try {
                  const worker = new Worker(new URL("data:text/javascript,", import.meta.url));
                  await worker.terminate();
                  restrictions.workerThreads = "allowed";
                } catch (error) {
                  restrictions.workerThreads = error.code;
                }
    
                try {
                  new WASI({ version: "preview1" });
                  restrictions.wasi = "allowed";
                } catch (error) {
                  restrictions.wasi = error.code;
                }
    
                try {
                  await import("node:sqlite");
                  restrictions.sqlite = "allowed";
                } catch (error) {
                  restrictions.sqlite = error.code;
                }
    
                process.stdout.write(JSON.stringify({
                  status: "ok",
                  code_hash: JSON.stringify(restrictions),
                }));
            "#,
        )
        .expect("worker script should be written");

        let service = NodeCompilerService {
            node_bin: "node".to_owned(),
            worker_path,
            timeout: Duration::from_secs(3),
        };
        let mut request = test_compile_request();
        request.compile_params = serde_json::json!({
            "deniedPath": denied_file,
            "outputPath": output_file,
        });

        let output = service
            .compile(request)
            .await
            .expect("restricted worker should complete");
        let restrictions = serde_json::from_str::<Value>(&output.code_hash)
            .expect("worker restrictions should be valid JSON");

        assert_eq!(restrictions["fsRead"], "ERR_ACCESS_DENIED");
        assert_eq!(restrictions["fsWrite"], "ERR_ACCESS_DENIED");
        assert_eq!(restrictions["eval"], "EvalError");
        assert_eq!(restrictions["childProcess"], "ERR_ACCESS_DENIED");
        assert_eq!(restrictions["workerThreads"], "ERR_ACCESS_DENIED");
        assert_eq!(restrictions["wasi"], "ERR_ACCESS_DENIED");
        assert_eq!(restrictions["sqlite"], "ERR_UNKNOWN_BUILTIN_MODULE");
        assert!(!output_file.exists());
    }

    #[tokio::test]
    async fn node_compiler_service_clears_sensitive_environment() {
        const CHILD_MARKER: &str = "VERIFIER_ENV_ISOLATION_TEST_CHILD";
        const TEST_NAME: &str =
            "compilers::tests::node_compiler_service_clears_sensitive_environment";

        if std::env::var_os(CHILD_MARKER).is_none() {
            let output = Command::new(std::env::current_exe().expect("current test executable"))
                .arg("--exact")
                .arg(TEST_NAME)
                .arg("--nocapture")
                .env(CHILD_MARKER, "1")
                .env("VERIFIER_API_KEY", "not-a-real-secret")
                .env(
                    "NODE_OPTIONS",
                    "--require=/acton-verifier-test-missing-node-options-module.cjs",
                )
                .env("NODE_PATH", "/acton-verifier-test-missing-node-path")
                .output()
                .await
                .expect("nested test process should run");

            assert!(
                output.status.success(),
                "nested environment-isolation test failed\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
            return;
        }

        let output = run_worker_script(
            r#"
                for await (const _chunk of process.stdin) {
                  // Drain the request before replying so the parent can finish writing stdin.
                }
                const sensitiveNames = [
                  "VERIFIER_API_KEY",
                  "NODE_OPTIONS",
                  "NODE_PATH",
                  "VERIFIER_ENV_ISOLATION_TEST_CHILD",
                ];
                process.stdout.write(JSON.stringify({
                  status: "ok",
                  code_hash: JSON.stringify({
                    environmentKeys: Object.keys(process.env).sort(),
                    leakedNames: sensitiveNames.filter(name => process.env[name] !== undefined),
                  }),
                }));
            "#,
            Duration::from_secs(3),
        )
        .await
        .expect("worker should start without inheriting parent environment");
        let environment = serde_json::from_str::<Value>(&output.code_hash)
            .expect("worker environment report should be valid JSON");
        let mut environment_keys = environment["environmentKeys"]
            .as_array()
            .expect("worker environment keys should be an array")
            .clone();
        #[cfg(target_os = "macos")]
        environment_keys.retain(|name| name.as_str() != Some("__CF_USER_TEXT_ENCODING"));

        assert_eq!(environment_keys, Vec::<Value>::new());
        assert_eq!(environment["leakedNames"], serde_json::json!([]));
    }

    #[tokio::test]
    async fn isolated_command_resolves_executable_and_clears_environment() {
        let mut command = isolated_command("node");
        let executable = PathBuf::from(command.as_std().get_program());
        assert!(
            executable.is_file(),
            "resolved Node executable does not exist: {}",
            executable.display()
        );

        let output = command
            .arg("--eval")
            .arg("process.stdout.write(String(process.env.PATH === undefined))")
            .output()
            .await
            .expect("isolated Node command should run");

        assert!(
            output.status.success(),
            "isolated Node command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"true");
    }

    #[tokio::test]
    async fn node_permissions_allow_selected_reads_and_deny_other_filesystem_access() {
        let allowed_directory = tempfile::tempdir().expect("temporary directory should be created");
        let denied_directory = tempfile::tempdir().expect("temporary directory should be created");
        let allowed_file = allowed_directory.path().join("allowed.txt");
        let denied_file = denied_directory.path().join("denied.txt");
        let output_file = allowed_directory.path().join("output.txt");
        std::fs::write(&allowed_file, "allowed").expect("allowed fixture should be written");
        std::fs::write(&denied_file, "denied").expect("denied fixture should be written");

        let output = isolated_command("node")
            .arg("--permission")
            .arg(format!(
                "--allow-fs-read={}",
                allowed_directory.path().display()
            ))
            .arg("--eval")
            .arg(
                r#"
                    const fs = require("node:fs");
                    const [allowedPath, deniedPath, outputPath] = process.argv.slice(1);
                    const allowed = fs.readFileSync(allowedPath, "utf8");
                    let denied;
                    try {
                      fs.readFileSync(deniedPath, "utf8");
                    } catch (error) {
                      denied = error.code;
                    }
                    let write;
                    try {
                      fs.writeFileSync(outputPath, "output");
                    } catch (error) {
                      write = error.code;
                    }
                    process.stdout.write(`${allowed}:${denied}:${write}`);
                "#,
            )
            .arg(&allowed_file)
            .arg(&denied_file)
            .arg(&output_file)
            .output()
            .await
            .expect("isolated Node command should run");

        assert!(
            output.status.success(),
            "isolated Node command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            output.stdout,
            b"allowed:ERR_ACCESS_DENIED:ERR_ACCESS_DENIED"
        );
        assert!(!output_file.exists());
    }

    #[tokio::test]
    async fn node_disallows_code_generation_from_strings() {
        let output = isolated_command("node")
            .arg("--disallow-code-generation-from-strings")
            .arg("--eval")
            .arg(
                r#"
                    try {
                      eval("1");
                    } catch (error) {
                      process.stdout.write(error.name);
                    }
                "#,
            )
            .output()
            .await
            .expect("isolated Node command should run");

        assert!(
            output.status.success(),
            "isolated Node command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"EvalError");
    }

    #[tokio::test]
    async fn node_disables_proto_and_experimental_sqlite() {
        let output = isolated_command("node")
            .arg("--disable-proto=throw")
            .arg("--no-experimental-sqlite")
            .arg("--eval")
            .arg(
                r#"
                    let protoError;
                    try {
                      ({}).__proto__;
                    } catch (error) {
                      protoError = error.code;
                    }
                    let sqliteError;
                    try {
                      require("node:sqlite");
                    } catch (error) {
                      sqliteError = error.code;
                    }
                    process.stdout.write(`${protoError}:${sqliteError}`);
                "#,
            )
            .output()
            .await
            .expect("isolated Node command should run");

        assert!(
            output.status.success(),
            "isolated Node command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            output.stdout,
            b"ERR_PROTO_ACCESS:ERR_UNKNOWN_BUILTIN_MODULE"
        );
    }
}
