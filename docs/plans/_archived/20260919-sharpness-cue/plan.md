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

# Sharpness cue: a relative score around the focus point

## Purpose

Spotting a missed focus or camera shake currently needs `Space` (the 1:1
view) on every file. This adds a per-file sharpness score, computed around the
camera's focus point (the frame center when none is recorded) from the JPEG
already read at scan time, stored in the folder index, and shown on the strip
**relative to the neighboring frames**. Absolute thresholds are unreliable
(edge content varies by subject), so the cue answers "which frame of this
burst is the sharpest", not "is this frame sharp".

Scope guard, first version only: no sort, no filter, no JpgFromRaw pass, no
settings toggle, no new key (`p` is reserved for pick; non-culling actions go
in the menu, and this version adds none). Everything comes from the embedded
preview JPEG; the RAW is never decoded or modified.

## Decisions taken before implementation

Taken from the numbers already in `README.md` ("Performance") and the
archived focus-check plan (`docs/plans/_archived/20260918-focus-check/`).
The user confirmed decision 1 (embedded preview). Reopen only with a reason;
the alternatives are in "Trade-offs and risks".

1. **Score the embedded preview, at scan time.** `scan::extract` already
   holds the 1616x1080 (α7 V) / 2112x1408 (M11-P) preview bytes from the
   bounded prefix read; scoring it adds one small grayscale decode and no IO.
   A JpgFromRaw crop would add a ~6MB ranged read and a 20-44ms row-bound
   partial decode per file to every first open (the scan is ~12ms/file
   today), and on a card reader the extra IO, not the CPU, is what would be
   felt. The preview catches gross misses and shake and ranks a burst; a
   JpgFromRaw refinement can come later as an on-demand pass reusing the same
   scoring function.
2. **The score is the variance of the Laplacian of the luma** over a fixed
   window centered on `partial::focus_point(preview_w, preview_h, focus)`
   (sensor -> preview scaling and the center fallback already live there),
   clamped to the image. One `f64` per file; no normalization, since it is
   only ever compared with neighbors.
3. **Relative means "against the strip neighbors"**: the cue for a cell is
   its score over the maximum score within a small window of adjacent cells
   in the *visible* (filtered) strip order, which is capture order for a
   single body. The cell holding that maximum is marked as the sharpest of
   its run. No burst detection by capture time in this version.
4. **Persisted in the SQLite index** as a `files.sharpness REAL` column,
   written by the same `write_batch` as the thumbnail, so reopening a folder
   costs nothing extra. Existing databases have their `files` table dropped
   and recreated (the v4 precedent) so every folder is rescanned once;
   `ratings` is kept.
5. **The cue is always shown** when a score exists; no toggle. If it turns
   out to be noise, a `View` menu check item is the place for one (not a key).

## Steps

- [x] Step 1: Core: `riffle_core::sharpness`, and `scan::Entry` carries the score
  - Done when:
    - `crates/core/src/sharpness.rs` (registered in `lib.rs`) has a pure
      function that, given a grayscale `&[u8]`, its width/height and a
      window rectangle, returns the variance of the 3x3 Laplacian over that
      window (interior pixels only; no padding), and a function that takes
      the preview JPEG bytes and an `Option<FocusLocation>`, decodes the
      preview to grayscale (`mozjpeg::Decompress::grayscale()`), places a
      fixed-size window (a `pub const`, initial value 256 px per axis; see
      risks) on `partial::focus_point(w, h, focus)` clamped into the image,
      and returns the score as `f64`
    - `scan::Entry` gains `sharpness: Option<f64>`; `scan::extract` fills it
      inside the same `catch_unwind` discipline as the thumbnail, and a
      failure to score is `None`, **not** a failed file (the thumbnail and
      metadata must still land). Existing callers (`crates/cli` `scan_dir`,
      `crates/app` `write_batch`) compile with the new field; `write_batch`
      ignores it until Step 2
    - Unit tests with synthetic images (mirror the gradient encoders in
      `decode.rs` / `partial.rs` tests): a constant image scores 0; a
      checkerboard scores higher than the same checkerboard box-blurred; a
      focus point near a corner yields a window clamped inside the image;
      `None` focus uses the center; the score depends only on the window
      (sharp content outside the window does not raise it); a preview that
      is not a JPEG yields `Err`, and through `extract` yields
      `sharpness: None` with a thumbnail still present
    - `riffle-cli scan` on `~/Downloads` (or any real folder) is run before
      and after on the same folder, same thread count, warm page cache, and
      both "per file on a worker" lines are recorded in `learnings.md` with
      those conditions. The number is a note, not a gate; only a per-file
      increase that changes the order of magnitude would reopen decision 1
    - `mise run ci` passes
  - Implementation approach:
    - Do not re-derive the sensor -> JPEG scaling; call
      `partial::focus_point`. The window is placed on the unrotated preview,
      like the crop; rotation is irrelevant to a variance
    - Decode the preview separately from `decode::thumbnail_jpeg` (which
      decodes at 2/8 scale); do not restructure the thumbnail path. If the
      grayscale full-scale decode measures high, `Decompress::scale(4)`
      (half size) is the knob to try, with the window halved; record what
      was chosen
    - Keep the Laplacian in integer arithmetic on `u8` with `i32`
      accumulators; use `f64` only for the mean/variance. No new crates
    - Leave `bench`, `crop`, `focusbox` in `crates/cli` untouched

