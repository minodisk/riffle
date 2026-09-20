# todo

## Cross-cutting / other

### App: unmeasured end-to-end per-page latency

End-to-end per-page latency (IPC + `createImageBitmap`) is unmeasured, since the GUI could not be driven from this development machine. Only the Rust-side file-read cost was measured; see the Phase 4 baseline table in the README.

#### TODO

- [ ] Add a timing readout in the status line, or a Rust-side benchmark that includes the IPC hop, before Phase 4 tuning work begins.

### App: `riffle-cli info`/`focusbox`/`bench` read whole files instead of the bounded prefix

`info` / `focusbox` / `bench` in `crates/cli/src/main.rs` still call `std::fs::read(path)` for the whole file rather than `reader::read_head`'s bounded prefix, unlike the scan path and unlike `crop` (which moved to `reader::read_full`, a ranged read, in Phase 5 Step 1). This may be inherent: these subcommands need the full-size `JpgFromRaw`, which sits past the 1 MiB prefix.

#### TODO

- [ ] Decide whether these subcommands can move to `reader::read_head`/`reader::read_full` with a whole-file fallback, or whether they inherently need the whole file and this is not worth changing.

### App: a multi-file drop is rejected wholesale, and drag-hover gives no early feedback

The `tauri://drag-drop` handler in `crates/app/ui/src/main.ts` rejects a multi-item drop outright, even when every item shares one parent folder. Separately, the `body.dragging` overlay in `crates/app/ui/style.css` looks the same whether or not the payload will be accepted, although Tauri's `drag-enter` event already carries the paths.

#### TODO

- [ ] Take the common parent folder of a multi-file drop instead of rejecting it.
- [ ] Indicate during drag-hover whether the drop will be accepted.

### App: real-folder scan and second-open numbers are still missing

Every Phase 3 performance figure in the README (5.55s first scan, 34.4ms second open, the per-file timings) was measured on 5000 symlinks to one inode, or on freshly `cp`-copied files — never on a real folder of 5000 distinct ARWs on real hardware. Only the user can close this.

#### TODO

- [ ] Measure first-scan and second-open times on a real folder of ~5000 distinct ARW files, and update the README's numbers. The instrumentation now exists: the app logs `scan ...` and `open ...` timing lines, `Help > Open Log Folder` reveals `Riffle.log`, and README's "Measuring on your own folder" spells out the procedure. Only running the measurement and filling in the numbers is left.

### App: a deep-row focus point still exceeds the 50ms budget

