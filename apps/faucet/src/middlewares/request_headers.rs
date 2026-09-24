use std::fmt;

use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue, StatusCode, header::USER_AGENT},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::debug;

const ALLOWED_USER_AGENT_PREFIX: &str = "acton/";
const ALLOWED_BROWSER_CLIENT_PREFIX: &str = "actonscan/";
pub const ACTON_CLIENT_HEADER: HeaderName = HeaderName::from_static("x-acton-client");
pub const DEVICE_UID_HEADER: HeaderName = HeaderName::from_static("x-device-uid");
const DEFAULT_DEVICE_UID: &str = "default";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientContext {
    pub device_uid: String,
    pub client_kind: AirdropClient,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AirdropClient {
    Acton,
    Actonscan,
}

impl AirdropClient {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Acton => "acton",
            Self::Actonscan => "actonscan",
        }
    }
}

impl fmt::Display for AirdropClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub async fn require_airdrop_headers(mut request: Request, next: Next) -> Response {
    let headers = request.headers();
    let user_agent = headers.get(USER_AGENT);
    let browser_client = headers.get(&ACTON_CLIENT_HEADER);
    let device_uid = headers.get(&DEVICE_UID_HEADER);
    let client_kind = if browser_client.is_some_and(is_allowed_browser_client) {
        Some(AirdropClient::Actonscan)
    } else if user_agent.is_some_and(is_allowed_user_agent) {
        Some(AirdropClient::Acton)
    } else {
        None
    };

    let is_device_uid_allowed = device_uid.is_some_and(is_allowed_device_uid_header);

    if let (Some(client_kind), true) = (client_kind, is_device_uid_allowed) {
        let device_uid = device_uid
            .and_then(|value| value.to_str().ok())
            .expect("validated device UID must be UTF-8");

        let device_uid = normalize_device_uid(device_uid);
        request.extensions_mut().insert(ClientContext {
            device_uid,
            client_kind,
        });
        return next.run(request).await;
    }

    if client_kind.is_none() {
        debug!(
            user_agent = header_value(user_agent),
            browser_client = header_value(browser_client),
            "Airdrop client headers failed validation"
        );
    }
    if !is_device_uid_allowed {
        debug!(
            header = %DEVICE_UID_HEADER,
            value = header_value(device_uid),
            "Airdrop request header failed validation"
        );
    }

    StatusCode::BAD_REQUEST.into_response()
}

fn is_allowed_browser_client(value: &HeaderValue) -> bool {
    value.to_str().is_ok_and(|client| {
        client
            .strip_prefix(ALLOWED_BROWSER_CLIENT_PREFIX)
            .is_some_and(|version| !version.trim().is_empty())
    })
}

fn is_allowed_user_agent(value: &HeaderValue) -> bool {
    value.to_str().is_ok_and(|user_agent| {
        let Some(version) = user_agent.strip_prefix(ALLOWED_USER_AGENT_PREFIX) else {
            return false;
        };

        semver::Version::parse(version).is_ok_and(|version| {
            version.build.is_empty() && matches!(version.pre.as_str(), "" | "trunk")
        })
    })
}

pub fn is_allowed_device_uid(value: &str) -> bool {
    value == DEFAULT_DEVICE_UID || matches!(value.len(), 32 | 36)
}

fn is_allowed_device_uid_header(value: &HeaderValue) -> bool {
    value.to_str().is_ok_and(is_allowed_device_uid)
}

fn normalize_device_uid(value: &str) -> String {
    value.replace('-', "").to_ascii_lowercase()
}

