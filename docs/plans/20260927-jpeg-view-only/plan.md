<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Open and preview JPEG-only folders in the strip

## Purpose

`File > Sequence JPEG Timestamps…` writes `<folder>-sequenced/`, a folder of
JPEGs, and today the only way to check its order and capture times is another
app: the strip lists ARW / DNG only, and the scan, the index, the sidecars and
the focus cue all assume RAW. This work lets a folder that holds only JPEGs
(`.jpg` / `.jpeg`, case-insensitive) open in the strip with thumbnails, the
preview and the meta pane's EXIF rows, ordered by capture time, so the
sequenced output can be checked inside Riffle.

The scope is **view-only**, as decided in `todo.md` ("App: open and preview
JPEG-only folders in the strip") and the archived
[`20260927-jpeg-view-only-todo`](../_archived/20260927-jpeg-view-only-todo/plan.md)
plan: rating, the pick / reject flag, the color label, sidecar (XMP / `.dop`)
writes, the focus cue and face detection, and the sharpness score do not apply
to JPEGs. Those actions are no-ops or disabled on a JPEG folder and no sidecar
is ever written next to a JPEG.

Decisions fixed here (alternatives under "Trade-offs and risks"):

- **JPEG-only folders only.** A folder is listed as JPEGs when it holds no
  ARW / DNG at all; a folder with any RAW keeps listing the RAWs only, exactly
  as today. So a RAW+JPEG shooting folder never doubles its strip.
- **Ordering reuses the strip's `capture` sort.** The sort key `capture` in
  `crates/app/ui/src/sort.ts` already orders by `DateTimeOriginal`, then
  sub-second, then file name, with files that have no capture time last. A
  JPEG-only folder opens in that order; no new ordering rule and no filename
  fallback beyond the one the sort already has.
- **The RAW pipeline is unchanged**: `crates/core/src/arw.rs` is not touched
  and `EXTRACTOR_VERSION` is not bumped, because what `extract` produces for
  an ARW / DNG stays byte-identical.

## Steps

- [x] Step 1: Core: read a JPEG's Exif and thumbnail through the existing `extract` / `read_preview` / `read_metadata` entry points
  - Done when:
    - `crates/core/src/jpeg.rs` (new, exported from `lib.rs`) parses a JPEG
      file's APP1 Exif into the same `arw::Arw`-shaped result the app already
      consumes: `orientation` (0x0112 in IFD0, default 1) and a `Shot` with
      `make`, `model`, `capture_time` (0x9003), `subsec` (0x9291),
      `lens_model` (0xa434), `exposure_time` (0x829a), `f_number` (0x829d),
      `estimated_f_number` from `ApertureValue` (0x9202, cleared when
      `f_number` is present, as `arw::exif` does), `focal_length` (0x920a),
      `exposure_bias` (0x9204) and `iso` (0x8827). Every MakerNote field,
      `focus`, `focus_frame` and `focus_distance_mm` stay `None`.
    - Both TIFF byte orders (`II` and `MM`) are handled. A JPEG with no APP1
      Exif segment, or a malformed one, is **not** an error: it yields
      orientation 1 and a default `Shot`, so a JPEG without Exif still shows
      a thumbnail and sorts last by name. A file that is not a JPEG (no SOI)
      is an error.
    - `scan::is_jpeg_file(path)` exists next to `is_raw_file` (case-
      insensitive `.jpg` / `.jpeg`); `sequence.rs`'s private `is_jpeg` is
      replaced by it.
    - `reader::read_preview`, `reader::read_full` and `reader::read_metadata`
      dispatch on `is_jpeg_file`: for a JPEG the whole file is the preview
      and the full-resolution JPEG (so `preview`, `focus_crop` and `metadata`
      in the app work without their own branches), and `read_metadata` reads
      only as much as the Exif needs (a bounded head like `HEAD_LIMIT`, with
      the same whole-file fallback).
    - `scan::extract` on a JPEG returns an `Entry` with the parsed
      orientation and `Shot`, a thumbnail, and `sharpness: None`; it runs
      neither `detect_around` nor `score_preview`. `extract_faces` is not
      changed (a JPEG row never reaches it; see Step 2).
    - The JPEG thumbnail is unrotated, baseline, `THUMBNAIL_QUALITY`, and
      its long edge is close to the ARW thumbnail's (404 px): a new
      `decode::thumbnail_jpeg_near` picks the DCT scale `n/8` from the source
      size and box-averages to 404 px, used only for JPEG files;
      `thumbnail_jpeg` stays at the fixed `2/8` scale so DNG thumbnails do not
      change. Measure a 24 MP JPEG's thumbnail bytes and decode time and
      record them in `learnings.md`; if `1/8` is still far above 404 px and
      the bytes are well above the ~19 KB ARW figure, a further downsample
      after the decode is acceptable, but decide that on the measurement, not
      up front.
    - Tests (`jpeg.rs` unit tests, building JPEGs with `mozjpeg` as
      `scan.rs`'s tests do and splicing a hand-built APP1 after SOI, both
      `II` and `MM`): each field above; sub-second inline (≤ 4 bytes) and at
      an offset; no APP1 gives the defaults; a non-JPEG is an error; a
      truncated segment is the defaults, not a panic. `scan.rs` tests:
      `extract` on a `.jpg` yields a thumbnail and `sharpness: None`;
      `extract_all` over a mixed list of `.ARW` fixtures and `.jpg` files
      delivers every index. `reader.rs` tests: `read_preview` / `read_full`
      on a JPEG return the file bytes. Existing ARW tests unchanged.
    - `cargo test -p riffle-core`, `cargo clippy --all-targets -- -D warnings`
      and `mise run ci` pass.
  - Implementation approach:
    - Do not generalize `arw.rs` to big-endian: `sequence.rs` already has an
      endian-aware, bounds-checked walker (`find_exif_tiff`, `Tiff`, `Entry`,
      `ifd_entries`, `u16` / `u32` / `bytes`). Make those `pub(crate)` (or
      move them into a small shared module the two files import) and add the
      readers `jpeg.rs` needs (ASCII of any length, RATIONAL, SHORT / LONG
      integer, honoring the inline-when-≤4-bytes rule). Keep the reading
      tolerant: a bad entry yields `None` for that field, only a missing SOI
      is `Err`.
    - `Shot` is `crates/core/src/arw.rs`'s struct; reuse it as is so
      `crate::exif::exif(&shot)` in the app, `write_batch` and `Metadata`
      need no new type.
    - `EXTRACTOR_VERSION` is documented as "what `extract` produces": verify
      by reading the diff that the ARW / DNG path is untouched and say so in
      the PR body; no bump.

- [x] Step 2: App backend: list, scan and serve a JPEG-only folder; refuse culling on JPEGs
  - Done when:
    - Step 1 is merged.
    - `read_listing` in `crates/app/src/commands.rs` lists the folder's
      JPEGs (sorted by file name like the RAWs) **only when the folder holds
      no ARW / DNG**, and in that case returns **no sidecars** whatever the
      format, so `reconcile_sidecars_of` neither parses a `foo.xmp` next to
      `foo.jpg` nor queues anything for the writer. `list_arw`, `scan_folder`
      (including the `AppListing` reuse), `reconcile`, `run_scan`,
      `folder_entries`, `thumbnail`, `last_viewed` / `set_last_viewed` and
      the watcher then work for a JPEG folder unchanged: `write_batch` stores
      the JPEG rows (`capture_time`, `subsec`, `exif` columns, `thumb`,
      `sharpness NULL`, `focus_w NULL`) and `folder_entries` returns them.
    - The second pass does not run on JPEGs: `faces_todo` already marks rows
      with `focus_w IS NULL` done at `FACES_VERSION`; a test in `index.rs`
      pins that a scanned JPEG row is not in `faces_todo` and ends with
      `eye_focus NULL`.
    - `set_rating` refuses a path that is not a RAW file (`is_raw_file`) with
      an error string before touching the index or the writer, so no client
      (the strip, MCP) can create a `ratings` row or a sidecar for a JPEG.
      `faces_of` refuses a non-RAW path the same way (face detection is out
      of scope, and a full-size JPEG would make it slow). `trash::plan`
      already refuses non-RAW paths; keep it.
    - `crates/app/src/folders.rs` `list` counts the JPEGs when the folder
      holds no RAW (same rule as the listing), so the tree shows a count for
      a `-sequenced` folder; the `raw_count` field name and the frontend stay
      as they are.
    - `preview` on a JPEG returns the file with its Exif orientation in the
      existing `PREVIEW_KIND_JPEG_V1` envelope; `metadata` returns the EXIF
      rows and `None` for every MakerNote row; `focus_crop` returns a crop
      centered on the image (no AF point). Measure the page time of a 24 MP
      JPEG with `Timing logs` on (`page invoke= decode= total=`) and record
      it in `learnings.md`; if it is far above an ARW's, add a DCT-scaled,
      **unrotated** re-encode capped at a long edge (e.g. 2048 px) in the
      `preview` path for JPEGs, keeping the orientation in the header. Decide
      on the measurement.
    - Tests (`commands.rs`, `folders.rs`, `index.rs`; app tests can build a
      JPEG with the `mozjpeg` dev-dependency and splice an APP1 as Step 1's
      tests do): `list_arw_in` on a JPEG-only folder returns the `.jpg` /
      `.JPEG` files sorted by name and no `.png` / `.txt`; on a mixed folder
      it returns the RAWs only; the JPEG listing carries no sidecars even
      with a `foo.xmp` and `foo.jpg.dop` present, and `reconcile_listed` over
      it queues nothing and leaves no `ratings` row; `run_scan` over a JPEG
      folder fills `entries` with `capture_time`, `has_thumb`, `sharpness:
      None`, `focus: None`, and `faces_todo` is empty afterwards; `set_rating`
      / `faces_of`'s inner functions refuse a `.jpg` path (extract the check
      into a testable helper if the command itself is not unit-testable);
      `folders::list` counts JPEGs only when no RAW is present (extend
      `raw_count_counts_only_arw_and_dng_files` with a JPEG-only case).
    - `mise run ci` passes.
  - Implementation approach:
    - Follow `docs/agents/tauri-app.md`: blocking IO stays in
      `spawn_blocking`; `read_listing` keeps its one-`read_dir` shape and its
      `file_type()`-based file test (`crate::folders::is_file`).
    - Keep the JPEG-or-RAW rule in one place (a helper both `read_listing`
      and `folders::list` call, or a shared predicate over the listed
      entries) so the tree count and the strip never disagree.
    - `scan_folder` logs `raws=`; either leave the label or log the kind;
      do not rename events or payload fields.
    - Between this step and Step 3 a judgment key on a JPEG folder shows the
      backend's refusal in the status line instead of doing nothing; that is
      the intended safety net, and Step 3 turns it into a no-op.

