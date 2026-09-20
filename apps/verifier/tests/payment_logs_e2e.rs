mod support;

use std::sync::Arc;

use support::{
    PAYMENT_ADDRESS, PAYMENT_TX_HASH, StaticPaymentBlockchainClient, payment_transaction,
};
use tracing::instrument::WithSubscriber;
use verifier::payment::{OnchainPaymentVerifier, PaymentLedger, PaymentVerifier};

const AMOUNT_NANO: u64 = 1_000_000;

// Keep log capture in its own test process: tracing callsite interest is global,
// while most verifier tests deliberately run without a subscriber in parallel.
#[tokio::test]
async fn first_payment_claim_reports_the_new_payment() {
    let code_hash = "ab".repeat(32);
    let client = Arc::new(StaticPaymentBlockchainClient::new(
        Some(payment_transaction(PAYMENT_TX_HASH, &code_hash)),
        Vec::new(),
    ));
    let verifier = OnchainPaymentVerifier::new(
        client,
        PaymentLedger::in_memory().expect("in-memory payment ledger should open"),
        PAYMENT_ADDRESS.to_owned(),
        AMOUNT_NANO,
    );
    verifier
        .recover(&[])
        .await
        .expect("empty payment history recovery should succeed");

    let log = tempfile::NamedTempFile::new().expect("log file should be created");
    let writer = log.reopen().expect("log writer should open");
    let subscriber = tracing::Dispatch::new(
        tracing_subscriber::fmt()
            .with_ansi(false)
            .without_time()
            .with_writer(move || writer.try_clone().expect("log writer should clone"))
            .finish(),
    );

    let claim = verifier
        .claim(PAYMENT_TX_HASH, &code_hash)
        .with_subscriber(subscriber)
        .await
        .expect("new payment should be reserved");
    assert_eq!(claim.claim_version, 1);

    let content = std::fs::read_to_string(log.path()).expect("logs should be readable");
    let event = content
        .lines()
        .find(|line| line.contains("new payment found and reserved"))
        .expect("new payment event should be logged");
    for expected in [
        &format!("transaction_hash={PAYMENT_TX_HASH}"),
        &format!("code_hash={code_hash}"),
        &format!("amount_nano={AMOUNT_NANO}"),
        &format!("payment_address={PAYMENT_ADDRESS}"),
        "network=testnet",
    ] {
        assert!(event.contains(expected), "{event}");
    }
}
