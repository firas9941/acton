use anyhow::Context;
use faucet_antifraud::{Antifraud, CheckError};
use faucet_valkey::AntifraudModule;
use toncenter::AccountStateEnum;
use tracing::warn;

use crate::app::AppState;

#[derive(Debug)]
pub(crate) struct AccountRejection {
    module: AntifraudModule,
    error: CheckError,
    pub(crate) message: &'static str,
}

struct AccountSnapshot {
    balance: u64,
    state: AccountStateEnum,
}

impl AppState {
    pub(crate) async fn check_account(
        &self,
        address: &str,
    ) -> anyhow::Result<Option<AccountRejection>> {
        if !self.antifraud.enabled() {
            return Ok(None);
        }

        let account = self.client.get_address_information(address).await?;
        let account = AccountSnapshot {
            balance: account.balance.parse().context("Invalid account balance")?,
            state: account.state,
        };

        for check in [check_uninit_wallet_balance, check_wallet_balance] {
            if let Some(rejection) = check(&self.antifraud, &account) {
                warn!(
                    module = rejection.module.name(),
                    address,
                    error = ?rejection.error,
                    "Antifraud module triggered"
                );
                self.record_antifraud_trigger(rejection.module, address)
                    .await;
                return Ok(Some(rejection));
            }
        }
        Ok(None)
    }
}

fn check_wallet_balance(
    antifraud: &Antifraud,
    account: &AccountSnapshot,
) -> Option<AccountRejection> {
    if !antifraud.wallet_balance_enabled() {
        return None;
    }
    antifraud
        .check_wallet_balance(account.balance)
        .err()
        .map(|error| AccountRejection {
            module: AntifraudModule::WalletBalance,
            error,
            message: "Wallet balance exceeds limit",
        })
}

