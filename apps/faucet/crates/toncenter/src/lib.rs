#[cfg(test)]
mod tests;

use anyhow::{Context, anyhow};
use faucet_config::ToncenterConfig;
use reqwest::header;
use serde::de::DeserializeOwned;
use std::time::Duration;
use tokio::time::sleep;
use toncenter_api::v2::requests::{
    AddressBalanceRequest, JsonRpcCall, JsonRpcRequest, RunGetMethodRequest, SendBocRequest,
};
use toncenter_api::v2::responses::{ResultOk, RunGetMethodResult};
use toncenter_api::v2::stack::LegacyStackEntry;
use toncenter_api::v2::{Int64Input, Response, TonlibErrorResponse};
use tracing::warn;

pub struct ToncenterClient {
    client: reqwest::Client,
    base_url: String,
    max_retries: u32,
    retry_base_delay: Duration,
}

impl ToncenterClient {
    pub fn new(config: &ToncenterConfig) -> anyhow::Result<Self> {
        let mut default_headers = header::HeaderMap::new();

        if let Some(api_key) = config.api_key.as_deref() {
            let api_key = header::HeaderValue::from_str(api_key)
                .context("Invalid Toncenter API key header value")?;

            default_headers.insert(header::HeaderName::from_static("x-api-key"), api_key);
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .connect_timeout(Duration::from_secs(config.connect_timeout_seconds))
            .default_headers(default_headers)
            .user_agent(user_agent())
            .build()
            .context("Failed to build Toncenter HTTP client")?;

        Ok(Self {
            client,
            base_url: config.url.clone(),
            max_retries: config.max_retries,
            retry_base_delay: Duration::from_millis(config.retry_base_delay_ms),
        })
    }

    pub async fn get_wallet_seqno(&self, address: &str) -> anyhow::Result<u32> {
        let result = self.run_get_method(address, "seqno").await?;
        for entry in result.stack {
            if let LegacyStackEntry::Number((_, value)) = entry {
                return match value {
                    Int64Input::Number(value) => {
                        u32::try_from(value).context("Invalid wallet seqno")
                    }
                    Int64Input::String(value) => match value.strip_prefix("0x") {
                        Some(hex) => u32::from_str_radix(hex, 16),
                        None => value.parse(),
                    }
                    .context("Invalid wallet seqno"),
                };
            }
        }
        Ok(0)
    }

    pub async fn run_get_method(
        &self,
        address: &str,
        method: &str,
    ) -> anyhow::Result<RunGetMethodResult> {
        self.post_jsonrpc_with_retry(
            JsonRpcCall::RunGetMethod(RunGetMethodRequest {
                address: address.to_owned(),
                method: method.into(),
                stack: Vec::new(),
                seqno: None,
            }),
            "runGetMethod",
        )
        .await
    }

    pub async fn send_boc(&self, boc: &str) -> anyhow::Result<ResultOk> {
        self.post_jsonrpc_with_retry(
            JsonRpcCall::SendBoc(SendBocRequest {
                boc: boc.to_owned(),
            }),
            "sendBoc",
        )
        .await
    }

    pub async fn get_address_balance(&self, address: &str) -> anyhow::Result<u64> {
        let balance: String = self
            .post_jsonrpc_with_retry(
                JsonRpcCall::GetAddressBalance(AddressBalanceRequest {
                    address: address.to_owned(),
                    seqno: None,
                }),
                "getAddressBalance",
            )
            .await?;
        balance.parse().context("Invalid account balance")
    }

    fn jsonrpc_url(&self) -> String {
        format!("{}/api/v2/jsonRPC", self.base_url)
    }

    fn retry_delay(&self, attempt: u32) -> Duration {
        let multiplier = 1u64 << attempt.min(8);
        self.retry_base_delay.saturating_mul(multiplier as u32)
    }

    fn is_retryable_status(status: reqwest::StatusCode) -> bool {
        status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
    }

    fn is_retryable_error(error: &reqwest::Error) -> bool {
        error.is_timeout() || error.is_connect() || error.is_request()
    }

    fn is_retryable_rpc_error(error: &TonlibErrorResponse) -> bool {
        let message = error.error.to_ascii_lowercase();
        error.code == 429
            || error.code >= 500
            || message.contains("rate limit")
            || message.contains("too many requests")
            || message.contains("timeout")
            || message.contains("temporary")
    }

    async fn post_jsonrpc_with_retry<T: DeserializeOwned>(
        &self,
        call: JsonRpcCall,
        operation: &str,
    ) -> anyhow::Result<T> {
        let url = self.jsonrpc_url();
        let payload = JsonRpcRequest {
            call,
            jsonrpc: Some(serde_json::json!("2.0")),
            id: Some(serde_json::json!("1")),
        };

        for attempt in 0..=self.max_retries {
            let request = self.client.post(&url).json(&payload);

            let response = match request.send().await {
                Ok(response) => response,
                Err(err) => {
                    if attempt < self.max_retries && Self::is_retryable_error(&err) {
                        warn!(
                            operation,
                            attempt = attempt + 1,
                            max_attempts = self.max_retries + 1,
                            error = %err,
                            "Toncenter request failed, retrying"
                        );
                        sleep(self.retry_delay(attempt)).await;
                        continue;
                    }

                    return Err(err).context(format!("Failed to send {} request", operation));
                }
            };

            let status = response.status();
            let body = response
                .text()
                .await
                .context(format!("Failed to read {} response body", operation))?;

            if !status.is_success() {
                if attempt < self.max_retries && Self::is_retryable_status(status) {
                    warn!(
                        operation,
                        attempt = attempt + 1,
                        max_attempts = self.max_retries + 1,
                        status = %status,
                        "Toncenter returned retryable HTTP status"
                    );
                    sleep(self.retry_delay(attempt)).await;
                    continue;
                }

                return Err(anyhow!(
                    "TON Center API returned status: {} for {}. Error: {}",
                    status,
                    operation,
                    body
                ));
            }

            let response: Response<T> = serde_json::from_str(&body)
                .context(format!("Failed to parse {} response as JSON", operation))?;

            if let Response::Error(error) = &response {
                if attempt < self.max_retries && Self::is_retryable_rpc_error(error) {
                    warn!(
                        operation,
                        attempt = attempt + 1,
                        max_attempts = self.max_retries + 1,
                        error = %error,
                        "Toncenter returned retryable JSON-RPC error"
                    );
                    sleep(self.retry_delay(attempt)).await;
                    continue;
                }

                return Err(anyhow!(
                    "TON Center JSON-RPC error for {}: {}",
                    operation,
                    error
                ));
            }

            return response.into_result().map_err(Into::into);
        }

        Err(anyhow!(
            "Exceeded retry budget for Toncenter operation: {}",
            operation
        ))
    }
}

fn user_agent() -> String {
    let git_hash = option_env!("GIT_HASH").unwrap_or("unknown");
    format!("faucet/{} ({git_hash})", env!("CARGO_PKG_VERSION"))
}
