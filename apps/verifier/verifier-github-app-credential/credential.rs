use std::{
    ffi::OsStr,
    io::{self, BufRead, BufReader, Read, Write},
};

use zeroize::Zeroizing;

use crate::error::GitHubAppAuthError;

/// Git limits each credential-protocol line, including its newline, to 65,535 bytes.
const MAX_CREDENTIAL_LINE_BYTES: u64 = 65_535;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialOperation {
    Get,
    Store,
    Erase,
}

impl TryFrom<&OsStr> for CredentialOperation {
    type Error = GitHubAppAuthError;

    fn try_from(value: &OsStr) -> Result<Self, Self::Error> {
        match value.to_str() {
            Some("get") => Ok(Self::Get),
            Some("store") => Ok(Self::Store),
            Some("erase") => Ok(Self::Erase),
            _ => Err(GitHubAppAuthError::UnsupportedOperation(
                value.to_string_lossy().into_owned(),
            )),
        }
    }
}

pub struct CredentialRequest<'a> {
    protocol: &'a str,
    host: &'a str,
}

impl<'a> CredentialRequest<'a> {
    pub fn parse(input: &'a str) -> Result<Self, GitHubAppAuthError> {
        let mut protocol = None;
        let mut host = None;

        for line in input.split('\n') {
            if line.is_empty() {
                break;
            }
            if line.contains('\r') || line.contains('\0') {
                return Err(GitHubAppAuthError::InvalidCredentialRequest(
                    "attributes must not contain CR or NUL",
                ));
            }

            let (name, value) =
                line.split_once('=')
                    .ok_or(GitHubAppAuthError::InvalidCredentialRequest(
                        "each attribute must contain an equals sign",
                    ))?;
            match name {
                "protocol" if protocol.is_some() => {
                    return Err(GitHubAppAuthError::InvalidCredentialRequest(
                        "duplicate protocol attribute",
                    ));
                }
                "protocol" => protocol = Some(value),
                "host" if host.is_some() => {
                    return Err(GitHubAppAuthError::InvalidCredentialRequest(
                        "duplicate host attribute",
                    ));
                }
                "host" => host = Some(value),
                _ => {}
            }
        }

        Ok(Self {
            protocol: protocol.ok_or(GitHubAppAuthError::InvalidCredentialRequest(
                "missing protocol attribute",
            ))?,
            host: host.ok_or(GitHubAppAuthError::InvalidCredentialRequest(
                "missing host attribute",
            ))?,
        })
    }

    pub fn ensure_github_https(&self) -> Result<(), GitHubAppAuthError> {
        if self.protocol != "https" || self.host != "github.com" {
            return Err(GitHubAppAuthError::UnsupportedCredentialTarget);
        }
        Ok(())
    }
}

pub fn read_credential_request() -> Result<Zeroizing<String>, GitHubAppAuthError> {
    let stdin = io::stdin();
    read_credential_request_from(stdin.lock())
}

fn read_credential_request_from(
    reader: impl Read,
) -> Result<Zeroizing<String>, GitHubAppAuthError> {
    let mut reader = BufReader::new(reader);
    let mut input = Zeroizing::new(String::new());

    loop {
        let line_start = input.len();
        let bytes_read = (&mut reader)
            .take(MAX_CREDENTIAL_LINE_BYTES + 1)
            .read_line(&mut input)
            .map_err(GitHubAppAuthError::ReadCredential)?;
        if bytes_read == 0 {
            break;
        }
        if bytes_read > MAX_CREDENTIAL_LINE_BYTES as usize {
            return Err(GitHubAppAuthError::CredentialLineTooLong);
        }
        if &input[line_start..] == "\n" {
            break;
        }
    }

    Ok(input)
}

pub fn write_credential(token: &str) -> Result<(), GitHubAppAuthError> {
    let stdout = io::stdout();
    write_credential_to(stdout.lock(), token)
}