- [x] Step 2: App: index schema v7 stores the score and `folder_entries` returns it
  - Done when:
    - `crates/app/src/index.rs`: `SCHEMA_VERSION = 7`; `files` gains
      `sharpness REAL`; `prepare` treats a v2-v6 database like v4 did for
      v2/v3: `DROP TABLE IF EXISTS files` then recreate, keeping `ratings`
      (and still applying the existing `ratings` `ALTER TABLE`s for the
      older versions). The doc comment above `SCHEMA_VERSION` gets a v7
      sentence in the same style
    - `write_batch` stores `entry.sharpness` (NULL when `None`); `entries()`
      reads it into a new `IndexedFile.sharpness: Option<f64>` (serialized
      as `null` when absent, like `focus`)
    - Tests in `index.rs`'s module, following the existing ones: a v6
      database (build it the way the existing migration tests build older
      versions) opens as v7 with its `ratings` rows intact and its `files`
      rows gone; a written entry's score round-trips through `entries()`;
      an entry with `sharpness: None` round-trips as `None`
    - `crates/app/ui/src/main.ts`'s `IndexedFile` interface gains
      `sharpness: number | null` (no behavior yet; keeps the type mirror
      honest)
    - `mise run ci` passes
  - Implementation approach:
    - Assumes Step 1 is merged
    - Follow the `focus_w`..`focus_y` handling in `write_batch` and
      `entries()` for the column plumbing; the error-row `INSERT` writes NULL
    - Do not touch `ratings`, `reconcile`, or the sidecar paths

