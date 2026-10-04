use axum::{
    body::Body,
    extract::{Path, Query},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use base64::prelude::*;
use flate2::read::{DeflateDecoder, GzDecoder, ZlibDecoder};
use serde::Deserialize;
use std::io::Read;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

/// Query parameters supported by `/svg`
#[derive(Debug, Deserialize)]
pub struct SvgQuery {
    #[serde(rename = "type")]
    pub viz_type: Option<String>,
    pub data: Option<String>,
    pub payload: Option<String>,
}

/// JSON request body supported by POST `/svg`
#[derive(Debug, Deserialize)]
pub struct SvgJsonRequest {
    #[serde(rename = "type")]
    pub viz_type: Option<String>,
    pub data: Option<String>,
    pub payload: Option<String>,
}

/// Robust payload decoder supporting:
/// - URL-safe base64 and standard base64 (with or without padding)
/// - Deflate decompression (raw deflate, zlib, and gzip)
/// - Uncompressed plain text fallback
pub fn decode_payload(encoded: &str) -> Result<String, String> {
    let trimmed = encoded.trim();
    if trimmed.is_empty() {
        return Err("Payload is empty".to_string());
    }

    // Normalize URL-safe Base64 to standard Base64 and restore padding
    let mut normalized = trimmed.replace('-', "+").replace('_', "/");
    while !normalized.len().is_multiple_of(4) {
        normalized.push('=');
    }

    let bytes = BASE64_STANDARD
        .decode(&normalized)
        .map_err(|e| format!("Base64 decode error: {e}"))?;

    // 1. Try raw Deflate decompression
    let mut deflate_decoder = DeflateDecoder::new(&bytes[..]);
    let mut decompressed = String::new();
    if deflate_decoder.read_to_string(&mut decompressed).is_ok() && !decompressed.is_empty() {
        return Ok(decompressed);
    }

    // 2. Try Zlib decompression
    let mut zlib_decoder = ZlibDecoder::new(&bytes[..]);
    decompressed.clear();
    if zlib_decoder.read_to_string(&mut decompressed).is_ok() && !decompressed.is_empty() {
        return Ok(decompressed);
    }

    // 3. Try Gzip decompression
    let mut gz_decoder = GzDecoder::new(&bytes[..]);
    decompressed.clear();
    if gz_decoder.read_to_string(&mut decompressed).is_ok() && !decompressed.is_empty() {
        return Ok(decompressed);
    }

    // 4. Fallback: uncompressed UTF-8 string
    String::from_utf8(bytes).map_err(|e| format!("Decompression / UTF-8 decode error: {e}"))
}

/// Format visual DSL based on type and body if not already wrapped in [docops,...]
pub fn format_dsl(viz_type: Option<&str>, body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.starts_with("[docops") {
        trimmed.to_string()
    } else if let Some(vt) = viz_type {
        let body_wrapped = if trimmed.starts_with("----") && trimmed.ends_with("----") {
            trimmed.to_string()
        } else {
            format!("----\n{trimmed}\n----")
        };
        format!("[docops,{vt}]\n{body_wrapped}")
    } else {
        trimmed.to_string()
    }
}

/// Builds an SVG HTTP response with appropriate caching headers
fn svg_response(svg: String, cacheable: bool) -> Response {
    let mut builder =
        Response::builder().header(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8");

    if cacheable {
        builder = builder.header(header::CACHE_CONTROL, "public, max-age=31536000, immutable");
    } else {
        builder = builder.header(header::CACHE_CONTROL, "no-cache");
    }

    builder.body(Body::from(svg)).unwrap_or_else(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to build response",
        )
            .into_response()
    })
}

/// Handler for GET `/svg` with query string parameters:
/// - `?type=<type>&data=<encoded>`
/// - `?payload=<encoded>`
pub async fn handle_get_query(Query(query): Query<SvgQuery>) -> Response {
    let dsl = match (&query.viz_type, &query.data, &query.payload) {
        (Some(viz_type), Some(data), _) => match decode_payload(data) {
            Ok(body) => format_dsl(Some(viz_type), &body),
            Err(err) => return (StatusCode::BAD_REQUEST, err).into_response(),
        },
        (_, _, Some(payload)) => match decode_payload(payload) {
            Ok(decoded) => format_dsl(query.viz_type.as_deref(), &decoded),
            Err(err) => return (StatusCode::BAD_REQUEST, err).into_response(),
        },
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                "Missing 'type' & 'data' or 'payload' query parameters. Example: /svg?type=pie&data=<base64>",
            )
                .into_response();
        }
    };

    let svg = docops_extension::generate_svg(&dsl);
    svg_response(svg, true)
}