- [x] Step 3: Frontend: view-only mode for a JPEG folder
  - Done when:
    - Step 2 is merged.
    - A pure helper (new `crates/app/ui/src/viewonly.ts`, with
      `viewonly.test.ts`) decides `viewOnly` from the listed paths: true when
      the list is non-empty and every path ends in `.jpg` / `.jpeg`
      (case-insensitive). `main.ts` computes it in `openDirectory` (and
      after a `resync` listing) and keeps it in one `let`.
    - In view-only mode:
      - Every judgment action (`rate1`–`rate5`, `pick`, `reject`, `unflag`,
        `clear`, the seven colors, `clearlabel`, `clearall`, `rejectRest`)
        is a no-op that neither calls `set_rating` nor pushes an undo entry;
        `undo` / `redo` have nothing to do. Navigation, selection, `focus`
        toggle, `zoom`, `compare`, `open`, the panel toggles and the filter
        menu keep working.
      - The strip's right-click menu shows only `Select All`:
        `contextMenuGroups` takes a view-only flag (or `main.ts` filters the
        groups) and `context.test.ts` covers it.
      - `drawFaceMarks` requests no faces (`faces_of` is never invoked), and
        the focus mark stays absent because `focus` is `null`.
      - The strip is ordered by capture time: the effective sort key is
        `capture` regardless of the persisted `sort_order`, the sort menu is
        disabled (or hidden) like the culling keys, and the persisted key is
        neither read for this folder nor overwritten, so the next RAW folder
        opens in the user's own order.
      - The status line names the mode once per folder open (e.g. `JPEG
        folder: view only`) through the existing `note` / `setStatus`
        mechanism, so a silent key is not mistaken for a bug.
      - The MCP `set_judgment` request is refused by `companion.ts`'s
        `setJudgment` with an error such as `culling does not apply to a
        JPEG folder` (the `ViewApi` gains a `viewOnly()` accessor or the
        `ViewState` carries the flag; `companion.test.ts` covers it).
        `get_view`, `get_photo`, `get_preview`, `show_photo`,
        `select_photos` and `set_view` keep working.
    - The resume-at-last-viewed, bursts (each sequenced file is its own
      burst) and the filter menu need no change; verify by hand that a
      `-sequenced` folder opens at its last viewed file in capture order.
    - `mise run ci` passes (`vp check`, `vp test`, fmt).
  - Implementation approach:
    - Keep `main.ts` to wiring: the mode decision, the menu-group filter and
      the companion refusal live in pure modules with tests, as `context.ts`
      / `companion.ts` / `resume.ts` do.
    - Gate `judge` / `record` / `rejectRest` at one point (e.g. `judge`
      returns 0 and `rejectRest` returns early when `viewOnly`) rather than
      per action, and make `runAction` still return `true` for a gated key
      so the key does not fall through to the browser.
    - The sort gate: `ordered()` uses `viewOnly ? "capture" : sortKey`; the
      menu's disabled state follows `setSortMenuOpen` / the existing
      `[hidden]` / `disabled` patterns in `index.html` and `style.css`.
      Remember the WebKitGTK and `[hidden]` notes in
      `docs/agents/tauri-app.md`.