- [x] Step 3: Frontend: the strip shows each cell's sharpness relative to its neighbors
  - Done when:
    - A new `crates/app/ui/src/sharpness.ts` exports a pure function that
      takes the scores of the visible files in strip order
      (`(number | null)[]`) and a window radius (a `const`, initial 2, i.e.
      up to 5 frames) and returns, per index, `null` (no score) or
      `{ ratio, best }`: `ratio` = score / max over the window's non-null
      scores (1 for the maximum), `best` = this index holds that maximum
      (ties: every tied index). A `sharpness.test.ts` (Vitest, like
      `exif.test.ts`) covers: a lone file is `best` with ratio 1; nulls are
      skipped and get `null`; the window is clamped at both ends; equal
      scores tie; a burst of 3 marks exactly the sharpest
    - `main.ts` keeps a `sharpness: Map<string, number>` filled from
      `folder_entries` rows (owned like `labels`, cleared with them); after
      `strip.setFiles(files)` in `refilter` (and where `refreshEntries`
      re-applies ratings), it computes the relative values over `files` and
      hands them to the strip with a new `strip.setSharpness(index, value)`
      that mirrors `setRating` (store per index, repaint the cell if
      on screen). `setFiles` clears the store like it clears `ratings`
    - `strip.ts` paints the cue on the cell without moving the existing
      badges: the rating stays top-right, the pick/reject dot top-left, the
      label tint on the name band. The cue is a small bar along one edge of
      the image box whose length is `ratio` and whose color changes for
      `best`; `style.css` gets the matching rules next to `.cell span.rating`
      / `.cell span.flag`. Exact placement is the implementer's call; it must
      not overlap the star badge
    - The meta pane shows the raw score of the current file in a
      `Sharpness` row (one decimal, or a fixed-precision integer; state which)
      so a user can see the absolute number the cue derives from. Nothing is
      added to the canvas
    - No new key, no menu item, no setting. `mise run ci` passes
    - **(manual, the user confirms)** in the running app on a real folder: a
      burst's sharpest frame carries the `best` color; a clearly missed
      frame's bar is visibly shorter than its neighbors; filtering the strip
      recomputes the cues over the visible files; a folder scanned by an
      older app version is rescanned once and then shows cues
  - Implementation approach:
    - Assumes Step 2 is merged
    - All async paths already run behind `folderToken` / `generation`;
      add no new counter (`docs/agents/tauri-app.md`, "one token, not one
      counter per feature")
    - Keep `sharpness.ts` free of DOM and Tauri so the test needs no mocks,
      as `exif.ts` is
    - The window is computed over `files` (the filtered view), not
      `allFiles`; say so in a comment where it is computed

- [x] Step 4: Documentation and measurements
  - Done when:
    - `README.md` (user-facing only): a **Sharpness cue** bullet in
      "Features" saying what the bar means (relative to the neighboring
      frames on the strip, sharpest of its run marked, computed around the
      focus point from the embedded preview, so it ranks a burst rather than
      judging a frame on its own, and does not replace the 1:1 view); the
      "Performance" section gains the Step 1 scan cost before/after **with
      its conditions** (folder, thread count, page cache state, n), following
      the existing tables; no phases, no verification logs
    - `CLAUDE.md` "Layout" mentions `crates/core/src/sharpness.rs` in the
      `riffle-core` parenthesis, in the existing style
    - `docs/agents/tauri-app.md` gains an entry only if a step hit something
      worth a rule (tagged Hit/Measured/Inferred per the file's convention);
      otherwise unchanged
    - `mise run ci` passes

## Trade-offs and risks

### Embedded preview vs JpgFromRaw crop (decision 1)

- **Preview (taken, confirmed by the user)**: no extra IO, one ~3-5ms
  grayscale decode per file inside the existing scan; catches gross misses and
  shake and ranks a burst. Misses a subtle front/back focus that only a 1:1
  view shows: the preview is a ~4.3x downsample of the α7 V frame, so a
  10-30 px blur circle at full resolution becomes 2-7 px, detectable
  relatively but not always.
- **JpgFromRaw crop**: the real sharpness, but per file a ~6MB ranged read
  plus a 20-44ms partial decode whose cost is set by the focus point's row
  (`docs/agents/tauri-app.md`, "A partial decode's cost is set by its row").
  At scan time that is roughly 3x the CPU and ~6x the IO of today's scan;
  on 5000 files, ~30GB of reads. As an on-demand pass (score the current file
  and its neighbors when the user lands on them) it avoids the scan cost but
  needs its own cache column, in-flight bookkeeping and a "not yet scored"
  state on the strip.
- Keep the scoring function generic over a grayscale buffer (Step 1) so a
  later JpgFromRaw variant is one new caller. If the preview cue proves
  useless on real bursts, the on-demand JpgFromRaw pass is the follow-up, not
  a scan-time one.

### Sliding window vs burst grouping (decision 3)

- **Sliding window over strip order (taken)**: trivial, works without
  capture times, reacts to the filter. Wrong across a burst boundary (the
  first frame of a new burst is compared with the tail of the old one) and
  when two bodies share a folder (the strip is file-name order; see the
  separate todo item on strip sorting).
- **Group by capture time** (frames within ~1-2 s form a run): the right
  unit, but needs `capture_time` + `subsec` parsing on the frontend, a gap
  threshold that is itself a guess, and leaves single frames with no
  neighbor at all.
- The pure function in `sharpness.ts` is the one place to swap in grouping
  later.

### Window size and the score's meaning

256 preview pixels (~1100 sensor px on the α7 V, ~1150 on the M11-P) is a
guess: large enough to hold an eye and some hair, small enough to stay on the
subject. Too large and the background's edges dominate; too small and a
focus point that sits on a flat cheek scores near 0 everywhere. Step 1 makes
it one `const`; if the manual check in Step 3 shows the cue tracking the
background, halve it. The variance of the Laplacian also rises with noise
(high ISO) and with contrast, which is why only the relative form is shown.

### Migration by drop-and-recreate (decision 4)

Every previously indexed folder is rescanned once (5.55s on the 5000-file
symlink folder in the README; real folders on a card are disk-bound). The
alternative, `ALTER TABLE ADD COLUMN sharpness REAL` plus a backfill pass
for NULL rows, avoids the rescan but needs a second code path that reads
previews outside the scan; not worth it for a cache column. `ratings` is
untouched either way, so no judgment is lost.

### Where the cue lives

A strip badge is taken because it needs no new key or menu entry and sits
next to the other per-file marks. A **sort** by score or a **filter**
("hide frames below X% of their run") would let the user act on the cue in
bulk; both are left out of the first version (a sort interacts with the
strip-order todo item; a filter needs a threshold, the very thing this cue
avoids). Add them once the cue has proven itself.

### Verification limits

GUI automation does not work on this machine
(`docs/agents/tauri-app.md`); the Step 3 on-screen checks are the user's
to confirm and are reported as unverified until then. The scan cost
before/after is a single-machine, warm-cache number and is written with its
conditions, per the same guide.

## Progress

- (2026-09-19) Step 1 complete
- (2026-09-20) Step 2 complete
- (2026-09-20) Step 3 complete
- (2026-09-20) Step 4 complete
