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

# Draw the Sony AF frame in the focus mark

## Purpose

The focus mark (`f`) draws only a crosshair at the recorded focus point,
because when it was written the app read only Sony `FocusLocation`. Sony
`FocusFrameSize` (0x2037) is now parsed into `Shot.focus_frame`
(`crates/core/src/arw.rs`) and used by the sharpness score, but it never
reaches the index or the viewer. Drawing the AF frame the camera actually
used, centered on the point, shows how large the area the body focused on was
(a face frame versus a spot), which a bare crosshair cannot. At the same time
the mark should stay silent on manual-focus shots, where the recorded
location is not trusted (the stance `crates/core/src/sharpness.rs`
already takes with `trusted_focus`).

Decided direction:

- Sony ARW with a valid frame: rectangle of the recorded size around the
  focus point plus the crosshair at its center. Point without a frame size
  (SIGMA BF): crosshair only. No AF point (SIGMA fp L, Leica M11-P): nothing.
- MF shots (Sony `FocusMode` == 0): no mark at all. DMF (6) is not MF. Bodies
  that write no `FocusMode` (non-Sony, older Sony) are treated as not MF.
- Face/eye-tracked shots use the same style as any other shot.
- Only the normal single view (`drawFocusMark` from `draw()`) changes. The
  1:1 view (`drawZoom`) and Compare (`drawCompare`) are untouched, and the 1:1
  view keeps centering on the focus point even for MF shots, as today.
- MF is stored as a boolean column, not the raw `FocusMode` code.
- `riffle-cli focusbox` uses the recorded frame when present, falling back to
  its current hardcoded size, so it stays a faithful mirror of the app.

## Steps