- [x] Step 4: Document the JPEG-only folder and close the todo item
  - Done when:
    - `README.md` and `README.ja.md` (in sync) say, next to the Sequence JPEG
      Timestamps feature and in "RAW formats and cameras", that a folder
      holding only JPEGs opens view-only: thumbnails, preview, EXIF rows,
      capture-time order, no rating / flag / label / sidecar / focus cue.
    - `docs/usage.md` gets a Features entry (next to **Sequence JPEG
      Timestamps…**) with the full behavior: the JPEG-only rule (a folder
      with any ARW / DNG lists the RAWs only), the extensions, capture-time
      order with the sort menu off, files without Exif last by name, which
      keys and menu items are off, the MCP `set_judgment` refusal, the tree
      count, and that no sidecar is read or written; the Folders entry's
      "how many RAW files it holds" gains the JPEG-only case.
    - `CLAUDE.md`'s Layout paragraph mentions `crates/core/src/jpeg.rs` and
      `crates/app/ui/src/viewonly.ts`.
    - The `todo.md` item "App: open and preview JPEG-only folders in the
      strip" is removed (its one checkbox is done by Steps 1–3).
    - `mise run lint` (including lychee) passes.

## Trade-offs and risks

- **Mixed folders (chosen: JPEGs only when no RAW is present).** The
  alternative, always listing JPEGs alongside RAWs, would double the strip in
  a RAW+JPEG shooting folder and make culling keys act on a list where half
  the files cannot take a judgment; it also needs a per-file (not per-folder)
  view-only rule. If the caller wants JPEGs visible in mixed folders, the
  per-file gate is `is_raw_file(path)` in `judge` and `set_rating` already
  refuses per path, but the strip's `viewOnly` flag becomes per file and
  the `-sequenced` use case gains nothing.
