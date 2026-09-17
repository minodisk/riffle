# Learnings

## Step 1: Extract `crates/core`

- The move was purely mechanical: `arw.rs` / `partial.rs` went over with
  `git mv` untouched, and `decode_rgb` / `apply_orientation` were lifted out of
  `crates/cli/src/main.rs` into `crates/core/src/decode.rs` with `pub` added.
  `focus_location` and `draw_rect` stayed in the CLI as planned.
- `mozjpeg` / `mozjpeg-sys` / `anyhow` moved to `crates/core`; `image` and
  `anyhow` stay in `crates/cli`. `anyhow` is needed in both because the CLI's
  own functions still return `Result`.
- `cargo clippy -D warnings` did not complain about missing docs on the newly
  `pub` items, so no extra doc comments were needed beyond the module `//!`.
- Verified `riffle-cli info ~/Downloads/_DSC6978.ARW` still prints
  `orientation: 8`, `preview: Some(Embedded { offset: 204962, length: 337198 })`,
  `full: Some(Embedded { offset: 544768, length: 5761112 })`.
- The hand-built TIFF fixture for the `arw::parse` tests only needs the 8-byte
  header plus IFD0; `next_ifd = 0` keeps the chain walk from running, so no
  SubIFD data has to be faked.
- `mise run fmt` + `mise run ci` passed on the first attempt.

## Step 2: Tauri app crate, Node/pnpm, CI on macOS

- **Frontend layout: option (a).** `frontendDist` points at the committed
  `crates/app/ui/` directory and `tsc` emits `.js` next to the `.ts` sources;
  `crates/app/ui/**/*.js` is gitignored. No `dist/` directory to keep in sync.
- `frontendDist` is resolved **relative to the directory holding
  `tauri.conf.json`**, which here is `crates/app/` (not the conventional
  `src-tauri/`). The plan's `"../ui"` therefore resolved to `crates/ui` and
  `tauri::generate_context!()` failed the build with
  "`frontendDist` ... but this path doesn't exist"; the value is `"ui"`.
- `tsc` refuses `outDir` equal to `rootDir` (`TS18003`: the `outDir` is
  auto-excluded, leaving no inputs). Emitting in place means simply omitting
  both.
- `main.ts` has no `import`/`export`, so TypeScript treats it as a script in
  global scope and `const status` collided with `lib.dom`'s global `status`.
  Renamed to `statusEl`. Once Step 4 adds imports the file becomes a module and
  the hazard goes away.
- `pnpm tauri icon` also writes `android/` and `ios/` icon sets; those were
  deleted since only desktop is a target. The remaining `crates/app/icons` is
  80 KB.
- The pnpm store path differs per OS, so CI asks `pnpm store path --silent` in
  a step and caches that instead of a hard-coded path.
- Timing: a **cold** `cargo clippy --all-targets` over the new Tauri crate took
  about 5 minutes locally (Apple Silicon); the full `mise run ci` with a warm
  target dir is well under a minute. **The first CI run on this PR still needs
  to be checked against the 30-minute `timeout-minutes`** — it is a cold
  macos-latest runner with an empty cargo cache, and that number could not be
  observed locally. Left the timeout at 30 for now.
- Verified the app binary launches cleanly (`cargo run -p riffle-app`, stays up,
  no stderr). The **window contents were not visually confirmed**: this machine
  denies `osascript` assistive access and `screencapture` screen recording, so
  "the window shows the app name and `ping` returns through
  `window.__TAURI__.core`" is unverified beyond the code path compiling and
  type-checking.

## Step 3

- The commands live in `crates/app/src/commands.rs`; `main.rs` only registers
  them and the dialog plugin. File reading (`std::fs::read`) is kept in
  `read_preview`, separate from `riffle_core::arw::parse`, so Phase 4 can swap
  in a seek-based reader without touching the parser.
- The preview payload header is 8 bytes little-endian: kind/version tag (u16,
  `PREVIEW_KIND_JPEG_V1` = 1), Orientation (u16), 4 reserved zero bytes. The
  reserved bytes keep the JPEG 8-byte aligned in the `ArrayBuffer`, which makes
  the Step 4 `subarray` in the worker cheap to reason about.
- `blocking_pick_folder` is fine in a **synchronous** `#[tauri::command]`:
  Tauri already runs sync commands off the main thread. `preview` is `async`
  and hands the read to `tauri::async_runtime::spawn_blocking` instead, so the
  webview never waits on file IO.
- `tempfile` turned out to be unnecessary: `std::env::temp_dir()` plus the
  process id is enough for the listing tests, so no extra dependency was added
  (only `tauri-plugin-dialog` moves `Cargo.lock`).
- `is_file()` in the listing filter matters: a *directory* named `sub.arw` is
  otherwise listed as a file. That case is covered by the test.

## Deferred issues (todo candidates)

- `tmp/` (scratch space used by the PR tooling) is untracked and shows up in
  every `git status`. Consider adding it to `.gitignore`. Basis: noted while
  staging Step 2; deliberately left out of this step's diff. Path:
  `.gitignore`.
- No frontend formatter or linter yet (Prettier / Biome), so `mise run fmt`
  only formats Rust while `crates/app/ui/**` is unchecked. Basis: the plan's
  "Open point" under "Frontend build step: `tsc` only, no bundler". Paths:
  `mise.toml`, `crates/app/ui/`.
