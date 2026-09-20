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

# Scan and second-open timing logs, and a menu item that opens the log folder

## Purpose

`todo.md` still carries "Measure first-scan and second-open times on a real
folder of ~5000 distinct ARW files, and update the README's numbers". Every
Phase 3 figure in the README was measured on symlinks or `cp`-copied files.
The only machine with such a folder is a Windows machine with no Rust
toolchain, so the measurement has to come from a CI-built installer
(`.github/workflows/release.yml` already builds `windows-latest`) and the
numbers have to be read back from a file afterwards.

OpenTelemetry (a collector on the Windows machine and a large dependency tree
for an n=1 run) and `crates/cli`'s `scan` benchmark (no toolchain there, and it
misses the SQLite writes, IPC and UI reflection) were both rejected. Instead:
emit `log::info!` timing lines through the `log` / `tauri-plugin-log` setup
the app already has, split by phase so a slow result can be attributed to
directory enumeration versus ARW extraction without another CI round-trip,
and add a menu item that opens the log folder so the file can be found on
Windows without digging through `%LOCALAPPDATA%` by hand.

Once done, a release build run on Windows writes lines from which the total
first-scan time, its split across phases, the file / error / thread counts and
the time of a subsequent open of the same folder can all be read off.

## Background (from investigation)

- `crates/app/src/main.rs:213` configures `tauri-plugin-log` with
  `LevelFilter::Info` in release, `Debug` in debug. **Every measurement line
  must be `log::info!`**; a `debug!` vanishes on the machine being measured.
- `tauri-plugin-log` defaults (v2, `src/lib.rs`): targets are `Stdout` and
  `LogDir { file_name: None }`; the file name is `package_info().name`, which
  `tauri-codegen` sets from `productName` (`Riffle`), so the file is
  `Riffle.log`. `DEFAULT_MAX_FILE_SIZE` is **40_000 bytes** and the rotation
  strategy is `KeepOne` (the previous file is kept as `Riffle_<timestamp>.log`).
  `app_log_dir()` on Windows resolves to
  `{FOLDERID_LocalAppData}\<identifier>\logs`.
- The existing `TimingLogs` state (`main.rs:149`) only gates *frontend*
  timing via the `debug` event, and the settings window's Debug section that
  toggles it is hidden unless `debug_build()` is true. The new lines must
  **not** be gated by it, or they would never appear in the installed build.
- The open flow (`crates/app/ui/src/main.ts` `openDirectory`) is:
  `list_arw` → `folder_entries` → `scan_folder` → `start_scan`. `scan_folder`
  (`commands.rs:723`) runs three `spawn_blocking` phases: `list_folder_in`
  (one `read_dir`), `stat` of every listed file + `Index::reconcile`, and
  `reconcile_sidecars_of`. It returns `todo` (the files needing extraction)
  and a `scan_id`. `start_scan` (`commands.rs:846`) spawns
  `index::run_scan(&index, &dir, &todo, scan_threads(), ...)`, which drives
  `riffle_core::scan::extract_all` on a rayon pool of `threads` threads and
  writes batches of `BATCH = 10` rows. A second open runs exactly the same
  commands with an empty `todo`, so the "second open" is the sum of
  `list_arw` + `folder_entries` + the `scan_folder` phases.
- `Menu::default` (tauri 2 `src/menu/menu.rs:142`) creates a `Help` submenu
  on every platform: empty on macOS, holding `About` on Windows/Linux. The app
  menu code already finds submenus by title with `app_menu::submenu`.
- `open_in_photolab` (`commands.rs:438`) already uses
  `app.opener().open_path(dir, Some(app))` from Rust; the opener's Rust API
  needs no capability entry (capabilities only gate the JS side).
  `open_path(path, None)` checks the path exists and hands it to the OS
  (Explorer on Windows, Finder on macOS).

## Steps

