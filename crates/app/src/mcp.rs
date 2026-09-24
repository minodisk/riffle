//! The embedded MCP server: a Streamable HTTP endpoint on the loopback
//! interface that lets any MCP client follow and assist a culling session.
//! It is off by default and turned on in the settings window.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{tool, tool_handler, tool_router, ServerHandler};
use serde_json::Value;
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter, Manager};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

/// The fixed loopback port, outside the ranges development servers commonly
/// take, so the connection examples stay the same from launch to launch.
pub const MCP_PORT: u16 = 41917;

/// How long a tool call waits for the main window to answer.
const BRIDGE_TIMEOUT: Duration = Duration::from_secs(5);

const MAIN_WINDOW: &str = "main";

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

type Reply = Result<Value, String>;

/// A request the main window answers through `mcp_reply`.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Request {
    id: u64,
    kind: &'static str,
    args: Value,
}

/// The request / reply bridge to the main window, which holds the view
/// state and the judgment path the tools read and drive.
pub struct Bridge {
    next: AtomicU64,
    pending: Mutex<HashMap<u64, oneshot::Sender<Reply>>>,
    send: Box<dyn Fn(Request) -> Result<(), String> + Send + Sync>,
}

impl Bridge {
    fn new(send: impl Fn(Request) -> Result<(), String> + Send + Sync + 'static) -> Self {
        Bridge {
            next: AtomicU64::new(0),
            pending: Mutex::new(HashMap::new()),
            send: Box::new(send),
        }
    }

    /// A bridge that emits `mcp-request` to the main window.
    pub fn to_main_window(app: AppHandle) -> Self {
        Bridge::new(move |request| {
            if app.get_webview_window(MAIN_WINDOW).is_none() {
                return Err("Riffle's main window is not open".to_string());
            }
            app.emit_to(MAIN_WINDOW, "mcp-request", request)
                .map_err(|e| e.to_string())
        })
    }

    /// Ask the main window for `kind` and wait for its reply.
    pub async fn call(&self, kind: &'static str, args: Value) -> Reply {
        self.call_within(kind, args, BRIDGE_TIMEOUT).await
    }

    async fn call_within(&self, kind: &'static str, args: Value, timeout: Duration) -> Reply {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        let (sender, receiver) = oneshot::channel();
        self.pending.lock().unwrap().insert(id, sender);
        if let Err(e) = (self.send)(Request { id, kind, args }) {
            self.pending.lock().unwrap().remove(&id);
            return Err(e);
        }
        let reply = tokio::time::timeout(timeout, receiver).await;
        self.pending.lock().unwrap().remove(&id);
        match reply {
            Ok(Ok(reply)) => reply,
            _ => Err(format!(
                "Riffle's main window did not answer within {timeout:?}"
            )),
        }
    }

    /// Complete the call `id`; `value` is the result, or the error message
    /// when `ok` is false. A reply after the timeout is dropped.
    pub fn reply(&self, id: u64, ok: bool, value: Value) {
        let Some(sender) = self.pending.lock().unwrap().remove(&id) else {
            return;
        };
        let reply = if ok {
            Ok(value)
        } else {
            Err(value
                .as_str()
                .map_or_else(|| value.to_string(), str::to_string))
        };
        let _ = sender.send(reply);
    }
}

fn tool_result(reply: Reply) -> CallToolResult {
    match reply {
        Ok(value) => CallToolResult::structured(value),
        Err(e) => CallToolResult::error(vec![ContentBlock::text(e)]),
    }
}

/// The MCP handler each client session gets.
#[derive(Clone)]
pub struct Companion {
    bridge: Arc<Bridge>,
}

