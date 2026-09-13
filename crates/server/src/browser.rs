use axum::{
    Json,
    body::Body,
    extract::{Extension, Path},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::{
    path::{Path as FsPath, PathBuf},
    sync::Arc,
};

#[derive(Clone, Debug, Default)]
pub struct BrowserConfig {
    pub enabled: bool,
    pub asset_root: Option<PathBuf>,
    pub allowed_origin: Option<HeaderValue>,
}

#[derive(Serialize)]
pub(crate) struct BootstrapResponse {
    pub version: u16,
    pub api_base: &'static str,
}

pub(crate) async fn bootstrap() -> Json<BootstrapResponse> {
    Json(BootstrapResponse {
        version: 2,
        api_base: "/api",
    })
}

pub(crate) async fn asset(
    Extension(root): Extension<Arc<PathBuf>>,
    Path(path): Path<String>,
) -> Response {
    let relative = path.trim_start_matches('/');
    let candidate = root.join(relative);
    let resolved = match candidate.canonicalize() {
        Ok(path) if path.starts_with(root.as_path()) && path.is_file() => path,
        _ if relative.is_empty() || !relative.contains('.') => root.join("index.html"),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    match tokio::fs::read(&resolved).await {
        Ok(bytes) => {
            let mut response = (StatusCode::OK, Body::from(bytes)).into_response();
            response
                .headers_mut()
                .insert(header::CONTENT_TYPE, content_type(&resolved));
            response.headers_mut().insert(
                header::CONTENT_SECURITY_POLICY,
                HeaderValue::from_static("default-src 'self'; base-uri 'self'; object-src 'none'; frame-ancestors 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'"),
            );
            response
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub(crate) async fn index(Extension(root): Extension<Arc<PathBuf>>) -> Response {
    asset(Extension(root), Path(String::new())).await
}

fn content_type(path: &FsPath) -> HeaderValue {
    let value = match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    };
    HeaderValue::from_static(value)
}
