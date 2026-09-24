//! The embedded MCP server: a Streamable HTTP endpoint on the loopback
//! interface that lets any MCP client follow and assist a culling session.
//! It is off by default and turned on in the settings window.

use std::time::Duration;

use rmcp::model::{Implementation, ServerCapabilities, ServerConfig};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{tool_handler, tool_router, ServerHandler};
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter, Manager};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

/// The fixed loopback port, outside the ranges development servers commonly
/// take, so the connection examples stay the same from launch to launch.
pub const MCP_PORT: u16 = 41917;

/// How long `stop` waits for open connections to close before it drops them.
const STOP_TIMEOUT: Duration = Duration::from_secs(2);

const INSTRUCTIONS: &str = "Riffle is a desktop app the user culls Sony ARW and Leica DNG \
photos with. This server connects you to the running app so you can assist while they cull: \
read what Riffle is showing (the current photo, the selection, its burst, the sharpness \
score, the AF point, the shooting settings and the current stars, pick / reject flag and \
color label), look at a small copy of a photo's preview, move the view, and record stars, \
picks / rejects and color labels the same way the user's keys do, so the app's undo and \
sidecar files stay consistent. Nothing here can move a file to the trash or delete it. \
Suggest judgments and explain them; write them only when the user asks you to.";

/// The MCP handler each client session gets.
#[derive(Clone)]
pub struct Companion;

#[tool_router(allow_empty)]
impl Companion {}

#[tool_handler]
impl ServerHandler for Companion {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("riffle", env!("CARGO_PKG_VERSION")))
            .with_instructions(INSTRUCTIONS)
    }
}

struct Running {
    port: u16,
    token: CancellationToken,
    task: JoinHandle<()>,
}

#[derive(Default)]
struct Status {
    enabled: bool,
    running: Option<Running>,
    error: Option<String>,
}

/// The server's on/off setting, the running server if any, and the last
/// bind error. A tokio mutex, since a switch holds it across the bind.
#[derive(Default)]
pub struct AppMcp(tokio::sync::Mutex<Status>);

/// The payload of the `mcp-state` event and of the `mcp_enabled` command.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct McpState {
    enabled: bool,
    port: u16,
    error: Option<String>,
}

impl Status {
    fn state(&self) -> McpState {
        McpState {
            enabled: self.enabled,
            port: self.running.as_ref().map_or(MCP_PORT, |r| r.port),
            error: self.error.clone(),
        }
    }
}

/// The `/mcp` route. Host validation keeps its loopback-only default, and
/// any request carrying an `Origin` header, which only a web page sends, is
/// refused.
fn router(token: &CancellationToken) -> axum::Router {
    let service: StreamableHttpService<Companion, LocalSessionManager> = StreamableHttpService::new(
        || Ok(Companion),
        Default::default(),
        StreamableHttpServerConfig::default()
            .with_cancellation_token(token.child_token())
            .enforce_origin_validation(),
    );
    axum::Router::new().nest_service("/mcp", service)
}

/// Serve `/mcp` on `listener` until the returned token is cancelled.
fn serve(listener: TcpListener) -> Result<Running, String> {
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let token = CancellationToken::new();
    let app = router(&token);
    let shutdown = token.clone();
    let task = tauri::async_runtime::spawn(async move {
        let served = axum::serve(listener, app)
            .with_graceful_shutdown(async move { shutdown.cancelled_owned().await })
            .await;
        if let Err(e) = served {
            log::error!("the MCP server stopped: {e}");
        }
    });
    Ok(Running { port, token, task })
}

async fn bind(port: u16) -> Result<Running, String> {
    let listener = TcpListener::bind(("127.0.0.1", port))
        .await
        .map_err(|e| format!("could not listen on 127.0.0.1:{port}: {e}"))?;
    serve(listener)
}

async fn shut(running: Running) {
    running.token.cancel();
    let mut task = running.task;
    if tokio::time::timeout(STOP_TIMEOUT, &mut task).await.is_err() {
        task.abort();
    }
}

