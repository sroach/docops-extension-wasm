use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use base64::prelude::*;
use flate2::read::{DeflateDecoder, GzDecoder, ZlibDecoder};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tower_http::cors::CorsLayer;

/// In-memory tracker for operational metrics with zero external overhead
#[derive(Debug)]
pub struct ServerMetrics {
    start_time: Instant,
    pub total_requests: AtomicU64,
    pub successful_renders: AtomicU64,
    pub failed_requests: AtomicU64,
    pub total_render_time_us: AtomicU64,
    pub total_bytes_rendered: AtomicU64,

    // Breakdown by visual type
    pub count_pie: AtomicU64,
    pub count_bar: AtomicU64,
    pub count_line: AtomicU64,
    pub count_combination: AtomicU64,
    pub count_badge: AtomicU64,
    pub count_adr: AtomicU64,
    pub count_scorecard: AtomicU64,
    pub count_button: AtomicU64,
    pub count_quadrant: AtomicU64,
    pub count_gherkin: AtomicU64,
    pub count_gauge: AtomicU64,
    pub count_recipe: AtomicU64,
    pub count_timeline: AtomicU64,
    pub count_steps: AtomicU64,
    pub count_release: AtomicU64,
    pub count_metrics_card: AtomicU64,
    pub count_other: AtomicU64,
}

impl Default for ServerMetrics {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            total_requests: AtomicU64::new(0),
            successful_renders: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            total_render_time_us: AtomicU64::new(0),
            total_bytes_rendered: AtomicU64::new(0),
            count_pie: AtomicU64::new(0),
            count_bar: AtomicU64::new(0),
            count_line: AtomicU64::new(0),
            count_combination: AtomicU64::new(0),
            count_badge: AtomicU64::new(0),
            count_adr: AtomicU64::new(0),
            count_scorecard: AtomicU64::new(0),
            count_button: AtomicU64::new(0),
            count_quadrant: AtomicU64::new(0),
            count_gherkin: AtomicU64::new(0),
            count_gauge: AtomicU64::new(0),
            count_recipe: AtomicU64::new(0),
            count_timeline: AtomicU64::new(0),
            count_steps: AtomicU64::new(0),
            count_release: AtomicU64::new(0),
            count_metrics_card: AtomicU64::new(0),
            count_other: AtomicU64::new(0),
        }
    }
}

/// JSON summary representation of server operational statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub uptime_seconds: u64,
    pub total_requests: u64,
    pub successful_renders: u64,
    pub failed_requests: u64,
    pub avg_render_time_ms: f64,
    pub total_bytes_rendered: u64,
    pub visuals_breakdown: VisualBreakdown,
}

/// Visual type usage distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualBreakdown {
    pub pie: u64,
    pub bar: u64,
    pub line: u64,
    pub combination: u64,
    pub badge: u64,
    pub adr: u64,
    pub scorecard: u64,
    pub button: u64,
    pub quadrant: u64,
    pub gherkin: u64,
    pub gauge: u64,
    pub recipe: u64,
    pub timeline: u64,
    pub steps: u64,
    pub release: u64,
    pub metrics_card: u64,
    pub other: u64,
}

