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

# Phase 5: the 1:1 focus check

## Purpose

The app shows 400px thumbnails (Phase 3) and the 1616x1080 embedded preview
(Phase 2/3), which is enough to see the frame but not whether the eye is
sharp. Phase 5 adds the third tier: pressing `Space` toggles a 1:1 view of the
current file, centred on the camera's focus point, cut out of the
full-resolution `JpgFromRaw` by the partial decode `riffle-cli crop` already
uses. Budget: 50ms from keypress to pixels.

Everything comes from the JPEGs embedded in the ARW; no RAW decoder, no
external process, and the RAW file is never modified.

Scope guard: no prefetch of the crop (Phase 4 owns prefetching), no ratings
(Phase 6, running in parallel), no panning, no free zoom levels.

## Decisions taken before implementation

Settled during planning from measurements on the real test file; the
measurements are in "Trade-offs and risks". Reopen only with a reason.

1. **`Space` is a toggle, not a hold.** Holding fires `keydown` auto-repeat,
   needs a `keyup` handler, and ties up a hand the culling workflow wants on
   the paging keys. A toggle also makes decision 2 possible.
2. **Paging while zoomed stays zoomed**, showing the next file's crop at that
   file's own focus point. Comparing the eye across a burst is the whole point
   of the tier; dropping back to the preview on every page turn would make the
   user press `Space` once per frame.
3. **No panning in this phase.** Arrows/WASD/HJKL are paging keys, the mouse
   has no role in the app yet, and panning implies decoding more than the
   focus region. The crop is where the camera focused; that is what is being
   judged.
4. **A file without `FocusLocation` (manual focus) falls back to the centre of
   the full JPEG**, decided on the Rust side so the frontend has one code path.
   Refusing would make `Space` a no-op on a whole class of files.
5. **1:1 means one JPEG pixel per device pixel** (`devicePixelRatio`-aware).
   One pixel per CSS pixel would show a 2x upscale on a Retina display, which
   defeats a sharpness check.
6. **The crop is the viewport in device pixels, capped per axis** (initial
   cap: 1024; see risks), and the payload is **raw RGBA**, not a re-encoded
   JPEG. Decode cost barely depends on crop size (512 → 20.7ms, 2048 → 30.2ms)
   but the payload does (4.2MB at 1024², 16.9MB at 2048²), and re-encoding
   costs 49ms at 1024² on its own. The rest of the canvas around the crop
   shows the existing preview bitmap scaled to the same 1:1 scale, so the
   frame stays in context and something is on screen the instant `Space` is
   pressed.
7. **The focus arithmetic lives in `riffle-core`, is shared by the CLI and the
   app, and is ported from `riffle-cli focusbox` / `drawFocusBox`**:
   `FocusLocation` is in unrotated sensor coordinates; scale it onto the
   unrotated full JPEG (`fx * jpeg_w / sensor_w`, same for y), crop there, and
   let the canvas rotation (the same transform `draw()` applies to the
   preview) carry the crop. Never crop after rotating. On the test file the
   JPEG is 7008x4672, the same as the sensor, so the scale is exactly 1 there
   — the code must still scale, other bodies may differ. `riffle-cli crop` and
   `bench` currently skip the scaling; Step 1 fixes `crop`.

## Steps

