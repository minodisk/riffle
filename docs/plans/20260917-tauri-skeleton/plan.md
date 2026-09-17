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

# Phase 2: Tauri app skeleton (folder picker, ARW enumeration, preview on canvas, arrow-key paging)

## Purpose

Phase 1 proved on the CLI that the 1616x1080 preview embedded in a Sony ARW can
be located and decoded in a few milliseconds. Phase 2 turns that into the first
runnable desktop app: pick a folder, enumerate its ARW files, show one preview
on a `<canvas>`, and page with the left/right arrow keys. Speed is explicitly
not a goal here; the point is a working end-to-end skeleton (Rust command →
IPC → worker → canvas) that Phase 4 (prefetch / ring buffer / cache) and
Phase 5 (1:1 focus check via `partial.rs`) can build on without re-plumbing.

Scope guard: no prefetching, no ring buffer, no on-disk cache, no thumbnail
grid, no focus box, no rating / flagging / XMP, no burst grouping.

## Decisions taken before implementation

These were open questions in planning and are now settled. Do not reopen them
during implementation; if one turns out to be wrong, say so and stop rather
than quietly switching.

1. **The IPC payload is the embedded JPEG bytes, not RGBA pixels.** `preview`
   returns the JPEG slice out of the ARW as-is and the worker decodes it with
   `createImageBitmap`. Rationale: ~400 KB per image instead of ~7 MB, which is
   what makes Phase 4's prefetch of ~41 images affordable (~16 MB vs ~286 MB),
   and it is the most direct reading of "createImageBitmap + worker".
   Consequence: the Orientation rotation is applied on the canvas, and
   `mozjpeg` is unused by the app until Phase 5.
2. **CI moves from ubuntu-latest to macos-latest.** Tauri needs no system
   dependencies on macOS, so the webkit2gtk apt install never has to be added;
   the existing `nasm` step can be **removed** as well, because macOS runners
   are ARM and `mozjpeg-sys` uses `gas` there. This also means CI checks a
   platform the project actually ships. Consequence: the Linux build is not
   checked, which is acceptable because Linux is not a target.

## Steps

- [x] Step 1: Extract the reusable ARW / JPEG code into a library crate `crates/core`
  - Done when:
    - `crates/core` (package `riffle-core`, `lib.rs`) exists in the workspace and
      exports `arw` (moved verbatim from `crates/cli/src/arw.rs`), `partial`
      (moved verbatim from `crates/cli/src/partial.rs`), and a small `decode`
      module holding `decode_rgb` and `apply_orientation` (moved out of
      `crates/cli/src/main.rs`, made `pub`)
    - `crates/cli` depends on `riffle-core` and keeps exactly the same
      subcommands and output; `riffle-cli info/focusbox/crop/bench` behave as
      before on a real ARW
    - `crates/core` has at least one unit test for `arw::parse` that does not
      need a real ARW file (a hand-built minimal little-endian TIFF with IFD0
      carrying tags 0x0201/0x0202/0x0112, plus a rejection test for a
      non-`II` header). `mise run ci` passes
  - Implementation approach:
    - Root `Cargo.toml` `members = ["crates/core", "crates/cli"]`. Move
      `mozjpeg`, `mozjpeg-sys`, `anyhow` to `crates/core`; `image` stays in
      `crates/cli` (PNG output is CLI-only)
    - Do not change behaviour or signatures of `arw.rs` / `partial.rs`; this is
      a pure move so the diff is reviewable. `focus_location` (exiftool
      shell-out) and `draw_rect` stay in the CLI
    - Module docs (`//!`) stay in English as they are