- [x] Step 1: Log the phases of the first scan with their counts
  - Done when:
    - `index::run_scan` emits one `log::info!` line when `extract_all` has
      returned and the last batch is flushed, carrying: the directory, the
      elapsed time of the extraction pass, the number of files handed to it
      (`files.len()`), the number completed (`done`), the error count and the
      thread count, with an explicit unit (e.g. `... in 5123ms`), and whether
      the scan was cancelled (`cancel` set) so a partial run is not mistaken
      for a full one.
    - `scan_folder` emits a line per phase — directory listing
      (`list_folder_in`, with the RAW count and sidecar count found), stat +
      `reconcile` (with the `todo` count, i.e. how many files need
      extraction), and `reconcile_sidecars_of` (with the dirty count) — and
      one summary line with the sum of those phases. Every line names the
      directory and the `scan_id` so it can be matched to the `run_scan` line
      of the same open.
    - `start_scan` logs the wall time from spawning to `scan-done` (the same
      number as `run_scan`'s minus nothing significant, but measured where
      the frontend sees it), or `run_scan`'s line is judged sufficient and
      this is skipped; decide during implementation and note it in
      `learnings.md`.
    - Lines are unambiguous on their own: each carries `scan`-family wording
      distinct from Step 2's `open` wording, units on every duration, and the
      counts spelled out (`files=5000 done=5000 errors=0 threads=14`).
    - Nothing is gated by `TimingLogs`; all lines are `log::info!`.
    - The existing tests in `crates/app/src/index.rs` and `commands.rs` pass
      unchanged (the `log` macros are no-ops without a logger); `mise run ci`
      passes.
  - Implementation approach:
    - `std::time::Instant` is already imported in `index.rs`; use it around
      the `extract_all` call in `run_scan`, and around each `spawn_blocking`
      block in `scan_folder`. No new dependency.
    - Keep the scan pipeline untouched: only add `Instant::now()` /
      `elapsed()` and `log::info!` calls. Do not change `ScanSummary`,
      `run_scan`'s signature or the batching.
    - Follow the one existing precedent for an info-level summary,
      `log::info!("index eviction: {summary:?}")` in
      `commands::spawn_eviction`: terse `key: value` prose, one line per event.
    - Use milliseconds (`elapsed().as_millis()`) throughout, or
      `{:?}` on the `Duration` (which prints its own unit) — pick one and use
      it in Step 2 as well so the lines are comparable.
    - `threads` is a parameter of `run_scan`, so the thread count is logged
      there; `scan_threads()` in `commands.rs` is where it is computed.

- [ ] Step 2: Log the second-open path as separate `open` lines
  - Done when:
    - `list_arw` logs its elapsed time and the RAW count for the directory
      (this is the listing the frontend waits on before it can show anything).
    - `folder_entries` logs its elapsed time and the number of rows returned
      (the SQLite read on the reader connection; on a first open this is 0
      rows and near-zero time, on a second open it is the whole cache read).
    - The lines are wordable as "open" lines (e.g. `open list ...`,
      `open entries ...`) so they are clearly distinguishable from Step 1's
      `scan` lines, and each names the directory.
    - Nothing else changes; `mise run ci` passes.
  - Implementation approach:
    - Both commands are thin wrappers in `commands.rs`; wrap the
      `spawn_blocking` body (or the whole command for `list_arw`, which is
      synchronous) with an `Instant`.
    - `folder_entries` is also called on every `scan-progress` event during
      a scan (`main.ts` `refreshEntries`, up to ~10/s). That is fine for the
      40 KB log budget only if each line is short; keep it to one short line
      and verify during implementation how many `folder_entries` lines a
      5000-file scan produces (progress is capped by `PROGRESS_INTERVAL` =
      100ms, so a ~30s scan is at most ~300 lines of ~100 bytes = ~30 KB,
      which is uncomfortably close to the 40 KB rotation). If it is too
      chatty, either log `folder_entries` only when the returned row count
      equals the listed count (i.e. the folder is fully indexed), or raise
      `max_file_size` on the log plugin builder in `main.rs` (see
      Trade-offs). Record the choice in `learnings.md`.
    - Because the "second open" total is spread across three commands, the
      README procedure (Step 4) tells the user to add `open list` +
      `open entries` + the `scan_folder` summary line of the same open.