/// Start the server on `MCP_PORT` unless it is running, and return its port.
async fn start(status: &mut Status) -> Result<u16, String> {
    if let Some(running) = &status.running {
        return Ok(running.port);
    }
    let running = bind(MCP_PORT).await?;
    let port = running.port;
    status.running = Some(running);
    Ok(port)
}

/// Stop the server if it is running, waiting briefly for open connections.
pub async fn stop(mcp: &AppMcp) {
    let running = mcp.0.lock().await.running.take();
    if let Some(running) = running {
        shut(running).await;
    }
}

/// The current state, as the settings window shows it.
pub async fn state(app: &AppHandle) -> McpState {
    app.state::<AppMcp>().inner().0.lock().await.state()
}

/// Turn the server on or off, remember a bind error, and emit `mcp-state`.
/// A failed bind is logged, not returned: the setting stays on, so the next
/// launch tries again.
pub async fn switch(app: &AppHandle, enabled: bool) -> McpState {
    let state = {
        let mut status = app.state::<AppMcp>().inner().0.lock().await;
        status.enabled = enabled;
        status.error = None;
        if enabled {
            if let Err(e) = start(&mut status).await {
                log::error!("failed to start the MCP server: {e}");
                status.error = Some(e);
            }
        } else if let Some(running) = status.running.take() {
            shut(running).await;
        }
        status.state()
    };
    let _ = app.emit("mcp-state", state.clone());
    state
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{header, Request, StatusCode};
    use tower::ServiceExt;

    const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;

    fn initialize(origin: Option<&str>) -> Request<Body> {
        let mut request = Request::post("/mcp")
            .header(header::HOST, format!("127.0.0.1:{MCP_PORT}"))
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ACCEPT, "application/json, text/event-stream");
        if let Some(origin) = origin {
            request = request.header(header::ORIGIN, origin);
        }
        request.body(Body::from(INITIALIZE)).unwrap()
    }

    #[test]
    fn a_request_without_an_origin_initializes() {
        tauri::async_runtime::block_on(async {
            let token = CancellationToken::new();
            let response = router(&token).oneshot(initialize(None)).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            token.cancel();
        });
    }

    #[test]
    fn a_request_with_an_origin_is_refused() {
        tauri::async_runtime::block_on(async {
            let token = CancellationToken::new();
            let response = router(&token)
                .oneshot(initialize(Some("http://evil.example")))
                .await
                .unwrap();
            assert!(response.status().is_client_error(), "{}", response.status());
            token.cancel();
        });
    }

    #[test]
    fn serve_binds_the_port_and_stop_frees_it() {
        tauri::async_runtime::block_on(async {
            let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
            let port = listener.local_addr().unwrap().port();
            let mcp = AppMcp::default();
            mcp.0.lock().await.running = Some(serve(listener).unwrap());
            assert!(tokio::net::TcpStream::connect(("127.0.0.1", port))
                .await
                .is_ok());
            assert!(bind(port).await.is_err());

            stop(&mcp).await;
            assert!(mcp.0.lock().await.running.is_none());
            assert!(tokio::net::TcpStream::connect(("127.0.0.1", port))
                .await
                .is_err());
            let rebound = bind(port).await.unwrap();
            shut(rebound).await;
        });
    }

    #[test]
    fn start_returns_the_running_port_without_binding_again() {
        tauri::async_runtime::block_on(async {
            let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
            let port = listener.local_addr().unwrap().port();
            let mut status = Status {
                running: Some(serve(listener).unwrap()),
                ..Status::default()
            };

            let started = start(&mut status).await.unwrap();
            assert_eq!(started, port);

            let running = status.running.take().unwrap();
            shut(running).await;
        });
    }

    #[test]
    fn get_info_names_riffle_and_enables_tools() {
        let info = Companion.get_info();
        assert_eq!(info.server_info.name, "riffle");
        assert!(info.capabilities.tools.is_some());
        assert!(info.instructions.is_some());
    }
}
