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

### App: the filmstrip re-requests every visible placeholder on each `scan-progress` event

`refresh` in `crates/app/ui/src/strip.ts`, driven from the `scan-progress` handler at ~10/s, re-requests every visible placeholder rather than only the indices the scan has newly passed. It is bounded by the 4-in-flight cap and the visible range, but it is avoidable IPC. Relatedly, the strip discovers whether a file has a thumbnail by invoking `thumbnail` and treating an `Err` as "not yet", rather than reading `has_thumb` from the `folder_entries` map, because keeping that map fresh during a scan would mean the same 10/s full re-read.

#### TODO

- [ ] Use the `done` counter or per-file `has_thumb` state to request only newly available thumbnails, if the strip turns out to be IPC-bound.

### App: a multi-file drop is rejected wholesale, and drag-hover gives no early feedback

The `tauri://drag-drop` handler in `crates/app/ui/src/main.ts` rejects a multi-item drop outright, even when every item shares one parent folder. Separately, the `body.dragging` overlay in `crates/app/ui/style.css` looks the same whether or not the payload will be accepted, although Tauri's `drag-enter` event already carries the paths.

#### TODO

- [ ] Take the common parent folder of a multi-file drop instead of rejecting it.
- [ ] Indicate during drag-hover whether the drop will be accepted.

### App: real-folder scan and second-open numbers are still missing

Every Phase 3 performance figure in the README (5.55s first scan, 34.4ms second open, the per-file timings) was measured on 5000 symlinks to one inode, or on freshly `cp`-copied files — never on a real folder of 5000 distinct ARWs on real hardware. Only the user can close this.

#### TODO

- [ ] Measure first-scan and second-open times on a real folder of ~5000 distinct ARW files, and update the README's numbers. The instrumentation now exists: the app logs `scan ...` and `open ...` timing lines, `Help > Open Log Folder` reveals `Riffle.log`, and README's "Measuring on your own folder" spells out the procedure. Only running the measurement and filling in the numbers is left.

### App: `scan-progress` carries no way to tell what became available

`scan-progress` carries only `{dir, scan_id, done, total}`, so #56 had to throttle the filmstrip refresh to ~1/s rather than re-request only the thumbnails that became available. Carrying the newly-written paths, or a done-index high-water mark, would let the strip ask for exactly what is ready.

#### TODO

- [ ] Extend the `scan-progress` payload so the filmstrip can request only the newly available cells (`crates/app/src/commands.rs`'s `scan-progress` emit, `crates/app/ui/src/strip.ts`, `crates/app/ui/src/main.ts`).

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

### App: rejected files cannot be cleared out from the app

Culling ends with the rejects still in the folder; removing them means going to
another tool.

#### TODO

- [ ] Add a menu item that moves every rejected file of the open folder, with
  its sidecars, to the OS trash (or a chosen folder), after a confirmation
  showing the count.

### App: custom menu-item icons don't tint for dark mode

muda never calls `setTemplate` on a custom menu `NSImage`, and Tauri exposes no
template flag for menu items (only for the tray icon), so a bundled PNG
(`Settings...` / `Undo`, `crates/app/icons/menu/`) cannot tint with the menu
appearance the way native icons do. The current PNGs are rendered in a neutral
grey (`#8E8E93`) as a legible-in-both-modes compromise. Upstreaming template
support in muda / Tauri is the proper fix.

#### TODO

- [ ] Investigate and, if feasible, upstream `setTemplate` support for custom
      menu item images in muda / Tauri, then switch `crates/app/src/main.rs`
      (`app_menu`) to a template image instead of the fixed-grey PNG fallback.

### App: PNG-backed menu icons render larger than native ones

muda hardcodes an 18pt height for every `Image`-backed menu item
(`to_nsimage(Some(18.))` in muda's macOS backend), while `NativeIcon` items are
used at their natural ~14–16pt size. As a result `Settings...`, `Undo` and
`Open Log Folder` (all PNG-backed) sit visibly larger than `Open in DxO
PhotoLab` and `Check for Updates…` (both `NativeIcon`-backed) in the macOS
menu. Confirmed by manual inspection during the dependency-refresh sanity
check (2026-09-20). Fixable without an upstream change: shrink the drawn
glyph inside the canvas in `tools/macos/export-menu-icons.swift` so the
transparent padding absorbs muda's stretch to 18pt. Related:
`tools/macos/export-menu-icons.swift`, `crates/app/icons/menu/*.png`,
`crates/app/src/main.rs`.

#### TODO

- [ ] Shrink the glyph drawn by `tools/macos/export-menu-icons.swift` so the
      exported PNGs read at ~14pt once muda stretches them to 18pt, and
      confirm all five menu icons look the same size.

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

### App: `Open Folder…` has no macOS menu icon

From `menu-accelerators`'s trade-offs: `NativeIcon::Folder` exists, but the
macos-menu-icons work found several `NativeIcon`s to be legacy colour
bitmaps rather than template images, and only verified ones were used
elsewhere, so `File > Open Folder…` ships as a plain `MenuItem` with no
icon. Confirmed directly (dependency-refresh, 2026-09-20, AppKit script on
macOS 26.6): `NSImage(named: "NSFolder")` (what `NativeIcon::Folder`
resolves to) has `isTemplate == false` — a colour Finder folder that would
clash with the template icons around it. File: `crates/app/src/main.rs`
(`app_menu`).

#### TODO

- [ ] Export an SF Symbol (e.g. `folder`) as a PNG via
      `tools/macos/export-menu-icons.swift`, the same way `Open Log Folder`
      is done, and assign it to `Open Folder…` instead of
      `NativeIcon::Folder`.

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

### App: a pick kept across a sidecar format switch still shows its flag dot in the wrong format

A pick kept by `reset_sidecars` across a Dop -> Xmp switch is rendered in XMP
mode even though the underlying format no longer matches. The strip draws the
pick dot regardless of the current sidecar format.

#### TODO

- [ ] Gate the flag-dot render on `sidecarFormat` in
      `crates/app/ui/src/strip.ts` (the flag dot, `setRating`) and
      `crates/app/ui/src/main.ts` (`applyRating`, `case "pick"`), consistent
      with `crates/app/src/index.rs`'s `reset_sidecars` behavior.
