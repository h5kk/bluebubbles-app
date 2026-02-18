//! MCP HTTP/SSE server using hyper 1.x.
//!
//! Listens on `127.0.0.1:<port>` and serves two routes:
//! - `GET /mcp/sse`  -- Server-Sent Events stream
//! - `POST /mcp`     -- JSON-RPC request handler
//!
//! All requests require `Authorization: Bearer <token>`.

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use http_body_util::{Full, StreamBody, BodyExt};
use hyper::body::{Bytes, Frame};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use tokio::net::TcpListener;
use tokio::sync::watch;
use tracing::{info, warn, debug};

use crate::mcp_auth::McpAuth;
use crate::mcp_tools;
use crate::state::AppState;

type BoxBody = http_body_util::combinators::UnsyncBoxBody<Bytes, Infallible>;

/// Shared context passed to every request handler.
struct ServerContext {
    auth: Arc<McpAuth>,
    app_state: Arc<AppState>,
    active_connections: Arc<AtomicU32>,
    _port: u16,
}

/// Start the MCP HTTP server. Returns once the server is bound and listening.
/// The actual serving loop runs in the provided `JoinHandle`.
pub async fn start(
    auth: Arc<McpAuth>,
    app_state: Arc<AppState>,
    port: u16,
    active_connections: Arc<AtomicU32>,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<(), String> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| format!("failed to bind MCP server on {addr}: {e}"))?;

    info!("mcp server listening on http://{addr}");

    let ctx = Arc::new(ServerContext {
        auth,
        app_state,
        active_connections,
        _port: port,
    });

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, remote)) => {
                        let ctx = ctx.clone();
                        debug!("mcp connection from {remote}");
                        tokio::task::spawn(async move {
                            let svc = service_fn(move |req| {
                                let ctx = ctx.clone();
                                async move { handle_request(req, ctx).await }
                            });
                            if let Err(e) = http1::Builder::new()
                                .serve_connection(hyper_util::rt::TokioIo::new(stream), svc)
                                .with_upgrades()
                                .await
                            {
                                debug!("mcp connection error: {e}");
                            }
                        });
                    }
                    Err(e) => {
                        warn!("mcp accept error: {e}");
                    }
                }
            }
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    info!("mcp server shutting down");
                    break;
                }
            }
        }
    }

    Ok(())
}

/// Route an incoming HTTP request.
async fn handle_request(
    req: Request<hyper::body::Incoming>,
    ctx: Arc<ServerContext>,
) -> Result<Response<BoxBody>, Infallible> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    // CORS preflight
    if method == Method::OPTIONS {
        return Ok(cors_response(
            Response::builder()
                .status(StatusCode::NO_CONTENT)
                .body(empty_body())
                .unwrap(),
        ));
    }

    // Auth check (skip for OPTIONS which already returned)
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !ctx.auth.validate(auth_header).await {
        warn!("mcp unauthorized request to {path}");
        return Ok(cors_response(
            Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header("content-type", "application/json")
                .body(full_body(r#"{"error":"unauthorized"}"#))
                .unwrap(),
        ));
    }

    // Route dispatch
    let response = match (method, path.as_str()) {
        (Method::GET, "/mcp/sse") => handle_sse(ctx).await,
        (Method::POST, p) if p == "/mcp" || p.starts_with("/mcp?") => {
            handle_jsonrpc(req, ctx).await
        }
        _ => {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("content-type", "application/json")
                .body(full_body(r#"{"error":"not found"}"#))
                .unwrap()
        }
    };

    Ok(cors_response(response))
}

/// Handle `GET /mcp/sse` -- Server-Sent Events stream.
async fn handle_sse(ctx: Arc<ServerContext>) -> Response<BoxBody> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let connections = ctx.active_connections.clone();

    connections.fetch_add(1, Ordering::Relaxed);

    // Create a channel for SSE events
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Frame<Bytes>, Infallible>>(16);

    // Send the initial endpoint event
    let endpoint_event = format!(
        "event: endpoint\ndata: /mcp?sessionId={session_id}\n\n"
    );
    let _ = tx
        .send(Ok(Frame::data(Bytes::from(endpoint_event))))
        .await;

    // Spawn keepalive pings
    let connections_clone = connections.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            if tx.send(Ok(Frame::data(Bytes::from(": keepalive\n\n")))).await.is_err() {
                // Client disconnected
                connections_clone.fetch_sub(1, Ordering::Relaxed);
                return;
            }
        }
    });

    let body_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    let body = StreamBody::new(body_stream);

    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
        .header("connection", "keep-alive")
        .body(BoxBody::new(body.map_err(|never| match never {})))
        .unwrap()
}

