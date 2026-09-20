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

# Clear the index cache from the settings window

## Purpose

`index.sqlite` under the app cache dir holds every thumbnail and the cached
EXIF of every folder ever opened. The only way to empty it today is to quit
and delete `index.sqlite*` by hand, which on Windows means typing
`%LOCALAPPDATA%\com.minodisk.riffle` into Explorer because `AppData` is
hidden. This adds a normal user affordance to the settings window: the
current cache size and a button that clears it after a confirmation. It also
turns the "measuring on your own folder" procedure in `README.md` into
"press the button, reopen the folder".

## Decisions (made by the user; not open)

- The size is shown next to the button, not a bare button.
- It lives in the settings window, in a **new `Cache` tab** (after `Keyboard
  Shortcuts`, before the hidden `Debug` tab). Rationale: the existing tabs
  (`Sidecar`, `Culling`, `Keyboard Shortcuts`) are about workflow, not
  storage; a size figure plus a destructive button inside `Culling` would
  sit next to an unrelated checkbox. Rejected: a `Help > Open Cache Folder`
  menu item (unnecessary once the button exists) and the `Debug` tab (hidden
  in distributable builds).
- **Units follow the platform's file manager**: decimal (`1 GB = 1000^3`,
  as Finder shows) on macOS; binary (`1024^3`, as Explorer shows) on Windows
  and Linux. Labels stay plain (`GB`, not `GiB`) because they mirror what the
  user sees in their file manager. No new dependency (no
  `@tauri-apps/plugin-os`): the base comes from Rust via
  `cfg!(target_os = "macos")`. The formatting is done in Rust and the command
  returns a display string (see Trade-offs for why).
- Accepted as recommended: reuse `evict_folder`, keep dirty ratings, measure
  the WAL after `VACUUM`, refuse while a scan runs, auto-reopen the open
  folder through an `index-cleared` event, and compute the size from the
  files on disk.

## Background (from investigation)

- `crates/app/src/index.rs`: `Index::evict` (line ~369) walks `folders` and
  calls `evict_folder(dir)`, which deletes `files WHERE dir`, `ratings WHERE
  dir AND dirty = 0`, and the `folders` row when no rating remains; it then
  runs `VACUUM` if any row went. `used_bytes()` (private, line ~400) is
  `(page_count - freelist_count) * page_size`. "Clear everything" is
  `evict_folder` over every `folders` row, plus the `VACUUM`.
