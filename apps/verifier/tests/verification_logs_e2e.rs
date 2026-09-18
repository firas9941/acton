mod support;

use axum::http::StatusCode;
use support::{
    app_state, failing_compiler_app_state, file_part, post_verify_with_user_agent, text_part,
};

const CODE_HASH_ONE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn valid_verify_parts() -> Vec<support::MultipartPart> {
    vec![
        text_part("code_hash", CODE_HASH_ONE),
        text_part("language", "tolk"),
        text_part("compile_params", r#"{"compiler_version":"1.4.1"}"#),
        text_part("sources", r#"[{"path":"main.tolk","is_entrypoint":true}]"#),
        file_part(
            "files",
            "main.tolk",
            "text/plain",
            "fun main(): int { return 0; }",
        ),
    ]
}

// Keep log capture in its own test process: tracing callsite interest is global,
// while the other API tests deliberately run without a subscriber in parallel.
#[tokio::test]
async fn verification_logs_report_outcomes_without_uploading_source_payloads_to_logs() {
    use tracing::instrument::WithSubscriber;

    let log = tempfile::NamedTempFile::new().expect("log file");
    let writer = log.reopen().expect("log writer");
    let subscriber = tracing::Dispatch::new(
        tracing_subscriber::fmt()
            .with_ansi(false)
            .without_time()
            .with_writer(move || writer.try_clone().expect("log writer clone"))
            .finish(),
    );
    let mut parts = valid_verify_parts();
    for part in &mut parts {
        if let support::MultipartPart::Text {
            name: "compile_params",
            value,
        } = part
        {
            *value = std::borrow::Cow::Borrowed(
                r#"{"compiler_version":"1.4.1","private_metadata":"audit-payload-must-not-be-logged"}"#,
            );
        }
    }
    let response = post_verify_with_user_agent(app_state(&[], CODE_HASH_ONE), parts, "acton/1.2.3")
        .with_subscriber(subscriber.clone())
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let failure = post_verify_with_user_agent(
        failing_compiler_app_state(&[], "expected compiler error"),
        valid_verify_parts(),
        "blueprint/0.42.0",
    )
    .with_subscriber(subscriber)
    .await;
    assert_eq!(failure.status(), StatusCode::BAD_REQUEST);
    let content = std::fs::read_to_string(log.path()).expect("logs");
    assert!(!content.contains("audit-payload-must-not-be-logged"));
    let events: Vec<_> = content
        .lines()
        .filter(|line| line.contains("operation=\"verify\""))
        .collect();
    assert_eq!(events.len(), 5, "{content}");
    for (event, (outcome, user_agent)) in events.iter().zip([
        ("started", "acton/1.2.3"),
        ("match", "acton/1.2.3"),
        ("completed", "acton/1.2.3"),
        ("started", "blueprint/0.42.0"),
        ("failed", "blueprint/0.42.0"),
    ])
    {
        assert!(
            event.contains("operation=\"verify\"") && event.contains(CODE_HASH_ONE),
            "{event}"
        );
        assert!(
            event.contains(&format!("outcome={outcome}"))
                || event.contains(&format!("outcome=\"{outcome}\"")),
            "{event}"
        );
        assert!(event.contains(&format!("user_agent={user_agent}")), "{event}");
    }
    assert!(events.last().unwrap().contains("duration_ms="));
}