/// Handle `POST /mcp` -- JSON-RPC requests.
async fn handle_jsonrpc(
    req: Request<hyper::body::Incoming>,
    ctx: Arc<ServerContext>,
) -> Response<BoxBody> {
    // Read the body
    let body_bytes = match req.into_body().collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            return jsonrpc_error_response(None, -32700, &format!("failed to read body: {e}"));
        }
    };

    // Parse JSON
    let rpc: serde_json::Value = match serde_json::from_slice(&body_bytes) {
        Ok(v) => v,
        Err(e) => {
            return jsonrpc_error_response(None, -32700, &format!("parse error: {e}"));
        }
    };

    let id = rpc.get("id").cloned();
    let method = rpc
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let params = rpc.get("params").cloned().unwrap_or(serde_json::Value::Null);

    debug!("mcp jsonrpc method={method}");

    match method {
        "initialize" => {
            let result = serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "bluebubbles-mcp",
                    "version": "0.1.0"
                }
            });
            jsonrpc_success_response(id, result)
        }
        "notifications/initialized" => {
            // Client ack, no response needed for notifications
            // But we still return a response since this is HTTP
            Response::builder()
                .status(StatusCode::NO_CONTENT)
                .body(empty_body())
                .unwrap()
        }
        "tools/list" => {
            let tools = mcp_tools::tool_definitions();
            jsonrpc_success_response(id, serde_json::json!({ "tools": tools }))
        }
        "tools/call" => {
            let tool_name = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let tool_args = params
                .get("arguments")
                .cloned()
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

            // Get the API client
            let api = match ctx.app_state.api_client().await {
                Ok(api) => api,
                Err(_) => {
                    let err_result = serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": "BlueBubbles server not connected. Connect in the app first."
                        }],
                        "isError": true
                    });
                    return jsonrpc_success_response(id, err_result);
                }
            };

            match mcp_tools::execute_tool(tool_name, tool_args, &api).await {
                Ok(result) => jsonrpc_success_response(id, result),
                Err(e) => {
                    let err_result = serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Error: {e}")
                        }],
                        "isError": true
                    });
                    jsonrpc_success_response(id, err_result)
                }
            }
        }
        "ping" => {
            jsonrpc_success_response(id, serde_json::json!({}))
        }
        _ => {
            jsonrpc_error_response(id, -32601, &format!("method not found: {method}"))
        }
    }
}

// ─── Response Helpers ────────────────────────────────────────────────────────

fn empty_body() -> BoxBody {
    BoxBody::new(Full::new(Bytes::new()).map_err(|never| match never {}))
}

fn full_body(s: &str) -> BoxBody {
    BoxBody::new(
        Full::new(Bytes::from(s.to_string())).map_err(|never| match never {}),
    )
}

fn cors_response(mut resp: Response<BoxBody>) -> Response<BoxBody> {
    let headers = resp.headers_mut();
    headers.insert("access-control-allow-origin", "*".parse().unwrap());
    headers.insert(
        "access-control-allow-methods",
        "GET, POST, OPTIONS".parse().unwrap(),
    );
    headers.insert(
        "access-control-allow-headers",
        "authorization, content-type".parse().unwrap(),
    );
    resp
}

fn jsonrpc_success_response(
    id: Option<serde_json::Value>,
    result: serde_json::Value,
) -> Response<BoxBody> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(serde_json::Value::Null),
        "result": result
    });
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(full_body(&body.to_string()))
        .unwrap()
}

fn jsonrpc_error_response(
    id: Option<serde_json::Value>,
    code: i64,
    message: &str,
) -> Response<BoxBody> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(serde_json::Value::Null),
        "error": {
            "code": code,
            "message": message
        }
    });
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(full_body(&body.to_string()))
        .unwrap()
}