fn write_credential_to(mut output: impl Write, token: &str) -> Result<(), GitHubAppAuthError> {
    writeln!(output, "username=x-access-token")
        .and_then(|()| writeln!(output, "password={token}"))
        .and_then(|()| writeln!(output))
        .map_err(GitHubAppAuthError::WriteCredential)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_credential_operations() {
        assert_eq!(
            CredentialOperation::try_from(OsStr::new("get")).expect("get operation should parse"),
            CredentialOperation::Get
        );
        assert_eq!(
            CredentialOperation::try_from(OsStr::new("store"))
                .expect("store operation should parse"),
            CredentialOperation::Store
        );
        assert_eq!(
            CredentialOperation::try_from(OsStr::new("erase"))
                .expect("erase operation should parse"),
            CredentialOperation::Erase
        );
        assert!(CredentialOperation::try_from(OsStr::new("unknown")).is_err());
    }

    #[test]
    fn accepts_github_https_credential_target() {
        let request = CredentialRequest::parse(
            "protocol=https\nhost=github.com\npath=owner/repository.git\n\n",
        )
        .expect("credential request should parse");

        assert!(request.ensure_github_https().is_ok());
    }

    #[test]
    fn rejects_unsupported_credential_targets() {
        for request in [
            "",
            "protocol=https\n\n",
            "host=github.com\n\n",
            "protocol=http\nhost=github.com\n\n",
            "protocol=ssh\nhost=github.com\n\n",
            "protocol=https\nhost=example.com\n\n",
            "protocol=https\nhost=api.github.com\n\n",
            "protocol=https\nhost=github.com.example.com\n\n",
        ] {
            assert!(
                CredentialRequest::parse(request)
                    .and_then(|request| request.ensure_github_https())
                    .is_err(),
                "credential target should be rejected: {request:?}"
            );
        }
    }

    #[test]
    fn rejects_malformed_credential_requests() {
        for request in [
            "protocol=https\nprotocol=https\nhost=github.com\n\n",
            "protocol=https\nhost=github.com\nhost=github.com\n\n",
            "protocol=https\nhost\n\n",
            "protocol=https\r\nhost=github.com\r\n\r\n",
            "protocol=https\nhost=github.com\0\n\n",
        ] {
            assert!(
                CredentialRequest::parse(request).is_err(),
                "credential request should be rejected: {request:?}"
            );
        }
    }

    #[test]
    fn stops_parsing_credential_request_at_the_first_blank_line() {
        let request = CredentialRequest::parse(
            "protocol=https\nhost=github.com\n\nprotocol=http\nhost=example.com\n",
        )
        .expect("credential request should parse");

        assert!(request.ensure_github_https().is_ok());
    }

    #[test]
    fn reads_credential_request() {
        let input = b"protocol=https\nhost=github.com\n\n";

        let request = read_credential_request_from(input.as_slice())
            .expect("credential request should be read");

        assert_eq!(&*request, "protocol=https\nhost=github.com\n\n");
    }

    #[test]
    fn accepts_credential_request_larger_than_one_line_limit() {
        let line = format!("padding={}\n", "a".repeat(40_000));
        let input = format!("{line}{line}\n");

        let request = read_credential_request_from(input.as_bytes())
            .expect("multi-line credential request should be read");

        assert!(request.len() > MAX_CREDENTIAL_LINE_BYTES as usize);
    }

    #[test]
    fn rejects_credential_line_over_git_limit() {
        let mut input = vec![b'a'; MAX_CREDENTIAL_LINE_BYTES as usize];
        input.push(b'\n');

        let error = read_credential_request_from(input.as_slice())
            .expect_err("oversized credential line should be rejected");

        assert!(matches!(error, GitHubAppAuthError::CredentialLineTooLong));
    }

    #[test]
    fn rejects_non_utf8_credential_request() {
        let input = [0xff];

        let error = read_credential_request_from(input.as_slice())
            .expect_err("non-UTF-8 credential request should be rejected");

        assert!(matches!(error, GitHubAppAuthError::ReadCredential(_)));
    }

    #[test]
    fn writes_git_credential_response() {
        let mut output = Vec::new();

        write_credential_to(&mut output, "github_pat_example")
            .expect("credential response should be written");

        assert_eq!(
            output,
            b"username=x-access-token\npassword=github_pat_example\n\n"
        );
    }

    #[test]
    fn reports_credential_write_errors() {
        struct FailingWriter;

        impl Write for FailingWriter {
            fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "test error"))
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let error = write_credential_to(FailingWriter, "github_pat_example")
            .expect_err("write error should be returned");

        assert!(matches!(
            error,
            GitHubAppAuthError::WriteCredential(source)
                if source.kind() == io::ErrorKind::BrokenPipe
        ));
    }
}
