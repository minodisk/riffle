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

# Per-page latency instrumentation

## Purpose

`todo.md`'s "App: unmeasured end-to-end per-page latency" item: only the Rust
side of a page turn was ever measured (README "Per-page preview read"); the IPC
hop and `createImageBitmap` were not, because the GUI cannot be driven from the
agent's machine. The 1:1 zoom path already has the instrumentation this needs
(`requestCrop` in `crates/app/ui/src/main.ts` logs
`zoom crop=… read=… decode=… ipc=… bitmap=… keypressToPixels=…ms` through
`debugLog`, gated by the settings window's `Timing logs` item), and README's
"End to end, keypress to pixels" section was measured from it. The page-turn
path (`move` -> `show` -> `requestPreview` -> `worker.ts` `createImageBitmap`
-> the `worker` `message` handler's `draw()`) has no marks at all.

This plan gives the page-turn path the same readout, forwards the timing lines
to `Riffle.log` so they can be read without DevTools, documents how, and leaves
only the actual measurement to the user.

Decisions made with the user: timing lines also go to `Riffle.log` (Step 2);
strip clicks are not stamped with `keypressToPixels`; no Rust-side `read`
field is added to the preview header.

## Steps

- [x] Step 1: Log a per-page timing line from the preview path
  - Done when: with `Timing logs` on, every page turn that ends in pixels
    writes one line `page invoke=<ms> decode=<ms> total=<ms>[ keypressToPixels=<ms>]`
    (naming may follow the `zoom …` line; the marks are what matters) where
    `invoke` is the `preview` invoke round trip (Rust read + IPC), `decode`
    is the worker `createImageBitmap` round trip (postMessage to `message`
    back), `total` is from `requestPreview`'s start to after `draw()`, and
    `keypressToPixels` is present only for the page the keypress itself asked
    for; a page turn superseded by another (`current !== seq`) logs nothing;
    `mise run ci` passes.
  - Implementation approach:
    - Frontend only, `crates/app/ui/src/main.ts`. Mirror `requestCrop` /
      `toggleZoom` exactly: a `pageKeypressAt: number | null` set in `move()`
      (that covers the `previous`/`next` keys and auto-advance's `move(1)`;
      strip clicks and folder opens start their own, un-keypressed request,
      same as a resize does for crops), consumed and cleared by the
      `requestPreview` it produces, nulled when `requestPreview` no-ops on
      `inFlight`.
    - `requestPreview` records `requestStartedAt` before the invoke and
      `invokeAt` in `.then`. Because the bitmap arrives asynchronously in the
      `worker` `message` handler, keep the per-request marks keyed by `seq`
      the way `orientations: Map<number, …>` already is (extend that map's
      value or add a sibling map; delete the entry when the response is
      handled or discarded).
    - In the `message` handler, after `draw()`, compute `decode` (now minus
      the postMessage time), `total`, and `keypressToPixels`, and call
      `debugLog(...)`. Keep the "after synchronous `draw()`" convention the
      zoom line uses so the two are comparable.
    - Do not add a Rust-side `read` field to the preview header.
    - No `worker.ts` change is needed: the decode round trip is timed on the
      main thread around the postMessage.
    - `main.ts` has no unit tests; if a helper is extracted (e.g. formatting
      the line), put it in a small module with a vitest like `zoom.ts` /
      `zoom.test.ts`, otherwise no new tests.

- [x] Step 2: Forward the timing lines to `Riffle.log`
  - Done when: with `Timing logs` on, the `page …` and `zoom …` lines appear
    in `Riffle.log` (the same file `Help > Open Log Folder` reveals) without
    DevTools open; with it off, nothing is sent; `mise run ci` and
    `cargo test -p riffle-app` pass.
  - Implementation approach:
    - Keep the gating on the frontend's `debugLogging` so the extra IPC call
      only happens when timing logs are on, and issue it after `draw()` so it
      is outside the measured interval.
    - First check whether `tauri-plugin-log` already exposes a JS API on
      `window.__TAURI__.log` under `withGlobalTauri: true`
      (`crates/app/tauri.conf.json`); if it does, add the typing to
      `crates/app/ui/src/tauri.d.ts` and call its `info`/`debug`. If not, add
      one `#[tauri::command] fn log_timing(line: String)` in
      `crates/app/src/main.rs` that does `log::info!("{line}")`, registered
      in the `generate_handler!` list next to `timing_logs`.
    - Route through `debugLog` itself (console plus log file) so the existing
      `zoom …` line gets the same sink without touching `requestCrop`.
    - The log plugin's `max_file_size` is already 1 MB; a page line is ~100
      bytes, so no cap change (see `docs/agents/tauri-app.md`, "A per-tick log
      line can rotate other lines out").

- [x] Step 3: Document the measurement and update `todo.md`
  - Done when: README's "Per-page preview read" section says the end-to-end
    number is now observable and where; "Measuring on your own folder" gains
    a short paragraph on per-page latency (turn on `Timing logs` in the
    settings window, which only shows in a `mise run tauri:release:devtools`
    or debug build; page through with the next/previous keys; read the
    `page …` lines in `Riffle.log`, with the field meanings); `todo.md`'s item
    is reworded so only the human measurement on real hardware remains (mirror
    the wording of the "real-folder scan and second-open numbers" item: "The
    instrumentation now exists: …"); `mise run ci` passes.
  - Implementation approach:
    - Files: `README.md` (the "Per-page preview read" and "Measuring on your
      own folder" sections), `todo.md`.
    - Do not add measured numbers; the user fills them in.

## Trade-offs and risks

- **Status-line readout vs log line.** `setStatus()` is rewritten on every
  `show()` and by judgment messages, so a number there would be overwritten;
  logging matches the existing zoom instrumentation.
- **Rust-side `read` in the preview header.** Not planned: the Rust read is
  already measured and small; if `invoke` dominates the first measurement,
  that is the follow-up.
- **Where "pixels" is marked.** After the synchronous `draw()` (as the zoom
  line does) rather than in a `requestAnimationFrame`, for comparability.
- **Timing logs item is dev-build only** (`cfg!(any(feature = "devtools",
  debug_assertions))`), so the user must measure on
  `mise run tauri:release:devtools`, exactly as for the zoom numbers.

## Progress

- (2026-09-21) Step 1 complete
- (2026-09-21) Step 2 complete
- (2026-09-21) Step 3 complete
