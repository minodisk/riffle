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

# Sony eye-AF frame as the sharpness window

## Purpose

Since face-aware sharpness landed, every scanned file pays a full-size RGB
decode plus YuNet inference (~16 ms per image on this machine; the scan's
per-file mean went 29 -> 67 ms on ARW and 45 -> 92 ms on DNG at 12 threads,
414 -> 178 files/s on 468 ARWs), and YuNet's recall on small faces is poor.
Sony bodies with eye/face tracking already write where they focused: the
plaintext MakerNote tags `AFTracking` (0x2021, BYTE; 1 = face tracking) and
`FocusFrameSize` (0x2037, 3 x SHORT `w h valid`) alongside the
`FocusLocation` the parser already reads. When face tracking was engaged the
frame is a small square on the face/eye, so the sharpness window can be
centred on `FocusLocation` and sized from the frame with no inference at all.
Face detection stays for everything else (Leica DNG, Sony frames without
tracking, tracking that never locked).

This plan also records the real-file measurements already taken on this
machine (`/mnt/d/Photos/2026/...`) that `docs/performance.md` still lists as
unmeasured, plus the before/after of the skip.

Investigation results behind the design are in `investigation.md`, and the
measurements already taken are in `measurements.md`, both in this folder.

## Decisions taken at approval

- Gate on `AFTracking` + `FocusFrameSize` + `FocusLocation`; `AFAreaMode` is
  not parsed (enciphered, unreliable). Not-engaged state excluded by the exact
  sensor-centre focus point (Option A below).
- Step 3 restores the `riffle-app` binary-size row in `docs/performance.md`
  and closes that todo item.
- `docs/agents/raw-metadata-parsing.md` is not written in this plan; its todo
  item stays open.

## Steps

