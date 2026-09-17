use std::{env, ffi::OsString, path::PathBuf};

use crate::error::GitHubAppAuthError;

const APP_ID_ENV: &str = "SOURCE_REPOSITORY_GITHUB_APP_ID";
const INSTALLATION_ID_ENV: &str = "SOURCE_REPOSITORY_GITHUB_APP_INSTALLATION_ID";
const PRIVATE_KEY_FILE_ENV: &str = "SOURCE_REPOSITORY_GITHUB_APP_PRIVATE_KEY_FILE";

#[derive(Debug)]
pub struct GitHubAppConfig {
    pub app_id: u64,
    pub installation_id: u64,
    pub private_key_file: PathBuf,
}

impl GitHubAppConfig {
    pub fn from_environment() -> Result<Self, GitHubAppAuthError> {
        Self::from_values(
            env::var_os(APP_ID_ENV),
            env::var_os(INSTALLATION_ID_ENV),
            env::var_os(PRIVATE_KEY_FILE_ENV),
        )
    }

    fn from_values(
        app_id: Option<OsString>,
        installation_id: Option<OsString>,
        private_key_file: Option<OsString>,
    ) -> Result<Self, GitHubAppAuthError> {
        Ok(Self {
            app_id: Self::required_positive_integer(APP_ID_ENV, app_id)?,
            installation_id: Self::required_positive_integer(INSTALLATION_ID_ENV, installation_id)?,
            private_key_file: Self::required_path(PRIVATE_KEY_FILE_ENV, private_key_file)?,
        })
    }

    fn required_positive_integer(
        name: &'static str,
        value: Option<OsString>,
    ) -> Result<u64, GitHubAppAuthError> {
        let value = value.ok_or(GitHubAppAuthError::MissingEnvironment(name))?;
        let value = value
            .into_string()
            .map_err(|_| GitHubAppAuthError::InvalidInteger(name))?;
        let value = value
            .parse::<u64>()
            .map_err(|_| GitHubAppAuthError::InvalidInteger(name))?;

        if value == 0 {
            return Err(GitHubAppAuthError::InvalidInteger(name));
        }

        Ok(value)
    }

    fn required_path(
        name: &'static str,
        value: Option<OsString>,
    ) -> Result<PathBuf, GitHubAppAuthError> {
        let value = value.ok_or(GitHubAppAuthError::MissingEnvironment(name))?;
        if value.is_empty() {
            return Err(GitHubAppAuthError::MissingEnvironment(name));
        }

        Ok(PathBuf::from(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_config_values() {
        let config = GitHubAppConfig::from_values(
            Some("4979165".into()),
            Some("162512672".into()),
            Some("/run/secrets/github-app.pem".into()),
        )
        .expect("config values should parse");

        assert_eq!(config.app_id, 4_979_165);
        assert_eq!(config.installation_id, 162_512_672);
        assert_eq!(
            config.private_key_file,
            PathBuf::from("/run/secrets/github-app.pem")
        );
    }

    #[test]
    fn rejects_missing_config_values() {
        let missing_app_id =
            GitHubAppConfig::from_values(None, Some("1".into()), Some("private-key.pem".into()))
                .expect_err("missing App ID should be rejected");
        let missing_installation_id =
            GitHubAppConfig::from_values(Some("1".into()), None, Some("private-key.pem".into()))
                .expect_err("missing installation ID should be rejected");
        let missing_private_key =
            GitHubAppConfig::from_values(Some("1".into()), Some("1".into()), None)
                .expect_err("missing private key path should be rejected");

        assert!(matches!(
            missing_app_id,
            GitHubAppAuthError::MissingEnvironment(APP_ID_ENV)
        ));
        assert!(matches!(
            missing_installation_id,
            GitHubAppAuthError::MissingEnvironment(INSTALLATION_ID_ENV)
        ));
        assert!(matches!(
            missing_private_key,
            GitHubAppAuthError::MissingEnvironment(PRIVATE_KEY_FILE_ENV)
        ));
    }

    #[test]
    fn rejects_invalid_positive_integers() {
        for value in ["", "0", "-1", "not-a-number"] {
            let error = GitHubAppConfig::required_positive_integer(APP_ID_ENV, Some(value.into()))
                .expect_err("invalid integer should be rejected");

            assert!(matches!(
                error,
                GitHubAppAuthError::InvalidInteger(APP_ID_ENV)
            ));
        }
    }

    #[test]
    fn rejects_empty_private_key_path() {
        let error = GitHubAppConfig::required_path(PRIVATE_KEY_FILE_ENV, Some(OsString::new()))
            .expect_err("empty private key path should be rejected");

        assert!(matches!(
            error,
            GitHubAppAuthError::MissingEnvironment(PRIVATE_KEY_FILE_ENV)
        ));
    }
}
