<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# MCP culling companion

## Purpose

Let any MCP client (Claude Code, Claude Desktop, or any other client that
speaks MCP) sit beside the user while they cull: read what Riffle is showing
(the current photo, the selection, the burst, the sharpness score, the AF
point, the shooting settings, the current stars / flag / label), look at a
small copy of the preview, move the UI (show a photo, change the selection,
open compare or the 1:1 view) and record stars, picks / rejects and labels
through exactly the path a key press takes, so the strip, undo and the sidecar
writer stay consistent. Nothing can trash or delete a file.

The server is an MCP Streamable HTTP endpoint embedded in the running app
(`rmcp` 3.4.x, `axum`, loopback only), turned on in the settings window and
off by default. Nothing in the app, the server's `instructions`, the tool
descriptions or the docs assumes a particular client; Claude Code and Claude
Desktop appear only as connection examples.

## Design (decided here; details in `design.md` once implementation starts)

- **Where the state lives**: all view state (current file, selection, bursts,
  compare / zoom mode, undo history) and the only consistent write path
  (`judge()` → `commit()` → `set_rating` in `crates/app/ui/src/main.ts`) are
  in the frontend. The Rust MCP server therefore does **not** mirror state;
  it asks the main window over a request / reply bridge: Rust emits an
  `mcp-request` event `{ id, kind, args }` to the `main` window, the frontend
  handles it and answers with an `invoke("mcp_reply", { id, ok, value })`;
  Rust keeps a `Mutex<HashMap<u64, oneshot::Sender<..>>>` and times out
  (5 s) when the window is gone. Reason over a mirrored state: one source of
  truth, no double bookkeeping of the ~15 maps in `main.ts`, and writes reuse
  `commit()` (undo entry, strip, `set_rating`, coalescing writer) unchanged.
  Cost: every tool call is one event round trip and the main window must be
  open; both are acceptable for a companion.
- **Rust-side reads** that need no view state (`get_photo`, `get_preview`)
  read the index reader (`AppIndexReader`) and `riffle_core::reader`, as the
  existing `folder_entries` / `metadata` / `preview` commands do.
- **Server lifecycle**: `crates/app/src/mcp.rs` owns an `AppMcp` state
  (`Mutex<Option<Running>>` with the `CancellationToken` and the bound port),
  started in `setup` when the `mcpEnabled` setting is true and by the
  settings toggle, stopped by the toggle and on exit. The server runs on
  Tauri's tokio runtime (`tauri::async_runtime::spawn`); `tokio` gets an
  explicit dependency with the `net` feature.
- **Security**: bind `127.0.0.1` on the fixed port `MCP_PORT` (constant, see
  trade-offs); `StreamableHttpServerConfig` keeps its loopback
  `allowed_hosts` default and calls `enforce_origin_validation()` so any
  request carrying an `Origin` header (a web page) is rejected. No bearer
  token in this plan (see trade-offs).
- **Tools** (names fixed here so the docs and tests can refer to them):
  `get_view`, `get_photo`, `get_preview`, `show_photo`, `select_photos`,
  `set_view`, `set_judgment`. No tool touches `trash.rs`.

## Steps

