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

# Docs batch: parsing and inference guides, the threshold-verification note, camera-neutral usage bullets

## Purpose

Four documentation items have been sitting in `todo.md` after their features
shipped: the practice of measuring a platform workaround's threshold on the
production code path (learned the hard way by the 6 MP Linux preview limit),
a guide to the RAW MakerNote / TIFF parsing quirks in `crates/core/src/arw.rs`
(Sony, Leica, Sigma, and the count-1 `SHORT` padding), a guide to running the
YuNet model through `tract-onnx` as `crates/core/src/faces.rs` does, and the
camera names still left in three `docs/usage.md` bullets after the READMEs
went camera-neutral. Writing them now means the next agent touching the
parser or the detector reads the pitfalls before repeating them, and
`docs/usage.md` describes behavior by what a camera records, as `README.md`
already does. Docs only; no code changes.

The four `todo.md` items this closes out (their headings, for the wrap-up's
`todo-curator`):

- `### Docs: record the practice of verifying platform-workaround thresholds on the production code path`
- `` ### Docs: write a guide for RAW metadata parsing (`docs/agents/raw-metadata-parsing.md`) ``
- `` ### Docs: write a guide for tract-onnx inference (`docs/agents/tract-onnx-inference.md`) ``
- `### Docs: docs/usage.md still names cameras in the Focus mark, Sharpness cue and Bursts bullets`

`todo.md` itself is not edited in a step; the develop wrap-up removes the
items.

## Decisions

- **Guide linking (user decision, 2026-09-28)**: the new guides are linked
  only from inside `docs/agents/` (cross-links); `CLAUDE.md` is not edited.
  Agents already find guides through "read the guides under `docs/agents/`"
  in `.claude/agents/{implementer,planner,local-review-reviewer}.md`.
- **Threshold practice location**: a new item under `## Verification` in
  `docs/agents/tauri-app.md`, with a pointer from the WebKitGTK transferred
  `ImageBitmap` Hit item.
- **`usage.md` scope**: only the Focus mark, Sharpness cue and Bursts bullets.
  The Maker note and sort bullets (per-vendor fields) and the "406
  hand-labeled α7 V frames" sentence (a measurement condition) stay as they
  are.

## Steps

