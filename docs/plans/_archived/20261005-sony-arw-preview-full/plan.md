<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../humans/usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Older Sony ARW bodies: 1:1 falls back to the preview

## Purpose

The α9 II, α7R IV, α7R IVA, α7C, α6400, α6600 and ZV-E10 write no full-size
JPEG: their IFD chain is IFD0 (the 1616x1080 preview; 1920x1080 on the ZV-E10;
1088x1080 / 1440x1080 on 1:1 / 4:3 shots) plus IFD1 (a 160x120 thumbnail), and
the SubIFD is the raw data. `arw::parse` takes the largest JPEG in the chain
and the SubIFDs as `full`, so on these files `full` is the thumbnail and the
1:1 view (`read_focus_crop` via `reader::read_full`) crops an upscaled 160x120
image. The user decided (2026-10-05) that `full` must never be a JPEG smaller
than the preview: fall back to the preview instead, the way ORF already does
(`full: preview`). All seven bodies fall inside the camera support rules
(released 2019-2021; largest embedded JPEG long side 1616 px >= 1280 px), so
once the fix lands they are listed in `docs/humans/cameras.md`, the docs that
say they are unlisted are updated, and the todo section is removed.

## Steps

- [x] Step 1: Fall back to the preview as `full` in `arw::parse`, list the seven bodies, update the docs and drop the todo
  - Done when:
    - A synthetic-TIFF unit test in `crates/core/src/arw.rs` (next to the
      existing ones, built with the `tiff` / `ifd` helpers) with IFD0 = a
      large preview JPEG and IFD1 = a small thumbnail JPEG asserts
      `a.full == a.preview` (same `offset` / `length`) rather than the
      thumbnail; the existing tests (`parses_ifd0_preview_and_orientation`
      with its `full.is_none()`, the DNG strip tests, the big-endian ones)
      still pass unchanged.
    - `cargo run -p riffle-cli -- info` / `bench` on
      `D:\photos\samples\ARW\ILCE-9M2_sony_a9_ii_10.arw`,
      `ILCE-6400_DSC00087.ARW`, `ZV-E10_DSC00002.ARW` (and one each of
      `ILCE-7RM4_*`, `ILCE-7RM4A_*`, `ILCE-7C_*`, `ILCE-6600_*`) show `full`
      at the preview's offset / length and the full decode gives 1616x1080
      (1920x1080 on the ZV-E10, 1440x1080 on the 4:3 α7C sample); the 1:1
      view in the app on one of them is the preview crop, not a blurry
      thumbnail. Record the per-body sizes in `learnings.md`.
    - `docs/humans/cameras.md` and `cameras.ja.md` carry the seven rows;
      `docs/humans/raw-formats.md` / `raw-formats.ja.md` and
      `docs/agents/raw-metadata-parsing.md` describe the new behavior;
      the `todo.md` section "### Core: older Sony ARW bodies take the 160x120
      thumbnail as the full-size JPEG" is removed.
    - `mise run ci` passes.
  - Implementation approach:
    - Core (`crates/core/src/arw.rs`, `parse`): after the loop that picks the
      largest `Embedded` from the chain / SubIFDs (and after the DNG strip
      branch, which is gated on `preview.is_none() && full.is_none()` and
      must stay as is), substitute the preview when it is bigger:
      `if let (Some(p), Some(f)) = (preview, full) { if f.length < p.length { full = Some(p) } }`.
      Compare by `length`: `Embedded` has no pixel size, and byte length is
      the measure `parse` already ranks JPEGs by (a 160x120 thumbnail is a
      few KB against a 300-900 KB preview). Keep the preview-only case
      (`full` stays `None`) so `parses_ifd0_preview_and_orientation` holds;
      see Trade-offs for the alternative. Update the `Arw::full` doc comment
      ("the preview when the file holds nothing larger") and the comment
      above the chain walk.
    - Consumers need no change: `reader::read_full` reads whatever `full`
      points at by range (the preview lies inside `HEAD_LIMIT`, which the
      bounded path already handles); `read_focus_crop` decodes it;
      `crates/core/src/scan.rs` and `crates/app/src/index.rs` never read
      `full`, so no `EXTRACTOR_VERSION` bump. The CLI `bench` /
      `scan` checks branch on `a.full.is_some()` and now exercise the preview
      decode on these bodies, which is the intended behavior. Verify this by
      grep during implementation rather than assuming it.
    - `docs/humans/cameras.md` / `.ja.md`: insert the rows in the table's
      Sony order (newest body of each line first, as the current rows are):
      `Sony α9 II` after `Sony α9 III`; `Sony α7R IV` and `Sony α7R IVA`
      after `Sony α7R V`; `Sony α7C` after `Sony α7CR`; `Sony α6400` and
      `Sony α6600` after `Sony α6700`; `Sony ZV-E10` after `Sony ZV-E1`.
      Column values, derived the way the existing Sony rows are (AF point =
      `FocusLocation` present and trusted; AF frame size = `FocusFrameSize`
      present; Face tracking = `AFTracking` 1 observed on a sample, so the
      lock-on value 2 is `–`; Sub-second = `SubSecTimeOriginal` present):
      | Sony α9 II | ✓ | – | – | ✓ |
      | Sony α7R IV | ✓ | – | – | – |
      | Sony α7R IVA | ✓ | – | – | – |
      | Sony α7C | ✓ | – | – | ✓ |
      | Sony α6400 | ✓ | – | – | – |
      | Sony α6600 | ✓ | – | – | – |
      | Sony ZV-E10 | ✓ | – | – | ✓ |
      Source: the "Per body (not listed)" table in
      `../_archived/20260930-sony-arw-coverage/learnings.md`; re-check with
      `riffle-cli info` on the samples if any value is in doubt. Add a
      sentence under the table (next to the α9 III / α7CR manual-focus note)
      that these seven bodies embed no full-size JPEG, so their 1:1 view
      shows the 1616x1080 preview (1920x1080 on the ZV-E10) at its own size,
      as ORF does; mirror it in the `.ja.md`.
    - `docs/humans/raw-formats.md` / `.ja.md`: rewrite the
      "ARW: no full-size JPEG on older bodies" section's last two sentences
      (currently "Riffle takes that thumbnail ... looks blurry. These bodies
      are not listed as supported until that is fixed.") to say Riffle
      skips any JPEG smaller than the preview and uses the preview as the
      1:1 view, so these bodies are listed; and amend the ARW
      "Full-size JPEG" table cell ("The largest JPEG in the other IFDs")
      to add "or the preview when nothing larger is embedded". Keep the
      pair in sync in the same PR.
    - `docs/agents/raw-metadata-parsing.md`: retitle and rewrite the entry
      "Older Sony ARW bodies carry no full-size JPEG, so `parse` takes the
      160x120 thumbnail as `full` (Measured)" to the new behavior (`parse`
      falls back to the preview when the largest other JPEG is smaller by
      byte length), and replace its "Rule" (do not list a Sony body without
      a full-size JPEG) with: a body without IFD2 is still listed; note in
      `cameras.md` that its 1:1 view is the preview. Point the Source at
      this plan's `learnings.md` as well.
    - `todo.md` (repo root, line ~764): delete the whole
      "### Core: older Sony ARW bodies take the 160x120 thumbnail as the
      full-size JPEG" section (Background + TODO). Do not touch the
      Lightroom-related items around it.
    - Commit as `fix(core): fall back to the preview when an ARW has no larger JPEG`
      (or similar Conventional Commit); one PR.