impl ServerMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_request(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_failure(&self) {
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_render(&self, dsl: &str, elapsed_us: u64, bytes_len: usize) {
        self.successful_renders.fetch_add(1, Ordering::Relaxed);
        self.total_render_time_us
            .fetch_add(elapsed_us, Ordering::Relaxed);
        self.total_bytes_rendered
            .fetch_add(bytes_len as u64, Ordering::Relaxed);

        let viz_type = extract_viz_type(dsl);
        match viz_type.as_str() {
            "pie" | "piechart" | "pieslice" => self.count_pie.fetch_add(1, Ordering::Relaxed),
            "bar" | "barchart" => self.count_bar.fetch_add(1, Ordering::Relaxed),
            "line" | "linechart" => self.count_line.fetch_add(1, Ordering::Relaxed),
            "combination" | "combo" => self.count_combination.fetch_add(1, Ordering::Relaxed),
            "badge" => self.count_badge.fetch_add(1, Ordering::Relaxed),
            "adr" => self.count_adr.fetch_add(1, Ordering::Relaxed),
            "scorecard" | "score" => self.count_scorecard.fetch_add(1, Ordering::Relaxed),
            "button" => self.count_button.fetch_add(1, Ordering::Relaxed),
            "quadrant" | "magic" => self.count_quadrant.fetch_add(1, Ordering::Relaxed),
            "gherkin" => self.count_gherkin.fetch_add(1, Ordering::Relaxed),
            "gauge" | "gaugechart" => self.count_gauge.fetch_add(1, Ordering::Relaxed),
            "recipe" => self.count_recipe.fetch_add(1, Ordering::Relaxed),
            "timeline" => self.count_timeline.fetch_add(1, Ordering::Relaxed),
            "steps" => self.count_steps.fetch_add(1, Ordering::Relaxed),
            "release" | "releasestrategy" => self.count_release.fetch_add(1, Ordering::Relaxed),
            "metrics" | "metricscard" => self.count_metrics_card.fetch_add(1, Ordering::Relaxed),
            _ => self.count_other.fetch_add(1, Ordering::Relaxed),
        };
    }

    pub fn summary(&self) -> MetricsSummary {
        let total_ok = self.successful_renders.load(Ordering::Relaxed);
        let total_us = self.total_render_time_us.load(Ordering::Relaxed);
        let avg_ms = if total_ok > 0 {
            (total_us as f64 / total_ok as f64) / 1000.0
        } else {
            0.0
        };

        MetricsSummary {
            uptime_seconds: self.start_time.elapsed().as_secs(),
            total_requests: self.total_requests.load(Ordering::Relaxed),
            successful_renders: total_ok,
            failed_requests: self.failed_requests.load(Ordering::Relaxed),
            avg_render_time_ms: (avg_ms * 100.0).round() / 100.0,
            total_bytes_rendered: self.total_bytes_rendered.load(Ordering::Relaxed),
            visuals_breakdown: VisualBreakdown {
                pie: self.count_pie.load(Ordering::Relaxed),
                bar: self.count_bar.load(Ordering::Relaxed),
                line: self.count_line.load(Ordering::Relaxed),
                combination: self.count_combination.load(Ordering::Relaxed),
                badge: self.count_badge.load(Ordering::Relaxed),
                adr: self.count_adr.load(Ordering::Relaxed),
                scorecard: self.count_scorecard.load(Ordering::Relaxed),
                button: self.count_button.load(Ordering::Relaxed),
                quadrant: self.count_quadrant.load(Ordering::Relaxed),
                gherkin: self.count_gherkin.load(Ordering::Relaxed),
                gauge: self.count_gauge.load(Ordering::Relaxed),
                recipe: self.count_recipe.load(Ordering::Relaxed),
                timeline: self.count_timeline.load(Ordering::Relaxed),
                steps: self.count_steps.load(Ordering::Relaxed),
                release: self.count_release.load(Ordering::Relaxed),
                metrics_card: self.count_metrics_card.load(Ordering::Relaxed),
                other: self.count_other.load(Ordering::Relaxed),
            },
        }
    }
}

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

/// Extracts the visualization type from DSL or raw block header (e.g. `[docops,pie]` -> `"pie"`)
pub fn extract_viz_type(dsl: &str) -> String {
    let trimmed = dsl.trim();
    if let Some(rest) = trimmed.strip_prefix("[docops") {
        let rest = rest.trim_start();
        if let Some(after_comma) = rest.strip_prefix(',') {
            if let Some(end) = after_comma.find(']') {
                return after_comma[..end].trim().to_lowercase();
            }
        }
    }
    if trimmed.starts_with('[') {
        if let Some(end) = trimmed.find(']') {
            let inside = &trimmed[1..end];
            let parts: Vec<&str> = inside.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 2 && parts[0].eq_ignore_ascii_case("docops") {
                return parts[1].to_lowercase();
            }
        }
    }
    "unknown".to_string()
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
pub async fn handle_get_query(
    State(metrics): State<Arc<ServerMetrics>>,
    Query(query): Query<SvgQuery>,
) -> Response {
    metrics.record_request();
    let dsl = match (&query.viz_type, &query.data, &query.payload) {
        (Some(viz_type), Some(data), _) => match decode_payload(data) {
            Ok(body) => format_dsl(Some(viz_type), &body),
            Err(err) => {
                metrics.record_failure();
                return (StatusCode::BAD_REQUEST, err).into_response();
            }
        },
        (_, _, Some(payload)) => match decode_payload(payload) {
            Ok(decoded) => format_dsl(query.viz_type.as_deref(), &decoded),
            Err(err) => {
                metrics.record_failure();
                return (StatusCode::BAD_REQUEST, err).into_response();
            }
        },
        _ => {
            metrics.record_failure();
            return (
                StatusCode::BAD_REQUEST,
                "Missing 'type' & 'data' or 'payload' query parameters. Example: /svg?type=pie&data=<base64>",
            )
                .into_response();
        }
    };

    let start = Instant::now();
    let svg = docops_extension::generate_svg(&dsl);
    let elapsed_us = start.elapsed().as_micros() as u64;
    metrics.record_render(&dsl, elapsed_us, svg.len());
    svg_response(svg, true)
}

/// Handler for GET `/svg/:payload` (single path param)
pub async fn handle_get_path_single(
    State(metrics): State<Arc<ServerMetrics>>,
    Path(payload): Path<String>,
) -> Response {
    metrics.record_request();
    match decode_payload(&payload) {
        Ok(decoded) => {
            let dsl = format_dsl(None, &decoded);
            let start = Instant::now();
            let svg = docops_extension::generate_svg(&dsl);
            let elapsed_us = start.elapsed().as_micros() as u64;
            metrics.record_render(&dsl, elapsed_us, svg.len());
            svg_response(svg, true)
        }
        Err(err) => {
            metrics.record_failure();
            (StatusCode::BAD_REQUEST, err).into_response()
        }
    }
}

/// Handler for GET `/svg/:type/:payload`
pub async fn handle_get_path_typed(
    State(metrics): State<Arc<ServerMetrics>>,
    Path((viz_type, payload)): Path<(String, String)>,
) -> Response {
    metrics.record_request();
    match decode_payload(&payload) {
        Ok(body) => {
            let dsl = format_dsl(Some(&viz_type), &body);
            let start = Instant::now();
            let svg = docops_extension::generate_svg(&dsl);
            let elapsed_us = start.elapsed().as_micros() as u64;
            metrics.record_render(&dsl, elapsed_us, svg.len());
            svg_response(svg, true)
        }
        Err(err) => {
            metrics.record_failure();
            (StatusCode::BAD_REQUEST, err).into_response()
        }
    }
}

/// Handler for POST `/svg` accepting raw text DSL or JSON body
pub async fn handle_post_svg(
    State(metrics): State<Arc<ServerMetrics>>,
    headers: HeaderMap,
    body: String,
) -> Response {
    metrics.record_request();
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
                    metrics.record_failure();
                    return (
                        StatusCode::BAD_REQUEST,
                        "JSON requires 'type' & 'data' or 'payload'",
                    )
                        .into_response();
                }
            },
            Err(e) => {
                metrics.record_failure();
                return (StatusCode::BAD_REQUEST, format!("Invalid JSON: {e}")).into_response();
            }
        }
    } else {
        body
    };

    let start = Instant::now();
    let svg = docops_extension::generate_svg(&dsl);
    let elapsed_us = start.elapsed().as_micros() as u64;
    metrics.record_render(&dsl, elapsed_us, svg.len());
    svg_response(svg, false)
}

