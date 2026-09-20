# Working on `crates/app`

Read this before touching Tauri commands, `crates/app/tauri.conf.json`, or the
`crates/app/ui/` frontend. It lists the pitfalls this repository has already
hit, each with the reason it happens.

Each item is tagged:

- **Hit**: actually broke something here.
- **Measured**: measured in this repo and steered a design decision; nothing
  broke, but the naive choice would have missed a stated budget or cost more.
- **Inferred**: taken from the sources or docs; nothing has broken on it yet.

Source: `docs/plans/_archived/20260917-tauri-skeleton/learnings.md` and the fix
in #11.

## Rust side

### Synchronous commands run on the main thread (Hit)

A `#[tauri::command]` without `async` runs **inline on the main thread**.

- Why: tauri-macros routes non-`async` commands through `body_blocking`, which
  calls the function directly from the IPC handler instead of spawning it.
- What broke: `pick_folder` called `blocking_pick_folder`, which parks the
  calling thread on a `sync_channel(0)` `recv`. The main thread stopped pumping
  the run loop, so the native dialog appeared but its buttons did nothing and
  the app hung.
- This passed `mise run ci`, the type check, and local review.
  It surfaced the first time the user launched the app. An earlier learnings
  note claimed sync commands run off the main thread; that note was wrong.

Rules:

- Anything that waits (dialogs, channels, locks held by the UI) goes in an
  `async` command. For callback APIs, await a
  `tauri::async_runtime::channel(1)`; see `pick_folder` in
  `crates/app/src/commands.rs`.
- An `async` command doing blocking IO wraps it in
  `tauri::async_runtime::spawn_blocking`, as `preview` does. Why: blocking
  inside the future stalls an async runtime worker instead.
- Confirmed again in Phase 3, with a related trap: HTML5 drag-and-drop never
  reaches the frontend. Tauri intercepts it, so `dataTransfer.files` carries no
  usable path and paths arrive only through the `tauri://drag-drop` (and
  `drag-enter` / `drag-over` / `drag-leave`) webview events, listened to via
  `event:listen` under `core:event`'s default `allow-listen`. Those events and
  `window.__TAURI__` are main-thread only; neither exists inside a worker.
- Exception: a synchronous command that only writes a small settings file
  (store set + save on the calling thread, as `update_keymap` and
  `set_auto_advance` do) does not need `spawn_blocking` — the write is a
  tiny JSON file, so the blocking cost is negligible. Reach for
  `spawn_blocking` when the IO is unbounded or can block on something other
  than a small local file.

### Resolve shortcut overrides order-independently, not in a single pass (Hit)

`from_overrides` in `crates/app/src/shortcuts.rs` checks each stored override
against the keymap built so far. A single pass in action order silently drops
a valid override that wants a key another override in the same batch frees
later: `{"pick": ["q"], "reject": ["p"]}` left `reject` on its default,
because `reject` comes before `pick`, so `p` still looked taken.

- Fix: apply the non-conflicting overrides first, then retry the conflicting
  ones until no more apply.
- When two overrides genuinely want the same key, the first in action order
  still wins.
- Source: `docs/plans/_archived/20260920-pick-shortcut-editable/learnings.md`,
  Step 1.

### An accelerator string is not validated until Tauri parses it (Inferred)

`accelerator()` in `crates/app/src/shortcuts.rs` is a plain table; muda is
not a dependency of `crates/app`, so nothing checks the string at compile
time. Tauri parses it with `.parse().ok()`, so an unrecognised name
silently becomes "no accelerator" instead of an error — a typo here fails
silently, not loudly.

A key override with only `shift` (or no modifier at all) also converts to
`None` on purpose: e.g. a plain `o` override leaves the corresponding menu
item without an accelerator rather than erroring.

- When adding or changing an accelerator mapping, don't rely on a build or
  test failure to catch a bad string or a modifier-less override; check the
  menu item's displayed accelerator by hand.
