//! HTTP contract snapshots exercise the real router and durable runtime together.

use acton_localnet::{CreateNetwork, Runtime, catalog, http};
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request},
};
use expect_test::expect;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::Notify;
use tower::ServiceExt;

async fn request(
    app: &Router,
    method: Method,
    path: &str,
    body: Value,
    token: Option<&str>,
) -> (u16, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("Content-Type", "application/json");

    if let Some(token) = token {
        request = request.header("Authorization", format!("Bearer {token}"));
    }

    let response = app
        .clone()
        .oneshot(request.body(Body::from(body.to_string())).expect("request"))
        .await
        .expect("response");
    let status = response.status().as_u16();
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body");

    (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
}

#[tokio::test]
async fn authenticated_api_is_scoped_to_one_network() {
    let root = tempfile::tempdir().expect("state directory");
    let first = catalog::create(
        root.path(),
        CreateNetwork {
            name: "first".to_owned(),
            ..Default::default()
        },
    )
    .await
    .expect("first network");
    let second = catalog::create(
        root.path(),
        CreateNetwork {
            name: "second".to_owned(),
            ..Default::default()
        },
    )
    .await
    .expect("second network");
    let runtime = Runtime::open(&first.path).await.expect("runtime");
    let app = http::router(
        runtime.clone(),
        "secret".to_owned(),
        Arc::new(Notify::new()),
    );
    let unauthorized = request(&app, Method::GET, "/v1/network", Value::Null, None).await;
    let network = request(
        &app,
        Method::GET,
        "/v1/network",
        Value::Null,
        Some("secret"),
    )
    .await;
    let health = request(&app, Method::GET, "/v1/health", Value::Null, Some("secret")).await;
    let catalog_route = request(
        &app,
        Method::GET,
        "/v1/networks",
        Value::Null,
        Some("secret"),
    )
    .await;
    let create_route = request(
        &app,
        Method::POST,
        "/v1/networks",
        json!({"name":"third"}),
        Some("secret"),
    )
    .await;
    let other_network = request(
        &app,
        Method::POST,
        &format!("/v1/networks/{}/start", second.network.id),
        Value::Null,
        Some("secret"),
    )
    .await;
    let snapshots = request(
        &app,
        Method::GET,
        "/v1/network/snapshots",
        Value::Null,
        Some("secret"),
    )
    .await;
    expect![[r#"
        {
          "catalog": 404,
          "create": 404,
          "health": 200,
          "name": "first",
          "network": 200,
          "otherNetwork": 404,
          "snapshots": [],
          "unauthorized": 401
        }"#]]
    .assert_eq(
        &serde_json::to_string_pretty(&json!({
            "unauthorized": unauthorized.0, "network": network.0, "name": network.1["name"],
            "health": health.0, "catalog": catalog_route.0, "create": create_route.0,
            "otherNetwork": other_network.0, "snapshots": snapshots.1,
        }))
        .expect("API snapshot"),
    );

    let second_runtime = Runtime::open(&second.path)
        .await
        .expect("independent ownership");
    let duplicate = Runtime::open(&first.path)
        .await
        .err()
        .expect("exclusive ownership");
    expect![["service_already_running"]].assert_eq(match duplicate {
        acton_localnet::Error::Conflict { code, .. } => code,
        _ => "unexpected error",
    });
    runtime.shutdown().await.expect("first shutdown");
    expect![["second"]].assert_eq(&second_runtime.get().await.name);
    second_runtime.shutdown().await.expect("second shutdown");
}

#[tokio::test]
async fn accepted_operations_survive_client_disconnect_and_service_shutdown_rejects_writes() {
    let root = tempfile::tempdir().expect("state directory");
    let location = catalog::create(
        root.path(),
        CreateNetwork {
            name: "bad-image".to_owned(),
            ..Default::default()
        },
    )
    .await
    .expect("network");
    let runtime = Runtime::open(&location.path).await.expect("runtime");

    // Pin a malformed image to fail before invoking Docker. This exercises the
    // actual asynchronous operation path without depending on a Docker daemon.
    let descriptor = json!({"version":2, "image":"bad image", "dockerTarget":{"kind":"context","value":"test"}, "projectName":"acton-localnet-test"});
    std::fs::write(location.path.join("runtime.json"), descriptor.to_string()).expect("descriptor");
    let app = http::router(
        runtime.clone(),
        "secret".to_owned(),
        Arc::new(Notify::new()),
    );
    let (status, accepted) = request(
        &app,
        Method::POST,
        "/v1/network/start",
        Value::Null,
        Some("secret"),
    )
    .await;
    let operation_id = accepted["id"].as_str().expect("operation id");

    drop(app);
    let operation = tokio::time::timeout(std::time::Duration::from_secs(3), async {
        loop {
            let operation = runtime
                .operation(operation_id)
                .await
                .expect("durable operation");
            if operation.status != acton_localnet::OperationStatus::Running {
                break operation;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("operation finished");

    expect![["202:start:Failed:failed"]].assert_eq(&format!(
        "{status}:{}:{:?}:{}",
        operation.kind, operation.status, operation.phase
    ));
    expect![["true:true"]].assert_eq(&format!(
        "{}:{}",
        operation
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("valid container image"),
        operation
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("Full log:")
    ));

    runtime
        .prepare_shutdown()
        .await
        .expect("stop accepting mutations");
    let app = http::router(
        runtime.clone(),
        "secret".to_owned(),
        Arc::new(Notify::new()),
    );
    let stopped = request(
        &app,
        Method::POST,
        "/v1/network/start",
        Value::Null,
        Some("secret"),
    )
    .await;
    expect![[r#"
        [
          409,
          {
            "code": "service_stopping",
            "message": "The localnet service is stopping"
          }
        ]"#]]
    .assert_eq(&serde_json::to_string_pretty(&stopped).expect("snapshot"));
}
