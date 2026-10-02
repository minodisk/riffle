# Learnings

## Step 1: keep the EXIF of a file whose extraction failed

- `scan::extract` now fails with `scan::Failure { message, orientation, shot }`
  (`shot` boxed for clippy's `result_large_err`; `Display` prints the message; `From<String>` / `From<&str>` build one with
  no metadata, which kept the many `Err("x".into())` test rows short).
  `for_each_path` became generic over the error type, so the analysis pass
  keeps `String`; the tests that call it directly needed `Ok::<_, String>`.
- `write_batch` is now one `INSERT` for both outcomes: an error row with no
  shot writes `Shot::default()`'s NULLs and a NULL orientation. `indexed_file`
  tells "no metadata" apart by that NULL orientation (an error row from before
  `EXTRACTOR_VERSION` 12 has it NULL too), so `exif` stays `None` there.
- The fixture in `a_file_whose_preview_is_not_a_jpeg_is_one_error_not_a_crash`
  fails at the thumbnail encode, after the parse, so the failure carries the
  parsed orientation and shot without the extra `read_metadata` call; a
  missing file covers the `shot: None` path.
- Frontend: `main.ts` reads `capture_time` / `exif` / `focus` from the entry
  regardless of `error`, and `filter.ts` / `burst.ts` never look at `error`,
  so nothing changed there. A failed file now also gets the meta pane's EXIF
  rows and, when its AF point was parsed, a focus mark.
- `EXTRACTOR_VERSION` went 11 -> 12 (origin/main had 11 when the branch was
  cut). If `fix/big-endian-dng` lands a bump first, take the next number.
- Correction: `fix/big-endian-dng` landed its bump first, so the conflict
  resolution on main took `EXTRACTOR_VERSION` 12 -> 13 (not 11 -> 12).

## Step 2: a Refresh item in the folder tree's right-click menu

- `Refresh` (action `refreshFolder`) sits in its own group after the copy
  group, so roots get it too; the multi-folder menu does not offer it.
- `folders.refresh` re-lists with the same `list_subfolders` +
  `setChildren` as `toggle`; a collapsed folder stays collapsed. A failed
  listing is handled like `toggle`'s (`markFailed(collapse(...))`, reported).
- The optional watch retry took one line: clearing `watched` before the
  render makes `syncWatches` send the current set again, bypassing its
  unchanged-set skip. `treewatch.rs` is untouched.
- `README.md` / `README.ja.md` do not list the tree menu items, so only
  `docs/humans/usage.md` changed.

## Deferred issues (todo candidates)

- Pending manual check (Step 2, real device; the step's checkbox was ticked
  on the automated criteria): on a network share (or a folder whose watch
  `set_tree_watches` cannot set), expand a folder, create a subfolder in it
  from another machine or the OS file manager, right-click the folder in
  Riffle's tree and choose `Refresh`. Expected: the new subfolder appears and
  the RAW count updates, with the folders already open under it still open.
  Files: `crates/app/ui/src/folders.ts` (`refresh`),
  `crates/app/ui/src/context.ts`.
- Pending manual check (Step 2, real device; checkbox ticked on the
  automated criteria): delete or unmount a folder shown in the tree (without
  its parent's watch removing the row, e.g. on a share), then choose
  `Refresh` on it. Expected: the row is marked failed (its tooltip shows the
  error), it collapses, and the error appears in the status line. Files:
  `crates/app/ui/src/folders.ts` (`refresh`).