- [x] Step 1: Embedded MCP server with an on/off setting and the connection details
  - Done when:
    - `crates/app/Cargo.toml` adds `rmcp = { version = "3", features = ["server", "macros", "transport-streamable-http-server"] }`, `axum = "0.8"`, `tokio = { version = "1", features = ["net", "sync"] }` and `tokio-util` (already in the lockfile); the exact features are verified against `rmcp` 3.4.1's `Cargo.toml` at implementation time.
    - New `crates/app/src/mcp.rs`: `Companion` handler (`#[tool_router]` impl with an empty router, `ServerHandler::get_info` with `ServerCapabilities::builder().enable_tools()` and a client-neutral `instructions` string that tells the connected assistant what the tools are for), `AppMcp` state, `start(app) -> Result<u16, String>` (binds `127.0.0.1:MCP_PORT`, mounts `StreamableHttpService` at `/mcp`, spawns `axum::serve(..).with_graceful_shutdown(token)`), `stop(&AppMcp)` (cancels the token, waits for the task), and `pub const MCP_PORT: u16` (a fixed port outside common ranges: `41917`).
    - `mcpEnabled` (bool, default false) is read in `commands::load_settings` like `autoAdvance`; new commands `mcp_enabled` / `set_mcp_enabled(enabled)` persist it under `mcpEnabled` and start / stop the server. `set_mcp_enabled` is `async` and does the bind off the main thread. The backend emits `mcp-state` with `{ enabled, port, error }` so the settings window shows "Listening on http://127.0.0.1:PORT/mcp" or the bind error. `main.rs` starts the server in `setup` when the setting is on and stops it in `RunEvent::ExitRequested` (before the writer flush is fine, order does not matter).
    - Settings window: a new `MCP` tab in `crates/app/ui/settings.html` with the checkbox, the status line, and, first, the endpoint URL `http://127.0.0.1:PORT/mcp` with a "Copy" button, which is all any MCP client needs. Below it, read-only `<pre>` connection examples with a "Copy" button each: Claude Code (`claude mcp add --transport http riffle http://127.0.0.1:PORT/mcp`) and Claude Desktop (the `claude_desktop_config.json` entry, see the Desktop trade-off for its exact form). The texts are built by a pure function in a new `crates/app/ui/src/mcp.ts` (tested), `settings.ts` wires it like the auto-advance checkbox.
    - Acceptance: with the setting on, `claude mcp add --transport http riffle http://127.0.0.1:PORT/mcp` then `claude mcp list` shows `riffle` connected with zero tools; with it off, `curl http://127.0.0.1:PORT/mcp` gets a connection refused; a request with `Origin: http://evil.example` is rejected (4xx). Whether current Claude Desktop connects with a `url` entry directly or needs `mcp-remote` is checked by hand and recorded in `learnings.md`; the Desktop example shown matches the result.
    - Tests: Rust unit tests in `mcp.rs` that `start` binds and `stop` frees the port, and that a request carrying an `Origin` header is refused (use `axum::Router` + `tower::ServiceExt::oneshot` or a real loopback listener in the test); a `mcp.test.ts` for the text builder. `mise run ci` passes.
  - Implementation approach:
    - Follow `rust-sdk/examples/servers/src/counter_streamhttp.rs`: `StreamableHttpService::new(|| Ok(Companion::new(app.clone())), LocalSessionManager::default().into(), StreamableHttpServerConfig::default().with_cancellation_token(ct.child_token()).enforce_origin_validation())`.
    - `axum` needs `tokio`'s `net`; confirm `tauri::async_runtime::spawn` can drive `axum::serve` (it is a tokio multi-thread runtime). If it cannot, spawn a dedicated `tokio::runtime::Runtime` on a thread like `sidecar::Writer::spawn` and record it in `learnings.md`.
    - A failed bind (port in use) must not stop the app from launching: log and emit `mcp-state` with `error`, as the index cache open failure is handled in `setup`.
    - Settings tab, checkbox and status follow the existing `auto-advance` pattern (invoke on load, `listen` on the event, `status.textContent` on error). Copy uses `navigator.clipboard.writeText`; verify it works in the Tauri webview on Linux (WebKitGTK) and fall back to selecting the text if not.
    - Commit as `feat(app): embed an MCP server behind a setting`.

- [x] Step 2: Frontend bridge and `get_view`
  - Done when:
    - `crates/app/src/mcp.rs` gets `bridge::call(app, kind, args) -> Result<Value, String>`: mints an id, stores a `tokio::sync::oneshot::Sender`, emits `mcp-request` to the `main` window (`app.get_webview_window("main")`, error "Riffle's main window is not open" when missing), awaits with a 5 s timeout. New `#[tauri::command] mcp_reply(app, id: u64, ok: bool, value: Value)` completes it.
    - New `crates/app/ui/src/companion.ts`: `handleRequest(kind, args, view: ViewApi): Promise<unknown>` is pure over a small `ViewApi` interface (getters for `files`, `index`, `selection`, `bursts`, `sharpness`, `ratings`, `flags`, `labels`, `entries`, mode flags, and the actions added in later steps) so it is unit-tested without the DOM; `main.ts` implements `ViewApi` over its existing module-level state and registers the `mcp-request` listener next to the other `listen` calls (~line 1827).
    - Tool `get_view` (no parameters) returns structured JSON: `folder`, `count` (visible files after filter), `current` `{ path, position }`, `selected: [paths]`, `mode: "normal" | "zoom" | "compare"`, `compare_active` (path or null), and `burst: [{ path, position, sharpness, rating, flag, label, visible }]` for the current file's burst (empty when it is alone; `visible` is false for a frame the filter hides), plus `sort` and whether a filter is active. Values come from the frontend maps; `rating` is `null` for unrated, `flag` is `"none" | "pick" | "reject"`, `label` the raw name or `null`.
    - Tests: `companion.test.ts` covers `get_view` over a fake `ViewApi` (no folder open, single file, burst, compare mode); Rust test that `bridge::call` times out with the expected error when nothing replies and resolves when `mcp_reply` is called.
    - `claude mcp list` shows the `get_view` tool and calling it from Claude Code returns the state of the open folder.
  - Implementation approach:
    - Tool results use `CallToolResult::structured(value)` plus a text block with the same JSON, since some clients only show text.
    - The frontend listener must catch every error and reply `ok: false` so Rust never waits for the timeout on a handled request.
    - `tauri.d.ts` has no `emitTo` typing; the frontend only needs `listen` and `invoke`, both already declared.
    - Commit as `feat(app): answer MCP get_view from the main window`.