- [x] Step 2: Scaffold the Tauri 2 app crate, wire Node/pnpm into `mise.toml`, and move CI to macOS
  - Done when:
    - `crates/app` (package `riffle-app`, a Tauri 2 binary crate with
      `build.rs`, `tauri.conf.json`, `capabilities/default.json`, `icons/`) is
      a workspace member and depends on `riffle-core`
    - A committed placeholder frontend (`index.html` + one TypeScript file
      compiled by `tsc` only, no bundler) opens in a window via
      `pnpm tauri dev` on macOS and shows the app name; `invoke` reaches a
      trivial Rust command (e.g. `ping`) through `window.__TAURI__.core`
    - `mise.toml`: `node` and `pnpm` added to `[tools]`; the `ci` task runs
      `pnpm install --frozen-lockfile` and `pnpm exec tsc --noEmit -p ...`
      (type-check) alongside the existing cargo / shellcheck / actionlint steps
    - `.github/workflows/ci.yml` runs on `macos-latest`, and the `nasm` install
      step is **removed** (macOS runners are ARM, where `mozjpeg-sys` uses
      `gas`). The pnpm store is cached alongside the existing cargo cache.
      `mise run ci` passes on macos-latest within the existing 30-minute
      timeout (measure the first cold run and record it in `learnings.md`; if
      it is close to the limit, raise the timeout in this step rather than
      later)
    - `README.md` gains a "Running the app" section: prerequisites per OS
      (Xcode CLT on macOS; MSVC Build Tools + WebView2 + `nasm` on Windows for
      `mozjpeg-sys`), `pnpm install`, `pnpm tauri dev`
  - Implementation approach:
    - Tauri CLI: add `@tauri-apps/cli` as a root `package.json` devDependency
      and drive it with `pnpm tauri dev` / `pnpm tauri build` (prebuilt npm
      binary, installed by the same `pnpm install` CI already runs). Do not
      add `cargo:tauri-cli` to `mise.toml`: CI does not need the CLI (it runs
      `cargo clippy` / `cargo test` directly) and compiling it there costs
      minutes for nothing
    - Frontend layout: sources in `crates/app/ui/` (`index.html`, `style.css`,
      `src/*.ts`, `tsconfig.json` with `module: es2022`, `target: es2022`,
      `lib: ["dom", "dom.iterable", "es2022", "webworker"]` split as needed for
      the worker file, `outDir` pointing at the directory `frontendDist` names).
      `frontendDist` must point at a directory that exists in a fresh checkout,
      because `tauri::generate_context!()` reads it at compile time and
      `cargo clippy` in CI would otherwise fail before any JS is emitted. Two
      known-good layouts: (a) `frontendDist: "../ui"` with `tsc` emitting `.js`
      in place and `crates/app/ui/**/*.js` gitignored, or (b)
      `frontendDist: "../dist"` with a committed `dist/index.html` and
      gitignored `dist/*.js`. Pick one and note it in `learnings.md`
    - `tauri.conf.json`: `identifier` in reverse-domain form (not the default,
      which Tauri rejects), `app.withGlobalTauri: true`, no `devUrl`
      (`tsc --watch` as `beforeDevCommand` is enough; no dev server). Commit the
      icons `tauri init` / `tauri icon` generates so Windows builds (which need
      them for the resource file) work
    - TypeScript sees `window.__TAURI__` through a small hand-written
      `tauri.d.ts` (`core.invoke<T>(cmd, args?): Promise<T>`) rather than the
      `@tauri-apps/api` npm package: without a bundler the browser cannot
      resolve a bare `@tauri-apps/api/core` import
    - `cargo clippy --all-targets -- -D warnings` now compiles the Tauri crate;
      the generated `main.rs` from `tauri init` includes
      `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`;
      keep it

- [x] Step 3: Rust commands: pick a folder, enumerate ARW files, hand the embedded preview to the frontend
  - Done when:
    - `pick_folder() -> Option<String>` opens the native directory picker via
      `tauri-plugin-dialog` (Rust side `blocking_pick_folder`) and returns the
      chosen path
    - `list_arw(dir: String) -> Vec<String>` returns the absolute paths of the
      files directly in `dir` whose extension is `arw` case-insensitively,
      sorted by file name; non-recursive; unreadable entries are skipped, an
      unreadable directory is an error string
    - `preview(path: String)` returns, as raw bytes via `tauri::ipc::Response`
      (no base64), a small fixed-size little-endian header followed by **the
      embedded JPEG bytes copied verbatim out of the ARW**. The header is
      documented in code and carries at least a version/kind tag and the
      Orientation value. `riffle_core::arw::parse` does the locating; the
      command is `async` / runs on a blocking thread so the webview never
      stalls on file IO
    - A file with no embedded preview is an error the frontend can show, not a
      panic
    - Unit tests in `crates/app` for the listing logic (extension filter and
      ordering, using `tempfile` or `std::env::temp_dir`), and a test that the
      preview payload header encodes the documented fields. `mise run ci` passes
  - Implementation approach:
    - `capabilities/default.json` gets `dialog:allow-open` (and
      `core:default`). `invoke` for `list_arw` / `preview` needs no extra
      permission because they are the app's own commands
    - Reading the whole ARW (tens of MB) to get the ~400 KB preview is
      acceptable for Phase 2 (speed is out of scope), but it is the obvious
      Phase 4 hot spot: keep the file-reading and the parsing separate so a
      seek-based reader can replace `std::fs::read` later without touching
      `arw::parse`
    - `Response` carries only bytes, so the orientation rides in the header
      rather than in a second `invoke` that would re-read the file. Width and
      height are not needed in the header — the JPEG carries them and
      `createImageBitmap` reports them
    - Errors: commands return `Result<_, String>`; the frontend shows the
      string. No retry logic