fn header_value(value: Option<&HeaderValue>) -> &str {
    match value {
        Some(value) => value.to_str().unwrap_or("<non-utf8>"),
        None => "<missing>",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        is_allowed_browser_client, is_allowed_device_uid, is_allowed_device_uid_header,
        is_allowed_user_agent, normalize_device_uid,
    };

    #[test]
    fn allows_actonscan_version_browser_client() {
        assert!(is_allowed_browser_client(
            &"actonscan/1.0.0".parse().unwrap()
        ));
        assert!(is_allowed_browser_client(
            &"actonscan/1.0.0-beta.1".parse().unwrap()
        ));
    }

    #[test]
    fn rejects_missing_or_unknown_browser_client_version() {
        assert!(!is_allowed_browser_client(&"actonscan/".parse().unwrap()));
        assert!(!is_allowed_browser_client(&"actonscan/ ".parse().unwrap()));
        assert!(!is_allowed_browser_client(
            &"explorer/1.0.0".parse().unwrap()
        ));
    }

    #[test]
    fn allows_acton_package_version_user_agent() {
        for user_agent in [
            "acton/0.1.0",
            "acton/0.3.1",
            "acton/0.3.2",
            "acton/0.4.0",
            "acton/0.4.1",
            "acton/0.4.2",
            "acton/0.5.0",
            "acton/1.0.0",
            "acton/1.1.0",
            "acton/1.1.0-trunk",
            "acton/1.2.0",
            "acton/10.20.30",
            "acton/10.20.30-trunk",
        ] {
            assert!(
                is_allowed_user_agent(&user_agent.parse().unwrap()),
                "{user_agent:?} should be allowed"
            );
        }
    }

    #[test]
    fn rejects_non_release_or_trunk_user_agents() {
        for user_agent in [
            "",
            "acton/",
            "acton/ ",
            "faucet/0.1.0",
            "Acton/1.0.0",
            "acton/stage2",
            "acton/0.5.0 (allchains-faucet-ton)",
            "acton/0.1.0 (debug)",
            "acton/1.2.3-beta.1+build.5",
            "acton/1.2.3-beta.1",
            "acton/1.2.3+build.5",
            "acton/1.1.0-trunk+build.5",
            "acton/1.1.0-trunk.1",
            "acton/1.1.0-trunk-trunk",
            "acton/1.1.0-TRUNK",
            "acton/v1.0.0",
            "acton/1",
            "acton/1.0",
            "acton/1.0.0.0",
            "acton/.1.0",
            "acton/1..0",
            "acton/1.0.",
            "acton/01.0.0",
            "acton/1.01.0",
            "acton/1.0.01",
            "acton/-1.0.0",
            "acton/1.0.x",
            "acton/1.0.0/extra",
            " acton/1.0.0",
            "acton/1.0.0 ",
            "acton/1.0.0-trunk ",
            "acton/1.0.0, acton/1.1.0",
        ] {
            assert!(
                !is_allowed_user_agent(&user_agent.parse().unwrap()),
                "{user_agent:?} should be rejected"
            );
        }
    }

    #[test]
    fn allows_device_uid_values_from_supported_platforms() {
        assert!(is_allowed_device_uid_header(&"default".parse().unwrap()));
        assert!(is_allowed_device_uid_header(
            &"00112233445566778899aabbccddeeff".parse().unwrap()
        ));
        assert!(is_allowed_device_uid_header(
            &"00112233-4455-6677-8899-aabbccddeeff".parse().unwrap()
        ));
        assert!(is_allowed_device_uid_header(
            &"00112233-4455-6677-8899-AABBCCDDEEFF".parse().unwrap()
        ));
        assert!(is_allowed_device_uid("default"));
    }

    #[test]
    fn normalizes_device_uid_to_compact_lowercase() {
        assert_eq!(
            normalize_device_uid("00112233445566778899AABBCCDDEEFF"),
            "00112233445566778899aabbccddeeff"
        );
        assert_eq!(
            normalize_device_uid("00112233-4455-6677-8899-AABBCCDDEEFF"),
            "00112233445566778899aabbccddeeff"
        );
    }

    #[test]
    fn rejects_invalid_device_uid() {
        assert!(!is_allowed_device_uid_header(&"".parse().unwrap()));
        assert!(!is_allowed_device_uid_header(&" ".parse().unwrap()));
        assert!(!is_allowed_device_uid_header(&"device-1".parse().unwrap()));
        assert!(!is_allowed_device_uid_header(&"another".parse().unwrap()));
        assert!(!is_allowed_device_uid_header(
            &"{00000000-0000-0000-0000-000000000000}".parse().unwrap()
        ));
    }
}
