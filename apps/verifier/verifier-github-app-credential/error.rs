use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitHubAppAuthError {
    #[error("missing required environment variable: {0}")]
    MissingEnvironment(&'static str),
    #[error("environment variable {0} must be a positive integer")]
    InvalidInteger(&'static str),
    #[error("missing Git credential helper operation")]
    MissingOperation,
    #[error("unsupported Git credential helper operation: {0}")]
    UnsupportedOperation(String),
    #[error("failed to read Git credential request: {0}")]
    ReadCredential(io::Error),
    #[error("Git credential request contains a line longer than 65,535 bytes")]
    CredentialLineTooLong,
    #[error("invalid Git credential request: {0}")]
    InvalidCredentialRequest(&'static str),
    #[error("GitHub App credentials are restricted to HTTPS github.com remotes")]
    UnsupportedCredentialTarget,
    #[error("failed to read GitHub App private key at {path}: {source}")]
    ReadPrivateKey { path: PathBuf, source: io::Error },
    #[error("GitHub App private key must be a PKCS#1 PEM-encoded RSA key")]
    InvalidPrivateKey,
    #[error("failed to encode GitHub App JWT: {0}")]
    EncodeJwt(serde_json::Error),
    #[error("failed to sign GitHub App JWT")]
    SignJwt,
    #[error("system clock is before the Unix epoch")]
    InvalidSystemTime,
    #[error("failed to build GitHub API client: {0}")]
    BuildClient(reqwest::Error),
    #[error("GitHub installation token request failed: {0}")]
    RequestToken(reqwest::Error),
    #[error("GitHub returned an empty or malformed installation token")]
    InvalidInstallationToken,
    #[error("failed to write Git credential response: {0}")]
    WriteCredential(io::Error),
}
