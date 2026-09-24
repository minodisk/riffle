# Learnings: MCP culling companion

## Step 1: embedded MCP server behind a setting

- `rmcp` 3.4.1's features match the plan: `server`, `macros` and
  `transport-streamable-http-server` (which pulls `server-side-http` and the
  session manager). `ServerInfo` is deprecated in 3.4 in favor of
  `ServerConfig` (the same `InitializeResult` type); use `ServerConfig`.
- `#[tool_router]` on an impl with no `#[tool]` fn is a compile error in
  3.4.1; an intentionally empty router needs `#[tool_router(allow_empty)]`.
- `#[tool_handler]` without `router = ...` calls `Self::tool_router()` on
  each request, so the handler needs no `tool_router` field (a field that is
  never read trips `dead_code` under `-D warnings`). `Companion` is a unit
  struct for now; step 2 adds the `AppHandle` it needs for the bridge.
- `axum::serve` runs fine on `tauri::async_runtime` (Tauri's tokio
  multi-thread runtime); no dedicated runtime thread was needed. The unit
  tests drive it with `tauri::async_runtime::block_on`, so no tokio `macros`
  / `rt` dev features were added.
- `tokio` also gets the `time` feature: `stop` bounds its wait on the
  graceful shutdown with a 2 s `tokio::time::timeout` and aborts the task
  after, so a lingering SSE stream cannot hold up the exit.
- The server state is a `tokio::sync::Mutex`, held across the bind, so two
  quick toggles cannot bind twice.
- `mcp_enabled` returns the whole `{ enabled, port, error }` state rather than
  a bool, so the settings window can show the status line (and a bind error
  from launch) on load, not only on the next `mcp-state` event.
- Checked against a live listener on 41917 with curl: `initialize` returns
  `serverInfo.name = "riffle"`, the instructions and `capabilities.tools`;
  `tools/list` returns `[]`; a POST with `Origin: http://evil.example` gets
  `403 Forbidden`; after stop, `curl` exits 7 (connection refused). `claude
  mcp add` / `claude mcp list` itself was not run, to keep the user's Claude
  config untouched; the curl handshake is the same exchange it performs.
- **Unverified: Claude Desktop's connection form.** No Claude Desktop is
  available in this environment, so whether a direct `url` entry in
  `claude_desktop_config.json` works was not checked by hand. The settings
  window shows the `npx -y mcp-remote http://127.0.0.1:41917/mcp` form, which
  only needs Node and a stdio-only Desktop. Confirm by hand before the docs
  step; if a `url` entry works, switch the example in
  `crates/app/ui/src/mcp.ts` (and its test).
- **Unverified: `navigator.clipboard.writeText` in WebKitGTK.** Not checked in
  a running app here. The Copy buttons fall back to selecting the text and
  saying so in the status line when the write is refused.

## Deferred issues (todo candidates)

- Verify by hand whether Claude Desktop accepts a direct `url` entry for a
  local Streamable HTTP server and, if so, replace the `mcp-remote` example.
  Basis: plan's "Claude Desktop's connection form is unverified" trade-off;
  could not be checked in step 1's environment. Files:
  `crates/app/ui/src/mcp.ts`, `crates/app/ui/src/mcp.test.ts`.
- Verify the settings window's Copy buttons in the Linux (WebKitGTK) and
  Windows / macOS webviews. Basis: step 1 implementation approach asks for it;
  no running app was available. Files: `crates/app/ui/src/settings.ts`.