- [ ] Step 1: Core: ranged read of `JpgFromRaw`, focus-point mapping, rectangular RGBA crop; `riffle-cli crop` uses them
  - Done when:
    - `riffle_core::reader` has a function that parses the metadata from the
      bounded prefix and returns the `Arw` plus the full-resolution JPEG bytes
      via a ranged read of `arw.full` (the JPEG starts inside the 1MiB prefix
      on the test file but ends ~6.3MB in, so the range read is the normal
      path, not a fallback). Falls back to reading the whole file when the
      prefix is too short to parse, like `read_preview`. Unit tests with a
      synthetic TIFF, mirroring the `read_preview` tests
    - `riffle_core::partial` (or a sibling) has a function that, given the
      decoded JPEG size, an `Option<FocusLocation>` and a requested crop
      width/height in unrotated JPEG pixels, returns the point of interest on
      the full JPEG (scaled focus point, or the centre when `None`) and the
      crop; the crop is rectangular (`w`, `h`), not only square, and can be
      produced as RGBA (`JCS_EXT_RGBA`) so no conversion pass is needed. The
      result carries the point of interest **in crop pixel coordinates** (the
      MCU snap shifts the crop's origin by up to 15px, so the crop's centre is
      not the focus point). The existing `decode_crop(jpeg, cx, cy, size)`
      keeps working (as a wrapper or unchanged) so `bench` is untouched
    - Unit tests with a synthetic encoded JPEG (like `decode.rs`'s gradient
      test): the crop's pixels equal the same region of a full `decode_rgb`,
      `x` is a multiple of the MCU width, the clamping at the edges, the
      centre fallback, and the sensor→JPEG scaling with a sensor size that
      differs from the JPEG size
    - `riffle-cli crop <file.ARW> <out.png> [size]` goes through the new read
      and mapping functions and prints the crop rectangle **and the point of
      interest in crop coordinates**. On `~/Downloads/_DSC6978.ARW` with the
      default 512 it still prints `crop 525x512 at (3344,1476)` and now also
      prints point `(269,256)`; the written PNG is 512x525 (rotated). Record
      the exact CLI output in `learnings.md`; never commit the file or the
      PNG
    - `mise run ci` passes
  - Implementation approach:
    - Follow `reader::read_preview` / `preview_from` for the head-then-range
      pattern; a shared helper taking the `Embedded` is fine, but do not change
      `read_preview`'s behaviour
    - `decode_crop`'s body already does clamp → `jpeg_crop_scanline` →
      `jpeg_skip_scanlines` → read rows → `jpeg_abort_decompress`; generalise
      it rather than duplicating it. Set `out_color_space` to
      `JCS_EXT_RGBA` for the RGBA variant and use 4 bytes per pixel in the
      stride
    - Port the scaling from `focusbox()` in `crates/cli/src/main.rs`
      (`sx = w / sensor_w`, `sy = h / sensor_h`); do not re-derive it

- [ ] Step 2: App: an async `focus_crop` command returning the crop as a raw RGBA payload
  - Done when:
    - `crates/app/src/commands.rs` has `#[tauri::command] pub async fn
      focus_crop(path, width, height)` that runs the Step 1 read and decode in
      `tauri::async_runtime::spawn_blocking` and returns a
      `tauri::ipc::Response`. `width`/`height` are the viewport in device
      pixels; the command swaps them for Orientation 6/8 before cropping
      (the viewport's width runs along the unrotated JPEG's height), and
      clamps each to the cap from decision 6 (a `const` next to the header
      constants)
    - A new payload kind `CROP_KIND_RGBA_V1 = 3` with its own header (a
      documented fixed-size little-endian layout like `payload()`'s table:
      kind, orientation, crop width, crop height, point-of-interest x and y in
      crop pixels; reserve to a round size), followed by the RGBA bytes. The
      existing 8-byte header and kinds 1/2 are untouched
    - Registered in `main.rs`'s `generate_handler!`; no capability change is
      needed (`core:default` already covers `invoke`)
    - Unit tests: the header encodes every field; a synthetic ARW (the
      `reader` test builder pointing at an encoded gradient JPEG) round-trips
      through the command's blocking body with the expected crop size and
      point; Orientation 8 swaps width and height
    - `mise run ci` passes
  - Implementation approach:
    - Mirror `preview`: `spawn_blocking(...).await.map_err(..)??`, errors as
      `String` prefixed with the path. Sync commands run on the main thread
      (`docs/agents/tauri-app.md`), so this must be `async`
    - Keep the blocking body a plain function (like `read_preview` in
      `commands.rs`) so the tests call it without Tauri
    - Do not touch `index.rs` or its schema; the command reads the ARW
      directly. The index's `FocusLocation` is not needed here because the
      bounded prefix already yields it in the same read