- Dirty `ratings` rows are judgements that have not reached their sidecar
  (README: "the folder index is a cache, but it also holds unwritten
  judgements"). `evict_folder` and `reset_sidecars` both keep them; a clear
  must too, or it silently loses a judgement.
- `commands::spawn_eviction` (`crates/app/src/commands.rs:674`) is the lock
  precedent: `Scans` lock first, then the writer lock (`index::lock(&index)`),
  and it returns without doing anything if `state.running.is_some()`.
  `docs/agents/tauri-app.md` ("Folder-index eviction: lock order and where it
  runs") records this order; keep it.
- The writer and reader connections are opened in `main.rs` `setup` from
  `app.path().app_cache_dir()?.join("index.sqlite")`; the path is not kept in
  any managed state. Both stay open for the process lifetime, so the file
  must not be deleted.
- Confirmation precedent: the app has no message dialog yet. `pick_folder`
  (`commands.rs:313`) is the pattern for any dialog: an `async` command calls
  the plugin's callback API and awaits a `tauri::async_runtime::channel(1)`.
  `tauri-plugin-dialog` 2.7.3's Rust `MessageDialogBuilder`
  (`app.dialog().message(..)`) has `.title(..)`,
  `.kind(MessageDialogKind::Warning)`,
  `.buttons(MessageDialogButtons::OkCancelCustom("Clear", "Cancel"))`,
  `.parent(&window)` and `.show(|confirmed: bool| ..)`. The Rust API needs no
  capability entry; `capabilities/default.json` only lists `dialog:allow-open`
  for the JS side, which this does not use.
- Display formatting precedent: `crates/app/src/exif.rs` formats the
  shooting settings in Rust for both the meta pane and the filter menu
  (`CLAUDE.md` names it as such), so a `format_bytes` in Rust follows an
  existing shape rather than introducing one.
- Settings window: `crates/app/ui/settings.html` is a tablist,
  `crates/app/ui/src/settings.ts` invokes commands and shows errors in
  `#status`; `tabs.ts`'s `nextTab` is generic over the visible tab ids, so a
  new tab needs no test change. `settings.css` has `.actions` for a
  right-aligned button row.
- Main window precedent for "the backend reset the index": the
  `sidecar-format` listener in `crates/app/ui/src/main.ts` (~line 1195)
  reopens `openDir` with `newFolderToken()`.
- `Writer::flush(DRAIN_TIMEOUT)` (`sidecar.rs:285`) writes everything pending
  and waits at most 2 s; `switch_sidecar_format` calls it before touching the
  index.
- WAL hazard for "the file shrinks": the index runs in WAL mode, `VACUUM`
  writes the rebuilt database through the WAL, and SQLite does not shrink
  `index.sqlite-wal` on its own until a checkpoint truncates it or the last
  connection closes. After a clear, `index.sqlite` may be small while
  `index.sqlite-wal` still holds the old size on disk. Step 1 measures this.

## Steps

- [x] Step 1: `Index::clear` — empty the index and reclaim the space
  - Done when:
    - `Index::clear(&mut self) -> Result<EvictSummary, String>` in
      `crates/app/src/index.rs` removes every `files` row, every clean
      `ratings` row and every `folders` row that keeps no rating (a dirty
      rating keeps its row and its `folders` row, exactly as `evict_folder`
      does), then `VACUUM`s when anything was deleted, and returns the
      counts in the existing `EvictSummary`.
    - After `clear`, the on-disk footprint actually shrinks: a test in the
      style of `a_vacuum_after_eviction_shrinks_the_file` asserts
      `page_count` dropped and asserts the sum of `index.sqlite` +
      `index.sqlite-wal` sizes on disk is small (the WAL must not keep the
      old database's size). If the measurement shows the WAL keeps its size,
      `clear` ends with `PRAGMA wal_checkpoint(TRUNCATE)` and the test pins
      it. Record the measured before/after sizes in `learnings.md`.
    - A test asserts a dirty rating survives `clear` (mirror
      `a_dirty_rating_survives_the_eviction_of_its_folder`), and one asserts
      `clear` on an empty index returns `EvictSummary::default()` and does
      not `VACUUM`.
    - `used_bytes` stays private (the displayed size comes from the files on
      disk, Step 2).
    - The existing tests in `index.rs` pass unchanged; `mise run ci` passes.
  - Implementation approach:
    - Implement `clear` by iterating `SELECT dir FROM folders` and calling
      the existing `evict_folder`, then the same `VACUUM` branch `evict` has;
      or factor the `VACUUM`-if-rows step out of `evict` so both share it.
      Do not write a second delete path with its own SQL: the "keep dirty
      ratings, drop the folder row only when nothing remains" rule must stay
      in one place.
    - Update the doc comment on `evict`/`evict_folder` so `clear` is named
      as the second caller of `evict_folder`.
    - Use the test helpers already in the module (`temp_dir`, `open`,
      `big_entry`, `set_opened_at`, `remove_temp_dir`, `NO_CAP`).

- [x] Step 2: `index_size` and `clear_index` commands, with the confirmation, the scan guard and the platform-unit formatting
  - Done when:
    - `commands::index_size(app) -> Result<String, String>`: an `async`
      command returning the cache's size **already formatted** for display
      (`"1.2 GB"`, `"312 KB"`, `"0 B"`); `"0 B"` (not an error) when the
      index cache is unavailable (`AppIndex(None)`) or the file does not
      exist.
    - A pure `fn format_bytes(bytes: u64, base: u64) -> String` in
      `commands.rs` (or a small `crates/app/src/bytes.rs` if `commands.rs`
      is judged too crowded) with plain labels `B`/`KB`/`MB`/`GB`, no
      decimals for `B`, one decimal from `KB` up, and a `const
      SIZE_BASE: u64 = if cfg!(target_os = "macos") { 1000 } else { 1024 };`
      next to it with a comment stating the Finder/Explorer rationale.
    - Rust unit tests pin: `0 -> "0 B"`; the boundaries at both bases
      (`999`/`1000` and `1023`/`1024` -> `KB`; `1000^2`/`1024^2` -> `MB`;
      `1000^3`/`1024^3` -> `GB`); a ~1.2 GB value at each base; and one test
      that asserts `SIZE_BASE` is `1000` under `cfg!(target_os = "macos")`
      and `1024` otherwise, so the platform choice is pinned by CI on every
      OS in the matrix, not only on the developer's Mac.
    - `commands::clear_index(app) -> Result<bool, String>`: an `async`
      command that (1) refuses with an `Err` message such as
      `"a scan is running; wait for it to finish"` when `Scans.running` is
      set, before showing any dialog; (2) shows a native confirmation
      (`app.dialog().message(..)`, kind `Warning`, buttons
      `OkCancelCustom("Clear", "Cancel")`, parented to the settings window,
      awaited over a channel as `pick_folder` does) and returns `Ok(false)`
      on cancel; (3) on confirm, in `spawn_blocking`: flushes the sidecar
      writer (`Writer::flush(DRAIN_TIMEOUT)`, as `switch_sidecar_format`
      does), takes the `Scans` lock, re-checks `running` under it (refuse
      again if a scan started during the dialog), then takes the writer lock
      and calls `Index::clear`, logging the summary with `log::info!` like
      `spawn_eviction`; (4) emits an `index-cleared` event and returns
      `Ok(true)`.
    - Both commands are registered in `generate_handler!` in `main.rs`.
    - Lock order is `Scans` then writer, never the reverse, and no `std`
      `MutexGuard` is held across an `.await`.
    - The main window listens for `index-cleared` in `main.ts` and, when a
      folder is open, reopens it with `newFolderToken()` exactly as the
      `sidecar-format` listener does, so the strip does not keep showing
      thumbnails the index no longer has and the folder is rescanned at
      once.
    - `mise run ci` passes. The dialog and the reopen are verified manually
      (`mise run tauri:release:devtools`): confirm clears and the main window
      rescans; cancel changes nothing; pressing the button mid-scan shows the
      refusal in the settings status line. Report what was not checked.
  - Implementation approach:
    - Size: recompute the path as `main.rs` does
      (`app.path().app_cache_dir()?.join("index.sqlite")`) and sum
      `std::fs::metadata(..).len()` over the base file and its `-wal` and
      `-shm` siblings (reuse the private `with_suffix` helper in `index.rs`,
      making it `pub(crate)`), treating a missing file as 0. Do the `stat`s
      in `spawn_blocking`; no lock is needed.
    - The refusal on a running scan is deliberate (Trade-offs); do not cancel
      the scan.
    - The dialog `parent` is `app.get_webview_window("settings")` when it
      exists; fall back to no parent rather than erroring.
    - Follow the "anything that waits goes in an `async` command" rule in
      `docs/agents/tauri-app.md`.
    - No new capability entry and no new dependency.

- [x] Step 3: The settings window's Cache tab
  - Done when:
    - `settings.html` gains a `Cache` tab and panel (after `Keyboard
      Shortcuts`, before the hidden `Debug` tab) showing a line such as
      `Index cache: 1.2 GB` (the string from `index_size` verbatim) and a
      `Clear Cache` button, with a one-line explanation that clearing drops
      every cached thumbnail so the next open of a folder scans it again
      (judgements are kept — they live in the sidecars).
    - The size is fetched with `index_size` when the window opens and again
      after a successful `clear_index`, so the figure visibly drops.
    - The button invokes `clear_index`; while it is in flight the button is
      disabled; an `Err` (including the running-scan refusal) is shown in
      `#status` like every other settings error; `Ok(false)` (cancelled)
      leaves the figure as is.
    - No formatting logic in TypeScript and no new frontend module: the
      backend string is displayed as-is.
    - `pnpm exec vp check`, `fmt`, `test` and `mise run ci` pass.
  - Implementation approach:
    - Mirror the existing patterns in `settings.ts`: top-level
      `invoke(..).then(..)` for the initial read, `status.textContent = ""`
      before an action and `String(error)` on failure, `.actions` for the
      button row.
    - The tab is a plain visible tab; `nextTab` needs no change.
    - Do not put anything in the `Debug` section.

- [x] Step 4: Documentation
  - Done when:
    - `README.md`'s feature list gains a `Clear Cache` entry next to the
      other settings-window items (auto-advance, sidecar format) saying
      where it is, what it removes (thumbnails and cached metadata of every
      folder; judgements are not touched), that the size shown uses the
      same units as the platform's file manager, and that a running scan has
      to finish first.
    - The "Measuring on your own folder" paragraph replaces the "quit and
      delete `index.sqlite` and its `-wal` / `-shm` files in
      `%LOCALAPPDATA%\...`" instruction with "clear the cache in Settings and
      reopen the folder"; the per-platform cache paths may stay as a note of
      where the file lives, or go — keep the paragraph user-facing (README
      content policy: no phases, verification logs or personal environment).
    - `docs/agents/tauri-app.md`'s eviction note mentions that `Index::clear`
      shares `evict_folder`, the same lock order, and the WAL-truncation fact
      Step 1 measured, so the next change to eviction sees both callers.
    - `mise run ci` (including the lychee link check) passes.

## Trade-offs and risks

### Where the unit formatting lives (chosen: Rust, preformatted string)

- **Chosen: `index_size` returns a preformatted string; `format_bytes` and
  `SIZE_BASE` live in Rust.** The base is a compile-time platform fact that
  only Rust knows without a new dependency; the only consumer of the number
  is one label; and `exif.rs` already establishes "display formatting in
  Rust, shared by the frontend". Returning the bytes plus the base would
  put one concern on both sides of the IPC boundary (a `{bytes, base}`
  struct, a TS type for it, a `formatBytes(n, base)` and tests in both
  languages) for no consumer that needs the raw number. The Rust unit tests
  run on every OS in the CI matrix, so the platform pin is checked where it
  matters.
- Not taken: `{bytes, base}` + `formatBytes` in TypeScript. It keeps
  presentation in the UI layer, which is the usual argument, but here the
  "presentation" is a two-branch constant that the UI cannot determine on
  its own.
- Consequence: if a future feature wants the raw byte count (e.g. an
  eviction-limit setting), add a separate command or a second field then;
  do not pre-build it.

### Button pressed while a scan is running

- **Chosen: refuse with a message, do not cancel the scan.** This is what
  `spawn_eviction` already does, it keeps the `Scans`-then-writer lock order
  trivially (no `running` handle to take and join across an `.await`), and a
  scan finishes within a minute. The refusal is checked before the dialog so
  the user is not asked to confirm something that then fails, and re-checked
  under the lock so a scan started during the dialog is not clobbered. The
  settings window does not know whether a scan is running, so the button is
  not disabled pre-emptively; the message in `#status` is the feedback.
- Not taken: cancel and join the running scan (as `scan_folder` does with
  `previous`), then clear, then let the auto-reopen rescan. It requires
  releasing the `Scans` lock to await the join and re-validating afterwards,
  i.e. the "snapshot the state you decided on" trap from
  `docs/agents/tauri-app.md`; not worth it for a maintenance button.
- Not an option: wait silently while holding the lock — that blocks the
  settings window's command with no feedback.

### Reopening the current folder after a clear

- **Chosen: emit `index-cleared` and let the main window reopen the open
  folder** (same code path as `sidecar-format`). Without it the strip keeps
  its in-memory thumbnails but any newly scrolled-in `thumbnail` call finds
  no row and shows a blank, and `folder_entries` returns nothing on the next
  refresh. Reopening also gives the measurement procedure its "first scan"
  immediately.

### Dirty (unwritten) ratings

`clear` keeps `ratings` rows with `dirty = 1`, as `evict_folder` and
`reset_sidecars` do, and the command flushes the writer first so almost none
remain. Consequence: the size after a clear is not exactly zero (a few KB of
`ratings` rows and the schema), and a folder with a still-dirty rating keeps
its `folders` row. Deleting them instead would lose a judgement that failed
to reach its sidecar, which the README promises not to do. This is stated in
the Cache panel's one-line explanation ("judgements are kept").

### How the size is computed, and what it includes

- **Chosen: the on-disk size**, i.e. `fs::metadata().len()` of
  `index.sqlite` + `index.sqlite-wal` + `index.sqlite-shm` (missing files
  count as 0). This is the number a user compares with what Finder /
  Explorer shows, it needs no database lock, and it makes the WAL hazard
  visible instead of hiding it: if `VACUUM` leaves a large `-wal`, the figure
  says so and Step 1's `wal_checkpoint(TRUNCATE)` is what makes it drop. It
  includes `-shm` (a few KB, irrelevant) and any not-yet-checkpointed WAL
  pages, so it can be slightly above the logical size mid-scan.
- Not taken: `Index::used_bytes` (pages in use). It is the figure eviction
  decides on and excludes the WAL entirely, so it understates disk usage
  when the WAL is large, and it needs a connection lock.
- Read **once when the settings window opens and again after a clear**, not
  live. A live figure would need polling or a scan-progress listener in the
  settings window for a number that only matters when the user is about to
  press the button. If refresh-on-focus turns out to be wanted, a `focus`
  listener calling `index_size` again is a one-liner; not included by
  default.

### Other risks

- `VACUUM` on a ~1 GB index (the eviction cap) takes seconds, not the ~0.2 s
  measured at 100 MB; it runs in `spawn_blocking` and the button is disabled
  while in flight, so the UI stays responsive, but there is no progress
  indicator. Note the measured time in `learnings.md` if a large index is
  at hand.
- The reader connection does not error during `VACUUM` (reasoned, not
  tested, per the eviction note); `folder_entries` calls landing during a
  clear may briefly wait on the 300 ms `busy_timeout` and return an error the
  main window shows in its status line. The auto-reopen after `index-cleared`
  supersedes any such in-flight refresh.
- `wal_checkpoint(TRUNCATE)` cannot complete while the reader holds a read
  transaction; the reader's transactions are short, so a `SQLITE_BUSY` there
  should be logged, not treated as a failed clear.
- GUI checks are manual on this Mac (`osascript` is denied); list what was
  and was not verified in each PR.

## Progress

- (2026-09-20) Step 1 complete
- (2026-09-20) Step 2 complete
- (2026-09-20) Step 3 complete