- [ ] Step 3: Add `Help > Open Log Folder` to the app menu
  - Done when:
    - A menu item with id `open-log-folder` and label `Open Log Folder`
      appears in the `Help` submenu on every platform (on Windows/Linux above
      the predefined `About`, separated from it; on macOS it is the only
      item). Selecting it opens `app.path().app_log_dir()` in the platform's
      file manager in one action.
    - The handling lives entirely in `app_menu::on_event` in
      `crates/app/src/main.rs`; no frontend change, no new command, no
      capability change.
    - A failure (log dir missing because nothing was logged yet, opener
      error) is reported with `log::error!` and does not panic.
    - `mise run ci` passes.
  - Implementation approach:
    - Mirror the existing items: a `const OPEN_LOG_FOLDER_ID`, a
      `MenuItem::with_id(handle, ..., "Open Log Folder", true, None::<&str>)`
      in `build`, found-by-title `submenu(&menu, "Help")?` with
      `prepend_items(&[&item, &separator])` (or `prepend(&item)` on macOS
      where `Help` is empty — the separator is harmless there but check it
      does not render as a lone line). If `Help` were ever absent, create it
      like the Linux `File` fallback does.
    - In `on_event`: `app.path().app_log_dir()` then
      `app.opener().open_path(dir, None::<&str>)` (`OpenerExt` is already
      imported in `commands.rs`; import it in `app_menu` too). Since the log
      plugin creates the folder on first write and this app logs at startup
      (eviction summary / update check), the folder exists by the time a
      user can click; still, `create_dir_all` before opening is a cheap
      guard and avoids `open_path`'s not-found error — decide during
      implementation.
    - Do not bind a shortcut: non-culling actions go in the menu, not on a key
      (project policy).