- [ ] Step 3: Frontend: `Space` toggles the 1:1 view, drawn centred on the focus point under the preview's rotation
  - Done when:
    - `Space` toggles a `zoomed` flag. While zoomed, `draw()` hands off to a
      new `drawZoom()` at its top and returns; nothing else in `draw()`,
      `show()` or the `keydown` handler is restructured. The `keydown`
      addition is one `else if (key === " ")` branch (`event.key` is `" "`
      for Space; it survives `toLowerCase()`). Space does not claim `1`–`5`,
      `x`, `f`, `o` or the paging keys
    - `drawZoom()`: translates to the canvas centre and applies the **same
      rotation as `draw()`**; first draws the current preview bitmap (`shown`)
      scaled so one full-JPEG pixel is one device pixel (scale =
      `jpegWidth / bitmap.width` known from the crop header or the point
      mapping; if the preview is not yet loaded draw nothing behind), offset
      so the focus point sits at the origin; then draws the crop bitmap on
      top, offset by minus its point-of-interest. The focus box (`f`) is not
      drawn in this mode
    - Requesting: a `requestCrop()` that mirrors `requestPreview()` —
      one request in flight, latest wins, guarded by the existing `seq`
      (per `docs/agents/tauri-app.md`, no new counter); passes
      `Math.round(canvas.clientWidth * devicePixelRatio)` and the height; on
      the result checks `seq`, reads the header, builds an `ImageData` over
      the RGBA bytes and `createImageBitmap(imageData)` (no JPEG decode, so
      no worker round-trip is needed; `putImageData` is not usable because it
      ignores the rotation transform). The previous crop bitmap is `close()`d
      when replaced or when the file changes
    - Paging while zoomed: `show()` gains one line — `if (zoomed)
      requestCrop();` — so the new file's crop is requested alongside its
      preview; the old crop is not drawn over the new preview (drop it when
      `seq` moves on). Toggling zoom off draws the preview as before;
      toggling back on for the same file reuses the crop bitmap if it is
      still held
    - Status line shows `1:1` while zoomed (through `setStatus`'s `extra`
      argument or an additive part; do not restructure `setStatus`). A crop
      error shows in the status line and leaves the preview visible
    - Timing marks: `performance.now()` at keypress, after `invoke` resolves,
      and after `createImageBitmap`, written with `console.debug` behind a
      single `const ZOOM_TIMING = false` flag flipped on for the manual check
      (or removed before merge; say which in `learnings.md`)
    - `tsc --noEmit` and `mise run ci` pass. `README.md` is left to Step 4
    - **(manual)** In the running app on `~/Downloads/_DSC6978.ARW`:
      - `Space` shows the subject's eye at 1:1 in the middle of the canvas,
        upright (Orientation 8), with the blurry preview around it; the crop
        region is the same region `riffle-cli crop` writes to its PNG, seen
        rotated the same way
      - `Space` again returns to the preview with the focus box
      - Paging with `j`/`l` while zoomed stays zoomed and moves to the next
        file's focus point; the old crop never appears over the new file
      - A manual-focus file (no `FocusLocation`) zooms to the frame centre
      - Report the three `console.debug` timings for a top-of-frame and a
        bottom-of-frame focus point, stating whether the folder had been
        opened before (page cache)
  - Implementation approach:
    - All additions in `crates/app/ui/src/main.ts`, localised: new state
      (`zoomed`, `crop`, `cropInFlight`), new functions (`requestCrop`,
      `drawZoom`), the three one-line hooks. No reformatting of untouched
      code; the parallel Phase 6 session edits the same file
    - Header constants (`CROP_HEADER_LEN`, `CROP_KIND_RGBA_V1`) next to the
      existing `PREVIEW_*` constants, matching `commands.rs`
    - Coordinates: in the rotated frame the crop's top-left is
      `(-point.x, -point.y)` in device pixels, and since `context.scale(dpr,
      dpr)` is applied in `draw()`, either skip that scale in `drawZoom()` or
      divide by `dpr`; check which the code does and keep one convention.
      Verify the direction of rotation by construction: `drawZoom()` uses the
      same `rotate()` branches as `draw()`, and the crop is cut in unrotated
      coordinates, exactly as the focus box is placed
    - Resize while zoomed: redraw with what is held; re-requesting is not
      required in this phase

- [ ] Step 4: Documentation, status and measurements
  - Done when:
    - `README.md` "Status": Phase 5 done, `Space` in the key table, a
      paragraph on the 1:1 tier (crop from `JpgFromRaw`, centred on the focus
      point, centre fallback, stays zoomed while paging, no panning); the
      `crop` CLI line shows the optional size; the "confirmed / verified
      without a GUI / awaiting the user's confirmation" split is extended
      with this phase's items, with the Step 3 manual checks listed as
      unconfirmed unless the user has confirmed them
    - A measurement table for the Rust side of the 1:1 path (ranged read,
      crop by size, crop by focus-point row), **each row stating the
      conditions**: one real file, warm page cache, in-process, n, and that
      the IPC hop and `createImageBitmap` are excluded unless the user
      reported the Step 3 timings — in which case those are recorded as the
      user's numbers with the conditions they stated. The planning numbers in
      "Trade-offs and risks" may be re-measured with the Step 1 CLI rather
      than copied
    - `CLAUDE.md` "Layout" mentions the `focus_crop` command if the layout
      text lists commands; otherwise unchanged
    - `docs/agents/tauri-app.md` gains what this phase hit, at minimum:
      `mozjpeg::Compress` defaults (trellis/Huffman optimisation) make a
      re-encode cost more than the decode it follows — measure before
      choosing a JPEG payload; and that `jpeg_skip_scanlines` on a baseline
      JPEG still entropy-decodes the skipped rows, so a crop's cost is set by
      its row, not its size. Tag each per the file's Hit/Inferred convention
    - `mise run ci` passes

