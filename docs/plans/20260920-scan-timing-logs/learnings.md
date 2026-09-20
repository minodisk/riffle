# Learnings

## Step 1

- Duration format: **milliseconds** (`elapsed().as_millis()`), written as
  `... in {n}ms` at the end of every line. Step 2 must use the same so the
  lines are comparable and greppable.
- Line wording: `scan list` / `scan reconcile` / `scan sidecars` /
  `scan prepare` (the `scan_folder` summary) in `commands.rs`, and
  `scan extract` in `index::run_scan`. Step 2 uses `open ...` wording.
- `start_scan`: **skipped**, no wall-time line of its own. Its body is a
  `spawn_blocking` around `index::run_scan` plus two `emit` calls, so its wall
  time is `run_scan`'s time plus event emission; `run_scan` already logs the
  elapsed time together with `files` / `done` / `errors` / `threads` /
  `cancelled`, which is strictly more informative. A second, nearly identical
  number would only make the log longer (the 40 KB rotation budget) and invite
  mis-reading.
- `run_scan` now builds the `ScanSummary` into a local before returning it so
  the log line can read `total` / `errors` from it. The counts and the
  batching are unchanged.
- The sidecar phase logs after the `match dirty`, using a `dirty_count`
  captured in the `Ok` arm (the `Vec` is consumed by the writer loop), so a
  failed sidecar reconcile still produces a timing line with `dirty=0`.

## Step 2

- Wording: `open list` (`list_arw`) and `open entries` (`folder_entries`),
  both `log::info!`, both ending in `in {n}ms` as Step 1 does.
- `list_arw` now canonicalizes into a local so the line can name the same
  directory string the other lines use.
- Log chattiness: **chose (b), raising `max_file_size`** to 1_000_000 on the
  log plugin builder in `main.rs`. The numbers: `refreshEntries`
  (`crates/app/ui/src/main.ts`) keeps at most one `folder_entries` invoke in
  flight and is driven by `scan-progress`, capped by `PROGRESS_INTERVAL` =
  100ms, so a ~30s scan of 5000 files yields up to ~300 `open entries` lines.
  With the plugin's prefix (`[date][time][riffle_app::commands][INFO] `) and a
  Windows path, one line is ~120 bytes: ~36 KB, which on its own all but fills
  the 40 KB `DEFAULT_MAX_FILE_SIZE` and would rotate the `scan` lines of the
  same run out of the current file.
- Option (a) (log only when the folder is fully indexed) was rejected:
  `folder_entries` does not know the listed count — only `scan_folder` does —
  so it would need cross-command state, which is more machinery than the
  measurement warrants. Raising the cap also keeps the mid-scan rows=N
  progression, which shows how fast the index fills.

## Step 3

- `create_dir_all` before `open_path`: **included**. It is one line and turns
  the "nothing has been logged yet" case (where `open_path` errors with a
  not-found) into an empty folder opening normally.
- The separator is not placed unconditionally: `help.items()?.is_empty()`
  decides, so macOS (where the default `Help` submenu is empty) gets the item
  alone and does not render a trailing lone separator line.
- `open_log_folder` returns `Box<dyn std::error::Error>` because it mixes
  `tauri::Error` (`app_log_dir`), `std::io::Error` (`create_dir_all`) and the
  opener's error; the caller logs it with `log::error!`.
- `open_path` takes an `impl Into<String>`, not a `Path`, so the `PathBuf` goes
  through `to_string_lossy()` (`open_in_photolab` passes an already-`String`
  path, which hid this).

## Merge conflict with concurrently-landed macOS menu icon work

- While this branch was open, `feat(app): show native icons on the macOS menu
  items that have one` (#190) and `feat(app): bundle SF Symbol icons for
  Settings and Undo menu items` (#193) landed on `main` and changed
  `crates/app/src/main.rs`'s `app_menu` module: `MenuItem` is now imported
  only under `#[cfg(not(target_os = "macos"))]`, since every macOS item got
  an `IconMenuItem` instead. This branch's Step 3 had added the `Open Log
  Folder` item as a plain `MenuItem::with_id(...)` with no `cfg` split, so
  merging the two was a textually clean merge (no conflict markers) that was
  semantically wrong: `MenuItem` no longer resolves on macOS at that call
  site.
- **A branch-only local CI pass does not catch this kind of conflict.** CI on
  this branch alone was green throughout, because the branch's own copy of
  the import block still had `MenuItem` in scope. The break only exists in
  the *merge result*, so nothing short of actually merging (or rebasing) main
  in and rebuilding would have shown it.
- **First diagnosis was wrong.** The `test (macos-latest)` job's failure was
  first suspected to be a macOS runner cache flake, since the same code had
  passed before and the other three jobs (`test (ubuntu-latest)`,
  `test (windows-latest)`, `lint`) were green. Re-running the job
  reproduced the identical `error[E0433]: cannot find type \`MenuItem\` in
  this scope` at the same line every time, which ruled out flakiness.
- **What made it look like phantom corruption**: the reported line number
  (`main.rs:139`) did not match the branch head's `open_log_folder` line at
  the time, because GitHub's merge commit for the PR is what CI actually
  builds, not the branch tip — so a `git blame`/`git show` against the branch
  head alone showed unrelated code at that line. The fix was to check out the
  actual merge result (rebase main onto the branch, or build the PR's merge
  ref) rather than reasoning from the branch head in isolation.
- Fix: gave `Open Log Folder` the same `#[cfg(target_os = "macos")]` /
  `#[cfg(not(target_os = "macos"))]` split as the other items, following the
  established pattern of a bundled SF Symbol PNG for items with no fitting
  `NativeIcon` (`folder.png`, rendered via `tools/macos/export-menu-icons.swift`
  alongside `gearshape.png` and `arrow.uturn.backward.png`).

## Deferred issues (todo candidates)

- (none)