- [x] Step 1: Parse `AFTracking` and `FocusFrameSize` from the Sony MakerNote
  - Done when:
    - `crates/core/src/arw.rs` reads tag 0x2021 (`TYPE_BYTE`, count 1) into
      `Shot::af_tracking: Option<u8>` (raw value, documented: 0 off, 1 face
      tracking, 2 lock-on AF, mirroring how `focus_mode` is kept raw) and tag
      0x2037 (`TYPE_SHORT`, count 3, value at offset) into
      `Shot::focus_frame: Option<FocusFrame { width: u16, height: u16 }>`,
      which is `None` when the third SHORT is 0 (exiftool's `n/a`)
    - Both are read from the same `maker` IFD as `FocusMode` /
      `FocusLocation` (the `maker_note_ifd` result), so a Leica note or a
      missing MakerNote yields `None` for both
    - Unit tests in `arw.rs` (extend `tiff_with_maker` or add a sibling
      builder in the same style) cover: both tags present; each absent; a
      frame whose validity flag is 0 -> `None`; a wrong type/count -> `None`
      rather than an error; the existing Leica fixtures still give `None`
    - Every `Shot { .. }` literal without `..Default::default()`
      (`crates/app/src/index.rs` tests, `crates/core/src/sharpness.rs` tests)
      compiles; `mise run ci` passes
  - Implementation approach:
    - Follow the existing constant naming (`TAG_AF_TRACKING`,
      `TAG_FOCUS_FRAME_SIZE`) and the `byte` / `shorts4` helper style; a
      3-SHORT reader can generalise `shorts4` or be a small sibling. Count-3
      SHORT is 6 bytes, so the value lives at the entry's offset, unlike
      `FocusMode`
    - Do not parse `AFAreaMode`: it lives in the enciphered 0x9402 block and
      is not a reliable "engaged" signal on real files (see `investigation.md`)
    - Nothing in the app reads the new fields yet; `Shot` is not persisted in
      the index, so no schema change

- [ ] Step 2: Route the sharpness window through the eye-AF frame and skip detection in the scan
  - Done when:
    - `crates/core/src/sharpness.rs` exposes the gate (e.g.
      `eye_af_frame(shot: &Shot) -> Option<(FocusLocation, FocusFrame)>`)
      that is `Some` only when `trusted_focus` is `Some`,
      `af_tracking == Some(1)`, `focus_frame` is `Some`, and the focus point
      is not exactly at the sensor centre (`x == sensor_w / 2 &&
      y == sensor_h / 2`, the not-engaged state observed as
      `3504 2336` + `832x740`)
    - `score_preview` gains the frame as an input (a new `Option<FocusFrame>`
      parameter, or a small enum in place of `focus`; decide in the step and
      keep the routing inside `sharpness.rs`) and, when the frame is given,
      scores a square window centred on the focus point whose side is the
      frame's long side mapped to preview pixels (`side * preview_w /
      sensor_w`, the same scale `partial::focus_point` uses) and clamped to
      `[EYE_WINDOW_MIN, WINDOW]`, ignoring `faces`
    - `crates/core/src/scan.rs::extract` calls `detect_faces` only when the
      gate is `None`; the eye-AF path passes an empty `faces` slice
    - The module doc of `sharpness.rs` lists the new first path; CLAUDE.md's
      layout sentence for `sharpness.rs` and the README "Sharpness cue"
      bullet mention it in one clause each
    - Unit tests: gate `None` for `af_tracking` 0/2/absent, for a `None`
      frame, for manual focus, and for the exact-centre point; `Some` for an
      off-centre point (including one with only x at centre); window side
      128 for a 153x154 frame on a 7008-wide sensor and 1616-wide preview,
      256 for a 1533x1535 frame; with a frame given, a face elsewhere in the
      preview does not move the window
    - Measured before/after on this machine with the release `riffle-cli
      scan` on `/mnt/d/Photos/2026/2026-07-11` (468 ARWs; 12 threads as in
      `measurements.md`, plus 1 thread), warm page cache, runs alternated
      between a build of `origin/main` and this branch; per-file mean / p95
      and files/s recorded in `learnings.md` together with how many of the
      folder's files took the eye-AF path (a one-off count, e.g. a temporary
      print or an exiftool sample; not committed instrumentation unless it is
      trivially small)
    - `mise run ci` passes
  - Implementation approach:
    - Assumes Step 1 is merged
    - The sensor-centre test needs `FocusLocation` only; do not hard-code
      `832x740`, it is body-specific. If the implementation finds engaged
      frames at the exact centre in a broader sample, switch to excluding by
      frame size and note it in `learnings.md`
    - Leica DNG and Sony frames without tracking keep the existing detector
      path unchanged; the 4-neighbour Laplacian over a square window is
      rotation-invariant, so scoring in stored coordinates stays valid

- [ ] Step 3: Record the real-file measurements and update the docs
  - Done when:
    - `docs/performance.md` "Face detection cost" replaces its "Not yet
      measured" paragraph with: detection latency on real α7 V ARW and M11-P
      DNG previews (`riffle-cli bench`), the scan before/after of adding
      detection, and the before/after of the Step 2 skip on the same folder,
      each with machine, thread count, folder size, date and cache state;
      plus a short recall note (the DNG zero-face count and the 7-person
      frame) with its sample sizes. Every number must come from
      `measurements.md` / `learnings.md` in this plan folder; nothing
      unsourced
    - `docs/performance.md` "Face detection cost" gets the `riffle-app`
      binary-size row (26.8 MB -> 47.8 MB, Linux WSL2, unstripped; sourced in
      `docs/plans/_archived/20260922-face-aware-sharpness/learnings.md`), and
      the todo item "Docs: restore the `riffle-app` binary-size row" is
      removed
    - `todo.md` "App: face/eye-aware focus check for culling": replace the
      MakerNote checkbox with the answer (Sony writes `AFTracking` +
      `FocusFrameSize` + `FocusLocation`, no per-face list; `AFAreaMode` is
      enciphered; Leica writes only `FocusDistance`), replace the "Measure on
      the Mac" checkbox with what was measured here, and add checkboxes for
      the two deferred items: small-face recall (DNG 29/36 zero faces, 2 of 7
      found) and the DCT-scaled RGB decode for detection
    - `mise run ci` passes (lychee link check included)
  - Implementation approach:
    - Assumes Step 2 is merged so the skip numbers exist
    - Follow the table style of the existing "Sharpness scoring cost" and
      "Real folders on Windows" sections

## Trade-offs and risks

- **`AFAreaMode` not parsed.** It is inside the enciphered Sony 0x9402 block
  (`c = b^3 mod 249` substitution), and on the sampled real files it
  disagrees with `AFTracking` in both directions. Implementing the decipher
  would only distinguish human from animal eye tracking, which the window
  sizing does not need.
- **Not-engaged detection.** Option A (taken): exclude when the focus point is
  exactly at the sensor centre. Body-independent; a tracked face dead centre
  falls back to detection (harmless, only slower). Option B: exclude when the
  frame equals the body's default (832x740 on the α7 V), body-specific.
- **`score_preview` signature.** Extra parameter vs a subject enum; Step 2
  decides.
- **Eye-AF path count.** Temporary instrumentation rather than a committed
  field.
- **Behaviour change for Sony frames with tracking engaged.** The window now
  sits on the camera's frame instead of YuNet's eye midpoint; this is
  reasoning, not a measured ranking change. Step 2 should spot-check a burst
  and note it in `learnings.md`.

## Progress

- (none yet)