/// Handler for GET `/svg/:payload` (single path param)
pub async fn handle_get_path_single(Path(payload): Path<String>) -> Response {
    match decode_payload(&payload) {
        Ok(decoded) => {
            let dsl = format_dsl(None, &decoded);
            let svg = docops_extension::generate_svg(&dsl);
            svg_response(svg, true)
        }
        Err(err) => (StatusCode::BAD_REQUEST, err).into_response(),
    }
}

/// Handler for GET `/svg/:type/:payload`
pub async fn handle_get_path_typed(Path((viz_type, payload)): Path<(String, String)>) -> Response {
    match decode_payload(&payload) {
        Ok(body) => {
            let dsl = format_dsl(Some(&viz_type), &body);
            let svg = docops_extension::generate_svg(&dsl);
            svg_response(svg, true)
        }
        Err(err) => (StatusCode::BAD_REQUEST, err).into_response(),
    }
}

/// Handler for POST `/svg` accepting raw text DSL or JSON body
pub async fn handle_post_svg(headers: HeaderMap, body: String) -> Response {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("text/plain");

    let dsl = if content_type.contains("application/json") {
        match serde_json::from_str::<SvgJsonRequest>(&body) {
            Ok(req) => match (&req.viz_type, &req.data, &req.payload) {
                (Some(vt), Some(data), _) => match decode_payload(data) {
                    Ok(decoded) => format_dsl(Some(vt), &decoded),
                    Err(_) => format_dsl(Some(vt), data),
                },
                (_, _, Some(payload)) => match decode_payload(payload) {
                    Ok(decoded) => format_dsl(req.viz_type.as_deref(), &decoded),
                    Err(_) => format_dsl(req.viz_type.as_deref(), payload),
                },
                _ => {
                    return (
                        StatusCode::BAD_REQUEST,
                        "JSON requires 'type' & 'data' or 'payload'",
                    )
                        .into_response()
                }
            },
            Err(e) => {
                return (StatusCode::BAD_REQUEST, format!("Invalid JSON: {e}")).into_response()
            }
        }
    } else {
        body
    };

    let svg = docops_extension::generate_svg(&dsl);
    svg_response(svg, false)
}

/// Handler for POST `/svg/:type`
pub async fn handle_post_typed_svg(Path(viz_type): Path<String>, body: String) -> Response {
    let dsl = format_dsl(Some(&viz_type), &body);
    let svg = docops_extension::generate_svg(&dsl);
    svg_response(svg, false)
}

