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

## Step 4: Frontend (button, canvas, worker, paging)

- **Relative imports: there are none.** `main.ts` and `worker.ts` share no
  module, so the only cross-file reference is the worker URL
  (`new Worker(new URL("./worker.js", import.meta.url), { type: "module" })`),
  which is a runtime string and already carries `.js`. `moduleResolution` was
  therefore left at `"bundler"`: the extensionless-import hazard has no way to
  bite while the frontend stays at two independent files, and switching to
  `node16` would also force `module: "node16"`, whose emit depends on the root
  `package.json` `type` field. **If a shared module is ever added, write the
  import as `./foo.js`** — nothing enforces it today.
- `DedicatedWorkerGlobalScope` is not in the `dom` lib, and adding `webworker`
  next to `dom` clashes on the shared globals. `worker.ts` declares the two
  members it uses (`addEventListener("message", ...)`, `postMessage`) as a
  local `WorkerScope` interface and casts `self` to it, so one tsconfig still
  covers both files.
- The sequence counter is bumped in `show()` and in `openFolder()`, and it is
  checked twice: once when `invoke` resolves (stop before posting to the
  worker) and once when the worker answers (close the stale `ImageBitmap` and
  drop it). The orientation travels in a `Map` keyed by that same sequence,
  because only the JPEG bytes can be transferred to the worker. An `inFlight`
  flag caps `preview` `invoke` calls at one outstanding request: `show()` only
  bumps the sequence, and `requestPreview()` is a no-op while a call is
  already pending. When that call settles, if the sequence moved on in the
  meantime it immediately issues exactly one follow-up for the latest index,
  so holding an arrow key never queues more than one request in flight.
- Drawing: the canvas backing store is `clientWidth/Height * devicePixelRatio`,
  the context is translated to the center and scaled by `dpr`, then rotated by
  ±90° for Orientation 6 / 8. The fit scale is computed against the **upright**
  dimensions (width/height swapped for a quarter turn) while `drawImage` uses
  the bitmap's own dimensions, so a portrait frame fits the window height.

### Phase 4 baseline latency (partial, Rust side only)

Per-page cost is dominated by `std::fs::read` of the whole ~48 MB ARW plus
`arw::parse`, measured with a throwaway binary against `riffle-core` (not
committed) on Apple Silicon:

| folder                              | n   | mean   | p50    | p95    | max    |
|-------------------------------------|-----|--------|--------|--------|--------|
| 5000 symlinks to one ARW (warm page cache) | 300 | 7.3 ms | 7.1 ms | 8.8 ms | 14.0 ms |
| 20 distinct 48 MB copies (first read)      | 20  | 19.3 ms | 19.2 ms | 26.6 ms | 26.6 ms |

How the folders were built: `~/Downloads/_DSC6978.ARW` (the only real ARW on
this machine, portrait, Orientation 8) symlinked 5000 times as
`IMG_0001.ARW`..`IMG_5000.ARW` into a scratch directory outside the repository,
plus 20 real `cp` copies for a read that cannot hit the page cache for the same
inode. Nothing derived from that ARW was committed.

**Caveat: this is not the end-to-end per-page latency.** The IPC hop and
`createImageBitmap` in the worker are not included, because they could not be
measured (see below). Treat ~20 ms as the floor set by reading the file, which
is the number Phase 4's seek-based reader has to beat.

### Verification status

Verified:

- `mise run ci` passes, `tsc --noEmit` passes, and the emitted `main.js` /
  `worker.js` parse as ES modules (`node --input-type=module --check`).
- The app binary launches and stays up with no stderr output.

Not verified:

- **The UI itself is unverified by eye.** `screencapture` now works on this
  machine, but `osascript` UI scripting is still denied assistive access:
  `System Events` returns an empty window list for `riffle-app` and injected
  keystrokes are silently dropped, so the window could not be brought forward
  or driven. Concretely, **the user still has to check by hand**: that the
  "Open folder" button and the `o` key open the picker, that the status line
  shows `n / total` and the file name, that the preview appears, that
  `ArrowRight` / `ArrowLeft` page and clamp at the ends, that holding an arrow
  key down leaves the right file on screen, and that the portrait
  (Orientation 8) file is drawn upright.
- The "5000 ARW files open and page end to end" criterion is therefore
  **not demonstrated**; only the Rust-side cost of 5000 entries was measured.

## Step 5: Documentation and status update

- The README "Status" section now carries three distinct things that were easy
  to conflate: what the app can do, what is **not** verified (the UI was never
  driven on this machine), and the Phase 4 baseline. The baseline table is a
  separate `### Phase 4 baseline` subsection so it cannot be mistaken for the
  existing "Measurements (Apple Silicon Mac, α7 V ARW, n=20)" table, which is
  the Phase 1 CLI decode timing and still valid.
- Both baseline rows are stated with their conditions (symlinks with a warm page
  cache vs. distinct 48 MB copies) and with the explicit note that the IPC hop
  and `createImageBitmap` are excluded.