- [x] Step 3: `get_photo` and `get_preview` (Rust-side reads)
  - Done when:
    - `get_photo(path?: string)` (default: the current path from `get_view`'s bridge) returns the index row for the path (`rating`, `flag`, `label`, `sharpness`, `focus` (AF point, frame, manual_focus), `orientation`, `capture_time`) merged with `commands::read_metadata` (camera, lens, aperture, shutter, ISO, focal length, exposure bias, focus distance, captured_at). Unknown path or a path outside the open folder is a tool error, not a panic. The row lookup adds `Index::entry(path)` (single-row query) to `crates/app/src/index.rs` rather than loading `entries(dir)`.
    - `get_preview(path?: string, long_edge?: u32)` returns one MCP image block (`ContentBlock::image(base64_jpeg, "image/jpeg")`), the embedded preview decoded with mozjpeg's DCT scaling to the largest `n/8` scale whose long edge is `<= long_edge` (default 1024, clamped to 256..=1616), rotated upright with `riffle_core::faces::upright_rgb` (handles the orientation-3 half turn that `apply_orientation` leaves alone), re-encoded at quality ~75. A text block with `{ path, width, height }` accompanies it.
    - New `riffle_core::decode::preview_jpeg(preview: &[u8], orientation: u16, long_edge: usize, quality: f32) -> Result<Vec<u8>>` in `crates/core/src/decode.rs`, next to `thumbnail_jpeg`, with a unit test on a synthetic JPEG (as `thumbnail_jpeg`'s test does) checking the output dimensions and orientation.
    - Rust tests for the tool argument validation (`long_edge` clamp, bad path); the tools appear in `claude mcp list`, and Claude Code shows the image.
  - Implementation approach:
    - Blocking IO and decode go through `tauri::async_runtime::spawn_blocking` as `preview` does (`docs/agents/tauri-app.md`).
    - Faces are not stored in the index (`scan.rs` detects them only to score sharpness), so no face data is returned; see trade-offs.
    - Commit as `feat(app): add MCP get_photo and get_preview`.

- [x] Step 4: UI-driving tools `show_photo`, `select_photos`, `set_view`
  - Done when:
    - `show_photo(path)`: makes the path current (like a strip click: collapses the selection to it and calls `show()`); a path hidden by the filter or not in the folder is a tool error.
    - `select_photos(paths: string[])`: sets the selection to the listed visible paths (1..=4 for compare's sake is not enforced; `comparisonCandidates` already slices to 4), makes the first one current, repaints the strip.
    - `set_view(mode: "normal" | "zoom" | "compare")`: toggles `zoomed` / `comparing` through `toggleZoom()` / `toggleCompare()` to reach the requested mode; `compare` with fewer than 2 candidates returns the same message the UI shows. Each returns the new `get_view` payload.
    - `main.ts` exposes these three actions on the `ViewApi`; `companion.ts` validates the arguments and calls them; tests cover the validation and that the fake `ViewApi` receives the right calls.
  - Implementation approach:
    - Reuse existing functions only (`show`, `single`, `paintSelection`, `toggleZoom`, `toggleCompare`); do not add a second way to change the selection. `selection` is built with `single()` / the `Selection` helpers in `selection.ts`.
    - Commit as `feat(app): drive the view from MCP tools`.

- [ ] Step 5: `set_judgment` through the UI's commit path
  - Done when:
    - `set_judgment(paths?: string[], rating?: 0..5, flag?: "none"|"pick"|"reject", label?: string|null)`: omitted fields keep each file's value; `paths` defaults to the current selection (or the compare-active file in compare mode, as `judge()` does). The frontend builds `Change`s from the current maps, pushes one undo entry, and calls `commit()` exactly as `judge()` does (so `set_rating` → `sidecar::Writer::set` runs unchanged and auto-advance is not applied). Returns the resulting `{ path, rating, flag, label }` per file.
    - `label` accepts the seven color names the keys use (`Red` ... `Purple`); any other string is a tool error (the sidecar keeps raw names, but the companion only sets the UI's vocabulary).
    - No tool in `mcp.rs` references `trash`; a Rust test asserts the tool list of `Companion` is exactly the seven names above.
    - Tests: `companion.test.ts` checks the built changes (partial update, idempotent no-op, hidden path rejected); Rust test for argument validation. Manual check recorded in the PR: a `set_judgment` from an MCP client writes the XMP / `.dop` with the same bytes a key press produces and `Cmd+Z` undoes it.
  - Implementation approach:
    - Extract the change-building part of `judge()` into a function usable by both key presses and the bridge, or call `judge()` with a synthesized `Command`; pick whichever keeps `judge()`'s behavior byte-identical (see `selection.ts::judgments`).
    - Commit as `feat(app): write judgments from MCP through the UI's commit path`.

- [ ] Step 6: Documentation
  - Done when:
    - `README.md` and `README.ja.md` (same PR) get a short "MCP companion" bullet under Features and a subsection under "Working with other software" with the endpoint URL and the Claude Code / Claude Desktop connection examples, framed as examples of MCP clients rather than the only ones.
    - `docs/usage.md` gets an "MCP companion" section: what each tool does, that it is off by default, loopback only, that trash is never exposed, and the connection examples.
    - `CLAUDE.md`'s Layout paragraph mentions `src/mcp.rs` and `ui/src/companion.ts`.
    - `docs/agents/tauri-app.md` gets the learnings from steps 1–5 (runtime used for axum, bridge timeouts, anything that broke).
    - `lychee` link check in `mise run lint` passes.
  - Implementation approach:
    - Commit as `docs: describe the MCP culling companion`.

## Trade-offs and risks

- **Claude Desktop's connection form is unverified.** The MCP docs show only stdio `command` entries in `claude_desktop_config.json`, while some reports say recent versions accept a `url` entry for a local Streamable HTTP server; Custom Connectors go through Anthropic's side and cannot reach `127.0.0.1`. Step 1 checks it by hand: show the direct `url` entry if it works, else `npx -y mcp-remote http://127.0.0.1:PORT/mcp` (needs Node). A Node-free stdio relay subcommand (`riffle mcp`) stays deferred.
- **Bearer token or not.** Not added: the server is off by default, loopback only, rejects any `Origin`, and a same-user local process could drive the app anyway. A token would also make every connection example longer. If wanted, generate it once into the settings store and show it in the examples; add it in Step 1.
- **Fixed port vs configurable.** A constant keeps the examples stable and the settings UI minimal; a clash surfaces as an error line. Making the port a setting is a small follow-up if a clash is reported.
- **Bridge vs mirrored state.** Chosen: request / reply to the frontend (one source of truth, writes reuse `commit()`). Risk: a tool call while the main window is closed or mid-reload fails after 5 s; acceptable for a companion. Mirrored state would make reads work without the window but would duplicate the frontend's maps and could not reuse undo.
- **Faces.** The detector's output is not persisted; only the score derived from it is. Returning face boxes would need either a schema bump storing them (re-extracts every folder) or running YuNet on demand per call (~hundreds of ms). Left out; `get_photo` returns the AF point / frame, `manual_focus` and the sharpness score instead.
- **Session mode.** `rmcp` 3.4 serves protocol `2026-07-28` statelessly regardless of `legacy_session_mode`; older clients get sessions from `LocalSessionManager`. Verify `claude mcp list` connects with the defaults; if not, try `json_response: true`.
- **Preview size.** Default long edge 1024 (~100–200 KB JPEG). Larger requests are clamped at the preview's native 1616 px; this is the embedded preview, never the RAW.
- **Tauri runtime.** If `axum::serve` cannot run on `tauri::async_runtime` (feature mismatch), a dedicated runtime thread is the fallback; record it in `learnings.md` and `docs/agents/tauri-app.md`.

## Progress

- (2026-09-25) Step 1 complete
- (2026-09-25) Step 2 complete
- (2026-09-25) Step 3 complete
- (2026-09-25) Step 4 complete
