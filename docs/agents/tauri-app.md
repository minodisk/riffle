# Working on `crates/app`

Read this before touching Tauri commands, `crates/app/tauri.conf.json`, or the
`crates/app/ui/` frontend. It lists the pitfalls this repository has already
hit, each with the reason it happens.

Each item is tagged:

- **Hit**: actually broke something here.
- **Measured**: measured in this repo and steered a design decision; nothing
  broke, but the naive choice would have missed a stated budget or cost more.
- **Inferred**: taken from the sources or docs; nothing has broken on it yet.

Source: `docs/plans/_archived/20260917-tauri-skeleton/learnings.md` and the fix
in #11.

## Rust side

### Synchronous commands run on the main thread (Hit)

A `#[tauri::command]` without `async` runs **inline on the main thread**.

- Why: tauri-macros routes non-`async` commands through `body_blocking`, which
  calls the function directly from the IPC handler instead of spawning it.
- What broke: `pick_folder` called `blocking_pick_folder`, which parks the
  calling thread on a `sync_channel(0)` `recv`. The main thread stopped pumping
  the run loop, so the native dialog appeared but its buttons did nothing and
  the app hung.
- This passed `mise run ci`, `tsc --noEmit`, and local review.
  It surfaced the first time the user launched the app. An earlier learnings
  note claimed sync commands run off the main thread; that note was wrong.

Rules:

- Anything that waits (dialogs, channels, locks held by the UI) goes in an
  `async` command. For callback APIs, await a
  `tauri::async_runtime::channel(1)`; see `pick_folder` in
  `crates/app/src/commands.rs`.
- An `async` command doing blocking IO wraps it in
  `tauri::async_runtime::spawn_blocking`, as `preview` does. Why: blocking
  inside the future stalls an async runtime worker instead.
- Confirmed again in Phase 3, with a related trap: HTML5 drag-and-drop never
  reaches the frontend. Tauri intercepts it, so `dataTransfer.files` carries no
  usable path and paths arrive only through the `tauri://drag-drop` (and
  `drag-enter` / `drag-over` / `drag-leave`) webview events, listened to via
  `event:listen` under `core:event`'s default `allow-listen`. Those events and
  `window.__TAURI__` are main-thread only; neither exists inside a worker.

### Measure before choosing a JPEG payload over raw pixels (Measured)

`mozjpeg::Compress`'s defaults turn on trellis quantisation and optimised
Huffman tables, so re-encoding a crop costs more than the partial decode that
produced it: 49ms at q85 for a 1037x1024 crop, against 21.5ms to decode it.

- Why: the defaults optimise for file size, and both passes run over every MCU.
- Phase 5's `focus_crop` therefore returns raw RGBA (4.2MB at the 1024 cap)
  rather than a ~180KB JPEG. That trades IPC bytes for CPU; take the
  measurement before trading back, and measure with the optimisations off, not
  with the defaults.

### A partial decode's cost is set by its row, not its size (Measured)

`jpeg_skip_scanlines` on a baseline JPEG still entropy-decodes the rows it
skips; it only skips the IDCT and colour conversion.

- Why: baseline Huffman data is not randomly addressable, so libjpeg must walk
  every MCU row from the start of the scan to reach the wanted one.
- Measured on the 7008x4672 `JpgFromRaw`: a 1024 crop costs 10.8ms at row 300
  and 44.1ms at row 4400, while growing the crop from 512 to 2048 at the same
  row only moves 18.4ms to 29.8ms.
- So a budget for a crop has to be stated for the worst row, not an average
  one, and shrinking the crop is not a way to make it fit.

### A crop's origin snaps to an MCU boundary; don't assume centred or bit-identical (Hit, twice)

`jpeg_crop_scanline` snaps the requested crop origin **down** to an MCU
boundary (16px in the tested files) and does not necessarily widen the crop to
compensate.

- Why: baseline JPEG DCT blocks (MCUs) are the smallest unit libjpeg can crop
  to; it never crops mid-MCU.
- What broke: two independent guesses were wrong. `FocusCrop`'s `point_x`/
  `point_y` are not the crop's centre (measured 269 vs a centre of 262 on one
  file). `crop_size()`'s width does not grow to keep the point centred (a
  64px-wide request at x=200 came back as width 64 at x=160, landing the point
  at 40, not 32) — that behaviour also isn't consistent across files (it does
  widen on a 4:2:2 test file). Always derive the point of interest as
  `centre - crop.origin` from the actual returned crop, never assume a fixed
  offset or that width grows.
- A crop decoded via `jpeg_crop_scanline` is also **not bit-identical** to the
  same region of a full `decode_rgb` near its left/right edges (chroma
  upsampling has one fewer neighbour there; differences up to 2 in the first
  columns, 1 in the last, on the tested file). Equality tests against a crop
  must exclude ~8px from each edge or use a tolerance there, not assert exact
  equality everywhere.
- Source: `docs/plans/_archived/20260918-focus-check/learnings.md`, Steps 1
  and 2.

### Draining background work at quit needs `build()` + `run()` (Hit)

Work that must finish before the process ends — Phase 6's sidecar writer has a
300ms debounce, so a quit inside that window would otherwise leave the write
for the next folder open — is drained in `tauri::RunEvent::ExitRequested`.

- Why: `tauri::Builder::run(context)` takes only the context and gives no place
  to hook the run event. Use `Builder::build(context)?` and then
  `App::run(|app, event| ...)`; the closure's first argument is an `&AppHandle`,
  so `app.state::<...>()` works there.
- The writer's `Drop` is deliberately **not** the drain: a managed state's
  `Drop` is not guaranteed to run during process teardown, and even when it
  does, the channel disconnect races the exit instead of being waited on. Send
  an explicit flush message and block on a reply channel with a bounded wait
  (~2s) instead.