#[tool_router]
impl Companion {
    #[tool(
        description = "What Riffle is showing now: the open folder, the number of photos \
        visible after the filter, the current photo and its 1-based position, the selected \
        paths, the view mode (normal, zoom for the 1:1 focus check, or compare), the active \
        compare pane, the sort order, whether a filter is active, and the current photo's \
        burst (empty when it is not part of one) with each frame's sharpness score, stars \
        (null when unrated), pick / reject flag, color label, and whether the filter shows it \
        (a frame it hides cannot be shown or selected)."
    )]
    async fn get_view(&self) -> CallToolResult {
        tool_result(
            self.bridge
                .call("get_view", Value::Object(Default::default()))
                .await,
        )
    }
}

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
/// bind error (a tokio mutex, since a switch holds it across the bind), and
/// the bridge every session's tools share.
pub struct AppMcp {
    status: tokio::sync::Mutex<Status>,
    bridge: Arc<Bridge>,
}

impl AppMcp {
    pub fn new(bridge: Bridge) -> Self {
        AppMcp {
            status: Default::default(),
            bridge: Arc::new(bridge),
        }
    }

    /// Complete a bridge call with the main window's answer.
    pub fn reply(&self, id: u64, ok: bool, value: Value) {
        self.bridge.reply(id, ok, value);
    }
}

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
fn router(token: &CancellationToken, bridge: Arc<Bridge>) -> axum::Router {
    let service: StreamableHttpService<Companion, LocalSessionManager> = StreamableHttpService::new(
        move || {
            Ok(Companion {
                bridge: bridge.clone(),
            })
        },
        Default::default(),
        StreamableHttpServerConfig::default()
            .with_cancellation_token(token.child_token())
            .enforce_origin_validation(),
    );
    axum::Router::new().nest_service("/mcp", service)
}

