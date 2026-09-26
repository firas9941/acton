use axum::{
    Extension, Router,
    body::{Body, to_bytes},
    http::{
        Method, Request, StatusCode,
        header::{ORIGIN, USER_AGENT},
    },
    middleware,
    routing::{get, post},
};
use faucet::middlewares::{ClientContext, require_actonscan_origin, require_airdrop_headers};
use tower::ServiceExt;

#[tokio::test]
async fn requires_airdrop_headers_on_protected_route() {
    let response = request_with_headers(Some("acton/0.1.0"), None, Some("default")).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response_body(response).await, "acton");

    let response = request_with_headers(
        Some("acton/0.1.0"),
        None,
        Some("00112233445566778899aabbccddeeff"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    let response = request_with_headers(
        Some("acton/0.1.0"),
        None,
        Some("00112233-4455-6677-8899-aabbccddeeff"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    let response = request_with_headers(
        Some("acton/0.1.0"),
        None,
        Some("00112233-4455-6677-8899-AABBCCDDEEFF"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    let response = request_with_headers(None, None, Some("default")).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let response = request_with_headers(Some("acton/"), None, Some("default")).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let response = request_with_headers(Some("faucet/0.1.0"), None, Some("default")).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let response = request_with_headers(Some("acton/0.1.0"), None, None).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let response = request_with_headers(Some("acton/0.1.0"), None, Some("")).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let response = request_with_headers(Some("acton/0.1.0"), None, Some(" ")).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let response = request_with_headers(Some("acton/0.1.0"), None, Some("device-1")).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let response = request_with_headers(Some("acton/0.1.0"), None, Some("another")).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_unofficial_user_agent_formats_on_airdrop_routes() {
    let app = airdrop_app();

    for path in ["/challenge", "/claim"] {
        for (user_agent, expected_status) in [
            ("acton/0.5.0", StatusCode::OK),
            ("acton/1.1.0-trunk", StatusCode::OK),
            ("acton/stage2", StatusCode::BAD_REQUEST),
            (
                "acton/0.5.0 (allchains-faucet-ton)",
                StatusCode::BAD_REQUEST,
            ),
            ("acton/1.1.0-beta.1", StatusCode::BAD_REQUEST),
            ("acton/1.1.0+build.5", StatusCode::BAD_REQUEST),
        ] {
            let request = Request::post(path)
                .header(USER_AGENT, user_agent)
                .header("x-device-uid", "default")
                .body(Body::empty())
                .unwrap();
            let response = app.clone().oneshot(request).await.unwrap();

            assert_eq!(response.status(), expected_status, "{path}: {user_agent}");
            let expected_body = if expected_status == StatusCode::OK {
                "acton"
            } else {
                ""
            };
            assert_eq!(response_body(response).await, expected_body);
        }
    }
}

#[tokio::test]
async fn allows_actonscan_browser_client_header() {
    for (client, expected_status) in [
        ("actonscan/1.0.0", StatusCode::OK),
        ("actonscan/", StatusCode::BAD_REQUEST),
        ("explorer/1.0.0", StatusCode::BAD_REQUEST),
    ] {
        let request = Request::post("/challenge")
            .header("x-acton-client", client)
            .header("x-device-uid", "default")
            .header(ORIGIN, "https://actonscan.com")
            .body(Body::empty())
            .unwrap();
        let response = airdrop_app().oneshot(request).await.unwrap();

        assert_eq!(response.status(), expected_status, "{client}");
        if expected_status == StatusCode::OK {
            assert_eq!(response_body(response).await, "actonscan");
        }
    }
}

#[tokio::test]
async fn requires_nonempty_origin_for_actonscan_challenges_and_claims() {
    let app = airdrop_app();

    for path in ["/challenge", "/claim"] {
        for (origin, expected_status) in [
            (None, StatusCode::BAD_REQUEST),
            (Some(""), StatusCode::BAD_REQUEST),
            (Some(" \t "), StatusCode::BAD_REQUEST),
            (Some("https://actonscan.com"), StatusCode::OK),
            (Some("http://localhost:5173"), StatusCode::OK),
            (Some("http://127.0.0.1:5173"), StatusCode::OK),
            // Only require a nonempty header; do not maintain an origin allowlist here.
            (Some("https://example.com"), StatusCode::OK),
        ] {
            for user_agent in [None, Some("acton/0.1.0")] {
                let mut request = Request::post(path)
                    .header("x-acton-client", "actonscan/1.0.0")
                    .header("x-device-uid", "default");
                if let Some(origin) = origin {
                    request = request.header(ORIGIN, origin);
                }
                if let Some(user_agent) = user_agent {
                    request = request.header(USER_AGENT, user_agent);
                }
                let response = app
                    .clone()
                    .oneshot(request.body(Body::empty()).unwrap())
                    .await
                    .unwrap();

                assert_eq!(
                    response.status(),
                    expected_status,
                    "{path}: origin={origin:?}, user_agent={user_agent:?}"
                );
                if expected_status == StatusCode::OK {
                    assert_eq!(response_body(response).await, "actonscan");
                }
            }
        }
    }
}

#[tokio::test]
async fn allows_actonscan_auth_requests_without_origin() {
    for (method, path) in [
        (Method::GET, "/auth/status"),
        (Method::GET, "/auth/session"),
        (Method::POST, "/auth/exchange"),
        (Method::DELETE, "/auth/session"),
    ] {
        let request = Request::builder()
            .method(method.clone())
            .uri(path)
            .header("x-acton-client", "actonscan/1.0.0")
            .header("x-device-uid", "default")
            .body(Body::empty())
            .unwrap();
        let response = airdrop_app().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK, "{method} {path}");
        assert_eq!(response_body(response).await, "actonscan");
    }
}

#[tokio::test]
async fn origin_requirement_only_applies_to_airdrop_post_handlers() {
    let app = airdrop_app();

    for path in ["/challenge", "/claim"] {
        for method in [Method::GET, Method::DELETE, Method::PUT, Method::OPTIONS] {
            let request = Request::builder()
                .method(method.clone())
                .uri(path)
                .header("x-acton-client", "actonscan/1.0.0")
                .header("x-device-uid", "default")
                .body(Body::empty())
                .unwrap();
            let response = app.clone().oneshot(request).await.unwrap();

            assert_eq!(
                response.status(),
                StatusCode::METHOD_NOT_ALLOWED,
                "{method} {path}"
            );
        }
    }
}

#[tokio::test]
async fn normalizes_device_uid_before_inserting_client_context() {
    let app = Router::new()
        .route(
            "/challenge",
            post(|Extension(client): Extension<ClientContext>| async move { client.device_uid }),
        )
        .route_layer(middleware::from_fn(require_airdrop_headers));

    for (device_uid, expected) in [
        (
            "00112233445566778899AABBCCDDEEFF",
            "00112233445566778899aabbccddeeff",
        ),
        (
            "00112233-4455-6677-8899-AABBCCDDEEFF",
            "00112233445566778899aabbccddeeff",
        ),
    ] {
        let request = Request::builder()
            .method(Method::POST)
            .uri("/challenge")
            .header(USER_AGENT, "acton/0.1.0")
            .header("x-device-uid", device_uid)
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_body(response).await, expected);
    }
}

fn airdrop_app() -> Router {
    let handler =
        |Extension(client): Extension<ClientContext>| async move { client.client_kind.as_str() };
    let airdrop_handler = post(handler).route_layer(middleware::from_fn(require_actonscan_origin));
    Router::new()
        .route("/challenge", airdrop_handler.clone())
        .route("/claim", airdrop_handler)
        .route("/auth/status", get(handler))
        .route("/auth/exchange", post(handler))
        .route("/auth/session", get(handler).delete(handler))
        .route_layer(middleware::from_fn(require_airdrop_headers))
}

async fn request_with_headers(
    user_agent: Option<&str>,
    acton_client: Option<&str>,
    device_uid: Option<&str>,
) -> axum::response::Response {
    let app = airdrop_app();

    let mut request = Request::builder().method(Method::POST).uri("/challenge");

    if let Some(user_agent) = user_agent {
        request = request.header(USER_AGENT, user_agent);
    }
    if let Some(acton_client) = acton_client {
        request = request.header("x-acton-client", acton_client);
    }
    if let Some(device_uid) = device_uid {
        request = request.header("x-device-uid", device_uid);
    }

    app.oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn response_body(response: axum::response::Response) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}