- This is the one place the "never block the main thread" rule above does not
  apply: the run loop is on its way out, and blocking there is what makes the
  drain a drain. Keep the wait bounded anyway.
- Source: `docs/plans/_archived/20260918-ratings-xmp-sidecars/learnings.md`,
  Step 3.

### Snapshot the state you decided on, not just the lock, across a release (Hit)

`reconcile_sidecars` (decides what to parse) and `store_sidecar_ratings`
(stores the parse and clears `dirty`) run with the lock released in between,
so a `set_rating` landing in that window was silently discarded by the
"sidecar wins" write.

- Why: releasing a lock between "decide" and "act" always opens a window for
  the state to change underneath the decision; re-checking "is the lock still
  free" is not enough, the *value* the decision was based on has to be
  re-checked too.
- Fix pattern: `reconcile_sidecars` hands back the `dirty` flag it observed
  for each path alongside the parse job, and `store_sidecar_ratings` only
  applies its update when the row's `dirty` still matches that snapshot —
  otherwise the newer write is left in place.
- Source: `docs/plans/_archived/20260918-ratings-xmp-sidecars/learnings.md`,
  Step 4.

### `frontendDist` resolves from the `tauri.conf.json` directory (Hit)

`tauri.conf.json` lives in `crates/app/`, not the conventional `src-tauri/`, so
the sibling `ui/` is `"frontendDist": "ui"`.

- Why: Tauri resolves the path relative to the directory holding
  `tauri.conf.json`.
- What broke: the plan's `"../ui"` pointed at `crates/ui`, and
  `tauri::generate_context!()` failed the build because the path did not exist.

## Frontend (`crates/app/ui`, `tsc` only, no bundler)

### Give the current folder one token, not one counter per feature (Hit, repeatedly)

Every async path that can outlive the folder it started for — a preview load, a
thumbnail fetch, a background scan, an open by drop — has to ask, when its
result comes back, whether that folder is still the current one. This repo
re-derived the same guard four times under four names (`seq`, `generation`,
`folderToken` / `openDir`, `dropCounter`), and every one was caught in review
rather than by a test.

- Why: any `await` or callback across the IPC boundary can resolve after the
  user has picked, dropped or paged to something else, and the frontend has no
  other signal that the result is now stale.
- When you add an async path that depends on "the current folder", check the
  existing token for the current open operation instead of minting a new
  counter, and check it before applying the result, not before starting.
- The four instances:
  `docs/plans/_archived/20260918-thumbnail-cache-filmstrip/learnings.md`
  (Steps 2, 5, 6, 7).
- Two corollaries from Phase 6's review (Step 5): comparing the directory
  string is **not** enough, because reopening the same folder is a new open
  with the same `dir` — `refreshEntries` checks `dir !== openDir ||
  token !== folderToken` for that reason. And a backend **event** listener,
  which is registered once and outlives every folder, needs the same kind of
  guard on its payload before it touches the UI; the `sidecar-error` listener
  drops a payload whose path is not in the current folder's index.
- Source: `docs/plans/_archived/20260918-ratings-xmp-sidecars/learnings.md`,
  Step 5.

### `tsc` rejects `outDir` equal to `rootDir` (Hit)

To emit `.js` next to the `.ts` sources, omit both options.

- Why: the `outDir` is auto-excluded from inputs, which leaves none (TS18003).

### A `.ts` file with no `import`/`export` is a global script (Hit)

Its top-level `const`/`let` share scope with `lib.dom` globals; `const status`
collided with `window.status`. Add an `import` or `export` (`main.ts` ends with
`export {};`).

- Why: TypeScript only treats a file as a module when it has a top-level
  `import` or `export`.

### Workers: declare the scope locally (Hit)

`worker.ts` declares the members it uses as a local `WorkerScope` interface and
casts `self` to it. Keep one `tsconfig.json` for both threads this way.

- Why: `DedicatedWorkerGlobalScope` is only in the `webworker` lib, and adding
  `webworker` next to `dom` clashes on the globals both declare.

### Write relative imports with `.js` (Inferred)

`import { x } from "./foo.js"`, never `"./foo"`.

- Why: `moduleResolution: "bundler"` type-checks extensionless imports, but no
  bundler rewrites them, so the webview requests `./foo` and gets a 404 at
  runtime, after type-checking has passed.
- Nothing enforces this; today the frontend has no relative imports at all.

## CI

### Do not hard-code the pnpm store path (Inferred)

Cache the output of `pnpm store path --silent` (see `.github/workflows/`).

- Why: the store location differs per OS.

## Verification

### Write a performance number with its measurement conditions (Hit, twice)

Two performance claims here had to be walked back in review because they read
as general when they were not: a 30-second target that rested on warm `cp`
reads, and a batch of README numbers measured on folders of symlinks to a
single file.

- Why: a number without its conditions — page cache state, and whether the
  files were distinct or 5000 symlinks to one inode — is read as a general
  claim and copied forward as one.
- State what was actually measured next to the number, and say plainly what
  still needs a real-folder measurement. `README.md` separates "confirmed",
  "verified without a GUI" and "awaiting the user's confirmation" for the same
  reason; keep new claims inside that split.

### GUI automation does not work on this Mac (Hit)

`osascript` is denied assistive access: the app's window cannot be brought
forward, and injected keystrokes are dropped silently.

- Why: macOS privacy settings on this machine, not something the repo controls.
- Plan any GUI check as a **manual confirmation by the user**, and list exactly
  what they should look at. Report unchecked behaviour as "not verified"; that
  is more useful than an implied pass.