fn check_uninit_wallet_balance(
    antifraud: &Antifraud,
    account: &AccountSnapshot,
) -> Option<AccountRejection> {
    if !antifraud.uninit_wallet_balance_enabled() {
        return None;
    }
    antifraud
        .check_uninit_wallet_balance(
            account.balance,
            account.state == AccountStateEnum::Uninitialized,
        )
        .err()
        .map(|error| AccountRejection {
            module: AntifraudModule::UninitWalletBalance,
            error,
            message: "Uninitialized account balance exceeds limit. Initialize the account before requesting more tokens.",
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Json, Router, routing::post};
    use faucet_config::{
        AntifraudConfig, SentAmountWindowCheckConfig, SubnetAmountWindowCheckConfig,
        SuccessfulClaimWindowCheckConfig, ToncenterConfig, WalletBalanceCheckConfig,
    };
    use serde_json::{Value, json};
    use std::sync::{Arc, Mutex};
    use toncenter::ToncenterClient;

    fn config() -> AntifraudConfig {
        AntifraudConfig {
            enabled: true,
            wallet_balance: WalletBalanceCheckConfig {
                enabled: true,
                max_wallet_balance: 25_000_000_000,
            },
            uninit_wallet_balance: WalletBalanceCheckConfig {
                enabled: true,
                max_wallet_balance: 4_000_000_000,
            },
            sent_amount_window: SentAmountWindowCheckConfig {
                enabled: false,
                max_amount: 0,
                window_seconds: 60,
            },
            subnet_amount_window: SubnetAmountWindowCheckConfig {
                enabled: false,
                max_amount: 0,
                ipv4_prefix_length: 24,
                window_seconds: 60,
            },
            successful_claim_window: SuccessfulClaimWindowCheckConfig {
                enabled: false,
                max_requests: 0,
                window_seconds: 60,
            },
        }
    }

    fn account(state: &str, balance: &str) -> Value {
        json!({
            "ok": true,
            "@extra": "test",
            "result": {
                "@type": "raw.fullAccountState",
                "balance": balance,
                "extra_currencies": [],
                "last_transaction_id": {
                    "@type": "internal.transactionId", "lt": "0", "hash": "hash"
                },
                "block_id": {
                    "@type": "ton.blockIdExt", "workchain": -1,
                    "shard": "-9223372036854775808", "seqno": 1,
                    "root_hash": "root", "file_hash": "file"
                },
                "code": "",
                "data": "",
                "frozen_hash": "",
                "sync_utime": 0,
                "state": state
            }
        })
    }

    #[tokio::test]
    async fn checks_fresh_account_state_and_balance_and_fails_closed() {
        let response = Arc::new(Mutex::new(Value::Null));
        let requests = Arc::new(Mutex::new(Vec::<Value>::new()));
        let captured_response = Arc::clone(&response);
        let captured_requests = Arc::clone(&requests);
        let router = Router::new().route(
            "/api/v2/jsonRPC",
            post(move |Json(request): Json<Value>| {
                captured_requests.lock().unwrap().push(request);
                let response = captured_response.lock().unwrap().clone();
                async move { Json(response) }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        let client = ToncenterClient::new(&ToncenterConfig {
            api_key: None,
            url: format!("http://{address}"),
            timeout_seconds: 5,
            connect_timeout_seconds: 5,
            max_retries: 0,
            retry_base_delay_ms: 1,
        })
        .unwrap();
        let antifraud = Antifraud::new(&config());

        // The same address changes balance and then becomes active; do not cache eligibility.
        for (state, balance, expected_uninit, expected_wallet) in [
            ("uninitialized", "0", None, None),
            ("uninitialized", "4000000000", None, None),
            (
                "uninitialized",
                "4000000001",
                Some(AntifraudModule::UninitWalletBalance),
                None,
            ),
            (
                "uninitialized",
                "25000000001",
                Some(AntifraudModule::UninitWalletBalance),
                Some(AntifraudModule::WalletBalance),
            ),
            ("active", "4000000001", None, None),
            ("frozen", "4000000001", None, None),
            (
                "active",
                "25000000001",
                None,
                Some(AntifraudModule::WalletBalance),
            ),
        ] {
            *response.lock().unwrap() = account(state, balance);
            let request_count = requests.lock().unwrap().len();
            let account = client.get_address_information("wallet").await.unwrap();
            let snapshot = AccountSnapshot {
                balance: account.balance.parse().unwrap(),
                state: account.state,
            };
            assert_eq!(
                check_uninit_wallet_balance(&antifraud, &snapshot)
                    .map(|rejection| rejection.module),
                expected_uninit
            );
            assert_eq!(
                check_wallet_balance(&antifraud, &snapshot).map(|rejection| rejection.module),
                expected_wallet
            );
            assert_eq!(requests.lock().unwrap().len(), request_count + 1);
            assert_eq!(
                requests.lock().unwrap().last().unwrap(),
                &json!({
                    "id": "1",
                    "jsonrpc": "2.0",
                    "method": "getAddressInformation",
                    "params": {"address": "wallet"}
                })
            );
        }

        let mut missing_state = account("uninitialized", "5000000000");
        missing_state["result"]
            .as_object_mut()
            .unwrap()
            .remove("state");
        for invalid_response in [
            missing_state,
            account("unknown", "5000000000"),
            account("uninitialized", "invalid"),
            account("uninitialized", "-1"),
            json!({"ok": false, "error": "temporary error", "code": 503}),
        ] {
            *response.lock().unwrap() = invalid_response;
            assert!(
                client
                    .get_address_information("wallet")
                    .await
                    .and_then(|account| account
                        .balance
                        .parse::<u64>()
                        .context("Invalid account balance"))
                    .is_err()
            );
        }

        let mut config = config();
        *response.lock().unwrap() = account("uninitialized", "25000000001");
        for (enabled, wallet_enabled, uninit_enabled, expected_uninit, expected_wallet) in [
            (
                true,
                false,
                true,
                Some(AntifraudModule::UninitWalletBalance),
                None,
            ),
            (
                true,
                true,
                false,
                None,
                Some(AntifraudModule::WalletBalance),
            ),
            (true, false, false, None, None),
            (false, true, true, None, None),
        ] {
            config.enabled = enabled;
            config.wallet_balance.enabled = wallet_enabled;
            config.uninit_wallet_balance.enabled = uninit_enabled;
            let antifraud = Antifraud::new(&config);
            let request_count = requests.lock().unwrap().len();
            let account = client.get_address_information("wallet").await.unwrap();
            let snapshot = AccountSnapshot {
                balance: account.balance.parse().unwrap(),
                state: account.state,
            };
            assert_eq!(snapshot.balance, 25_000_000_001);
            assert_eq!(snapshot.state, AccountStateEnum::Uninitialized);
            assert_eq!(
                check_uninit_wallet_balance(&antifraud, &snapshot)
                    .map(|rejection| rejection.module),
                expected_uninit
            );
            assert_eq!(
                check_wallet_balance(&antifraud, &snapshot).map(|rejection| rejection.module),
                expected_wallet
            );
            assert_eq!(requests.lock().unwrap().len(), request_count + 1);
            assert_eq!(
                requests.lock().unwrap().last().unwrap()["method"],
                "getAddressInformation"
            );
        }
        server.abort();
    }
}
