use reqwest::header::{ACCEPT, HeaderValue, USER_AGENT};
use serde::Deserialize;
use zeroize::Zeroizing;

use crate::{config::GitHubAppConfig, error::GitHubAppAuthError};

const GITHUB_API_VERSION: &str = "2026-03-10";

#[derive(Deserialize)]
struct InstallationTokenResponse {
    token: String,
}

pub async fn request_installation_token(
    config: &GitHubAppConfig,
    jwt: &str,
) -> Result<Zeroizing<String>, GitHubAppAuthError> {
    let client = reqwest::Client::builder()
        .build()
        .map_err(GitHubAppAuthError::BuildClient)?;
    let url = format!(
        "https://api.github.com/app/installations/{}/access_tokens",
        config.installation_id
    );

    let request = client
        .post(url)
        .header(USER_AGENT, HeaderValue::from_static("ton-verifier"))
        .header(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        )
        .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
        .bearer_auth(jwt);

    let response = request
        .send()
        .await
        .map_err(GitHubAppAuthError::RequestToken)?;

    let response = response
        .error_for_status()
        .map_err(GitHubAppAuthError::RequestToken)?;

    let response = response
        .json::<InstallationTokenResponse>()
        .await
        .map_err(GitHubAppAuthError::RequestToken)?;

    let token = Zeroizing::new(response.token);
    validate_installation_token(&token)?;
    Ok(token)
}

fn validate_installation_token(token: &str) -> Result<(), GitHubAppAuthError> {
    if token.is_empty() || token.contains('\r') || token.contains('\n') || !token.is_ascii() {
        return Err(GitHubAppAuthError::InvalidInstallationToken);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_installation_tokens() {
        assert!(validate_installation_token("github_pat_example").is_ok());
        assert!(validate_installation_token("").is_err());
        assert!(validate_installation_token("token\nvalue").is_err());
        assert!(validate_installation_token("token\rvalue").is_err());
        assert!(validate_installation_token("токен").is_err());
    }
}