- [ ] Step 4: Document the log folder and the measurement procedure
  - Done when:
    - `README.md` mentions `Help > Open Log Folder` in the feature list next
      to the other menu items (`Open in DxO PhotoLab`, `Undo`, ...) and names
      where the file is per platform (`Riffle.log` under the app log dir:
      `%LOCALAPPDATA%\com.minodisk.riffle\logs\` on Windows,
      `~/Library/Logs/com.minodisk.riffle/` on macOS,
      `~/.local/share/com.minodisk.riffle/logs/` on Linux).
    - The README's Phase 3 performance section gains a short "how to
      measure on your own folder" paragraph: what the `scan` and `open` lines
      mean and which cache file to delete to repeat a first scan (the
      procedure in Trade-offs below, in user-facing form; no personal test
      environment details).
    - `todo.md`'s "real-folder scan and second-open numbers" item is updated
      to say the instrumentation exists and only the measurement itself
      remains (do not tick it; the numbers are not in yet).
    - `mise run ci` passes (lychee link check included).
  - Implementation approach:
    - README content policy: user-facing only; no phases, verification logs
      or personal machine details in the README.

## Trade-offs and risks

### Where the menu item lives

- **Chosen: `Help > Open Log Folder`** (decided by the user at plan approval).
  `Menu::default` creates `Help` on every platform, so there is no platform
  branch, and "Help > Open Logs Folder" is where users of VS Code, Slack,
  Obsidian etc. look for it. It also keeps a diagnostic action away from the
  culling workflow items in `File`.
- Alternative, not taken: next to `Check for Updates…` (macOS app menu /
  `File` elsewhere), which already groups the "about the app itself" items. It
  would reuse the existing `#[cfg(target_os)]` insert blocks but mixes a
  diagnostic into the settings group and adds a fourth item to `File` on
  Windows.
- Not taken: a button in the settings window's Debug section, because that
  section is hidden in distributable builds (`debug_build`), i.e. exactly
  where it is needed.

### Log file size and rotation

`tauri-plugin-log` rotates `Riffle.log` at 40 KB (`DEFAULT_MAX_FILE_SIZE`),
keeping one rotated copy. The log is appended across sessions, so the
first-scan lines and the later second-open lines share a file with the
startup eviction / update lines; a ~30s scan also produces up to ~300
`folder_entries` calls (Step 2). Options if that is too tight: (a) log
`folder_entries` only when the folder is fully indexed, (b) call
`.max_file_size(...)` on the log builder in `main.rs` (a one-line change,
no new dependency). Either way the rotated `Riffle_<timestamp>.log` in the
same folder still holds the older lines, so nothing is lost silently — the
README procedure should say to look at both files.

### Where the "second open" total comes from

The second open is three IPC commands, so its total is a sum of three lines
rather than one number. A single summary would require the frontend to
report back or a cross-command state; both are more machinery than the
measurement warrants. The README paragraph in Step 4 states the sum
explicitly.

### Concrete Windows paths

- Log: `C:\Users\<user>\AppData\Local\com.minodisk.riffle\logs\Riffle.log`
  (`%LOCALAPPDATA%\com.minodisk.riffle\logs\`), plus the rotated
  `Riffle_<timestamp>.log` if the 40 KB cap was hit.
- Index cache (the thing that makes an open a "second" open):
  `%LOCALAPPDATA%\com.minodisk.riffle\index.sqlite`, with `index.sqlite-wal`
  and `index.sqlite-shm` beside it (`app_cache_dir()` is
  `{FOLDERID_LocalAppData}\<identifier>` on Windows).
- Settings store (`last_folder`, keymap, sidecar format) lives under
  `%APPDATA%\com.minodisk.riffle\` (`app_data_dir`); leave it alone, it does
  not affect timing and its `last_folder` is what makes the relaunch reopen
  the folder automatically.

### Measurement procedure (cold first scan vs warm second open)

1. Install the CI-built installer of the release containing Steps 1–3.
2. Quit Riffle if running. Delete
   `%LOCALAPPDATA%\com.minodisk.riffle\index.sqlite*` (all three files) so
   the index is empty. For an OS-page-cache-cold disk read as well, reboot
   (or measure right after the folder was copied to a drive that has not
   been read since); otherwise the "first scan" is index-cold but
   disk-warm, which should be stated with the number.
3. Launch Riffle and open the folder (pick or drag-drop). Wait until the
   progress reaches the total (the `scan-done` refresh). This produces the
   `scan` lines: `scan_folder`'s phase lines with `todo=~5000`, and
   `run_scan`'s extraction line with `files`, `done`, `errors`, `threads`
   and the elapsed time.
4. Quit Riffle and launch it again. `last_folder` reopens the same folder
   automatically; this is a genuine second open (fresh process, warm index).
   This produces the `open list`, `open entries` (~5000 rows) and
   `scan_folder` lines with `todo=0`; add their times for the second-open
   total. Reopening the folder without quitting also counts, but with a
   warmer process.
5. `Help > Open Log Folder`, copy `Riffle.log` (and `Riffle_*.log` if
   present). Lines carry timestamps, so the two runs are told apart by time
   and by `todo=` / `files=` being ~5000 versus 0.
6. Note the machine, the drive type (internal NVMe / external / card reader)
   and whether step 2 included a reboot alongside the numbers when updating
   the README.

### Other risks

- `open_path(dir, None)` errors if the directory does not exist. The log
  plugin creates it on the first write, and the app logs at startup, so this
  should not occur; Step 3 guards it anyway.
- Local verification: `mise run tauri:release` on macOS bundles the app as
  CI does (minus updater artifacts) and writes to
  `~/Library/Logs/com.minodisk.riffle/Riffle.log`, so the line format and the
  menu item can be checked before the Windows CI build.

## Progress

- (2026-09-20) Step 1 complete