/// Serve `/mcp` on `listener` until the returned token is cancelled.
fn serve(listener: TcpListener, bridge: Arc<Bridge>) -> Result<Running, String> {
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let token = CancellationToken::new();
    let app = router(&token, bridge);
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

async fn bind(port: u16, bridge: Arc<Bridge>) -> Result<Running, String> {
    let listener = TcpListener::bind(("127.0.0.1", port))
        .await
        .map_err(|e| format!("could not listen on 127.0.0.1:{port}: {e}"))?;
    serve(listener, bridge)
}

async fn shut(running: Running) {
    running.token.cancel();
    let mut task = running.task;
    if tokio::time::timeout(STOP_TIMEOUT, &mut task).await.is_err() {
        task.abort();
    }
}

/// Start the server on `MCP_PORT` unless it is running, and return its port.
async fn start(status: &mut Status, bridge: Arc<Bridge>) -> Result<u16, String> {
    if let Some(running) = &status.running {
        return Ok(running.port);
    }
    let running = bind(MCP_PORT, bridge).await?;
    let port = running.port;
    status.running = Some(running);
    Ok(port)
}

/// Stop the server if it is running, waiting briefly for open connections.
pub async fn stop(mcp: &AppMcp) {
    let running = mcp.status.lock().await.running.take();
    if let Some(running) = running {
        shut(running).await;
    }
}

/// The current state, as the settings window shows it.
pub async fn state(app: &AppHandle) -> McpState {
    app.state::<AppMcp>().inner().status.lock().await.state()
}

/// Turn the server on or off, remember a bind error, and emit `mcp-state`.
/// A failed bind is logged, not returned: the setting stays on, so the next
/// launch tries again.
pub async fn switch(app: &AppHandle, enabled: bool) -> McpState {
    let state = {
        let mcp = app.state::<AppMcp>();
        let mut status = mcp.inner().status.lock().await;
        status.enabled = enabled;
        status.error = None;
        if enabled {
            if let Err(e) = start(&mut status, mcp.bridge.clone()).await {
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

    fn silent() -> Arc<Bridge> {
        Arc::new(Bridge::new(|_| Ok(())))
    }

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
            let response = router(&token, silent())
                .oneshot(initialize(None))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            token.cancel();
        });
    }

    #[test]
    fn a_request_with_an_origin_is_refused() {
        tauri::async_runtime::block_on(async {
            let token = CancellationToken::new();
            let response = router(&token, silent())
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
            let mcp = AppMcp::new(Bridge::new(|_| Ok(())));
            mcp.status.lock().await.running = Some(serve(listener, silent()).unwrap());
            assert!(tokio::net::TcpStream::connect(("127.0.0.1", port))
                .await
                .is_ok());
            assert!(bind(port, silent()).await.is_err());

            stop(&mcp).await;
            assert!(mcp.status.lock().await.running.is_none());
            assert!(tokio::net::TcpStream::connect(("127.0.0.1", port))
                .await
                .is_err());
            let rebound = bind(port, silent()).await.unwrap();
            shut(rebound).await;
        });
    }

    #[test]
    fn start_returns_the_running_port_without_binding_again() {
        tauri::async_runtime::block_on(async {
            let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
            let port = listener.local_addr().unwrap().port();
            let mut status = Status {
                running: Some(serve(listener, silent()).unwrap()),
                ..Status::default()
            };

            let started = start(&mut status, silent()).await.unwrap();
            assert_eq!(started, port);

            let running = status.running.take().unwrap();
            shut(running).await;
        });
    }

    #[test]
    fn get_info_names_riffle_and_enables_tools() {
        let info = Companion { bridge: silent() }.get_info();
        assert_eq!(info.server_info.name, "riffle");
        assert!(info.capabilities.tools.is_some());
        assert!(info.instructions.is_some());
    }

    #[test]
    fn the_tools_are_get_view() {
        let names: Vec<_> = Companion::tool_router()
            .list_all()
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect();
        assert_eq!(names, ["get_view"]);
    }

    #[test]
    fn a_call_resolves_with_the_reply() {
        tauri::async_runtime::block_on(async {
            let sent = Arc::new(Mutex::new(Vec::new()));
            let log = sent.clone();
            let bridge = Arc::new(Bridge::new(move |request| {
                log.lock().unwrap().push(request);
                Ok(())
            }));
            let replier = bridge.clone();
            let requests = sent.clone();
            let answer = tauri::async_runtime::spawn(async move {
                loop {
                    let id = requests
                        .lock()
                        .unwrap()
                        .first()
                        .map(|r: &super::Request| r.id);
                    if let Some(id) = id {
                        replier.reply(id, true, serde_json::json!({ "count": 3 }));
                        return;
                    }
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            });

            let reply = bridge.call("get_view", Value::Null).await;
            answer.await.unwrap();
            assert_eq!(reply, Ok(serde_json::json!({ "count": 3 })));
            assert_eq!(sent.lock().unwrap()[0].kind, "get_view");
            assert!(bridge.pending.lock().unwrap().is_empty());
        });
    }

    #[test]
    fn an_error_reply_carries_its_message() {
        tauri::async_runtime::block_on(async {
            let bridge = Arc::new(Bridge::new(|_| Ok(())));
            let replier = bridge.clone();
            tauri::async_runtime::spawn(async move {
                while replier.pending.lock().unwrap().is_empty() {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
                replier.reply(0, false, Value::String("no folder".into()));
            });
            assert_eq!(
                bridge.call("get_view", Value::Null).await,
                Err("no folder".to_string())
            );
        });
    }

    #[test]
    fn a_call_times_out_when_nothing_replies() {
        tauri::async_runtime::block_on(async {
            let bridge = Bridge::new(|_| Ok(()));
            let reply = bridge
                .call_within("get_view", Value::Null, Duration::from_millis(20))
                .await;
            assert_eq!(
                reply,
                Err("Riffle's main window did not answer within 20ms".to_string())
            );
            assert!(bridge.pending.lock().unwrap().is_empty());
            bridge.reply(0, true, Value::Null);
        });
    }

    #[test]
    fn a_call_fails_at_once_when_the_request_cannot_be_sent() {
        tauri::async_runtime::block_on(async {
            let bridge = Bridge::new(|_| Err("Riffle's main window is not open".to_string()));
            assert_eq!(
                bridge.call("get_view", Value::Null).await,
                Err("Riffle's main window is not open".to_string())
            );
            assert!(bridge.pending.lock().unwrap().is_empty());
        });
    }
}