/// Health check endpoint
pub async fn handle_health() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Root overview and documentation handler
pub async fn handle_index() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>DocOps Visual API</title>
    <style>
        :root {
            --bg: #0F172A;
            --surface: #1E293B;
            --border: #334155;
            --text: #F8FAFC;
            --muted: #94A3B8;
            --accent: #3B82F6;
            --code-bg: #0B1120;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            background: var(--bg);
            color: var(--text);
            line-height: 1.6;
            padding: 2rem 1rem;
            max-width: 800px;
            margin: 0 auto;
        }
        h1, h2, h3 { color: #FFFFFF; font-weight: 700; }
        h1 { margin-bottom: 0.5rem; }
        .subtitle { color: var(--muted); margin-bottom: 2rem; font-size: 1.1rem; }
        .card {
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 1.5rem;
            margin-bottom: 1.5rem;
        }
        code, pre {
            font-family: "JetBrains Mono", Consolas, Monaco, monospace;
            background: var(--code-bg);
            border-radius: 6px;
        }
        code { padding: 0.2rem 0.4rem; font-size: 0.9em; color: #60A5FA; }
        pre { padding: 1rem; overflow-x: auto; font-size: 0.85rem; border: 1px solid var(--border); }
        .badge {
            display: inline-block;
            background: rgba(59, 130, 246, 0.2);
            color: #60A5FA;
            padding: 0.2rem 0.6rem;
            border-radius: 9999px;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
        }
    </style>
</head>
<body>
    <h1>DocOps Visual API Server</h1>
    <div class="subtitle">High-performance native Rust microservice for on-demand SVG visual rendering</div>

    <div class="card">
        <h3>API Endpoints</h3>
        <ul>
            <li><code>GET /svg?type=&lt;type&gt;&amp;data=&lt;base64/deflate&gt;</code> — Render visual from query parameters</li>
            <li><code>GET /svg/:type/:payload</code> — Render visual with path parameters</li>
            <li><code>GET /svg/:type</code> — Render full <code>[docops,...]</code> encoded block (single param)</li>
            <li><code>POST /svg</code> — Render visual from raw DSL string or JSON payload</li>
            <li><code>POST /svg/:type</code> — Render visual body for a specific type</li>
            <li><code>GET /health</code> — Service health check</li>
        </ul>
    </div>

    <div class="card">
        <h3>Client-Side Encoding (CompressionStream)</h3>
        <p style="color: var(--muted); font-size: 0.9rem;">Zero-dependency client snippet using standard Web Streams (supported in modern browsers &amp; Node.js 18+):</p>
        <pre>async function encodeDocOps(text, format = "deflate-raw") {
  const stream = new Blob([text]).stream().pipeThrough(new CompressionStream(format));
  const compressed = await new Response(stream).arrayBuffer();
  const bytes = new Uint8Array(compressed);
  let binary = "";
  for (let i = 0; i &lt; bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}</pre>
    </div>

    <div class="card">
        <h3>Example Markdown / HTML Usage</h3>
        <pre>&lt;!-- Standard img tag in Markdown --&gt;
![Website Traffic](http://localhost:3000/svg?type=pie&amp;data=eNqLVjDXM9Qz0TMw1DMwMF...)</pre>
    </div>
</body>
</html>"#,
    )
}

/// Creates the Axum application router
pub fn create_app() -> Router {
    Router::new()
        .route("/", get(handle_index))
        .route("/health", get(handle_health))
        .route("/svg", get(handle_get_query).post(handle_post_svg))
        .route(
            "/svg/{type}",
            get(handle_get_path_single).post(handle_post_typed_svg),
        )
        .route("/svg/{type}/{payload}", get(handle_get_path_typed))
        .layer(CorsLayer::permissive())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "docops_server=info,tower_http=info".into()),
        )
        .init();

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("Invalid host/port configuration");

    let app = create_app();

    println!("⚡ DocOps SVG Server listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {addr}: {e}"));

    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod server_tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use flate2::write::DeflateEncoder;
    use flate2::Compression;
    use std::io::Write;
    use tower::util::ServiceExt; // for oneshot

    fn encode_deflate_base64(input: &str) -> String {
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(input.as_bytes()).unwrap();
        let compressed = encoder.finish().unwrap();
        BASE64_URL_SAFE_NO_PAD.encode(&compressed)
    }

    fn encode_plain_base64(input: &str) -> String {
        BASE64_STANDARD.encode(input.as_bytes())
    }

    #[test]
    fn test_decode_payload_deflate() {
        let text = "title= Test\nProduct A | 50\nProduct B | 50";
        let encoded = encode_deflate_base64(text);
        let decoded = decode_payload(&encoded).unwrap();
        assert_eq!(decoded, text);
    }

    #[test]
    fn test_decode_payload_plain_base64() {
        let text = "title= Plain Test\nItem | 100";
        let encoded = encode_plain_base64(text);
        let decoded = decode_payload(&encoded).unwrap();
        assert_eq!(decoded, text);
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = create_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_svg_query_endpoint() {
        let app = create_app();
        let body_data = "----\ntitle= Website Traffic\n---\nProduct A | 30\nProduct B | 70\n----";
        let encoded = encode_deflate_base64(body_data);

        let uri = format!("/svg?type=pie&data={encoded}");
        let response = app
            .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "image/svg+xml; charset=utf-8"
        );
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let svg = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Website Traffic"));
    }

    #[tokio::test]
    async fn test_get_svg_path_endpoint() {
        let app = create_app();
        let body_data = "----\ntitle= Path Test\n---\nProduct A | 40\nProduct B | 60\n----";
        let encoded = encode_deflate_base64(body_data);

        let uri = format!("/svg/pie/{encoded}");
        let response = app
            .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let svg = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Path Test"));
    }

    #[tokio::test]
    async fn test_post_svg_raw_endpoint() {
        let app = create_app();
        let dsl = r#"[docops,pie]
----
title= POST Raw Test
---
Alpha | 50
Beta | 50
----"#;

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/svg")
                    .header("content-type", "text/plain")
                    .body(Body::from(dsl))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let svg = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("POST Raw Test"));
    }

    #[tokio::test]
    async fn test_post_svg_json_endpoint() {
        let app = create_app();
        let body_data = "----\ntitle= JSON Test\n---\nItem 1 | 30\nItem 2 | 70\n----";
        let encoded = encode_deflate_base64(body_data);
        let json_body = format!(r#"{{"type":"pie","data":"{encoded}"}}"#);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/svg")
                    .header("content-type", "application/json")
                    .body(Body::from(json_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let svg = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("JSON Test"));
    }

    #[tokio::test]
    async fn test_post_typed_svg_endpoint() {
        let app = create_app();
        let body_data = "title= Typed POST Test\n---\nGamma | 20\nDelta | 80";

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/svg/pie")
                    .header("content-type", "text/plain")
                    .body(Body::from(body_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let svg = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Typed POST Test"));
    }

    #[tokio::test]
    async fn test_invalid_base64_returns_bad_request() {
        let app = create_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/svg?type=pie&data=@@invalid_base64@@")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
