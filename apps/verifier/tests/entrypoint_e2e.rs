use std::{
    env,
    ffi::OsString,
    fs,
    path::PathBuf,
    process::{Command, Output},
};

use tempfile::TempDir;

type TestEnvEntry = (&'static str, OsString);

fn entrypoint_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docker/entrypoint.sh")
}

fn run_entrypoint(temp_dir: &TempDir, environment: &[TestEnvEntry]) -> Output {
    let mut command = Command::new("/bin/sh");
    command
        .arg(entrypoint_path())
        .arg("true")
        .env_clear()
        .env(
            "PATH",
            env::var_os("PATH").unwrap_or_else(|| OsString::from("/usr/bin:/bin")),
        )
        .env("VERIFIER_CONFIG", temp_dir.path().join("config.toml"))
        .env("SOURCE_REPOSITORY_PATH", "");

    for (name, value) in environment {
        command.env(name, value);
    }

    command.output().expect("entrypoint should run")
}

fn assert_success(name: &str, environment: &[TestEnvEntry]) {
    let temp_dir = tempfile::tempdir().expect("temporary directory should be created");
    let output = run_entrypoint(&temp_dir, environment);
    assert!(
        output.status.success(),
        "expected success for {name}; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_failure(name: &str, expected: &str, environment: &[TestEnvEntry]) {
    let temp_dir = tempfile::tempdir().expect("temporary directory should be created");
    let output = run_entrypoint(&temp_dir, environment);
    assert!(!output.status.success(), "expected failure for {name}");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected),
        "expected error for {name} to contain {expected:?}; stderr: {stderr}"
    );
}

fn variable(name: &'static str, value: impl Into<OsString>) -> (&'static str, OsString) {
    (name, value.into())
}

#[test]
fn accepts_supported_authentication_modes() {
    assert_success(
        "none",
        &[
            variable("SOURCE_REPOSITORY_AUTH_MODE", "none"),
            variable(
                "SOURCE_REPOSITORY_URL",
                "https://github.com/owner/repository.git",
            ),
        ],
    );
    assert_success(
        "url",
        &[
            variable("SOURCE_REPOSITORY_AUTH_MODE", "url"),
            variable(
                "SOURCE_REPOSITORY_URL",
                "https://x-access-token:secret@github.com/owner/repository.git",
            ),
        ],
    );

    let temp_dir = tempfile::tempdir().expect("temporary directory should be created");
    let ssh_key = temp_dir.path().join("ssh-key");
    fs::write(&ssh_key, []).expect("SSH key fixture should be created");
    let environment = vec![
        variable("SOURCE_REPOSITORY_AUTH_MODE", "ssh"),
        variable(
            "SOURCE_REPOSITORY_URL",
            "git@github.com:owner/repository.git",
        ),
        variable("SOURCE_REPOSITORY_SSH_KEY_FILE", ssh_key.into_os_string()),
    ];
    let output = run_entrypoint(&temp_dir, &environment);
    assert!(
        output.status.success(),
        "expected success for ssh; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn rejects_unknown_authentication_mode() {
    assert_failure(
        "unknown mode",
        "SOURCE_REPOSITORY_AUTH_MODE must be one of: none, url, ssh",
        &[variable("SOURCE_REPOSITORY_AUTH_MODE", "github_app")],
    );
}

#[test]
fn rejects_incomplete_or_mixed_authentication() {
    assert_failure(
        "SSH without a key",
        "SOURCE_REPOSITORY_AUTH_MODE=ssh requires SOURCE_REPOSITORY_SSH_KEY_FILE",
        &[variable("SOURCE_REPOSITORY_AUTH_MODE", "ssh")],
    );
    assert_failure(
        "URL without credentials",
        "SOURCE_REPOSITORY_AUTH_MODE=url requires credentials in an HTTPS SOURCE_REPOSITORY_URL",
        &[
            variable("SOURCE_REPOSITORY_AUTH_MODE", "url"),
            variable(
                "SOURCE_REPOSITORY_URL",
                "https://github.com/owner/repository.git",
            ),
        ],
    );
    assert_failure(
        "credentials in none mode",
        "credentials in SOURCE_REPOSITORY_URL require SOURCE_REPOSITORY_AUTH_MODE=url",
        &[
            variable("SOURCE_REPOSITORY_AUTH_MODE", "none"),
            variable(
                "SOURCE_REPOSITORY_URL",
                "https://x-access-token:secret@github.com/owner/repository.git",
            ),
        ],
    );
    assert_failure(
        "URL credentials with an SSH key",
        "SOURCE_REPOSITORY_AUTH_MODE=url cannot be combined with SOURCE_REPOSITORY_SSH_KEY_FILE",
        &[
            variable("SOURCE_REPOSITORY_AUTH_MODE", "url"),
            variable(
                "SOURCE_REPOSITORY_URL",
                "https://x-access-token:secret@github.com/owner/repository.git",
            ),
            variable("SOURCE_REPOSITORY_SSH_KEY_FILE", "/tmp/unused-key"),
        ],
    );
}
