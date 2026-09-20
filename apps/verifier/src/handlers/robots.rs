use axum::{http::header::CONTENT_TYPE, response::IntoResponse};

const ROBOTS_TXT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/robots.txt"));

pub async fn handler() -> impl IntoResponse {
    ([(CONTENT_TYPE, "text/plain; charset=utf-8")], ROBOTS_TXT)
}
