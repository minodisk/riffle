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

## Step 2: frontend bridge and `get_view`

- The bridge is a `Bridge` struct (pending `oneshot` senders keyed by a
  counter, plus a boxed `send` closure) rather than a free
  `bridge::call(app, ..)`. Production builds it with
  `Bridge::to_main_window(app)`, which emits `mcp-request` with
  `emit_to("main", ..)`; the tests build it with a closure, so neither the
  bridge nor the router needs a Tauri `AppHandle` (and no `tauri` `test`
  feature / `MockRuntime` generics were needed). `Companion` holds an
  `Arc<Bridge>` instead of the `AppHandle` step 1's note expected; step 3's
  Rust-side reads will need the `AppHandle` too, so add it to `Companion`
  then.
- The bridge lives in `AppMcp` next to (not inside) the tokio status mutex,
  so `mcp_reply` never waits on a `switch` that holds that mutex across a
  bind or the 2 s shutdown wait. `mcp_reply` is a sync command: it only
  locks a std mutex briefly.
- `CallToolResult::structured(value)` in rmcp 3.4.1 already adds a text
  block with the same JSON, so no extra text block is built by hand.
- A tool returning `CallToolResult` directly works with `#[tool]`; a bridge
  failure becomes `CallToolResult::error` (a tool error the client sees),
  not a protocol error.
- The timeout message formats the `Duration` with `{:?}` ("5s"), so the
  test can use a 20 ms timeout and still check the exact text.
- `main.ts` exposes the view to `companion.ts` as an object of getters over
  its module-level `let`s, so the answer always reads the live state. The
  filter toggle's "is any filter on" expression, duplicated twice, became
  `filterActive()` so `get_view` can report it too.
- Burst positions and `current.position` are 1-based, as the strip badge and
  the position line show them.
- Not verified by hand: `claude mcp list` showing `get_view` and calling it
  against a running app (no GUI session in this environment). The tool list
  is covered by a unit test on `Companion::tool_router()`.

## Step 3: `get_photo` and `get_preview`

- `Companion` got the index reader (`Option<Arc<Mutex<Index>>>`, the same
  `Arc` `AppIndexReader` holds) instead of the `AppHandle` step 2's note
  expected: the two reads need nothing else from the app, and the handler
  stays constructible in tests without a Tauri runtime. `AppMcp` now holds a
  whole `Companion` that `router` clones per session.
- "Outside the open folder" is decided on the canonicalized parent directory
  against the canonicalized `folder` from a `get_view` bridge call (the
  frontend's `openDir` is the folder as picked, not canonicalized, while the
  paths come from `list_arw` under the canonical folder). The resolved path
  (canonical parent + file name) is exactly the index key. A photo the
  filter hides is still readable. Because the folder comes from the bridge,
  both tools need the main window, like every other tool.
- `Index::entry(path)` shares its `SELECT` and row mapping with `entries`
  (`INDEXED_FILE` / `indexed_file` in `index.rs`) instead of duplicating the
  30-column mapping.
- `preview_jpeg` returns `(jpeg, width, height)` rather than the plan's bare
  `Vec<u8>`, following `decode_rgb`'s tuple, so the tool's text block can
  report the size without decoding the output again. Like `decode_rgb` it
  wraps mozjpeg in `catch_unwind`, since the preview bytes come from files.
  The scale is the largest `n` in 1..=8 with `ceil(native * n / 8) <=
  long_edge` (libjpeg rounds the scaled size up): 1616 at 1024 gives 5/8,
  1010x675.
- Tool parameters use `Parameters<T>` with `#[derive(JsonSchema)]` and
  `#[schemars(crate = "rmcp::schemars")]`, so no direct `schemars`
  dependency; `base64 = "0.22"` (already in the lockfile) encodes the image.
  An `Option<u32>` field shows as `minimum: 0` in the schema; the clamp to
  256..=1616 is stated in the field's description.
- Not verified by hand: calling `get_photo` / `get_preview` from an MCP
  client against a running app, and the client showing the image (no GUI
  session nor sample ARW / DNG in this environment). Covered by unit tests
  on the path resolution, the index lookup, the clamp, the tool list and
  `preview_jpeg`.

## Step 4: `show_photo`, `select_photos`, `set_view`

- `ViewApi` gained three actions (`showPhoto`, `selectPhotos`, `setMode`);
  `companion.ts` validates the arguments (a path must be in `files`, i.e.
  visible after the filter; `paths` is deduplicated and must be non-empty;
  `mode` must be one of the three) before calling any of them, and each
  tool answers with the `get_view` payload read after the action. The Rust
  side only types the arguments (`ViewMode` is a lowercase serde enum, so the
  schema lists the three modes) and forwards them over the bridge; the
  frontend repeats the checks because the bridge carries plain JSON.
