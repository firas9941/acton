use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use base64::{Engine as _, engine::general_purpose};
use ring::{
    rand::SystemRandom,
    signature::{RSA_PKCS1_SHA256, RsaKeyPair},
};
use rustls_pki_types::{PrivatePkcs1KeyDer, pem::PemObject};
use serde::Serialize;
use zeroize::Zeroizing;

use crate::{config::GitHubAppConfig, error::GitHubAppAuthError};

#[derive(Serialize)]
struct JwtHeader {
    alg: &'static str,
    typ: &'static str,
}

const JWT_HEADER: JwtHeader = JwtHeader {
    alg: "RS256",
    typ: "JWT",
};

#[derive(Serialize)]
struct JwtClaims {
    iat: u64,
    exp: u64,
    iss: String,
}

pub struct GitHubAppJwt {
    header: JwtHeader,
    claims: JwtClaims,
}

impl GitHubAppJwt {
    pub fn create(config: &GitHubAppConfig) -> Result<Zeroizing<String>, GitHubAppAuthError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| GitHubAppAuthError::InvalidSystemTime)?
            .as_secs();

        Self::new(config.app_id, now).sign(config)
    }

    fn new(app_id: u64, now: u64) -> Self {
        Self {
            header: JWT_HEADER,
            claims: JwtClaims {
                iat: now.saturating_sub(60),
                exp: now.saturating_add(9 * 60),
                iss: app_id.to_string(),
            },
        }
    }

    fn sign(self, config: &GitHubAppConfig) -> Result<Zeroizing<String>, GitHubAppAuthError> {
        let header = general_purpose::URL_SAFE_NO_PAD
            .encode(serde_json::to_vec(&self.header).map_err(GitHubAppAuthError::EncodeJwt)?);
        let payload = general_purpose::URL_SAFE_NO_PAD
            .encode(serde_json::to_vec(&self.claims).map_err(GitHubAppAuthError::EncodeJwt)?);
        let signing_input = format!("{header}.{payload}");

        let private_key = Zeroizing::new(fs::read(&config.private_key_file).map_err(|source| {
            GitHubAppAuthError::ReadPrivateKey {
                path: config.private_key_file.clone(),
                source,
            }
        })?);
        let key_der = Zeroizing::new(
            PrivatePkcs1KeyDer::from_pem_slice(&private_key)
                .map_err(|_| GitHubAppAuthError::InvalidPrivateKey)?,
        );
        let key_pair = RsaKeyPair::from_der(key_der.secret_pkcs1_der())
            .map_err(|_| GitHubAppAuthError::InvalidPrivateKey)?;
        let mut signature = Zeroizing::new(vec![0_u8; key_pair.public().modulus_len()]);
        key_pair
            .sign(
                &RSA_PKCS1_SHA256,
                &SystemRandom::new(),
                signing_input.as_bytes(),
                &mut signature,
            )
            .map_err(|_| GitHubAppAuthError::SignJwt)?;
        let signature =
            Zeroizing::new(general_purpose::URL_SAFE_NO_PAD.encode(signature.as_slice()));

        Ok(Zeroizing::new(format!(
            "{signing_input}.{}",
            signature.as_str()
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ring::signature::{RSA_PKCS1_2048_8192_SHA256, UnparsedPublicKey};

    use super::*;

    const TEST_PRIVATE_KEY: &[u8] = include_bytes!("test-data/rsa-private-key.pkcs1.pem");

    #[test]
    fn serializes_rs256_jwt_header() {
        let header = serde_json::to_value(&JWT_HEADER).expect("JWT header should serialize");

        assert_eq!(header, serde_json::json!({ "alg": "RS256", "typ": "JWT" }));
    }

    #[test]
    fn creates_github_app_jwt_claims() {
        let jwt = GitHubAppJwt::new(42, 1_000);

        assert_eq!(jwt.claims.iat, 940);
        assert_eq!(jwt.claims.exp, 1_540);
        assert_eq!(jwt.claims.iss, "42");
    }

    #[test]
    fn creates_verifiable_rs256_jwt() {
        // This generated key is public test data and must never be used outside tests.
        let config = GitHubAppConfig {
            app_id: 42,
            installation_id: 1,
            private_key_file: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("verifier-github-app-credential/test-data/rsa-private-key.pkcs1.pem"),
        };

        let token = GitHubAppJwt::new(config.app_id, 1_000)
            .sign(&config)
            .expect("JWT should be signed");
        let segments = token.split('.').collect::<Vec<_>>();
        assert_eq!(segments.len(), 3);

        let header = general_purpose::URL_SAFE_NO_PAD
            .decode(segments[0])
            .expect("JWT header should be base64url encoded");
        let payload = general_purpose::URL_SAFE_NO_PAD
            .decode(segments[1])
            .expect("JWT payload should be base64url encoded");
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&header)
                .expect("JWT header should be JSON"),
            serde_json::json!({ "alg": "RS256", "typ": "JWT" })
        );
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&payload)
                .expect("JWT payload should be JSON"),
            serde_json::json!({ "iat": 940, "exp": 1_540, "iss": "42" })
        );

        let signature = general_purpose::URL_SAFE_NO_PAD
            .decode(segments[2])
            .expect("JWT signature should be base64url encoded");
        let signing_input = format!("{}.{}", segments[0], segments[1]);
        let private_key_der = PrivatePkcs1KeyDer::from_pem_slice(TEST_PRIVATE_KEY)
            .expect("test private key should be valid PKCS#1 PEM");
        let key_pair = RsaKeyPair::from_der(private_key_der.secret_pkcs1_der())
            .expect("test private key should be valid PKCS#1 DER");
        UnparsedPublicKey::new(&RSA_PKCS1_2048_8192_SHA256, key_pair.public().as_ref())
            .verify(signing_input.as_bytes(), &signature)
            .expect("JWT signature should verify");
    }
}