- The Phase 1 CLI usage block was moved above the "not confirmed at runtime"
  note; otherwise inserting the Phase 2 text left it stranded under the
  Phase 4 baseline heading.
- `CLAUDE.md` gained a short `## Layout` section ahead of `## Language`.

## Deferred issues (todo candidates)

- `tmp/` (scratch space used by the PR tooling) is untracked and shows up in
  every `git status`. Consider adding it to `.gitignore`. Basis: noted while
  staging Step 2; deliberately left out of this step's diff. Path:
  `.gitignore`.
- No frontend formatter or linter yet (Prettier / Biome), so `mise run fmt`
  only formats Rust while `crates/app/ui/**` is unchecked. Basis: the plan's
  "Open point" under "Frontend build step: `tsc` only, no bundler". Paths:
  `mise.toml`, `crates/app/ui/`.
- End-to-end per-page latency (IPC + `createImageBitmap`) is unmeasured, since
  the GUI cannot be driven from this machine. Consider a timing readout in the
  status line, or a Rust-side benchmark that includes the IPC hop, before
  Phase 4 tunes anything. Basis: Step 4's "Phase 4 baseline" acceptance item,
  only partially satisfiable. Paths: `crates/app/ui/src/main.ts`.
- The `ping` command in `crates/app/src/main.rs` is left over from Step 2 and
  now has no caller. Basis: noticed while replacing the placeholder frontend in
  Step 4; removing it is out of this step's scope. Path:
  `crates/app/src/main.rs`.
- `README.md`'s anchor link to `#running-the-app` is not checked by anything;
  there is no Markdown link checker in `mise run ci`. Basis: noticed while
  adding the link in Step 5. Paths: `mise.toml`, `README.md`.
- Write a guide at `docs/agents/tauri-app.md`, to be read by any task touching
  `crates/app/` (Tauri commands, `tauri.conf.json`, or the `crates/app/ui/`
  frontend). `docs/agents/` is empty today, so the Phase 2 gotchas have nowhere
  to be consolidated and would otherwise be rediscovered. It should carry, from
  the Step 2–4 sections above: that `frontendDist` resolves relative to the
  directory holding `tauri.conf.json` (so a sibling `ui/` is `"ui"`, not
  `"../ui"`, and a wrong value fails `generate_context!()`); that `tsc` refuses
  `outDir` equal to `rootDir` (TS18003), so emitting in place means omitting
  both; that a `.ts` file with no import/export is a global script whose
  top-level names can collide with `lib.dom` globals; that
  `DedicatedWorkerGlobalScope` is absent from the `dom` lib and the `webworker`
  lib conflicts with it, so a worker needs a locally declared interface and a
  cast; that relative frontend imports need an explicit `.js` extension because
  `moduleResolution: "bundler"` will not enforce it; that synchronous
  `#[tauri::command]`s already run off the main thread while async ones doing
  blocking IO need `spawn_blocking`; that CI must take the pnpm store path from
  `pnpm store path --silent` rather than hard-coding it; and that on this macOS
  machine `osascript` assistive access is denied, so a task needing GUI
  verification should plan for manual confirmation instead of assuming
  automation. Done when that file exists and the points are in it. Basis: the
  learnings-extractor's record-only proposal, sourced from Steps 2–4 of this
  file. Paths: `docs/agents/tauri-app.md`.
- This repository has no `.claude/settings.json` and no
  `.claude/settings.local.json` at all, so the wrap-up's `settings-promoter`
  step had nothing to promote and returned `FAILED` (its `NO_CHANGES` state
  covers "no diff", not "no file"). Two separate things follow. First, the
  ported skills repeatedly assume a permission allowlist exists — for instance
  the local-review step says `Bash(date *)` is "already on the
  `.claude/settings.json` allowlist", and several scripts insist on relative
  paths so they match allow rules that are not there. Those claims are
  currently false here. Second, nothing records which permissions this workflow
  actually needs, so a fresh clone re-approves everything by hand. Decide
  whether to create a checked-in `.claude/settings.json` with the allowlist the
  skills assume, and if not, correct the skill text that claims one exists.
  Basis: observed at wrap-up step 3.4 of this plan. Paths:
  `.claude/settings.json`, `.claude/skills/develop/SKILL.md`,
  `.claude/skills/pr/SKILL.md`.
- Deferred judgment, not yet worth filing on its own: the fixture trick from
  Step 1 — a hand-built TIFF for `arw::parse` tests needs only the 8-byte header
  plus IFD0, because `next_ifd = 0` stops the chain walk and no SubIFD has to be
  faked. It is one narrow fact about one parser's tests rather than a general
  pattern, so it stays here for now. Revisit when `crates/core`'s ARW parsing
  tests are next touched: if it still holds and is still needed, decide then
  whether it belongs in a guide. Paths: `crates/core/src/arw.rs`.
