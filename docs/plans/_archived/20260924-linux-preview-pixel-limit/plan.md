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

# Resize oversized previews in the decode worker on Linux

## Purpose

On Linux (WebKitGTK 2.50.4, seen under WSLg with `mise run dev`) an
`ImageBitmap` created inside a Worker and transferred to the main thread draws
fully transparent once it is roughly 16 MP or larger (3000x4097 draws,
4000x4000 does not). Since #376 SIGMA fp L DNGs have no mid-size strip JPEG,
so the preview tier falls back to the full-size 9520x6328 JPEG (~28 MB, 60 MP):
the strip shows the file but the main view stays blank, with no error and no
status note. Windows (WebView2) is unaffected.

Decoding on the main thread works, and so does
`createImageBitmap(blob, { resizeWidth, resizeHeight, resizeQuality: "high" })`
inside the worker (60 MP -> 3200x2127 in ~600 ms). The fix keeps the worker
and, only on Linux and only for previews over a backend-provided pixel limit
(~12 MP), passes aspect-preserving resize options. Every other path stays
byte-for-byte as it is today.

The decision to gate by platform in Rust with `cfg!(target_os = "linux")` was
made by the user and is not up for re-litigation.

## Steps

- [x] Step 1: Expose a Linux-only preview pixel limit from the backend and have the decode worker resize previews above it
  - Done when:
    - `crates/app/src/commands.rs` has
      `pub const PREVIEW_PIXEL_LIMIT: Option<u32>` defined with
      `if cfg!(target_os = "linux") { Some(12_000_000) } else { None }` (same
      style as `SIZE_BASE` there and `MACOS` in `shortcuts.rs`), with a doc
      comment naming the WebKitGTK transferred-bitmap symptom and the measured
      threshold (12.3 MP drew, 16 MP did not), and a sync
      `#[tauri::command] pub fn preview_pixel_limit() -> Option<u32>` returning
      it, registered in the `generate_handler!` list in `crates/app/src/main.rs`.
      A unit test in the existing `tests` module of `commands.rs` asserts the
      value against `cfg!(target_os = "linux")` (pattern: the `SIZE_BASE` test
      at ~line 1794).
    - A new frontend module `crates/app/ui/src/decode.ts` exports two pure
      functions with tests in `crates/app/ui/src/decode.test.ts`:
      - `jpegSize(bytes: Uint8Array): { width: number; height: number } | null`
        walks the JPEG marker segments from SOI and returns the frame size
        from the first SOFn marker (0xC0-0xCF except 0xC4 DHT, 0xC8 JPG,
        0xCC DAC; height is the big-endian u16 at segment offset 5, width at 7),
        `null` when no SOF is found before SOS or the end of the buffer.
        Tests build minimal byte sequences (SOI + APP0 + SOF0, SOF2, an
        SOF-less buffer, a truncated buffer).
      - `fitWithin(width: number, height: number, maxPixels: number): { resizeWidth: number; resizeHeight: number } | null`
        returns `null` when `width * height <= maxPixels`, else the largest
        integer size with the same aspect ratio whose pixel count is at or
        under `maxPixels` (scale = `Math.sqrt(maxPixels / (width * height))`,
        floor both sides, never below 1). Tests: 9520x6328 at 12 MP fits under
        the limit and keeps the ratio within one pixel; 6000x4000 (24 MP) is
        resized; 4000x3000 (12 MP exactly) and small images return `null`.
    - `crates/app/ui/src/worker.ts`'s `DecodeRequest` gains
      `maxPixels: number | null`. When `maxPixels` is `null` the call stays
      exactly `createImageBitmap(new Blob([jpeg], { type: "image/jpeg" }))`.
      When it is a number, the worker calls `jpegSize` on the bytes, then
      `fitWithin`; if that returns a size it passes
      `{ resizeWidth, resizeHeight, resizeQuality: "high" }` as the second
      argument, otherwise the plain call. An unparsable JPEG (`jpegSize` null)
      also takes the plain call, so the failure mode is today's, not a new one.
    - `crates/app/ui/src/main.ts` fetches the limit once at startup, next to
      the `timing_logs` / `auto_advance` fetches (~line 1962), as a kept promise
      (e.g. `const previewPixelLimit = window.__TAURI__.core.invoke<number | null>("preview_pixel_limit").catch(() => null)`),
      and `requestPreview()` awaits it before `worker.postMessage`, passing
      `maxPixels`. Awaiting the resolved promise costs only a microtask on
      later pages and closes the first-preview race; the `current !== seq`
      re-check that already follows the `preview` invoke must still apply after
      the await (or the await must happen before the header parse, inside the
      same `.then` chain) so a stale page never posts.
    - The compare view (`loadCompare` in `main.ts`) is left as is: it already
      decodes on the main thread, which the bug does not affect. Say so in the
      PR body.
    - `docs/agents/tauri-app.md` gets a short "Hit" item under the workers
      section (near "Workers: declare the scope locally") describing the
      WebKitGTK transparent-transferred-bitmap threshold and pointing at
      `PREVIEW_PIXEL_LIMIT`.
    - Manual check on Linux (`mise run dev`, WSLg): a SIGMA fp L DNG shows in
      the main view; the `page …` timing line shows the decode. On a
      non-Linux build, or by temporarily forcing `maxPixels` to `null`, the
      worker call is unchanged.
    - `mise run ci` passes (vp check/test, cargo fmt/clippy/test).
  - Implementation approach (as far as it is known):
    - Do NOT touch `crates/app/src/index.rs` (edited concurrently elsewhere).
    - Keep the `preview` payload header untouched (the reserved bytes stay
      zero, the doc comment stays true): the size is read from the JPEG's SOF
      in the worker instead, so the Rust side only adds the limit constant and
      command.
    - `worker.ts` keeps its local `WorkerScope` declaration (see
      `docs/agents/tauri-app.md`, "Workers: declare the scope locally");
      `resizeQuality: "high"` is in the DOM lib's `ImageBitmapOptions`, so no
      new declarations are needed.
    - The scan for SOF stops at SOS (0xDA); standalone markers (0xD0-0xD7 RST,
      0x01 TEM, 0xD8 SOI) carry no length field and must be skipped without
      reading one. Payloads are up to ~28 MB but the SOF sits within the first
      few KB, so the scan is negligible.
    - Conventional Commit: `fix(app): resize oversized previews in the decode worker on Linux`.