## Trade-offs and risks

- **Preview-only files (`full` stays `None`) vs `full = preview` whenever
  nothing else exists.** The plan substitutes the preview only when a
  smaller `full` was found, which keeps `parses_ifd0_preview_and_orientation`
  (`full.is_none()`) and the DNG strip gate untouched. The ORF-style
  alternative (`full = full.or(preview)`) would also give the 1:1 view
  something on a hypothetical ARW with no IFD1 thumbnail, but changes an
  existing test and the semantics of `full.is_none()` the CLI `bench` /
  `scan` check rely on. No sample needs it (all seven bodies have the IFD1
  thumbnail). Chosen: the minimal rule (approved 2026-10-05).
- **Comparing by byte length, not pixels.** `Embedded` carries no
  dimensions; a pathological file whose full-size JPEG is more compressed
  than its preview would be misjudged, but NEF has the same caveat and no
  Sony sample shows it (the full-size JPEGs run several MB).
- **Face tracking `–` on all seven.** The samples read `AFTracking` 2
  (lock-on), never 1 (face), so the column stays `–` as the existing rule
  ("`–` means not observed on the sample") dictates; it is not a claim the
  bodies lack it.
- **α7R IV AF point.** The learnings table gives `FocusLocation` 9504x6336
  without an "off-center" note; present counts as `✓` as for the α1 (point
  at the center). Verify with `riffle-cli info` if in doubt.
- The 1:1 view on these bodies is now a 1616x1080 crop, i.e. the same pixels
  as the preview pane; a user may expect more magnification. This is the
  decided behavior and the docs state it.

## Progress

- (2026-10-05) Step 1 complete (GUI 1:1 check pending the user)
