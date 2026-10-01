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

# Two todo items: EXIF on error rows, a folder tree Refresh item

## Purpose

Two fully specified `todo.md` items, one PR each:

1. An error row of the folder index stores no metadata, although
   `read_metadata` parses it, so bursts, the filter menu and capture-time order
   leave out a file whose extraction failed.
2. A folder whose tree watch could not be set (`set_tree_watches` only
   `log::warn!`s) goes stale with no way to re-list it short of collapsing and
   expanding; a Refresh item in the folder tree's right-click menu re-lists any
   folder on demand.

After this, a failed file still sorts and filters by its EXIF, and any folder
in the tree can be re-listed by hand.

Sources: `todo.md` sections "App: keep the EXIF of a file whose extraction
failed" and "App: Refresh from the folder tree's context menu for a folder
whose watch failed". Remove each section from `todo.md` in the PR of its step.

Out of scope: the `todo.md` items "Core: a malformed full-size JPEG makes the
1:1 view end the whole app" and "Core: big-endian DNGs do not open" are being
done in parallel on other branches (`fix/partial-decode-error-exit`,
`fix/big-endian-dng`). Do not touch `crates/core/src/partial.rs` or
`crates/core/src/arw.rs`'s byte-order handling. `fix/big-endian-dng` will likely
bump `EXTRACTOR_VERSION` too; see Step 1.

## Steps

- [x] Step 1: Keep the EXIF of a file whose extraction failed in its error row
  - Done when:
    - `riffle_core::scan::extract` (and `extract_all`'s `on_item`) hands a
      failure together with the metadata that was parsed, when any was:
      e.g. `Result<Entry, Failure>` with
      `pub struct Failure { pub message: String, pub orientation: u16, pub shot: Option<Shot> }`
      (shape to be settled in the step; `Display` for `Failure` prints the
      message so the CLI's `scan` stage and the app's logs stay as they are).
    - `Index::write_batch` writes the error row with `orientation`,
      `capture_time`, `subsec`, the focus columns and the `make` .. `focal_den`
      / `frame_w` / `frame_h` / `manual_focus` columns filled from the shot
      when there is one (`thumb` NULL, `error` set as today), and
      `indexed_file` reads `exif` for an error row too (today it forces
      `exif = None` when `error` is set). The row keeps `faces_todo`'s
      "error rows are done" rule.
    - Unit tests in `crates/app/src/index.rs`: the existing
      `a_failed_file_is_remembered_as_an_error_row_without_a_thumbnail` keeps
      its `exif.is_none()` assertion for a `shot: None` failure, plus a test
      where the failure carries a shot and the entry comes back with
      `capture_time` and `exif.make` and `error` set. A test in
      `crates/core/src/scan.rs` on the existing
      `a_file_whose_preview_is_not_a_jpeg_is_one_error_not_a_crash` fixture
      asserts the failure carries the fixture's orientation / shot.
    - `EXTRACTOR_VERSION` is bumped by one above the value on `origin/main`
      when the step's branch is cut (re-check right before the PR; if
      `fix/big-endian-dng` merged a bump meanwhile, take the next number),
      with the doc comment saying the new value keeps the metadata of a file
      whose extraction failed, so existing error rows are re-extracted.
    - Frontend: nothing has to change for the feature itself. `main.ts`
      builds `captureTime` from `entry.capture_time` and `exif` from
      `entry.exif` regardless of `error`, and `applyFailures` still marks the
      cell. Confirm `filter.ts` / `burst.ts` / sort code need no edit; if the
      meta pane now shows EXIF rows for a failed file, that is intended.
    - `todo.md` section removed; `mise run ci` passes.
  - Implementation approach:
    - On the failure path of `scan::extract_unless`: `read_preview` returns
      only an `Err` when the preview is missing, HEVC-undecodable or out of
      range, although the RAW parse succeeded inside it; the thumbnail encode
      failure happens after the parsed file is in hand. On a `read_preview`
      error, call `reader::read_metadata(path)` once (one more bounded head
      read, only on the failure path) and attach its `orientation` / `shot`
      if it succeeds; on a thumbnail failure, attach what was already read.
      Do not change `reader.rs`'s return types.
    - Callers to adapt: `crates/app/src/index.rs` (`write_batch` signature,
      the `pending` / `flush` / `on_item` types in `run_scan`),
      `crates/cli/src/main.rs` (`check_file`'s `scan` stage, the benchmark's
      `extract_all` callback), tests in both crates.
    - Add a short "error rows keep the parsed metadata" note to the
      `write_batch` doc comment.

- [x] Step 2: Add a Refresh item to the folder tree's right-click menu
  - Done when:
    - `folderMenuGroups` in `crates/app/ui/src/context.ts` offers a `Refresh`
      item on a single folder (root or not; not on a multi-folder selection),
      and `context.test.ts`'s expected menus are updated accordingly.
    - `crates/app/ui/src/folders.ts` exports `refresh(path)`: it runs the
      same `list_subfolders` call `toggle` / `tree-changed` run and applies
      `setChildren` (which keeps the folders open under it and clears
      `failed`); a failed listing is reported through `reportError` and marks
      the row `failed` (like `toggle`). A collapsed folder is refreshed too
      (its RAW count and cached children update; it stays collapsed).
      `main.ts`'s folder-menu `switch` gets the `refresh` case.
    - `docs/humans/usage.md`'s folder-tree paragraph (the list of right-click items)
      mentions `Refresh` in its place in the menu and what it does, including
      that it re-lists a folder whose watch could not be set (a network
      share, a refused permission). Grep `README.md` / `README.ja.md`; edit
      both only if they list tree menu items.
    - `todo.md` section removed; `mise run ci` passes.
    - Pending manual checks (real device, not blockers): the item appears and
      re-lists an expanded folder after creating a subfolder on a share the
      watcher cannot watch; the row shows its error when the folder is gone.
  - Implementation approach:
    - Menu placement: its own group right after the copy group (Copy Path /
      Copy Folder Name), before `Rename…`, so it is available on roots too.
      Label `Refresh`, no shortcut, like its siblings.
    - Optional: if a few lines, have Refresh re-invoke `set_tree_watches`
      with the current set (bypassing `syncWatches`'s unchanged-set skip) so
      a watch that failed is retried; otherwise leave it out and note it in
      `learnings.md`. `crates/app/src/treewatch.rs` needs no change otherwise.
    - `folders.ts` is a DOM module without its own test file; keep the new
      function small and delegate to the tested `context.ts` / `tree.ts`.

## Trade-offs and risks

- Step 1 bumps `EXTRACTOR_VERSION`, re-extracting every row of every folder on
  its next scan. `fix/big-endian-dng` may bump it too; whichever lands second
  takes the next number, and before a release users see one rescan.
- Step 1 stores the metadata in the row (not read on demand), costing one
  extra `read_metadata` head read per failed file on the failure path only.
- Step 1 changes `scan::extract`'s public error type (used by the CLI and the
  app only). `fix/partial-decode-error-exit` edits `crates/cli/src/main.rs`
  `check_file` too; expect a small conflict there.
- Step 2 offers Refresh on roots (a root's subfolder list can go stale too).

## Progress

- (2026-10-02) Step 1 complete: `scan::Failure` carries the parsed orientation / shot, error rows keep the metadata, `EXTRACTOR_VERSION` 12 -> 13