## Trade-offs and risks

### Measurements behind the decisions

All on `~/Downloads/_DSC6978.ARW` (Orientation 8, `FocusLocation`
7008 4672 3613 1732, JpgFromRaw 7008x4672 baseline 4:2:2 at offset 544768,
5,761,112 bytes), Apple Silicon Mac, **warm page cache, one file, in-process,
n=20, release build; IPC and `createImageBitmap` excluded**. Not committed
anywhere; re-measure in Step 1/4 rather than copy.

| What | Median |
|------|--------|
| Ranged read of the 5.76MB JpgFromRaw | 1.7ms |
| `decode_crop` 512 / 768 / 1024 / 1536 / 2048 at the focus point | 20.7 / 21.4 / 24.1 / 27.8 / 30.2ms |
| `decode_crop` 1024 at cy = 300 / 2336 / 4400 | 11.0 / 26.9 / 42.7ms |
| `mozjpeg::Compress` re-encode of the 1037x1024 crop, q85 / q92 | 48.9 / 70.9ms |
| RGB → RGBA copy, 1037x1024 / 2061x2048 | 2.8 / 13.7ms |

`riffle-cli crop` itself: `crop 525x512 at (3344,1476)`, 21.5–34ms across
runs (the first run of a fresh process is the slow one).

### The 50ms budget is not safe for a bottom-of-frame focus point

Skipped rows dominate: 43ms at row 4400 before the IPC hop and the bitmap. The
plan does not have a fix inside this phase; options for later, to be chosen on
the user's Step 3 timings: prefetch the crop of the neighbouring files (Phase
4's ring buffer), or decode with DCT scaling for the placeholder. Do not claim
the budget is met until the user reports end-to-end numbers.

### Crop cap and payload size (decision 6)

- Cap 1024 per axis: 4.2MB RGBA per keypress; on a 2x display the sharp
  region is a 512 CSS-px square in the middle of the canvas with the blurry
  preview around it.
- Cap = viewport (2240x1520 at the default window on 2x): 13.6MB per
  keypress through Tauri's IPC, whose throughput could not be measured
  headlessly.
- The step takes 1024 and leaves the constant to be tuned from the user's
  reported timings. If the IPC hop turns out cheap, raise it; if the 512
  CSS-px region is too small to judge, the same knob applies.

### Raw RGBA vs JPEG payload

JPEG would cut the payload to ~180KB and reuse the worker's
`createImageBitmap` path, but the re-encode costs 49ms at 1024² with
`mozjpeg::Compress` defaults; turning the optimisations off was not measured.
If IPC proves to be the bottleneck, that measurement is the first thing to
take before switching.

### Toggle vs hold, and the exit key

Toggle is taken (decision 1). `Escape` as a second way out is free but not
planned; add it only if the user asks. If the user prefers hold-to-zoom, the
`keyup` handler is an additive change to Step 3, but decision 2 (stay zoomed
while paging) then goes away.

### Placeholder from the preview

Drawing the 1616-wide preview at 1:1 scale is a ~4.3x upscale, visibly soft
by design; it exists so the frame stays in context and `Space` reacts on the
same frame. Alternative: a dark canvas until the crop lands — simpler, but
the user sees nothing for 30–50ms and loses the frame context. Taken:
placeholder.

### Verification limits

The coordinate arithmetic is verified numerically only on the Rust side (the
CLI and the app share the function; the CLI prints (3344,1476) and (269,256)
on the test file). The frontend's placement and rotation are right by
construction — same `rotate()` branches as `draw()`, crop in unrotated
coordinates — but only the **(manual)** checks in Step 3 confirm it on
screen, and the end-to-end time can only come from the user's timings. Report
both as unverified until then.

### Schema

No index schema change is needed: `FocusLocation` and Orientation come out of
the bounded prefix in the same read that locates `JpgFromRaw`. If Phase 4
later wants to cache crop parameters, that is a Phase 4 question.

## Progress

- (none yet)