A focus point in a deep row of the unrotated JPEG measured 58-65ms keypress to pixels (n=2, before the #56 and #60 fixes); see "The 1:1 focus check path" in the README. Options are prefetching the neighbouring files' crops (Phase 4's ring buffer) or a DCT-scaled placeholder; nothing is chosen.

#### TODO

- [ ] Retake the deep-row measurement post-fix, then decide between crop prefetch and a DCT-scaled placeholder.

### App: the silent update path is unverified end-to-end

`README.md`'s Updating paragraph describes a background download and install on launch and the "Check for Updates…" menu item, but nothing has confirmed on a real build that an installed copy detects a newer release, installs it silently, and launches as the new version next time.

#### TODO

- [ ] Verify the update path once a newer release is out: install the current release, publish the next one, then launch the installed build (and separately, use **Check for Updates…**). Done when the background flow installs the newer release after the signature check and it is used on the next launch, and the menu item reports the up-to-date / installed / already-installed outcomes correctly, on macOS, Windows, and Linux AppImage.

### Docs: write a guide for RAW metadata parsing (`docs/agents/raw-metadata-parsing.md`)

Leica DNG support found several non-obvious facts in `crates/core/src/{arw,reader}.rs`'s MakerNote/TIFF parsing that cost time in Steps 1 and 3: the Sony gate skips only when the note lacks `SONY` *and* `Make` is present and non-Sony, so non-Sony test fixtures must set `Make`; a `SubIFDs` entry with `count == 1` stores the IFD offset inline, not an offset to an array; the Leica MakerNote is `LEICA\0` + `02 00` then a little-endian IFD at note offset 8, with `FocusDistance` at tag 0x0304 (LONG, millimetres); `ApertureValue` (APEX) converts via `2^(AV/2)`; and a "non-Sony note is skipped" test needs a valid empty IFD in the fixture, not a `0xffff` sentinel count.

#### TODO

- [ ] When the next feature touches the MakerNote/TIFF parsing in `crates/core/src/{arw,reader}.rs` (another maker's MakerNote, or a new synthetic-TIFF fixture), create `docs/agents/raw-metadata-parsing.md` capturing the points above, linking `docs/plans/_archived/20260918-leica-dng-support/learnings.md` for the underlying measurements instead of duplicating them.

### Docs: consider a guide for verifying Pillow pixel edits

`app-icon-enlarge-marks` hit two Pillow pitfalls while verifying an in-place
pixel edit to `crates/app/icons/source.png`: (1) `Image.getbbox()` defaults to
`alpha_only=True` on RGBA images (Pillow >= 9.5), so a "no difference" check
built on `ImageChops.difference(...).getbbox()` can silently pass while the
RGB channels differ — use `getbbox(alpha_only=False)` or a per-pixel scan;
(2) measuring a resized/pasted mark inside the padded crop box that produced
it clips the bbox and understates the mark's size — the measurement window
must be wider than the source crop.

#### TODO

- [ ] The next time a plan does Pillow-based pixel verification, check whether
      it hits the same two pitfalls. If it does, write
      `docs/agents/verifying-pixel-edits.md` (or a section in an existing
      `tauri-app.md`-style guide) covering both, linking
      `docs/plans/_archived/20260921-app-icon-enlarge-marks/learnings.md` for
      the concrete numbers. If it does not recur, drop this item.

### App: the manual GUI checks for Move Rejected to Trash are still open

From `trash-rejected`'s implementation: GUI automation is unavailable on this
Mac and a native confirmation dialog cannot be driven by an agent, so nothing
of the menu item's visible behaviour has been run by a human. Two points are
specifically unverified: whether `NativeIcon::TrashFull` renders as a template
image in the macOS File menu (assumed, like `FollowLinkFreestanding`), and
whether the Trash's "Put Back" entry is actually created — the command pins
`DeleteMethod::NsFileManager` to avoid the Finder route's Automation
permission, and the `trash` crate documents that on some macOS systems files
moved that way get no "Put Back" entry (trash-rs#14); dragging them out of the
Trash still restores them. Files: `crates/app/src/main.rs` (`app_menu`),
`crates/app/src/commands.rs` (`trash_rejected`, `trash_context`),
`crates/app/src/trash.rs`, `crates/app/ui/src/trash.ts`,
`crates/app/ui/src/main.ts`.

#### TODO

- [ ] On macOS, verify: the menu item's icon renders as a template image at the
      same size as the other native-icon items; the confirmation names the
      right count (and the singular for one file) with `Move to Trash` /
      `Cancel`; Cancel leaves the folder untouched; confirming moves the RAW
      plus its `.xmp` and `.ARW.dop` to the Trash and the strip updates to the
      next passing file; the zero-reject, no-folder and mid-scan cases each
      write their message to `#status`; a "Put Back" from the Trash restores
      the file with its judgement, and if no "Put Back" entry exists, that
      dragging it out does.
- [ ] Verify the same flow on Windows and Linux (the `trash` crate's other
      backends have never been run here).

### App: custom menu-item icons don't tint for dark mode

muda never calls `setTemplate` on a custom menu `NSImage`, and Tauri exposes no
template flag for menu items (only for the tray icon), so a bundled PNG
(`Settings...` / `Undo` / `Open Folder…` / `Open Log Folder`,
`crates/app/icons/menu/`) cannot tint with the menu appearance the way native
icons do. The current PNGs are rendered in a neutral grey (`#8E8E93`) as a
legible-in-both-modes compromise.

A spike (menu-icon-glyph-size plan, reverted, not committed) confirmed the fix
is a one-line change in muda 0.19.3's `menuitem_set_icon`
(`src/platform_impl/macos/mod.rs`): adding `nsimage.setTemplate(true)` makes
every bundled PNG icon tint with the menu appearance exactly like the
OS-provided `Cut` / `Copy` / `Paste` items, and the baked `#8E8E93` grey
becomes irrelevant (a template image contributes only its alpha channel). No
upstream muda issue exists for this (all 92 issues, open and closed, checked;
nearest are #262, #240, #97, none of them this). Tauri's tray icon already
has `set_icon_as_template`; the concept was simply never extended to menu
items.

Patching requires a `path`/`git`-sourced fork (a crates.io-to-crates.io patch
via `[patch.crates-io]` is rejected outright) that still satisfies Tauri's
`muda = "^0.19"`, so a fork must stay on muda 0.19.3 rather than move to
0.20.

#### TODO

- [ ] Open an upstream PR against `tauri-apps/muda` adding
      `nsimage.setTemplate(true)` to `menuitem_set_icon`.
- [ ] Until it lands, adopt a `path`/`git`-patched fork of muda 0.19.3 via
      `[patch.crates-io]` in `Cargo.toml`, pinned to 0.19.3 to satisfy
      Tauri's `muda = "^0.19"`.
- [ ] Once menu-item images are templates, drop the now-dead `#8E8E93` fill
      step from `tools/macos/export-menu-icons.swift`.
- [ ] Done when `Settings...`, `Undo`, `Open Folder…` and `Open Log Folder`
      tint white in dark mode and black in light mode in the running app, and
      the grey fill step is gone from the export script.

Related: `tools/macos/export-menu-icons.swift`, `Cargo.toml`,
`crates/app/icons/menu/*.png`.

### App: the real-device checks for the File menu accelerators are still open

From `menu-accelerators`'s implementation: the GUI could not be driven from
the agent session, so the double-fire, stale-key-equivalent,
settings-capture and menu-set-from-`setup` checks are unconfirmed on macOS,
and Windows / Linux are unconfirmed entirely. Files: `crates/app/src/main.rs`
(`app_menu`), `crates/app/src/shortcuts.rs`, `crates/app/src/commands.rs`
(`update_keymap`), `crates/app/ui/src/main.ts`.

#### TODO

- [ ] On macOS, verify: one `Cmd+O` press opens the folder picker exactly
      once (no double fire from keydown + menu accelerator); one
      `Shift+Cmd+O` hands the folder to PhotoLab exactly once; rebinding
      `open`/`photolab` updates the menu accelerator and kills the old key,
      with the macOS key equivalent not staying stale; a menu set from
      `setup` shows correctly and doesn't steal focus from the settings
      window; pressing `Cmd+O`/`Shift+Cmd+O` while a shortcuts row is
      capturing does not trigger the menu action; both File menu items work
      via mouse click.
- [ ] Verify the `Some`/`None` accelerator behaviour on Windows and Linux
      (only reasoned from muda 0.19.3's sources so far, never run).

### App: the Clear Cache button's manual GUI verification is still open

From `clear-cache-stuck-guard`'s implementation: the button's guard used to
get stuck (never re-enabling after the first scan), so none of its GUI
behaviour has been run by a human. GUI automation is unavailable on this
Mac and a native confirm dialog cannot be driven by an agent. Files:
`crates/app/src/commands.rs`, `crates/app/ui/settings.html`,
`crates/app/ui/src/settings.ts`.

#### TODO

- [ ] Run `mise run tauri:release:devtools` and check, in order: (1) after
      a folder's scan finishes, Settings > Cache shows the button enabled
      and the note hidden; (2) pressing Clear Cache shows the confirmation
      dialog immediately, and Cancel leaves the size and `#status`
      unchanged with the button re-enabled; (3) Clear Cache then Clear
      shows `Clearing the index cache…` until the size drops and the main
      window rescans; (4) while a scan is running (or opening a large
      folder with Settings already open), the button is disabled and the
      note visible without any press, and both clear when the scan ends;
      (5) opening Settings during a large folder's prepare phase (before
      the first `scanning N / M` line) shows the button already disabled;
      (6) an error path, if reachable, writes the refusal to `#status` and
      it stays until the next press; (7) note whether the very first press
      after opening the settings window ever does nothing, and if so
      record the window focus state at that moment.

### App: unmeasured cost of a rescan on an unchanged large folder

The focus/manual rescan added by the live-folder-refresh feature (`crates/app/src/commands.rs`'s `scan_folder`, `crates/app/src/index.rs`) stats every file and reconciles the sidecar index even when nothing changed, plus a `folder_entries` read of every row on `scan-done`. This cost was not measured on a real large folder; if it turns out to be visible, the watcher's debounce window may need to grow.

#### TODO

- [ ] Measure a focus/manual rescan's wall time on an unchanged folder of ~5000 real files, and note whether it is noticeable enough to widen the debounce.

### App: a file picked up mid-copy may be scanned from a partial read

The folder watcher (`crates/app/src/watch.rs`, `crates/app/src/commands.rs`) can fire a rescan while a file is still being copied into the open folder, extracting a partial preview and writing that size/mtime into the index; the copy's completion later fires another event and `reconcile` re-extracts. The plan accepted this as expected behavior but it was never observed either way.

#### TODO

- [ ] Verify by hand (copy a large ARW/DNG into an open, watched folder) whether a partial mid-copy read ever produces a visibly wrong thumbnail/rating before the follow-up event corrects it, and whether any guard is warranted.

### App: the viewer empty-state manual checklist is still open

`viewer-empty-state`'s step 2 (overlay element, click-to-open, keymap-driven
re-render) could not be verified by running the app: `mise run tauri:dev`
needs an interactive session, unavailable in the implementation environment.
The behaviour was checked by reading the code paths only. Files:
`crates/app/ui/index.html`, `crates/app/ui/style.css`,
`crates/app/ui/src/main.ts`.

#### TODO

- [ ] Run `mise run tauri:dev` and verify: first launch with no remembered
      folder shows the clickable "no folder" prompt with the real `open` key;
      clicking it opens the native picker; reopening with a remembered folder
      hides it without a lingering flash; opening an empty folder shows the
      "no files" message; applying a filter that hides everything shows the
      "filtered" message and resetting the filter restores the view;
      rebinding the `open` key in Settings updates the shown key without
      restart; resizing the window and 1:1 zoom still render the canvas
      correctly now that `#viewer` (not `#canvas`) carries the flex sizing.

### App: a Rust test is flaky under load — `index::tests::the_reader_does_not_wait_on_an_open_write_transaction`

The test asserts a wall-clock budget of 100 ms for a read taken while a
write transaction is open; it failed once at 161 ms during
`clear-cache-stuck-guard`'s Step 3 local `mise run ci` run (an otherwise
idle machine) and passed on immediate re-run. File:
`crates/app/src/index.rs`.

#### TODO

- [ ] Loosen the timing bound, or otherwise make the assertion robust to
      scheduling noise, so an unrelated CI run does not intermittently fail
      on it.

### App: the filter menu closes when a thumbnail is clicked

Now that the filter menu opens as a fly-out beside the sidebar (leaving the
filmstrip visible), a user may want to click a thumbnail to inspect it while
keeping the filter menu open. Today the outside-`mousedown` handler in
`crates/app/ui/src/main.ts` (~line 1475) closes the menu on any click outside
`#filter`, including a strip click. Left out of the fly-out change as a
behaviour change; noted in
`docs/plans/_archived/20260920-filter-menu-flyout/plan.md`'s trade-offs.

#### TODO

- [ ] Decide whether a thumbnail click while the filter menu is open should
      keep the menu open, and implement if so.