- `main.ts` reuses `single`, `paintSelection`, `show`, `toggleZoom` and
  `toggleCompare`. The strip click's "focus moved, or only the selection
  changed" branch became `focusFile(path)`, shared by the two selection
  actions. The only new selection builder is `selectionOf(paths)` in
  `selection.ts` (the set, anchored on the first path), since no existing
  helper builds a selection from an arbitrary list.
- `set_view compare` reports failure by reading `comparing` after the
  toggle: `toggleCompare` only sets the status line when there are fewer
  than two candidates. Its message moved to `COMPARE_NEEDS_FRAMES` in
  `compare.ts` so the UI and the tool error say the same thing. `zoom` or
  `compare` with no photo shown is refused up front (`toggleZoom` silently
  does nothing then).
- Not verified by hand: driving a running app from an MCP client (no GUI
  session in this environment).

## Step 5: `set_judgment`

- `judge()`'s tail (building the changes with `judgments`, the undo entry,
  `commit()`) became `record(paths, focused, command, forceLabel, anchor)` in
  `main.ts`; `judge()` calls it with the same arguments it used inline, so a
  key press is unchanged. `ViewApi.judge(paths, command)` calls it with the
  shown file as the anchor (so the view does not jump to a judged file) and
  the shown file as the focused one when it is among `paths`, else the first
  path. The command `companion.ts` builds sets fixed values, so the focused
  file only orders the batch. `labelKnown` comes out of `send()` unchanged,
  so the sidecar bytes match a key press; auto-advance lives in
  `runAction`, which the bridge never goes through.
- An omitted `label` (keep) and a `null` one (clear) must differ, so the
  Rust argument is `Option<Option<String>>` with a small
  `deserialize_with = "present"` helper (no `serde_with` dependency).
- `rating: 0` clears the stars (the map holds `null`), as `set_rating`
  treats 0. A call with none of rating / flag / label is refused rather than
  a silent no-op.
- The seven label names are repeated in `mcp.rs` and `companion.ts` (and
  already in `context.ts`); not shared, since each is one line.
- Not verified by hand: that a `set_judgment` from an MCP client writes the
  same XMP / `.dop` bytes as a key press and that `Cmd+Z` undoes it (no GUI
  session in this environment). The code path is shared by construction.

## Step 6: documentation

- The settings live in the `MCP` tab of the settings modal in
  `crates/app/ui/index.html` (the plan's `settings.html` no longer exists
  after the settings-modal plan); the docs name the tab and its
  `Let MCP clients connect` checkbox.
- The docs keep the `npx -y mcp-remote` form for Claude Desktop and say a
  direct `url` entry is unverified, matching `crates/app/ui/src/mcp.ts`; the
  existing deferred item covers switching it once checked.
- README.ja.md links its own subsection as `#mcp-コンパニオン` (GitHub's slug
  keeps the katakana); lychee's `--include-fragments` accepts it.

## Deferred issues (todo candidates)

- Verify by hand whether Claude Desktop accepts a direct `url` entry for a
  local Streamable HTTP server and, if so, replace the `mcp-remote` example.
  Basis: plan's "Claude Desktop's connection form is unverified" trade-off;
  could not be checked in step 1's environment. Files:
  `crates/app/ui/src/mcp.ts`, `crates/app/ui/src/mcp.test.ts`.
- Verify the settings window's Copy buttons in the Linux (WebKitGTK) and
  Windows / macOS webviews. Basis: step 1 implementation approach asks for it;
  no running app was available. Files: `crates/app/ui/src/settings.ts`.
- Verify by hand that `get_view` from an MCP client (e.g. Claude Code)
  returns the open folder's state in a running app. Basis: step 2's
  acceptance criterion; no GUI session was available. Files:
  `crates/app/src/mcp.rs`, `crates/app/ui/src/main.ts`,
  `crates/app/ui/src/companion.ts`.
- Verify by hand that `get_photo` returns the index row and shooting
  settings, and that `get_preview` shows the image in an MCP client (e.g.
  Claude Code), against a running app with a real ARW / DNG folder. Basis:
  step 3's acceptance criterion; no GUI session or sample file was
  available. Files: `crates/app/src/mcp.rs`, `crates/core/src/decode.rs`.
- Verify by hand that `show_photo`, `select_photos` and `set_view` move the
  strip, the selection and the view mode of a running app from an MCP
  client. Basis: step 4 could not run the app (no GUI session). Files:
  `crates/app/src/mcp.rs`, `crates/app/ui/src/main.ts`,
  `crates/app/ui/src/companion.ts`.
- Verify by hand that `set_judgment` from an MCP client writes the XMP /
  `.dop` with the same bytes a key press produces, and that `Cmd+Z` undoes
  it. Basis: step 5's manual check for the PR; no GUI session was available.
  Files: `crates/app/src/mcp.rs`, `crates/app/ui/src/main.ts`,
  `crates/app/ui/src/companion.ts`.