- Source: `docs/plans/_archived/20260920-menu-accelerators/learnings.md`,
  Step 1 (unverified on a real device; see "GUI automation does not work on
  this Mac").

### Measure before choosing a JPEG payload over raw pixels (Measured)

`mozjpeg::Compress`'s defaults turn on trellis quantisation and optimised
Huffman tables, so re-encoding a crop costs more than the partial decode that
produced it: 49ms at q85 for a 1037x1024 crop, against 21.5ms to decode it.

- Why: the defaults optimise for file size, and both passes run over every MCU.
- Phase 5's `focus_crop` therefore returns raw RGBA (4.2MB at the 1024 cap)
  rather than a ~180KB JPEG. That trades IPC bytes for CPU; take the
  measurement before trading back, and measure with the optimisations off, not
  with the defaults.

### A partial decode's cost is set by its row, not its size (Measured)

`jpeg_skip_scanlines` on a baseline JPEG still entropy-decodes the rows it
skips; it only skips the IDCT and colour conversion.

- Why: baseline Huffman data is not randomly addressable, so libjpeg must walk
  every MCU row from the start of the scan to reach the wanted one.
- Measured on the 7008x4672 `JpgFromRaw`: a 1024 crop costs 10.8ms at row 300
  and 44.1ms at row 4400, while growing the crop from 512 to 2048 at the same
  row only moves 18.4ms to 29.8ms.
- So a budget for a crop has to be stated for the worst row, not an average
  one, and shrinking the crop is not a way to make it fit.

### A crop's origin snaps to an MCU boundary; don't assume centred or bit-identical (Hit, twice)

`jpeg_crop_scanline` snaps the requested crop origin **down** to an MCU
boundary (16px in the tested files) and does not necessarily widen the crop to
compensate.

- Why: baseline JPEG DCT blocks (MCUs) are the smallest unit libjpeg can crop
  to; it never crops mid-MCU.
- What broke: two independent guesses were wrong. `FocusCrop`'s `point_x`/
  `point_y` are not the crop's centre (measured 269 vs a centre of 262 on one
  file). `crop_size()`'s width does not grow to keep the point centred (a
  64px-wide request at x=200 came back as width 64 at x=160, landing the point
  at 40, not 32) — that behaviour also isn't consistent across files (it does
  widen on a 4:2:2 test file). Always derive the point of interest as
  `centre - crop.origin` from the actual returned crop, never assume a fixed
  offset or that width grows.
- A crop decoded via `jpeg_crop_scanline` is also **not bit-identical** to the
  same region of a full `decode_rgb` near its left/right edges (chroma
  upsampling has one fewer neighbour there; differences up to 2 in the first
  columns, 1 in the last, on the tested file). Equality tests against a crop
  must exclude ~8px from each edge or use a tolerance there, not assert exact
  equality everywhere.
- Source: `docs/plans/_archived/20260918-focus-check/learnings.md`, Steps 1
  and 2.

### mozjpeg panics, rather than returning `Err`, on non-JPEG bytes (Hit)

`Decompress::new_mem` and friends panic on bytes that are not a JPEG instead
of returning a `Result::Err`.

- Any code path that cannot guarantee its input is a real JPEG (e.g. scoring an
  arbitrary embedded preview) must wrap its own call into mozjpeg in
  `catch_unwind` and convert the panic to an `Err`; callers then just call
  `.ok()` on it.
- Source: `docs/plans/_archived/20260919-sharpness-cue/learnings.md`, Step 1.

### Draining background work at quit needs `build()` + `run()` (Hit)

Work that must finish before the process ends — Phase 6's sidecar writer has a
300ms debounce, so a quit inside that window would otherwise leave the write
for the next folder open — is drained in `tauri::RunEvent::ExitRequested`.

- Why: `tauri::Builder::run(context)` takes only the context and gives no place
  to hook the run event. Use `Builder::build(context)?` and then
  `App::run(|app, event| ...)`; the closure's first argument is an `&AppHandle`,
  so `app.state::<...>()` works there.
- The writer's `Drop` is deliberately **not** the drain: a managed state's
  `Drop` is not guaranteed to run during process teardown, and even when it
  does, the channel disconnect races the exit instead of being waited on. Send
  an explicit flush message and block on a reply channel with a bounded wait
  (~2s) instead.
- This is the one place the "never block the main thread" rule above does not
  apply: the run loop is on its way out, and blocking there is what makes the
  drain a drain. Keep the wait bounded anyway.
- Source: `docs/plans/_archived/20260918-ratings-xmp-sidecars/learnings.md`,
  Step 3.

### Snapshot the state you decided on, not just the lock, across a release (Hit)

`reconcile_sidecars` (decides what to parse) and `store_sidecar_ratings`
(stores the parse and clears `dirty`) run with the lock released in between,
so a `set_rating` landing in that window was silently discarded by the
"sidecar wins" write.

- Why: releasing a lock between "decide" and "act" always opens a window for
  the state to change underneath the decision; re-checking "is the lock still
  free" is not enough, the *value* the decision was based on has to be
  re-checked too.
- Fix pattern: `reconcile_sidecars` hands back the `dirty` flag it observed
  for each path alongside the parse job, and `store_sidecar_ratings` only
  applies its update when the row's `dirty` still matches that snapshot —
  otherwise the newer write is left in place.
- Source: `docs/plans/_archived/20260918-ratings-xmp-sidecars/learnings.md`,
  Step 4.

### `frontendDist` resolves from the `tauri.conf.json` directory (Hit)

`tauri.conf.json` lives in `crates/app/`, not the conventional `src-tauri/`, so
the Vite output under the sibling `ui/` is `"frontendDist": "ui/dist"`.

- Why: Tauri resolves the path relative to the directory holding
  `tauri.conf.json`.
- What broke: the plan's `"../ui"` pointed at `crates/ui`, and
  `tauri::generate_context!()` failed the build because the path did not exist.

### `tauri icon` also writes `ios/` and `android/` icons (Hit)

Regenerate the icon set with
`pnpm exec tauri icon crates/app/icons/source.png -o crates/app/icons`. It
leaves `tauri.conf.json` alone, but it also creates `icons/ios/` and
`icons/android/`, which the app has no use for.

