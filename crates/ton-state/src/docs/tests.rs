use axum::body::Body;
use axum::http::Request;
use expect_test::expect;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

#[tokio::test]
async fn reference_serves_only_supported_operations() {
    let app = super::router();
    let response = app
        .clone()
        .oneshot(Request::get("/openapi.json").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status().as_u16();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let document: Value = serde_json::from_slice(&body).unwrap();
    let mut methods = Vec::new();

    for (path, item) in document["paths"].as_object().unwrap() {
        for (method, operation) in item.as_object().unwrap() {
            methods.push(format!("{} {path}", method.to_uppercase()));
            assert!(operation.get("tags").is_none());
        }
    }

    // Every shared response type must be registered, including nested references.
    let mut pending = vec![&document];
    while let Some(value) = pending.pop() {
        match value {
            Value::Object(object) => {
                if let Some(reference) = object.get("$ref") {
                    let pointer = reference.as_str().unwrap().strip_prefix('#').unwrap();
                    assert!(
                        document.pointer(pointer).is_some(),
                        "unresolved {reference}"
                    );
                }
                pending.extend(object.values());
            }
            Value::Array(array) => pending.extend(array),
            _ => {}
        }
    }

    let home = app
        .clone()
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let page = app
        .oneshot(Request::get("/docs").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let contract = json!({
        "status": status,
        "servers": document["servers"],
        "methods": methods,
        "stream_content_types": document["paths"]["/api/streaming/v2/sse"]["post"]
            ["responses"]["200"]["content"].as_object().unwrap().keys().collect::<Vec<_>>(),
        "redirect": [home.status().as_str(), home.headers()["location"].to_str().unwrap()],
        "page": [page.status().as_str(), page.headers()["content-type"].to_str().unwrap()],
    });

    expect![[r#"
        {
          "methods": [
            "POST /api/streaming/v2/sse",
            "GET /api/v2/getAddressBalance",
            "GET /api/v2/getAddressInformation",
            "GET /api/v2/getMasterchainInfo",
            "POST /api/v2/sendBoc"
          ],
          "page": [
            "200",
            "text/html; charset=utf-8"
          ],
          "redirect": [
            "307",
            "/docs"
          ],
          "servers": [
            {
              "url": "/"
            }
          ],
          "status": 200,
          "stream_content_types": [
            "text/event-stream"
          ]
        }
    "#]]
    .assert_eq(&format!(
        "{}\n",
        serde_json::to_string_pretty(&contract).unwrap()
    ));
}