- [x] Step 1: Store the AF frame size and the MF flag in the index, draw the frame in the focus mark, and document it
  - Done when:
    - `crates/core`: a public helper says whether a `Shot` was taken in
      manual focus (e.g. `pub fn manual_focus(shot: &Shot) -> bool` next to
      `trusted_focus` in `crates/core/src/sharpness.rs`, or a method on
      `Shot` in `arw.rs`), returning `shot.focus_mode == Some(0)`, and
      `trusted_focus` uses it so the two cannot drift. Unit test: MF → true,
      AF-C (3) / DMF (6) / `None` → false.
    - `crates/app/src/index.rs`: the `files` table gains three columns for the
      frame width, frame height and the MF flag (e.g. `frame_w INTEGER`,
      `frame_h INTEGER`, `manual_focus INTEGER NOT NULL DEFAULT 0`);
      `write_batch` writes `shot.focus_frame` and the MF helper into them
      (NULL / 0 in the error-row insert); `entries` reads them back on the
      existing `Focus` struct (serialized inside the `focus` entry the
      frontend already receives). `SCHEMA_VERSION` is 13 and its doc comment
      describes v13 (a v10 to v12 database gains the columns in place with
      `ALTER TABLE` and keeps `files`, `ratings` and `folders`); `12` is
      added to the `prepare` whitelist; the new `ALTER` guard is keyed
      `(10..13).contains(&version)` (files is dropped and recreated with the
      columns for `version < 10`); the existing extractor guard stays
      `(10..12)`. `EXTRACTOR_VERSION` is 2 so every existing row (which has
      NULL frame / default MF) is re-extracted once. Existing migration tests
      that assert `user_version == 12` are updated to 13, and fixtures that
      fake v10 / v11 / v12 also drop the three new columns (a fixture built
      with `open` has the current schema; see the guide pitfall).
    - Index tests: a round-trip test writes an `Entry` with
      `focus_frame: Some(FocusFrame { .. })` and `focus_mode: Some(0)` and
      reads back the frame size and `manual_focus == true`; an entry with no
      frame reads back `None` / `false`. A v12-fixture migration test shows
      the columns are gained in place and `files` / `ratings` are kept.
    - `crates/app/ui`: the `Focus` interface in `src/main.ts` gains the
      frame size (nullable) and the MF flag. The geometry of the mark is
      extracted into a pure function in a new `src/focus.ts` (e.g.
      `focusMark(focus, drawWidth, drawHeight)` returning `null` for MF /
      no focus, else the point in centered unrotated preview coordinates plus
      an optional rectangle `{ x, y, width, height }`), tested in
      `src/focus.test.ts`: MF → null; point without frame → point and no
      rect; point with frame → rect centered on the point with
      `width * drawWidth / sensor_w` and `height * drawHeight / sensor_h`.
      `drawFocusMark` calls it and strokes the rectangle with the same
      two-pass style as the crosshair (black 4px outline under `#3f3` 2px),
      inside the same rotation transform, so every orientation is right
      without orientation-specific code. `drawZoom`, `placeholderRect` and
      `drawCompare` are unchanged (`ZoomFocus` is structurally compatible
      with the widened `Focus`).
    - The comment above `drawFocusMark` no longer claims the tag records only
      a point; it says the frame is drawn when `FocusFrameSize` is valid,
      the crosshair alone otherwise, and nothing for MF.
    - `riffle-cli focusbox` (`crates/cli/src/main.rs`) uses
      `shot.focus_frame` when present instead of the hardcoded
      `frame: u32 = 219`, falling back to 219 otherwise.
    - `docs/usage.md` ("Focus mark", line ~25) and `README.md` (line ~62)
      describe the new behavior: rectangle of the recorded AF frame plus
      crosshair on Sony bodies that record it, crosshair only when only a
      point is recorded (SIGMA BF), nothing without an AF point (M11-P) or on
      manual-focus shots. `CLAUDE.md` does not describe the mark; only touch
      it if the `index.rs` description there should mention the new columns
      (it currently does not list columns, so probably not).
    - `mise run ci` passes.
  - Implementation approach:
    - Keep the MF decision in Rust at extraction time (one column), not in
      the frontend, so the frontend never sees `focus_mode` codes. Do not
      null out `focus` for MF rows: the 1:1 view and `focus_crop` must keep
      their current behavior (`focus_crop` reads the ARW directly anyway).
    - Row values: `frame_w` / `frame_h` are NULL when `focus_frame` is
      `None`; read them back as an `Option` pair the way `focus_w` /
      `focus_h` are already zipped in `entries`.
    - Follow `docs/agents/tauri-app.md` "Bump `EXTRACTOR_VERSION`, not
      `SCHEMA_VERSION`, when extraction output changes" and the stranded
      guard section: here both bump, because the layout changes (schema)
      and existing rows must be refilled (extractor). Audit every existing
      `!= SCHEMA_VERSION` guard while bumping.
    - Update the `SCHEMA_VERSION` doc comment and the `EXTRACTOR_VERSION`
      doc comment (it says "starts at 1"; note 2 = AF frame and MF flag).
    - Frontend style constants `FOCUS_MARK_ARM` / `FOCUS_MARK_GAP` stay; the
      rectangle uses no gap. Its stroke sits in CSS pixels because
      `draw()` already applied `scale(dpr, dpr)`.

## Trade-offs and risks

- **Schema bump is unavoidable.** New columns change the table layout, so
  `SCHEMA_VERSION` goes to 13 as well as `EXTRACTOR_VERSION` to 2 (in-place
  `ALTER TABLE`, keeping `files`, `ratings`, `folders`). The extractor bump
  then does the refill. Doing the refill through the schema bump alone
  (dropping `files`) would throw away thumbnails for nothing.
- **Risk: migration test fixtures.** Faking v10/v11/v12 with a fixture
  built by `open` requires dropping the three new columns too, or the
  in-place `ALTER` fails and the cache is discarded (the exact failure
  recorded in `docs/plans/_archived/20260924-index-extractor-version/learnings.md`).
- **Risk: frame size semantics.** `FocusFrame` is in the same sensor
  coordinates as `FocusLocation` (per `arw.rs` docs and `sharpness.rs`'s
  `frame_side`); if a body writes a frame larger than the sensor or an
  oddly sized one, the rectangle simply follows the data. No clamping is
  planned.

## Progress

- 2026-09-24: Step 1 done (SCHEMA_VERSION 13 / EXTRACTOR_VERSION 2, AF frame + MF flag in the index, frame drawn in the focus mark, focusbox uses the recorded frame)