- Delete those two folders before committing.
- Source: `docs/plans/_archived/20260920-app-icon/learnings.md`, Step 1.

### Enabling the updater plugin pulls in `serde_json` at compile time (Hit)

Adding a `plugins.updater` block to `tauri.conf.json` makes
`tauri::generate_context!()` expand to code that references `serde_json`
directly, so `crates/app` needs it as a direct dependency or the build fails
with "could not find `serde_json` in the list of imported crates".

- Why: the generated context code assumes the crate already depends on
  `serde_json`; Tauri's own transitive dependency does not satisfy that.
- Source: `docs/plans/_archived/20260918-github-releases-auto-update/learnings.md`,
  Step 1.

### Self-update defers the install to quit on Windows only (Hit)

`tauri-plugin-updater-2.11.0`'s macOS and Linux (AppImage) `install_inner`
only extract and rename bundles (`std::fs::rename`, with an authorised
fallback); they never exit or signal the process, so `update.rs` keeps
`download_and_install` there and `RunEvent::ExitRequested` runs normally on a
later real quit. Windows is different: the running exe is locked, so its
`install_inner` launches the installer and calls `std::process::exit(0)`
directly. `update.rs` therefore only calls `Update::download` on Windows (in
the background, signature-verified) and keeps `(Update, Vec<u8>)` in
`UpdateRun`; `main.rs`'s `ExitRequested` arm calls `update::install_pending`
right after the sidecar writer's flush, so the installer starts only once
every pending write is on disk. The updater is built with
`restart_after_install(false)` (the Windows default is `true`), so quitting
means quitting. Compiled, not exercised against a real install.

- Why: only Windows needs the exe unlocked before it can overwrite itself;
  macOS/Linux replace files the running process isn't holding open. Deferring
  on macOS would run `install_inner`'s `run_on_main_thread` fallback from the
  main-thread quit callback, a deadlock risk.
- When touching the update or sidecar-flush paths, keep `install_pending`
  after the flush in `ExitRequested`, and don't assume `install` returns on
  Windows.
- Source: `docs/plans/_archived/20260919-silent-auto-update/learnings.md`,
  Step 1; `docs/plans/_archived/20260920-windows-deferred-update/`.

### App items go into the default menu's own submenus (Hit)

The menu bar keeps only the platform's default submenus. `app_menu::build`
finds them in `Menu::default` by title and inserts into them: `Settings...`
(`CmdOrCtrl+,`) after About in the macOS app menu (in `File` elsewhere), and
`Open in DxO PhotoLab` at the top of `File`. Linux's default has no `File`, so
one is prepended there. `Edit` ships a predefined Undo/Redo pair at its top
that owns `CmdOrCtrl+Z`; the app's `Undo` (emitting `undo` to the frontend)
replaces that pair rather than being added next to it. The replacement only
fires when the item at position 0 is still `MenuItemKind::Predefined`, so a
Tauri reordering cannot make it silently remove the wrong item. Settings themselves (sidecar format, shortcuts, the
dev-only timing logs) live in a separate `settings` window
(`ui/settings.html`), not in menu check items, so the menu reads no plugin
state and is built in `Builder::menu`.

The settings window is its own JS context, so a change reaches the main window
as a backend event (`shortcuts-changed`, `sidecar-format`, `debug`), and state
both windows read (timing logs) lives in Rust. The window is listed in
`capabilities/default.json` so it can invoke, and it is closed when `main` is
destroyed so it never keeps the app running alone.

- Why: a submenu per setting cluttered the menu bar; macOS apps put
  `Settings...` in the app menu.

### Rebuild the menu with `set_menu`, not `set_accelerator(None)`, to clear a stale macOS key equivalent (Inferred)

The menu is no longer passed to `Builder::menu`; `setup` calls
`app_menu::refresh` right after `load_settings`, and `update_keymap` calls
it again whenever an accelerator changes. Rebuilding the whole menu through
`AppHandle::set_menu` is what clears a stale macOS key equivalent when an
override moves a key away from an item; muda's `set_accelerator(None)` on
the existing item does not clear it.

- When a code path changes which action owns an accelerator, rebuild via
  `app_menu::refresh` (which calls `set_menu`) rather than mutating an
  existing `MenuItem`'s accelerator in place.
- Source: `docs/plans/_archived/20260920-menu-accelerators/learnings.md`,
  Step 1 (unverified on a real device).

### Menu icons: native where one exists, a bundled SF Symbol otherwise (Inferred)

On macOS the four app items carry an icon. `Open in DxO PhotoLab` and
`Check for Updates…` use `IconMenuItem::with_id_and_native_icon` with
`NativeIcon::FollowLinkFreestanding` / `NativeIcon::Refresh`, which are
template images and tint with the menu. `NativeIcon` has neither an undo nor a
modern gear, so `Settings...` and `Undo` use `IconMenuItem::with_id` with an
`Image::from_bytes(include_bytes!(...))` of a PNG committed under
`crates/app/icons/menu/` (which is why `crates/app/Cargo.toml` enables Tauri's
`image-png` feature). Other platforms keep the plain `MenuItem` behind `cfg`.

