use axum::{
    body::Bytes,
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
};

#[cfg(not(debug_assertions))]
use include_dir::{Dir, include_dir};
#[cfg(debug_assertions)]
use std::path::PathBuf;
use std::path::{Component, Path};

#[cfg(not(debug_assertions))]
static FRONTEND_DIR: Dir<'static> =
    include_dir!("$CARGO_MANIFEST_DIR/../../packages/faucet-ui/dist");

#[cfg(debug_assertions)]
const FRONTEND_DIST: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../packages/faucet-ui/dist");

pub(super) async fn handler(uri: Uri) -> Response {
    let request_path = uri.path();
    if is_backend_path(request_path) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let Some(asset_path) = asset_path(request_path) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    serve_asset(&asset_path)
        .await
        .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
}

fn is_backend_path(path: &str) -> bool {
    [
        "/auth",
        "/challenge",
        "/claim",
        "/healthz",
        "/metrics",
        "/openapi.json",
        "/readyz",
        "/robots.txt",
        "/stats",
        "/version",
    ]
    .iter()
    .any(|prefix| {
        path == *prefix
            || path
                .strip_prefix(prefix)
                .is_some_and(|suffix| suffix.starts_with('/'))
    })
}

fn asset_path(request_path: &str) -> Option<String> {
    let path = request_path.trim_start_matches('/');
    let path = if path.is_empty() || !asset_path_has_extension(path) {
        "index.html"
    } else {
        path
    };

    if is_safe_asset_path(path) {
        Some(path.to_owned())
    } else {
        None
    }
}

fn asset_path_has_extension(path: &str) -> bool {
    path.rsplit('/')
        .next()
        .is_some_and(|name| name.contains('.'))
}

fn is_safe_asset_path(path: &str) -> bool {
    !path.contains('\\')
        && Path::new(path)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

#[cfg(debug_assertions)]
async fn serve_asset(asset_path: &str) -> Option<Response> {
    let path = frontend_dist_path(asset_path)?;
    let bytes = tokio::fs::read(&path).await.ok()?;
    Some(asset_response(asset_path, Bytes::from(bytes)))
}

#[cfg(debug_assertions)]
fn frontend_dist_path(asset_path: &str) -> Option<PathBuf> {
    if !is_safe_asset_path(asset_path) {
        return None;
    }
    Some(Path::new(FRONTEND_DIST).join(asset_path))
}

#[cfg(not(debug_assertions))]
async fn serve_asset(asset_path: &str) -> Option<Response> {
    let file = FRONTEND_DIR.get_file(asset_path)?;
    Some(asset_response(
        asset_path,
        Bytes::from_static(file.contents()),
    ))
}

fn asset_response(asset_path: &str, bytes: Bytes) -> Response {
    ([(header::CONTENT_TYPE, content_type(asset_path))], bytes).into_response()
}

fn content_type(asset_path: &str) -> &'static str {
    match asset_path.rsplit('.').next() {
        Some("css") => "text/css; charset=utf-8",
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json" | "map") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        Some("wasm") => "application/wasm",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::{asset_path, is_backend_path};

    #[test]
    fn frontend_routes_use_the_spa_entry() {
        for route in ["/", "/oauth/callback", "/some/nested/route"] {
            assert_eq!(asset_path(route), Some("index.html".to_owned()));
        }
    }

    #[test]
    fn backend_routes_are_not_spa_fallbacks() {
        for route in [
            "/auth/unknown",
            "/challenge/unknown",
            "/claim/unknown",
            "/healthz/unknown",
            "/openapi.json/unknown",
            "/stats/unknown",
        ] {
            assert!(is_backend_path(route));
        }
    }
}