- [ ] Step 1: Write the two new `docs/agents/` guides and the threshold-verification note, and cross-link the guides
  - Done when:
    - `docs/agents/raw-metadata-parsing.md` exists and covers, each checked
      against the current `crates/core/src/arw.rs`: (a) the Sony MakerNote
      gate in `maker_note_ifd` (skips only when the note lacks the `SONY`
      header *and* `Make` is present and not `SONY`-prefixed, so a missing
      `Make` still gets the Sony parse and a non-Sony synthetic fixture must
      set `Make`); (b) a `SubIFDs` entry with `count == 1` holds the IFD
      offset inline, not an offset to an array (and a synthetic-TIFF SubIFDs
      fixture needs two entries to exercise the array path); (c) the Leica
      MakerNote layout (`LEICA\0` + `02 00`, little-endian IFD at note offset
      8, `FocusDistance` tag 0x0304 LONG in millimeters), that `ApertureValue`
      (APEX) converts as `2^(AV/2)` and only when `FNumber` is absent, and
      that a "non-Sony note is skipped" fixture needs a valid empty IFD, not
      a `0xffff` sentinel count; (d) Sony `FocusFrameSize` (0x2037) arrives
      as `UNDEFINED[6]` on real α7 V files and the parser remaps it to
      `SHORT[3]` while still accepting `SHORT[3]`; (e) a count-1 `SHORT`
      entry's unused high 16 bits can carry nonzero per-file garbage (SIGMA
      fp L DNGs) and `integer()` masks with `& 0xFFFF`; (f) the Sigma
      conventions: a MakerNote entry whose value fits in the entry
      (`count <= 4` for a `SHORT[2]`, generally `<= header_len`) is inline,
      so the inline check precedes the offset/range check (`count <=
      SIGMA_HEADER_LEN` in `sigma_af_point`), and `Make` differs by body
      within one vendor (`Sigma`/`Sigma BF` vs `SIGMA`/`SIGMA fp L`), so the
      gate is a case-insensitive `SIGMA` prefix on `Make` plus an exact
      `Model`. Measurements are linked, not duplicated: Leica ->
      `docs/plans/_archived/20260918-leica-dng-support/learnings.md`
      (Steps 1 and 3), eye-AF ->
      `docs/plans/_archived/20260922-sony-eye-af-window/learnings.md`
      (Step 1), padding ->
      `docs/plans/_archived/20260924-tiff-short-padding/learnings.md`
      (Step 1), Sigma ->
      `docs/plans/_archived/20260924-sigma-bf-af-point/learnings.md`
      (Step 1).
    - `docs/agents/tract-onnx-inference.md` exists and covers, as
      `crates/core/src/faces.rs` does it: `with_ignore_value_info(true)` /
      `with_ignore_output_shapes(true)` for a model whose fixed-size shape
      annotations (640 px) do not match the chosen input (320 px), and
      `with_input_fact` to pin the input; finding outputs by outlet label
      (`model.outlet_label`, the ONNX names `cls_8` / `obj_8` / `bbox_8` /
      `kps_8` ...) rather than the tract node names (`Sigmoid_85`, ...);
      the tract 0.23 API (`into_optimized()?.into_runnable()?` giving
      `Arc<TypedRunnableModel>`, `run(tvec!(..))`, outputs read with
      `try_as_plain_ram()?.as_slice::<f32>()`); `default-features = false`
      on `tract-onnx` (drops `tract-transformers`, smaller binary and
      build); the input layout (BGR NCHW, raw 0..255 floats, fixed square
      so one plan serves every file); feeding an upright image (`upright_rgb`,
      `to_upright`) and mapping detections back to stored coordinates
      (`to_stored`, orientation 3 handled as a half turn because
      `decode::apply_orientation` only rotates 6/8); and sharing the built
      plan across rayon workers through a `OnceLock` (`detector()` caches a
      `Result<Detector, String>` so a build failure is not retried per file).
      It links `docs/plans/_archived/20260922-face-aware-sharpness/learnings.md`
      for the measurements (binary size, timings).
    - `docs/agents/tauri-app.md` records the practice under `## Verification`:
      verify a platform workaround's numeric threshold on the exact production
      code path before shipping it, instead of taking a quoted or secondhand
      number, with the 6 MP Linux limit as the case (#383 shipped 12 MP from
      "12.3 MP drew, 16 MP did not"; the MiniBrowser bisect on the real path,
      a worker `createImageBitmap` with resize options, the bitmap
      transferred, `drawImage` to a canvas, the center pixel read, results
      POSTed to a local Python HTTP server, found ~6.87 MP by pixel count,
      shape independent). The existing WebKitGTK "draws a large transferred
      `ImageBitmap` transparent" item gets one sentence pointing at it. Source
      line links
      `docs/plans/_archived/20260924-linux-preview-pixel-limit-6mp/learnings.md`.
    - Cross-links inside `docs/agents/` only: `tauri-app.md`'s "Sony MakerNote
      fields: verify each tag's type ..." item points at
      `raw-metadata-parsing.md`, and the RAW and tract guides link each other
      where relevant (e.g. the orientation mapping). `CLAUDE.md` is not
      edited.
    - `mise run ci` passes (lychee `--include-fragments` resolves every
      link and `#fragment` in the new and edited files).
  - Implementation approach (as far as it is known):
    - Follow `docs/agents/tauri-app.md`'s shape for both guides: a first
      paragraph saying when to read it, per-item `###` headings tagged
      `(Hit)` / `(Measured)` / `(Inferred)`, a short "Why" / "What broke" /
      rule structure, and a `Source:` line linking the archived learnings
      (relative to `docs/agents/`, e.g.
      `../plans/_archived/20260918-leica-dng-support/learnings.md#step-1`).
      Every item here was actually hit, so most tags are `(Hit)`.
    - Do not restate the measured numbers (exiftool comparisons, offsets,
      timings); link the learnings sections instead.
    - Verify each claim against the code before writing it (in
      `crates/core/src/arw.rs` around `maker_note_ifd`, `integer`, the
      `focus_frame` match in `exif`, the `TAG_SUB_IFDS` branch of `parse`,
      `leica_focus_distance`, `sigma_af_point`, and in
      `crates/core/src/faces.rs` `build` / `detector` / `detect` /
      `to_stored` / `upright_rgb`). `crates/core/src/reader.rs` has no
      maker-specific parsing despite the todo's `{arw,reader}.rs` wording;
      name `arw.rs` as the home.
    - Fragments: `## Step 1` slugs to `#step-1`; the face-aware-sharpness
      learnings' Step 1 heading has a suffix, so link that file without a
      fragment unless the slug is confirmed with lychee locally.

- [ ] Step 2: Reword the Focus mark, Sharpness cue and Bursts bullets in `docs/usage.md` camera-neutrally
  - Done when:
    - The Focus mark bullet (`docs/usage.md` around lines 124-145), the
      Sharpness cue bullet (around 385-398) and the Bursts bullet (around
      399-415) name no camera or body ("Sony", "SIGMA BF", "M11-P",
      "Leica", "a Sony body", "Leica DNG", "Leica files") and describe the
      behavior by what the camera records: an AF frame plus point, a point
      only, no AF point / manual focus; face tracking recorded or not; a
      sub-second capture time recorded or not. The wording follows
      `README.md`'s bullets (lines 87-119) without copying them verbatim
      where `usage.md` carries more detail. The "406 hand-labeled α7 V
      frames" measurement-condition sentence stays.
    - Each of the three bullets keeps its `[What the camera records](./cameras.md)`
      link.
    - Nothing else in `docs/usage.md` changes (the Maker note and sort
      bullets are out of scope).
    - `mise run ci` passes.
  - Implementation approach (as far as it is known):
    - Independent of Step 1.
    - No translation of `docs/usage.md` exists (`README.ja.md` mirrors
      `README.md` only), so nothing to sync.

## Trade-offs and risks

- **Risk: lychee fragments.** `mise run lint` runs lychee with
  `--include-fragments`; a wrong slug for a learnings heading fails CI.
  Headings of the form `## Step 1` are safe (`#step-1`); headings with
  suffixes should be linked without a fragment or checked locally.
- **Risk: stale claims.** The guides quote parser behavior; the done-when
  requires verifying each against the current code, since the todo text was
  written at feature time (e.g. it names `reader.rs`, which holds none of
  this logic).
- **Discoverability.** With no index, a guide is found by listing
  `docs/agents/` or by following a cross-link; accepted by the user in favor
  of not maintaining a list in `CLAUDE.md`.

## Progress

- (none yet)
