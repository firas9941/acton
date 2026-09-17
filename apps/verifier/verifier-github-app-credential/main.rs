mod config;
mod credential;
mod error;
mod github;
mod jwt;

use std::env;

use config::GitHubAppConfig;
use credential::{
    CredentialOperation, CredentialRequest, read_credential_request, write_credential,
};
use error::GitHubAppAuthError;
use github::request_installation_token;
use jwt::GitHubAppJwt;

#[tokio::main]
async fn main() -> Result<(), GitHubAppAuthError> {
    let operation = env::args_os()
        .nth(1)
        .ok_or(GitHubAppAuthError::MissingOperation)?;
    let operation = CredentialOperation::try_from(operation.as_os_str())?;
    run(operation).await
}

async fn run(operation: CredentialOperation) -> Result<(), GitHubAppAuthError> {
    let credential_request = read_credential_request()?;

    match operation {
        CredentialOperation::Store | CredentialOperation::Erase => Ok(()),
        CredentialOperation::Get => {
            let credential_request = CredentialRequest::parse(&credential_request)?;
            credential_request.ensure_github_https()?;
            let config = GitHubAppConfig::from_environment()?;
            let jwt = GitHubAppJwt::create(&config)?;
            let token = request_installation_token(&config, &jwt).await?;
            write_credential(&token)
        }
    }
}