/// Handler for POST `/svg/:type`
pub async fn handle_post_typed_svg(
    State(metrics): State<Arc<ServerMetrics>>,
    Path(viz_type): Path<String>,
    body: String,
) -> Response {
    metrics.record_request();
    let dsl = format_dsl(Some(&viz_type), &body);
    let start = Instant::now();
    let svg = docops_extension::generate_svg(&dsl);
    let elapsed_us = start.elapsed().as_micros() as u64;
    metrics.record_render(&dsl, elapsed_us, svg.len());
    svg_response(svg, false)
}

/// Operational statistics / metrics endpoint (JSON)
pub async fn handle_stats(State(metrics): State<Arc<ServerMetrics>>) -> Json<MetricsSummary> {
    metrics.record_request();
    Json(metrics.summary())
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
            <li><code>GET /stats</code> — In-memory metrics &amp; visual usage statistics (JSON)</li>
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

/// Creates the Axum application router with a fresh metrics state
pub fn create_app() -> Router {
    create_app_with_metrics(Arc::new(ServerMetrics::new()))
}

/// Creates the Axum application router with a custom metrics instance
pub fn create_app_with_metrics(metrics: Arc<ServerMetrics>) -> Router {
    Router::new()
        .route("/", get(handle_index))
        .route("/health", get(handle_health))
        .route("/stats", get(handle_stats))
        .route("/metrics", get(handle_stats))
        .route("/svg", get(handle_get_query).post(handle_post_svg))
        .route(
            "/svg/{type}",
            get(handle_get_path_single).post(handle_post_typed_svg),
        )
        .route("/svg/{type}/{payload}", get(handle_get_path_typed))
        .layer(CorsLayer::permissive())
        .with_state(metrics)
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

    #[tokio::test]
    async fn test_stats_endpoint() {
        let metrics = Arc::new(ServerMetrics::new());
        let app = create_app_with_metrics(Arc::clone(&metrics));

        // 1. Initial metrics check
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let summary: MetricsSummary = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(summary.total_requests, 1);
        assert_eq!(summary.successful_renders, 0);
        assert_eq!(summary.failed_requests, 0);

        // 2. Perform a successful pie render
        let body_data = "----\ntitle= Stats Test Pie\n---\nSlice A | 50\nSlice B | 50\n----";
        let encoded = encode_deflate_base64(body_data);
        let uri = format!("/svg?type=pie&data={encoded}");
        let _ = app
            .clone()
            .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
            .await
            .unwrap();

        // 3. Perform a successful bar render
        let dsl_bar = "[docops,bar]\n----\ntitle= Stats Test Bar\n---\nItem 1 | 100\n----";
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/svg")
                    .header("content-type", "text/plain")
                    .body(Body::from(dsl_bar))
                    .unwrap(),
            )
            .await
            .unwrap();

        // 4. Perform a failed request
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/svg?type=pie&data=@@invalid@@")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // 5. Query stats again
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let summary: MetricsSummary = serde_json::from_slice(&body_bytes).unwrap();

        // Total requests = 1 (initial stats) + 1 (pie) + 1 (bar) + 1 (failed) + 1 (final stats) = 5
        assert_eq!(summary.total_requests, 5);
        assert_eq!(summary.successful_renders, 2);
        assert_eq!(summary.failed_requests, 1);
        assert_eq!(summary.visuals_breakdown.pie, 1);
        assert_eq!(summary.visuals_breakdown.bar, 1);
        assert!(summary.total_bytes_rendered > 0);
    }
}