Regenerate those PNGs with `swift tools/macos/export-menu-icons.swift`, and
only when a symbol, its size, weight or colour changes; AppKit's rasterisation
can differ between macOS releases, so the committed files are the source of
truth. The script never runs at build or run time.

- Limitation: muda does not call `setTemplate` on a custom menu image and
  Tauri exposes no template flag for menu items, so the bundled PNGs do not
  tint for dark mode. They are rendered in a fixed neutral grey (`#8E8E93`)
  that stays legible in both appearances.

### Adding a macOS menu item needs the same `cfg` split as its siblings (Hit)

Every macOS menu item in `app_menu` is behind `#[cfg(target_os = "macos")]`
(using `IconMenuItem`) with a `#[cfg(not(target_os = "macos"))]` twin (plain
`MenuItem`), because `MenuItem` is only imported under the latter cfg. A new
item added as a plain, un-cfg'd `MenuItem::with_id(...)` merges *textually*
clean with unrelated concurrent menu-icon changes but breaks macOS
compilation (`MenuItem` no longer in scope there).

- A branch-only CI pass does not catch this: the branch's own copy of the
  import still has `MenuItem` in scope, and the break only exists in the
  merge result. If a failure is isolated to one OS's CI job right after a
  merge/rebase and looks like a runner flake, re-run it once to rule out
  flakiness, then check out the actual merge ref (not the branch head) —
  GitHub builds the PR's merge commit, so line numbers and blame on the
  branch head alone can point at unrelated code.
- Give any new platform-specific menu item the existing `cfg` split up
  front, following the bundled-icon pattern above.
- Source: `docs/plans/_archived/20260920-scan-timing-logs/learnings.md`,
  "Merge conflict with concurrently-landed macOS menu icon work".

### A case-insensitive file system makes `exists()` match the wrong spelling (Hit)

On macOS APFS, `sidecar_path(arw).exists()` is true for `H.ARW.DOP` even when
the code built the path as `h.arw.dop` (or vice versa), because the file
system does case-insensitive but case-preserving lookups.

- Why: the OS resolves the path regardless of case, so an existence/rename
  check can silently hit a sidecar with different case than the one just
  built.
- Any code that renames or writes a sidecar based on `exists()` needs to
  tolerate landing on the other case's file rather than assuming its own
  spelling was used; do not assert on the exact resulting name in tests, only
  that exactly one sidecar exists.
- Source: `docs/plans/_archived/20260918-photolab-dop-sidecar/learnings.md`,
  Step 2.

### `.dop` indentation follows the table, and keys are not stable across versions (Hit)

PhotoLab indents an anonymous item's `{`/`}` at the same depth as its fields,
but a keyed table's closing `}` at the key's own depth. "Copy the closing
line's indentation" is correct for inserts into `Items[0]` but under-indents
an insert into a keyed table such as `Sidecar`.

- Key names change across PhotoLab versions (`CafId` became `CafID` in
  10.0.2); do not treat the sidecar's key set as stable.
- `dop::write_label` with `None` on a sidecar that has no label still updates
  the two timestamps. A caller that wants a true no-op must not call it.
- Source: `docs/plans/_archived/20260919-color-labels/learnings.md`, Step 2.

### Removing an XMP element needs its end tag (Hit)

`xmp.rs`'s `locate` takes the property's local name (`LocalName` compares
against `&str`). For an element-form property it cannot return on the `Text`
event: it waits for the end tag so it can report the whole element's range for
removal.

- Clearing an element-form label removes its whole line only when nothing but
  whitespace shares that line; otherwise it removes just the element.
- Values are spliced raw with no escaping, so a value containing `"` or `<`
  would break the XMP. Fine for the label vocabularies; check it again for any
  new use of this splice path.
- Source: `docs/plans/_archived/20260919-color-labels/learnings.md`, Step 1.

### quick-xml 0.42 namespace resolution: `&str`, not `&[u8]`, and a borrowed `QName` (Hit)

