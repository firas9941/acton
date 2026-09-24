use expect_test::expect;
use http_body_util::BodyExt;

use super::*;

#[tokio::test]
async fn malformed_messages_use_v2_errors() {
    let mut outcomes = Vec::new();
    for boc in [
        "?".to_owned(),
        STANDARD.encode([0]),
        STANDARD.encode(vec![0; 65_536]),
    ] {
        let response = parse_message(SendBocRequest { boc })
            .err()
            .unwrap()
            .into_response();
        let status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        outcomes.push(format!("{status}: {}", String::from_utf8_lossy(&body)));
    }

    expect![[r#"
        400 Bad Request: {"ok":false,"error":"boc must contain valid base64","code":400}
        400 Bad Request: {"ok":false,"error":"invalid inbound external message","code":400}
        413 Payload Too Large: {"ok":false,"error":"boc exceeds 65535 bytes","code":413}"#]]
    .assert_eq(&outcomes.join("\n"));
}