- **Ordering (chosen: the existing `capture` sort).** It orders by
  `DateTimeOriginal`, sub-second, then plain string file name; files without
  a capture time come last by name. `crates/core/src/sequence.rs` uses a
  natural (`natord`) file-name tie-break instead. The two differ only when
  two files share the same second and sub-second and have names whose
  numeric parts differ in width, which a `-sequenced` output (unique
  seconds) never has; camera names are fixed-width. Not replicating `natord`
  in `sort.ts` keeps ARW ordering unchanged. If the caller wants the same
  tie-break as the sequence preview, switching `compareStrings` on names to
  a natural compare is a separate, RAW-visible change.
- **Capture order forced in view-only (chosen) vs. leaving the sort key to
  the user.** Forcing it makes "check the sequenced order" work on the first
  open; the cost is a disabled sort menu in JPEG folders. Leaving the user's
  key means a `name` sort shows the folder in filename order, which for a
  two-body export is exactly the order the feature exists to correct.
- **Preview payload.** Sending the full JPEG (often 24 MP) to the decode
  worker is the simplest path and keeps zoom-free viewing at full detail,
  but paging may be slower than an ARW's 1616-px preview and Linux's 6 MP
  worker limit applies. Step 2 measures before deciding on a backend
  downscale; either way the envelope and orientation handling stay the same.
- **Thumbnail size.** `thumbnail_jpeg`'s fixed `2/8` scale assumes the
  preview tier; a 24 MP JPEG at `1/8` is still ~750 px and a few times the
  ARW thumbnail's bytes. Step 1 measures; a post-decode downsample is
  allowed if needed. Index eviction (`MAX_BYTES`) bounds the total anyway.
- **`raw_count` keeps its name** though it counts JPEGs in a JPEG-only
  folder; renaming would touch the IPC shape, `tree.ts` and its tests for a
  label that is only a number. Documented in usage instead.
- **No `EXTRACTOR_VERSION` bump.** Correct only if the ARW / DNG output is
  untouched; Step 1 verifies that by diff. Changing `thumbnail_jpeg`'s
  scale must keep the 1616-px case at `2/8` (404 px), or every RAW thumbnail
  changes and a bump is due.
- **Face detection through `read_preview`.** Once `read_preview` returns a
  JPEG's bytes, any caller that runs detection on it (`faces_of`,
  `extract_faces`) would work on a full-size image; the plan guards
  `faces_of` in the backend and the request in the frontend, and relies on
  `faces_todo`'s `focus_w IS NULL` rule for the scan. A future caller of
  `read_preview` must keep that in mind.
- **Between Step 2 and Step 3** a judgment key on a JPEG folder surfaces the
  backend refusal as a status-line error rather than a silent no-op. No
  sidecar can be written in that window.
- **Not covered:** subfolders, HEIC / PNG / TIFF, JPEGs inside a RAW folder,
  writing anything to a JPEG.

## Progress

- (2026-09-27) Step 1 complete
- (2026-09-28) Step 2 complete
- (2026-09-28) Step 3 complete