## Trade-offs and risks

- **How the worker learns the size.** Three options were weighed; the step
  takes the first (approved by the user).
  1. Parse the JPEG SOF header in the worker (chosen): ~20 lines of pure TS,
     unit-testable, one decode, non-Linux path untouched, no payload change.
  2. Decode once, check `bitmap.width * bitmap.height`, and on Linux over the
     limit close it and decode again with resize: the fewest lines and no
     parser, but a wasted full decode of a 60 MP JPEG (~600 ms or more) on
     every page turn to such a file on Linux.
  3. Have the backend fill the header's reserved 4 bytes with width/height:
     needs a Rust SOF parser in `riffle_core`, changes the payload contract
     and its doc comment, touches more crates. Rejected as not minimal.
- **Race at startup.** If the first preview were posted before the
  `preview_pixel_limit` invoke resolved, that one page would decode unresized
  on Linux. Awaiting the kept promise removes the race at the cost of one
  await in `requestPreview`.
- **The limit value.** 12 MP is a margin under the measured 16 MP failure; the
  exact WebKitGTK threshold was not bisected. If a Linux user reports blank
  previews between 12 and 16 MP the constant is the single knob.
- **Resized previews lose 1:1 detail on Linux only**, but the fitted main view
  is far smaller than 12 MP anyway and the focus check uses `focus_crop`, so
  nothing visible changes.

## Progress

- (2026-09-24) Step 1 complete
