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

fn run_entrypoint_with_command(
    temp_dir: &TempDir,
    environment: &[TestEnvEntry],
    child_command: &[&str],
) -> Output {
    let mut command = Command::new("/bin/sh");
    command
        .arg(entrypoint_path())
        .args(child_command)
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

fn run_entrypoint(temp_dir: &TempDir, environment: &[TestEnvEntry]) -> Output {
    run_entrypoint_with_command(temp_dir, environment, &["true"])
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

    let temp_dir = tempfile::tempdir().expect("temporary directory should be created");
    let private_key = temp_dir.path().join("github-app.pem");
    fs::write(&private_key, []).expect("GitHub App key fixture should be created");
    let environment = vec![
        variable("SOURCE_REPOSITORY_AUTH_MODE", "github_app"),
        variable(
            "SOURCE_REPOSITORY_URL",
            "https://github.com/owner/repository.git",
        ),
        variable("SOURCE_REPOSITORY_GITHUB_APP_ID", "4979165"),
        variable("SOURCE_REPOSITORY_GITHUB_APP_INSTALLATION_ID", "162512672"),
        variable(
            "SOURCE_REPOSITORY_GITHUB_APP_PRIVATE_KEY_FILE",
            private_key.into_os_string(),
        ),
    ];
    let output = run_entrypoint_with_command(
        &temp_dir,
        &environment,
        &[
            "/bin/sh",
            "-c",
            "printf '%s\\n' \"$GIT_CONFIG_COUNT\" \"$GIT_CONFIG_KEY_0=$GIT_CONFIG_VALUE_0\" \"$GIT_CONFIG_KEY_1=$GIT_CONFIG_VALUE_1\" \"$GIT_CONFIG_KEY_2=$GIT_CONFIG_VALUE_2\" \"$GIT_TERMINAL_PROMPT\"",
        ],
    );
    assert!(
        output.status.success(),
        "expected success for github_app; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        concat!(
            "3\n",
            "credential.helper=\n",
            "credential.helper=!/usr/local/bin/verifier-github-app-credential\n",
            "credential.useHttpPath=true\n",
            "0\n",
        )
    );
}

#[test]
fn rejects_unknown_authentication_mode() {
    assert_failure(
        "unknown mode",
        "SOURCE_REPOSITORY_AUTH_MODE must be one of: none, url, ssh, github_app",
        &[variable("SOURCE_REPOSITORY_AUTH_MODE", "unknown")],
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
    assert_failure(
        "GitHub App without an App ID",
        "SOURCE_REPOSITORY_GITHUB_APP_ID must be a positive integer",
        &[
            variable("SOURCE_REPOSITORY_AUTH_MODE", "github_app"),
            variable(
                "SOURCE_REPOSITORY_URL",
                "https://github.com/owner/repository.git",
            ),
        ],
    );
    assert_failure(
        "GitHub App with URL credentials",
        "SOURCE_REPOSITORY_AUTH_MODE=github_app requires a URL without credentials",
        &[
            variable("SOURCE_REPOSITORY_AUTH_MODE", "github_app"),
            variable(
                "SOURCE_REPOSITORY_URL",
                "https://x-access-token:secret@github.com/owner/repository.git",
            ),
        ],
    );
}

#[test]
fn validates_ssh_configuration_and_exports_git_environment() {
    assert_failure(
        "unreadable SSH key",
        "SOURCE_REPOSITORY_SSH_KEY_FILE is not readable",
        &[
            variable("SOURCE_REPOSITORY_AUTH_MODE", "ssh"),
            variable("SOURCE_REPOSITORY_SSH_KEY_FILE", "/missing/ssh-key"),
        ],
    );
    assert_failure(
        "invalid strict host key checking",
        "invalid SOURCE_REPOSITORY_SSH_STRICT_HOST_KEY_CHECKING value",
        &[
            variable("SOURCE_REPOSITORY_AUTH_MODE", "ssh"),
            variable("SOURCE_REPOSITORY_SSH_KEY_FILE", "/dev/null"),
            variable("SOURCE_REPOSITORY_SSH_STRICT_HOST_KEY_CHECKING", "invalid"),
        ],
    );

    let temp_dir = tempfile::tempdir().expect("temporary directory should be created");
    let ssh_key = temp_dir.path().join("ssh-key");
    fs::write(&ssh_key, []).expect("SSH key fixture should be created");
    let environment = vec![
        variable("SOURCE_REPOSITORY_AUTH_MODE", "ssh"),
        variable("SOURCE_REPOSITORY_SSH_KEY_FILE", ssh_key.as_os_str()),
        variable("SOURCE_REPOSITORY_SSH_STRICT_HOST_KEY_CHECKING", "yes"),
    ];
    let output = run_entrypoint_with_command(
        &temp_dir,
        &environment,
        &["/bin/sh", "-c", "printf '%s' \"$GIT_SSH_COMMAND\""],
    );
    assert!(
        output.status.success(),
        "expected SSH environment export; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!(
            "ssh -i {} -o IdentitiesOnly=yes -o StrictHostKeyChecking=yes",
            ssh_key.display()
        )
    );
}

#[test]
fn rejects_a_nonempty_non_git_checkout() {
    let temp_dir = tempfile::tempdir().expect("temporary directory should be created");
    let checkout = temp_dir.path().join("checkout");
    fs::create_dir(&checkout).expect("checkout directory should be created");
    fs::write(checkout.join("unexpected-file"), []).expect("fixture should be written");

    let environment = vec![
        variable("SOURCE_REPOSITORY_AUTH_MODE", "none"),
        variable(
            "SOURCE_REPOSITORY_URL",
            temp_dir.path().join("remote").as_os_str(),
        ),
        variable("SOURCE_REPOSITORY_PATH", checkout.as_os_str()),
    ];
    let output = run_entrypoint(&temp_dir, &environment);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("source repository path exists and is not an empty git repository")
    );
}