- [ ] Step 4: Frontend: folder button, canvas, decode worker, arrow-key paging
  - Done when:
    - Clicking "Open folder" (or pressing `o`) runs `pick_folder` then
      `list_arw`, shows `n / total` and the current file name in a status line,
      and displays the first file's preview
    - `ArrowRight` / `ArrowLeft` move to the next / previous file, clamped at
      the ends. Holding a key down (auto-repeat) does not corrupt the display:
      responses that arrive for a file that is no longer current are dropped
      (sequence counter)
    - Decoding happens in a `Worker` (module worker): the main thread receives
      the `ArrayBuffer` from `invoke`, transfers it to the worker, the worker
      builds the `ImageBitmap` with
      `createImageBitmap(new Blob([jpegBytes], { type: "image/jpeg" }))`, and
      transfers it back. The main thread draws it with `drawImage` scaled to
      fit the canvas (`devicePixelRatio`-aware), applying the Orientation
      rotation (1 / 6 / 8) from the header via canvas transform
    - A folder of 5000 ARW files opens and can be paged through end to end on
      macOS; each page shows the right file (spot-check by name) with the
      right orientation on a portrait (Orientation 8) file. Record the
      observed per-page latency in `learnings.md` as the Phase 4 baseline
    - Errors (no preview in file, unreadable file) show in the status line and
      do not break paging
  - Implementation approach:
    - Files: `crates/app/ui/src/main.ts` (state: `files: string[]`,
      `index`, `seq`; key handling; drawing), `crates/app/ui/src/worker.ts`
      (decode only), `crates/app/ui/index.html`, `crates/app/ui/style.css`.
      No framework, no bundler; `<script type="module">` and
      `new Worker(new URL("./worker.js", import.meta.url), { type: "module" })`
    - `invoke` is only available on the main thread (`window.__TAURI__` is not
      present in workers), which is why the main thread fetches the bytes and
      the worker only decodes
    - The IFD0 preview is a bare JPEG stream without its own EXIF orientation,
      so do not rely on `imageOrientation: "from-image"`; use the header value
    - Keep the canvas sized to the window (`resize` listener re-draws the last
      bitmap); do not add zoom, pan, or any UI beyond the button, canvas and
      status line

- [ ] Step 5: Documentation and status update
  - Done when:
    - `README.md` "Status" says Phase 2 is done and describes the app's current
      capability (folder → preview → arrow keys), links the run instructions
      from Step 2, and notes the Phase 4 baseline latency measured in Step 4
    - `CLAUDE.md` mentions the workspace layout (`crates/core`, `crates/cli`,
      `crates/app`) and that frontend code lives under `crates/app/ui` and is
      type-checked by `mise run ci`
    - `mise run ci` passes

## Trade-offs and risks

### Frontend build step: `tsc` only, no bundler

The user asked for TypeScript, which forces a compile step and therefore Node +
pnpm + `typescript` in `mise.toml` and CI. The plan takes **`tsc` only** (no
Vite or similar): ES modules load fine from Tauri's asset protocol, the code is
two files, and every extra tool is one more thing CI must run. Cost: `import`
specifiers must carry `.js` extensions, and there is no HMR (`tsc --watch` plus
restarting the window is the dev loop).

Open point: whether to add a frontend formatter (Prettier / Biome) to
`mise run fmt` and `mise run ci`. Not required by the acceptance criteria; left
out unless it is asked for.

### CI cost of adding a Tauri crate to the workspace

`cargo clippy --all-targets` and `cargo test` cover the whole workspace, so
`crates/app` is compiled in CI. On macos-latest that needs no system packages,
but the cold `cargo` build of Tauri and wry is still several minutes before the
existing `target` cache (keyed on `Cargo.lock`) warms up. The crate stays in
the default CI run, because a crate CI does not build is a crate CI does not
check. Step 2 measures the cold run and raises `timeout-minutes` if it comes
close to 30.

### Folder picker: Rust-side dialog

`blocking_pick_folder` inside a Rust `pick_folder` command keeps the frontend to
a single `invoke` and avoids the plugin's JS package, which without a bundler
would have to be reached through `window.__TAURI__.dialog`. If blocking dialogs
on the command thread cause trouble, run it as an `async` command on a blocking
task; the JS-side `open({ directory: true })` is the equivalent fallback.

### Reading whole ARW files

Phase 2 reads the entire ARW to parse it, since `arw::parse` takes a full
buffer. On a folder of 5000 files this only affects the current image (no
prefetch yet), so it meets the acceptance criteria, but per-page latency will
be dominated by the read, not the decode. That is the number Step 4 records as
the Phase 4 baseline; changing `arw::parse` to a seek-based reader is
deliberately deferred to Phase 4.

### Orientation handling now lives in two places

Choosing the JPEG-bytes payload means the canvas applies the rotation in
TypeScript, while the CLI keeps `apply_orientation` in Rust. Phase 5 draws the
focus box on a Rust-side partial decode, so it will need the same reasoning
again on that path. This duplication is accepted for the payload size win; if
it starts causing bugs, revisit it when Phase 5 lands rather than now.

## Progress

- (2026-09-17) Step 1 complete
- (2026-09-17) Step 2 complete
- (2026-09-17) Step 3 complete