`ResolveResult::Bound(Namespace)`'s `as_ref()` yields `&str` — compare it
against `XMP_NS` directly, not as bytes. A helper that resolves an
attribute's namespace and returns `ResolveResult<'a>` must tie the
lifetime to the *reader*, not to a local `String` holding the name: the
only variant that borrows (`Bound`) points into the reader's own
namespace buffer, so binding the lifetime to a temporary buffer fails to
compile (or is subtly wrong if it does).

- Source: `docs/plans/_archived/20260920-robustness-cleanup/learnings.md`, Step 2.

### Bumping `SCHEMA_VERSION` can strand an old per-version column guard (Hit)

A migration guard that adds a column for "any version other than the current
one" (e.g. `ratings.label_known` at v6) breaks once a later change bumps
`SCHEMA_VERSION` again (v6 -> v7): the guard now also fires for v6 databases
that already have the column, and the `ALTER TABLE` fails.

- Key such a guard to the version that introduced the column (`version < 6`),
  not to the ever-moving current version, and re-check the versions asserted by
  the existing migration tests whenever `SCHEMA_VERSION` moves.
- The same trap applies to a table drop: `files` used to be dropped for any
  `version != SCHEMA_VERSION`, which would have thrown away every thumbnail on
  the v7 -> v8 upgrade. It is now keyed to `version < 7`, and v8 seeds the new
  `folders` table from the surviving `files` rows.
- Source: `docs/plans/_archived/20260919-sharpness-cue/learnings.md`, Step 2;
  `docs/plans/_archived/20260920-app-quick-fixes/learnings.md`, Step 3.

### Folder-index eviction: lock order and where it runs (Measured)

Eviction (`commands::spawn_eviction`) runs once from `setup`, right after
`app.manage(Scans)`, on a plain `std::thread`. It holds the `Scans` lock and
then the writer lock for the whole evict + `VACUUM`, and skips if
`Scans.running` is already set, so a `scan_folder`/`start_scan` issued
meanwhile just waits. Keep the order `Scans` then writer. The size cap counts
pages in use (`page_count - freelist_count`), since a delete only moves pages
to the freelist until `VACUUM` runs.

`Index::clear` (behind the settings window's Clear Cache button, via
`commands::clear_index`) is the second caller of `evict_folder` and keeps the
same order: `Scans` lock first (re-checked under it after the confirmation
dialog, since the dialog is awaited with no lock held), then the writer lock.
A running scan is refused, never cancelled.

- Measured (`Index::clear`'s unit test, 2 folders x 5 rows of a 200KB
  thumbnail): `VACUUM` alone gives nothing back to the file system, because
  the rebuilt database is written through the WAL — the on-disk footprint
  (`index.sqlite` + `-wal`) went from 2,121,808 B to 2,212,448 B, i.e. it
  *grew*, though `page_count` dropped by more than 10x. Ending with
  `PRAGMA wal_checkpoint(TRUNCATE)` took it to 40,960 B. Any eviction path
  that is meant to shrink the file needs that checkpoint after the `VACUUM`.
- `rusqlite`'s `pragma_query(None, "wal_checkpoint(TRUNCATE)", ..)` compiles
  but silently does nothing (it quotes the argument, so `TRUNCATE` is lost);
  use `query_row("PRAGMA wal_checkpoint(TRUNCATE)", ..)` instead. A
  `SQLITE_BUSY` from it is fine to just log and continue.
- Measured (M3 Pro, release, ignored test `vacuum_cost_on_a_100_mb_index`):
  evicting half of 5000 rows of 20.8KB thumbnails (106MB in use) and vacuuming
  took ~220ms, leaving 53MB. That a WAL reader does not error during `VACUUM`
  is reasoned, not covered by a dedicated concurrent test.
- Source: `docs/plans/_archived/20260920-app-quick-fixes/learnings.md`, Step 3;
  `docs/plans/20260920-clear-index-cache/learnings.md`, Step 1.

### A per-tick log line can rotate other lines out of the 40 KB default (Measured)

`tauri-plugin-log`'s `DEFAULT_MAX_FILE_SIZE` is 40 KB. A line logged once per
progress tick (e.g. one per `scan-progress` event, capped at 100 ms) at
~120 bytes each can reach ~300 lines (~36 KB) for a single ~30s/5000-file
operation — nearly filling the budget on its own and rotating out other
commands' timing lines from the same run. Before adding a log line inside a
polling/progress loop, check its expected line count × size against the
plugin's cap; raise `max_file_size` on the log plugin builder in `main.rs`
rather than trying to suppress the log (gating on state the command doesn't
have is more machinery than it's worth).

- Source: `docs/plans/_archived/20260920-scan-timing-logs/learnings.md`, Step 2.

### `focus_crop`'s header carries the full JPEG size (Hit)

`riffle_core::partial::Crop` carries the full JPEG's `image_width`/
`image_height`. The `focus_crop` header encodes them as two `u16`s at offsets
28/30 under kind tag 5 (`CROP_KIND_RGBA_V3`); the next header change takes tag
6. The zoom placeholder (`crates/app/ui/src/zoom.ts`) falls back to the sensor
size until the current file's crop arrives.

- Source: `docs/plans/_archived/20260920-app-quick-fixes/learnings.md`, Step 1.

### This crate is on Rust edition 2021: no `if ... && let` chains (Hit)

`if let Some(x) = a && let Some(y) = b` does not compile here; nest the
`if let`s.

- Source: `docs/plans/_archived/20260920-app-quick-fixes/learnings.md`, Step 2.

## Frontend (`crates/app/ui`, Vite+)

### Undo must re-anchor conditionally, not unconditionally (Hit)

`refilter(anchor)` re-anchors the view on the given file when it is still
visible under the current filter, but anchoring on a file the filter now
hides moves the current file to a neighbour instead. So the shared
apply-and-invoke `commit(...)` behind judge and undo applies the state first,
then evaluates an optional anchor thunk against the new state: it anchors on
the target file only when it still passes the filter, and otherwise leaves the
current view alone.

- Why: anchoring on the just-undone file after every undo silently jumps the
  view when that file no longer passes the active filter.
- Source: `docs/plans/_archived/20260919-undo-judgements/learnings.md`, Step 1.

### `ErrorList`'s `Map` keeps a superseded entry in its original position (Hit)

Re-`set`ting an existing key on a `Map` used as `ErrorList`'s backing store
does not move that entry to the end; a superseded error stays where it was
first inserted, not where it was last updated.

- Why: don't assume replacing a `Map` entry reorders it — display order
  relying on "most recent first/last" needs an explicit ordering field, not
  `Map` iteration order.
- Source: `docs/plans/_archived/20260920-sidecar-error-display/learnings.md`,
  Step 1 (pinned by `errors.test.ts`).

### A judgement's own move must not double up with `refilter`'s move (Inferred)

`judge` returns whether it changed anything. The keydown handler records the
current path *before* the switch/commit runs, then — only if that path is
still the current file afterward — calls `move(1)` for auto-advance.

- Why: `commit`'s `refilter` step already moves the view off a file that
  drops out of the active filter as a result of the judgement. If the
  keydown handler also unconditionally called `move(1)`, a filtered-out file
  would advance twice. Checking "is the just-judged file still current"
  before moving is what prevents the double skip.
- `move` already clamps at the last file, so the auto-advance call does not
  need its own bounds check.
- Source: `docs/plans/_archived/20260919-auto-advance/learnings.md`, Step 3.

### A derived-state refresh has to run even when `refilter` short-circuits (Hit)

`refilter` returns early when the visible file list did not change, so per-file
UI state derived from the list (e.g. the sharpness cue's `applySharpness()`)
cannot rely on `refilter` alone to recompute it.

- Call the derivation explicitly wherever the underlying data is refreshed
  (`refreshEntries`), and also inside `refilter` after `setFiles`, which clears
  the strip's per-index store.
- Source: `docs/plans/_archived/20260919-sharpness-cue/learnings.md`, Step 3.

### Give the current folder one token, not one counter per feature (Hit, repeatedly)

Every async path that can outlive the folder it started for — a preview load, a
thumbnail fetch, a background scan, an open by drop — has to ask, when its
result comes back, whether that folder is still the current one. This repo
re-derived the same guard four times under four names (`seq`, `generation`,
`folderToken` / `openDir`, `dropCounter`), and every one was caught in review
rather than by a test.

- Why: any `await` or callback across the IPC boundary can resolve after the
  user has picked, dropped or paged to something else, and the frontend has no
  other signal that the result is now stale.
- When you add an async path that depends on "the current folder", check the
  existing token for the current open operation instead of minting a new
  counter, and check it before applying the result, not before starting.
- The four instances:
  `docs/plans/_archived/20260918-thumbnail-cache-filmstrip/learnings.md`
  (Steps 2, 5, 6, 7).
- Two corollaries from Phase 6's review (Step 5): comparing the directory
  string is **not** enough, because reopening the same folder is a new open
  with the same `dir` — `refreshEntries` checks `dir !== openDir ||
  token !== folderToken` for that reason. And a backend **event** listener,
  which is registered once and outlives every folder, needs the same kind of
  guard on its payload before it touches the UI; the `sidecar-error` listener
  drops a payload whose path is not in the current folder's index.
- Source: `docs/plans/_archived/20260918-ratings-xmp-sidecars/learnings.md`,
  Step 5.

### Two folder paths: `openDirectory` resets, `resync` keeps state (Inferred)

`main.ts` has two ways to point the UI at what is on disk, and a new "the
folder changed" event has to pick the right one.

- `openDirectory(folder, token)` is the **reset** path: it mints a folder
  token, clears `entries`, `ratings`, `picks`, `labels`, `sharpness`,
  `touched`, `history` and `errors`, sets `index = 0` and scrolls the strip
  back to the top. Use it when the *judgements* are no longer valid — the
  `sidecar-format` and `index-cleared` listeners, which run after the backend
  reset the index.
- `resync()` is the **keep-state** path: the same open, so no new token; it
  re-lists the folder, runs the same `scan_folder` -> `start_scan` diff
  through the shared `startScan` helper, and re-anchors with
  `refilter(currentPath, true)`, which keeps the strip's scroll offset. Use it
  when only the *files* may have changed — the window focus and
  `File > Reload Folder` triggers.
- `resync` defers to `scan-done` while a scan runs (`scanRunning`), because
  `scan_folder` cancels and joins the running scan first; a focus change
  during a 5000-file first scan would otherwise restart it. Repeat triggers
  collapse into the single `resyncPending` flag.

### Style the strip placeholder on `.cell img:not([src])`, never on `.cell img` (Hit)

`createCell` in `crates/app/ui/src/strip.ts` appends an `<img>` with no `src`
and sets `src` only once the thumbnail payload arrives; cells are recreated
rather than reused on refresh, so `src` is never stale. That makes
`.cell img:not([src])` exactly "placeholder or failed load". Put the grey
placeholder background there, not on `.cell img` itself: the image box is the
144px square footprint and `object-fit: contain` letterboxes anything that is
not square, so a background on `.cell img` shows as grey bands around every
loaded thumbnail (24px above and below a 3:2 one).

- Source: `docs/plans/_archived/20260920-aspect-independent-strip-cells/learnings.md`,
  Step 1.

### Name a modified key from `event.code`, not `event.key` (Hit)

On macOS, Option and Shift change `event.key` (⌃⌥1 reports `¡`, not `1`), so
a key pressed with any modifier is named from `event.code` (`Digit1` -> `1`,
`Comma` -> `,`) after the held modifiers in `ctrl+alt+shift+meta` order; a
plain key keeps the lower-cased `event.key`. The shortcuts panel's capture must
use the same naming, and a lone modifier must be skipped before the derived
name is built, or holding Ctrl+Alt records `ctrl+alt+altleft`. Checking
`event.key` alone is not enough: it can arrive in an unexpected case
(`control`, not `Control`) or be unrecognizable while `event.code` still names
the modifier (`ControlLeft`/`Right`, `AltLeft`/`Right`, `ShiftLeft`/`Right`,
`MetaLeft`/`Right`, and the legacy `OSLeft`/`OSRight`). Reject a lone modifier
by checking both: `event.code` against that list, and a lower-cased `event.key`
against `control`/`alt`/`shift`/`meta`.

- Why: the keymap compares names; a name built one way on dispatch and another
  way on capture never matches.
- The `control` chip that still appeared after the lone-modifier guard landed
  was a stale `crates/app/ui/src/keys.js` shadowing `keys.ts` in the dev
  server, not a gap in the guard; do not widen the guard for it.
- Source: `docs/plans/20260919-color-labels/learnings.md`, Step 5,
  `docs/plans/_archived/20260920-ignore-lone-modifier-keys/learnings.md`, and
  `docs/plans/20260920-ignore-stale-ui-js/learnings.md`.

### Carry every judgement field on every write (Hit)

`set_rating` writes the whole judgement (stars, pick, label), so the frontend
keeps a `labels` map from `folder_entries` and passes the current label with a
star or flag keypress; otherwise a rating clears the label. Until the label is
known (`folder_entries` not yet arrived), the backend is told `labelKnown:
false` and keeps whatever the sidecar holds; that state is stored
(`ratings.label_known`, schema v6) so a crash before the write does not strip
the sidecar's label on the replay.

- Source: `docs/plans/20260919-color-labels/learnings.md`, Steps 4 and 6.

### A `.ts` file with no `import`/`export` is a global script (Hit)

Its top-level `const`/`let` share scope with `lib.dom` globals; `const status`
collided with `window.status`. Every file under `src/` imports or exports
something today; keep it that way rather than adding `export {};` (Oxlint flags
that as `no-useless-empty-export`).

- Why: TypeScript only treats a file as a module when it has a top-level
  `import` or `export`.

### Workers: declare the scope locally (Hit)

`worker.ts` declares the members it uses as a local `WorkerScope` interface and
casts `self` to it. Keep one `tsconfig.json` for both threads this way.

- Why: `DedicatedWorkerGlobalScope` is only in the `webworker` lib, and adding
  `webworker` next to `dom` clashes on the globals both declare.

### Write relative imports with `.js` (Measured)

`import { x } from "./foo.js"`; Vite resolves the `.js` suffix to the `.ts`
source in dev and build, and it matches what the type check expects.

### A real `.js` beside a `.ts` shadows the source (Hit)

Because those imports carry the `.js` suffix, an actual `foo.js` sitting next
to `foo.ts` wins: the dev server serves the `.js` and the `.ts` is never
compiled. A `tsc`-era build left such files behind in `crates/app/ui/src/`, so
the dev build ran code months older than the sources. `.gitignore` now covers
`crates/app/ui/src/**/*.js`, so such files no longer show up in `git status`,
and `tsconfig.json` sets `"noEmit": true` so a stray `tsc` cannot regenerate
them.

- Why: when the running app disagrees with the source you are reading, check
  `ls crates/app/ui/src/*.js` (or `git clean -nX crates/app/ui/src`) before
  debugging the code, and restart the dev server after deleting them — Vite
  caches the old transform.
- Source: `docs/plans/20260920-ignore-stale-ui-js/learnings.md`, Step 1.

### `vp check` type-checks with TypeScript-Go and covers `vite.config.ts` (Hit)

- Its newer DOM lib types `HTMLElement.hidden` as `boolean | "until-found"`, so
  passing it where a `boolean` is expected fails (TS2345); `tsc` 5 accepted it.
  Coerce with `!!`.
- The root `vite.config.ts` is type-checked too, so `process` / `node:path`
  need `@types/node` as a devDependency.
- `vitest` has to be a direct devDependency even though `vite-plus` brings it
  at runtime: pnpm does not hoist it, and `import ... from "vitest"` in a test
  fails the type check with TS2307.
- Source: `docs/plans/20260919-vite-plus/learnings.md`, Steps 1 and 5.

### Vite+ config facts (Measured)

- `settings.html` is a second entry in `build.rollupOptions.input`; both HTML
  files land at the root of `ui/dist`, so `WebviewUrl::App("settings.html")`
  is unchanged.
- `test.include` is relative to the Vite `root` (`crates/app/ui`), so it reads
  `src/**/*.test.ts`.
- `vp fmt` honours `.gitignore`. Its scope (and `lint.ignorePatterns`) is the
  frontend plus the root JS tooling files; Markdown, the release-please JSON
  and `.claude/**` are excluded.
- Adding `vp fmt` to `mise run fmt` before the reformat commit would have made
  every pre-commit `mise run fmt` reformat the frontend; land the reformat
  together with (or before) the task change.
- Source: `docs/plans/20260919-vite-plus/learnings.md`, Steps 1-4.

### On Windows, run `vp` through `node`, not `pnpm exec`, in mise tasks (Hit)

The `test` task calls `node ./node_modules/vite-plus/bin/vp test`.

- Why: the mise task shell is bash; `pnpm exec vp` there resolves to the `.cmd`
  shim, which runs under cmd.exe with bash's POSIX-style `PATH` and cannot find
  `node`.
- Source: `docs/plans/20260919-vite-plus/learnings.md`, Step 5.

## CI

### Do not hard-code the pnpm store path (Inferred)

Cache the output of `pnpm store path --silent` (see `.github/workflows/`).

- Why: the store location differs per OS.

### `tauri-action@v0` has no `uploadWorkflowArtifacts` input (Hit)

Setting it silently produces zero build artifacts (the run logs "Unexpected
input(s) 'uploadWorkflowArtifacts'" and continues). Upload bundles explicitly
with a separate `actions/upload-artifact@v4` step over
`target/release/bundle/**` instead of relying on the action to do it.

- Source: `docs/plans/_archived/20260918-github-releases-auto-update/learnings.md`,
  Steps 2 and 4.

### Cache Rust builds by `matrix.os`, not `runner.os` (Hit)

`macos-latest` (arm64) and `macos-15-intel` (x86_64) both report
`runner.os == macOS`; a cache key built from `runner.os` would let them
restore each other's `target`, which is wrong across architectures. Key on
`matrix.os` instead.

### release-please: the `rust` strategy fails on a workspace root with no `[package]` (Hit)

`release-please release-pr --dry-run --release-type rust` (v17.9.0) aborts
with `is not a package manifest (might be a cargo workspace)`, because
`CargoToml.updateContent` tries to update the root `Cargo.toml` as a package
even when it only declares `[workspace]`. Use `simple` with `extra-files`
pointing at each crate's `Cargo.toml` instead.

- For a `Cargo.lock` extra-file, `$.package[?(@.source === undefined)].version`
  is the working JSONPath discriminator for "this workspace's own crates" —
  equality on `@.name` matches nothing, and `startsWith`/`match` are rejected
  by jsonpath-plus's safe evaluator.
- `release-pr --dry-run` reads `release-please-config.json` from the **remote**
  branch, not the local working tree, so testing a not-yet-merged config needs
  a scratch clone/bare-repo setup with the config pushed to its `main`.
- Source: `docs/plans/_archived/20260918-github-releases-auto-update/learnings.md`,
  Step 4.

### A green branch-only CI run does not prove the merge result compiles (Hit)

See "Adding a macOS menu item needs the same `cfg` split as its siblings"
above: two branches can each pass CI alone and still break when merged,
because each branch's own copy of an import/cfg is still in scope on that
branch. Only building the actual merge/rebase result (not the branch tip)
catches it; a single-OS failure right after a merge that looks like a
runner flake is a signal to check the merge ref before assuming flakiness.

## Verification

### Write a performance number with its measurement conditions (Hit, twice)

Two performance claims here had to be walked back in review because they read
as general when they were not: a 30-second target that rested on warm `cp`
reads, and a batch of README numbers measured on folders of symlinks to a
single file.

- Why: a number without its conditions — page cache state, and whether the
  files were distinct or 5000 symlinks to one inode — is read as a general
  claim and copied forward as one.
- State what was actually measured next to the number, and say plainly what
  still needs a real-folder measurement.

### GUI automation does not work on this Mac (Hit)

`osascript` is denied assistive access: the app's window cannot be brought
forward, and injected keystrokes are dropped silently.

- Why: macOS privacy settings on this machine, not something the repo controls.
- Plan any GUI check as a **manual confirmation by the user**, and list exactly
  what they should look at. Report unchecked behaviour as "not verified"; that
  is more useful than an implied pass.

### `riffle-app` is bin-only: use `cargo test <name>`, not `--lib` (Hit)

`cargo test --lib` fails because `crates/app` has no library target.

- Run `cargo test <test_name>` (optionally scoped with `cd crates/app`) instead.

### A rating/pick test needs a scan-populated `files` row before `rating_of` reads back (Hit)

`rating_of` joins `files`, which only a folder scan fills in. A unit test
that calls `write_batch` (or the extracted `switch_format` body in
`crates/app/src/commands.rs`) and then reads the rating back must first
call `index::stat` to populate that row, the same way
`a_foreign_sidecar_is_read_on_the_first_open_without_any_files_row` does —
otherwise the read returns nothing regardless of whether the write
succeeded.

- Source: `docs/plans/_archived/20260920-robustness-cleanup/learnings.md`, Step 4.
