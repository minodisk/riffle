# todo

## Cross-cutting / other

### App: real-device check of the AF eye Good / Bad / Unknown icons and mark colors

The `af-eye-good-ok-bad` feature renamed the filter menu's `AF eye` items to `Good` / `OK` / `Bad` / `Unknown` and gave each judged state its own icon and color on the strip, the filter menu and the `f` focus mark; the `af-eye-good-bad-cascade` plan (Step 1, `docs/plans/20261010-af-eye-good-bad-cascade/plan.md`) then merged `OK` into `Bad`, so the states are `Good` / `Bad` / `Unknown`. CI and the unit tests cover the state -> icon / color mapping (`stripState`, `FOCUS_MARK_ICONS`, `FOCUS_MARK_COLORS` in `focus.test.ts`) and the filter logic. Nobody has looked at the rendered result in the running app, including whether the gray `#999` reads on photos. Plan: `docs/plans/_archived/20261010-af-eye-good-ok-bad/plan.md`.

Files: `crates/app/ui/src/focus.ts`, `crates/app/ui/src/strip.ts`, `crates/app/ui/src/icons.ts`, `crates/app/ui/index.html`, `CLAUDE.md`.

#### TODO

- [ ] On any desktop platform, open a folder with faced AF frames and wait for `analyzing N / M` to finish. Check that the strip shows a bright green `scan-face` on good frames, a gray `scan` on bad frames (a face near the AF point, the frame not good, its AF eyes in focus or not) and nothing on unknown ones. Check that the filter menu's `AF eye` items (`Good` / `Bad` / `Unknown`) show the same icons (`Unknown` an empty aligned slot). Check that the `f` mark is bright green for good, gray for bad and white for unknown, with the gray readable on photos.
- [ ] Reword the `src/focus.ts` sentence in `CLAUDE.md`, which still says `photoTier` "puts the strip's face icon on a good frame", to describe the per-state icon (`stripState` / `FOCUS_MARK_ICONS`).

### Core: the `.dop` reader ignores a picked PhotoLab virtual copy

Found in `burst-keep-score` Step 1 (the inventory, `docs/plans/_archived/20261008-burst-keep-score/data.md`). `dop::read_flag` (`crates/core/src/dop.rs`, `locate`) reads only the first `Items` entry, so it returns `None` for 199 frames of `2026-06-05` and 228 of `2026-09-13-b` whose virtual copy (a later item) is picked in PhotoLab. Riffle shows those frames unflagged, and `trash.rs` `collect_folder` would not treat them as picked. Whether Riffle should read any item's flag, and which item it writes, is a product decision.

Files: `crates/core/src/dop.rs`, `crates/app/src/trash.rs`.

#### TODO

- [ ] Decide which `Items` entry's flag Riffle reads and writes, then change `read_flag` and its tests to match.

### App: re-measure a Good / Bad for frames with no face

The `af-eye-good-bad-cascade` plan (Step 3, Decision C in `docs/plans/20261010-af-eye-good-bad-cascade/plan.md`, numbers in `sharpness-fallback.md`, script `facefree.py` in the same folder) measured whether the sharpness score separates picks on frames where no face is analyzed (an AF point with no face near it, or no AF point and no face on the whole preview). It did not meet the pre-registered criterion (held-out AUC >= 0.70 pick vs non-pick and a cut at 2x the base precision marking 20% or more): absolute AUC 0.579, best held-out lift 1.25, so those frames stay `Unknown` (Step 4). The pick vs reject leg could not be run: only 9 face-free rejects exist (7 in `2026-09-19`, 2 in `2026-09-13-b`), and no folder has the 30 the criterion needs.

Files: `crates/app/ui/src/focus.ts` (where a rule would go, next to the good cuts), `crates/app/src/index.rs` (an `analyzed` flag on `Focus` / `FaceReady` so "analyzed, no face" differs from "not yet").

#### TODO

- [ ] Once a folder holds 30 or more face-free rejects (`.dop` `ShouldProcess` or XMP `xmpDM:good="False"`), or a cue other than the sharpness score exists for frames with no face, re-dump with `riffle-cli features`, re-run `facefree.py` against the same pre-registered criterion (pick vs reject included), and ship a face-free Good / Bad only if it passes.

### App: the MCP `get_view` eyes summary carries no openness

`burst-keep-score` Step 4b made the meta pane's `Eyes` row show the openness (0-100 from the EAR) but kept `get_view`'s `state` / `probability` / `pose` fields as planned. Whether `EyesSummary` should carry the EAR or the openness is open.

Files: `crates/app/ui/src/companion.ts` (`EyesSummary`), `crates/app/src/mcp.rs`.

#### TODO

- [ ] Decide whether `get_view`'s eyes summary carries the EAR or the openness, and add it with its test.

### App: real-device check of the 1:1 view on files with a malformed full-size JPEG

The `partial-decode-error-exit` work (`docs/plans/_archived/20261002-partial-decode-error-exit/plan.md`) made `decode_region` in `crates/core/src/partial.rs` return `Err` on a fatal libjpeg error instead of calling `exit(1)`. Unit tests (empty input, non-JPEG input, a JPEG truncated inside the header) and `riffle-cli crop` / `check` on the real files cover the core path. `crop` returned `libjpeg fatal error: Not a JPEG file: starts with 0x3c 0x44` for `NIKON_D70_Nikon.nef` and `... 0x80 0x03` for `CGO3P_YUN00007.dng`. `check` over the samples ran to its summary in 23.0 s. The desktop app's 1:1 view (the `focus_crop` command, which calls the same `decode_focus_crop`) was never exercised, and neither was what the UI shows for the error. Files: `crates/app/src/commands.rs` (`focus_crop`), `crates/core/src/partial.rs`.

#### TODO

- [ ] On any platform, open `D:\photos\samples\NEF\NIKON_D70_Nikon.nef` and `D:\photos\samples\DNG\CGO3P_YUN00007.dng` in the app and switch to the 1:1 view. Expect the app to stay up and the `focus_crop` error (`libjpeg fatal error: Not a JPEG file ...`) to surface however the 1:1 view shows a failed crop. Note what the UI shows; if it shows nothing useful, file a follow-up.

### App: real-device check of the preview-failed state when the selected file's preview fails

#### Background

The preview-failure-clears-stale-image feature clears the main preview and shows a "preview-failed" overlay (`The preview of this file could not be shown.`) when the current file's preview fails. Before this, the previous file's image stayed on screen. The automated criteria are covered by `empty.test.ts` and `mise run ci`. The real-app behavior (a real undecodable NEF, fast paging across it, `z` and Compare on it) was never exercised. Plan: `docs/plans/_archived/20261002-preview-failure-clears-stale-image/plan.md`.

Files: `crates/app/ui/src/main.ts`, `crates/app/ui/src/empty.ts`

#### TODO

- [ ] On Windows, open `D:\Photos\samples\NEF\`, view a normal NEF, then `NIKON_D70_Nikon.nef`. Expect the main preview no longer shows the previous NEF and reads "The preview of this file could not be shown.", with the error in the meta pane's status block.
- [ ] On Windows, page on from the bad file to a normal file. Expect it to show normally and the overlay to disappear.
- [ ] On Windows, page back and forth quickly across the bad file. Expect an older file's image never stays on screen.
- [ ] On Windows, press `z` (1:1) and open Compare on the bad file. Expect neither to throw.

### Core: validate the combined AF-eye score on the two reserved labeled folders

`docs/plans/_archived/20260926-af-eye-in-focus-probability/plan.md` reserved
`D:\Photos\tests\2026-08-29-focus-sample` and
`D:\Photos\tests\2026-09-13-b-focus-sample` (100 ARW each) as a
post-implementation check, but as of Step 1 neither carried XMP pick/reject
labels yet, so they were not run.

#### TODO

- [ ] Once `2026-08-29-focus-sample` and `2026-09-13-b-focus-sample` carry XMP
      pick/reject labels, run
      `riffle-cli candidates D:\Photos\tests\2026-08-29-focus-sample D:\Photos\tests\2026-09-13-b-focus-sample`
      and record AUC / precision / coverage. The cue now scores the mesh
      eye regions (`docs/plans/_archived/20261008-mesh-eye-focus/`), whose
      gain over the window rests on few off frames (9 held-out off frames
      among the meshed ones), so also run that plan's `fit.py` window /
      region comparison on them, and the eyelid contour mask comparison of
      `docs/plans/_archived/20261008-mesh-eye-mask/` (not adopted on the
      held-out set, but it ranked the training set's meshed frames better:
      meshed AUC 0.974 against the window's 0.951 on the same frames, where
      the rectangle reached 0.948 against the window's 0.946). Files: `crates/cli/src/main.rs`,
      `crates/core/src/candidate.rs`,
      `docs/plans/_archived/20261008-mesh-eye-focus/fit.py`,
      `docs/plans/_archived/20261008-mesh-eye-mask/fit.py`.

### Docs: the "Focus candidate pass" numbers in docs/humans/performance.md are missing the app's own scan/faces log lines

Step 3 of `docs/plans/_archived/20260925-focus-candidate/plan.md` asked for the
`scan extract` / `scan faces` log lines of one app open of the 2134-file
Sony folder to be recorded in `docs/humans/performance.md` "Focus candidate pass",
alongside the `riffle-cli scan` / `riffle-cli candidates` numbers already
there. The implementing agent could not drive the GUI, so only the CLI
numbers are recorded; `docs/humans/performance.md` says so in a note under the
section.

#### TODO

- [ ] Open the 2134-file Sony folder once in the app and record its
      `scan extract` / `scan faces` log lines in `docs/humans/performance.md`
      "Focus candidate pass", next to the existing CLI numbers.

### App: unmeasured end-to-end per-page latency

End-to-end per-page latency (IPC + `createImageBitmap`) is unmeasured, since the GUI could not be driven from this development machine. Only the Rust-side file-read cost was measured; see "Per-page preview read" in docs/humans/performance.md.

#### TODO

- [ ] Measure keypress-to-pixels per page turn on real hardware and add the numbers to "Per-page preview read" in docs/humans/performance.md. The instrumentation now exists: with `Timing logs` on, the app logs a `page invoke=… decode=… total=… keypressToPixels=…` line per page turn to `Riffle.log`, and "Measuring on your own folder" in docs/humans/performance.md spells out the procedure. Only running the measurement and filling in the numbers is left.

### App: cold first scan on an internal SSD is far slower than the extrapolation

A real cold first scan on Windows 11 (internal SSD, 22 threads, Sony ARW) costs ~16-19ms per file, ~82-97s extrapolated to 5000 files against the 30s target; see "Real folders on Windows" in docs/humans/performance.md. Excluding the folder from Defender did not help, and a warm-cache scan runs at ~1ms per file, so neither Defender nor CPU is the cause. The cause is unknown. A later data point (Windows 11, v0.2.0, 2026-09-22): a cold first scan after an index schema change took 25.5s on 3045 Sony ARW (~8.4ms/file, ~42s extrapolated to 5000), against the earlier 16-19ms/file; it is unknown whether the OS cache was cold for that run.

#### TODO

- [ ] Run `riffle-cli scan` on a cold real folder on Windows at thread counts 1 / 4 / 8 / 22 (cold each run) to separate IO concurrency from per-file cost, and compare the bounded 1MiB read against reading the whole file.

### App: a deep-row focus point still exceeds the 50ms budget

A focus point in a deep row of the unrotated JPEG measured 58-65ms keypress to pixels (n=2, before the #56 and #60 fixes); see "The 1:1 focus check path" in docs/humans/performance.md. Options are prefetching the neighboring files' crops (Phase 4's ring buffer) or a DCT-scaled placeholder; nothing is chosen.

#### TODO

- [ ] Retake the deep-row measurement post-fix, then decide between crop prefetch and a DCT-scaled placeholder.

### App: the silent update path is unverified end-to-end

The Updating paragraph in `docs/humans/usage.md` describes a background download and install on launch and the "Check for Updates…" menu item, but nothing has confirmed on a real build that an installed copy detects a newer release, installs it silently, and launches as the new version next time. The background flow is verified on Windows 11 (2026-09-22): 0.1.10 downloaded 0.2.0, installed it on quit, and launched as 0.2.0.

#### TODO

- [x] Background update flow on Windows (Windows 11, 0.1.10 -> 0.2.0, 2026-09-22): downloaded, installed on quit, launched as 0.2.0.
- [ ] Verify the **Check for Updates…** menu item reports the up-to-date / installed / already-installed outcomes correctly on macOS, Windows, and Linux AppImage.
- [ ] Verify the background flow on macOS and Linux AppImage: install the current release, publish the next one, then launch the installed build. Done when the newer release is installed after the signature check and used on the next launch.

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

From `trash-rejected`'s implementation and its follow-ups: GUI automation is
unavailable on this Mac, so nothing of the action's visible behavior has been
run by a human on macOS or Linux. The action is now only the folder tree's
right-click items (`Move Rejected to Trash…`,
`Move Rejected to Trash, Including Subfolders…`, and the
`Move Rejected in N Folders to Trash…` pair for a multi-selection); the File
menu item, and with it its template icon, was removed by
`file-menu-folder-items` (#537). The confirmation is the in-window
`#trash-dialog` (`crates/app/ui/index.html`, `crates/app/ui/src/trash.ts`),
driven by the `trash_rejected_preview` / `trash_rejected_run` commands
(`crates/app/src/commands.rs`). On macOS the mover is `trash::trash_file`
(`crates/app/src/trash.rs`, `undo-trash-rejected` #547), which calls
`NSFileManager.trashItemAtURL` directly rather than the `trash` crate's
`DeleteMethod::NsFileManager`; one point is specifically unverified: whether
the Trash's "Put Back" entry is actually created, since the `trash` crate
documents that on some macOS systems files moved through `NSFileManager` get
no "Put Back" entry (trash-rs#14); dragging them out of the Trash still
restores them. Files: `crates/app/src/commands.rs`, `crates/app/src/trash.rs`,
`crates/app/ui/src/trash.ts`, `crates/app/ui/src/main.ts`. A manual run on
Windows on 2026-09-28 (`mise run tauri:dev`) passed for the earlier flow with
the OS's own confirm dialog (it predates the in-window dialog of #529): the
confirmation showed the count and the button labels, Cancel changed nothing,
confirming moved the RAW, its `.xmp` and `.ARW.dop` to the Recycle Bin with
the right original location, the strip moved to the next file, restoring all
three showed the file still as a reject, and a folder with 0 rejects got a
message. The Windows checks of the tree items and the dialog are in
`### App: Windows real-device check of the merged folder-tree, scan-wait and strip-scroll work`.
Sources: `docs/plans/_archived/20260928-trash-rejected-from-tree/learnings.md`
(#518, #521, #526, #529) and
`docs/plans/_archived/20260928-undo-trash-rejected/learnings.md` Step 2
(#547).

#### TODO

- [ ] On macOS, right-click a folder in the tree and pick
      `Move Rejected to Trash…`: `#trash-dialog` names the right count (and
      the singular for one file) with `Move to Trash` / `Cancel`; Cancel
      leaves the folder untouched; confirming moves the RAW plus its `.xmp`
      and `.ARW.dop` to the Trash and the strip shows the next passing file;
      a folder with zero rejects writes its message to `#status` with no
      dialog; a press during a scan is held by `whenIdle` until the scan
      ends (`crates/app/ui/src/idle.ts`); a "Put Back" from the Trash
      restores the file with its judgment, and if no "Put Back" entry
      exists, dragging it out does. (Source:
      `docs/plans/_archived/20260928-trash-rejected-from-tree/learnings.md`,
      #521, #529.)
- [ ] Verify the same flow on Linux (the `trash` crate's freedesktop backend
      has never been run here).
- [ ] On macOS, trash the rejects of a folder, then `Edit > Undo`: the files
      come back with the reject flag, and a file with the same name placed
      at the original location beforehand is reported
      (`already exists at the original location`) and not overwritten.
      (Source: `docs/plans/_archived/20260928-undo-trash-rejected/learnings.md`
      Step 2, #547.)

### App: the muda template-icon fork is a temporary bridge

Custom menu-item icons now tint with the menu appearance (white in dark mode,
black in light mode) via a `[patch.crates-io]` fork of muda 0.19.3
(`minodisk/muda`, `rev = "ef4fbfda53416382a2eeca7da683a64a8fe0e8ed"`) that
calls `nsimage.setTemplate(true)` unconditionally in `menuitem_set_icon`. This
is a temporary bridge, not the permanent fix: upstream PR
https://github.com/tauri-apps/muda/pull/413 carries the same change behind an
opt-in `set_icon_as_template` API; it merged on 2026-09-29 and shipped in
muda v0.21.0 (2026-09-30).

As of 2026-10-05: Tauri v2.12.1, the latest v2 release, depends on
`muda = "0.20"`, so it does not resolve muda 0.21; Tauri's menu-item API still
has no template flag (only tray icons have one); the workspace is still on
tauri 2.11.6 with the fork's muda 0.19.3.

#### TODO

- [ ] Once a Tauri v2 release depends on muda >= 0.21 (where muda#413's
      `set_icon_as_template` lives) **and** Tauri exposes the template flag
      for menu items, drop the `[patch.crates-io]` entry from the workspace
      `Cargo.toml` and switch to the upstream opt-in API.
- [ ] Re-verify in the running app that all bundled menu PNGs still tint
      correctly with the menu appearance after the switch.

Related: `Cargo.toml`, `docs/agents/tauri-app.md`,
`tools/macos/export-menu-icons.swift`.

### App: real-device checks for the View menu

`view-menu-panes` (docs/plans/_archived/20260930-view-menu-panes/plan.md) cut the native
`View` menu down to three `CheckMenuItem`s (`Folders`, `Metadata`,
`Filmstrip`) whose checks follow the shown panes, with the pane toggles moved
to modifier defaults (`Ctrl+Alt+Arrow` on Windows / Linux, `Alt+Cmd+Arrow` on
macOS) so they show an accelerator. CI covers the build and the tests. The
macOS `cfg` branch of `app_menu::build` / `refresh` and the revert of muda's
native toggle on click (muda toggles the check before it sends the event) were
confirmed on a real Mac on 2026-10-05 (`mise run tauri:dev`): the items, their
accelerators, the checks, the single fire per key and the rebuild on a rebind
all behaved. The Windows GUI checks (the menu, the checks, the accelerators)
were never run: the step was ticked on the automated criteria only.
`view-pane-names` (docs/plans/_archived/20260930-view-pane-names/plan.md) then
renamed the first two items from `Left Pane` / `Right Pane` to `Folders` /
`Metadata` (ids, actions and stored keys unchanged); its Windows label check
was likewise never run, and the first Windows TODO below covers it.

Files: `crates/app/src/main.rs` (`app_menu`, `apply_panels`),
`crates/app/src/commands.rs` (`set_panels`, `AppPanels`),
`crates/app/src/shortcuts.rs` (defaults).

#### TODO

- [ ] On Windows, run `mise run tauri:dev` and confirm at launch that `View`
      shows `Folders`, `Metadata`, `Filmstrip` with `Ctrl+Alt+ArrowLeft` /
      `ArrowRight` / `ArrowDown` and checks matching the panes the last
      session left.
- [ ] On Windows, press each of the three keys once: the pane toggles exactly
      once (no double fire) and the check follows. `Tab`, `f`, `z`, `v` still
      work and are not in `View`. `F6` / `F7` / `F8` now do nothing.
- [ ] On Windows, click `View > Filmstrip` twice: hides, shows, check right
      each time. Open Settings, click `View > Folders`: nothing changes and
      the check stays.
- [ ] On Windows, rebind `toggleStrip` to `ctrl+alt+s`: the item shows it,
      `Ctrl+Alt+S` toggles once, and `Reset` restores `Ctrl+Alt+ArrowDown`.
      With the folder tree focused, `Ctrl+Alt+ArrowLeft` still hides the tree
      (tree passthrough).
- [ ] On Windows, watch for an Intel graphics hotkey taking `Ctrl+Alt+Arrow`
      (screen rotation). If it does, note it in `docs/humans/usage.md` as a driver
      setting to turn off (not a blocker; do not change the default).

### App: the real-device checks for the File menu accelerators are still open

From `menu-accelerators`'s implementation: the GUI could not be driven from
the agent session, so the double-fire, stale-key-equivalent,
settings-capture and menu-set-from-`setup` checks are unconfirmed on macOS,
and Windows / Linux are unconfirmed entirely. A later manual run on Windows
(`mise run tauri:dev`, `Ctrl` in place of `Cmd`) passed all six checks:
`Ctrl+O` opens the picker exactly once; `Shift+Ctrl+O` fires once (judged
only from a single status line, which a second identical message would
overwrite); rebinding `open` updates the menu accelerator and kills
`Ctrl+O`; the menu shows correctly with the settings window open and steals
no focus; a capturing shortcuts row swallows `Ctrl+O` / `Shift+Ctrl+O`;
both File items work by mouse. Files: `crates/app/src/main.rs`
(`app_menu`), `crates/app/src/shortcuts.rs`, `crates/app/src/commands.rs`
(`update_keymap`), `crates/app/ui/src/main.ts`.

Since `undo-redo-keymap`, `Edit > Undo` / `Edit > Redo` are keymap actions
too (`Cmd+Z` / `Shift+Cmd+Z` on macOS, `Ctrl+Z` / `Ctrl+Shift+Z`
elsewhere), run by the frontend keydown with keymap-derived menu
accelerators. Before that change, `Ctrl+Z` did nothing on Windows 11
(v0.2.0) while clicking the Edit items worked. A manual run on Windows on
2026-09-28 (`mise run tauri:dev`) passed: `Ctrl+Z` / `Ctrl+Shift+Z` each
fire exactly once, the Edit menu shows the accelerators, rebinding `undo`
updates the menu in place and kills the old key with no `menu item undo not
found` in the log, and both Edit items work by mouse. Select All passed the
same run: `Ctrl+A` with the strip focused selects every file and keeps the
current one, it does nothing to the strip with the folder tree focused,
inside a settings text input it selects the input's text, and `Edit > Select
All` shows the accelerator and updates in place when `selectAll` is
rebound. (`Ctrl+A` first did nothing because of a global PowerToys Keyboard
Manager remap of left `Ctrl+A` to `Home`, not the app; the tell-tale is a
keydown sequence of `Unidentified` then `Home`.) macOS is still unrun.

#### TODO

- [ ] On macOS, verify: one `Cmd+O` press opens the folder picker exactly
      once (no double fire from keydown + menu accelerator); rebinding
      `open` updates the menu accelerator and kills the old key,
      with the macOS key equivalent not staying stale; a menu set from
      `setup` shows correctly and doesn't steal focus from the settings
      window; pressing `Cmd+O` while a shortcuts row is
      capturing does not trigger the menu action; `Open Folder…` works via
      mouse click.
- [ ] On macOS, verify: one `Cmd+Z` press undoes exactly once and one
      `Shift+Cmd+Z` redoes exactly once (no double fire from keydown + the
      Edit menu key equivalent); rebinding `undo`/`redo` updates the Edit
      menu accelerator and kills the old key; both Edit items work via mouse
      click. Files: `crates/app/src/main.rs` (`app_menu`),
      `crates/app/src/shortcuts.rs`, `crates/app/ui/src/main.ts`.
- [ ] On macOS, verify Select All: pressing `Cmd+A` once with the strip
      focused selects every file (the current file stays current); pressing
      it with the folder tree focused does nothing to the strip; pressing it
      inside a settings text input selects that input's text. Check
      `Edit > Select All` shows the accelerator and that rebinding
      `selectAll` updates it. Files: same as above, plus
      `crates/app/src/commands.rs`, `crates/app/ui/src/settings.ts`.
- [ ] Verify the `Some`/`None` accelerator behavior on Linux (only
      reasoned from muda 0.19.3's sources so far, never run; Windows passed).

### App: the Clear Cache button's manual GUI verification is still open

From `clear-cache-stuck-guard`'s implementation: the button's guard used to
get stuck (never re-enabling after the first scan), so none of its GUI
behavior has been run by a human. GUI automation is unavailable on this
Mac and a native confirm dialog cannot be driven by an agent. A later manual
run on Windows passed (1) and showed the dialog in (2) immediately; the
Cancel half of (2) and checks (3)-(7) were blocked by a Clear Cache refusal
bug, since fixed (`focus-rescan-main-window`), and are still to be run.
`settings-modal` then moved the settings into a modal in the main window,
parented the confirmation on `main` and dropped the `index-clearing`
status text; none of that has been run by hand either. Files:
`crates/app/src/commands.rs`, `crates/app/ui/index.html`,
`crates/app/ui/src/settings.ts`, `crates/app/ui/src/main.ts` (the
`tauri://focus` listener). `wait-for-scan` then replaced the disabled
button during a scan with holding the clear until the scan ends (checks
(4) and (5) below were rewritten for it). A run on Windows was attempted
on 2026-09-28 with `mise run tauri:release:devtools` but not carried out:
the process was killed for low memory on the PC before the checks ran.

#### TODO

- [ ] Run `mise run tauri:release:devtools` and check, in order: (1) after
      a folder's scan finishes, Settings > Cache shows the button enabled
      and the note hidden; (2) pressing Clear Cache shows the confirmation
      dialog immediately, and Cancel leaves the size and `#settings-status`
      unchanged with the button re-enabled; (3) Clear Cache then Clear
      keeps the button disabled until the size drops and the main window
      rescans, and never refuses with `a scan is running` (closing the
      confirmation refocuses `main`, whose focus listener skips `resync()`
      while the modal is open); (4) pressing Clear Cache while a scan is
      running (or with a large folder opening under the settings modal)
      holds it: the button stays enabled, `#clear-index-note` shows the
      waiting text, and the clear runs by itself when the scan ends, after
      which the size figure updates; closing the modal while it is held
      drops it; (5) pressing Clear Cache during a large folder's prepare
      phase (before the first `scanning N / M` line) holds it the same
      way; (6) an error path, if reachable, writes the refusal to
      `#settings-status` and it stays until the next press; (7) note
      whether the very first press after opening the settings modal ever
      does nothing, and if so record the window focus state at that
      moment; (8) the native confirmation looks right over the settings
      modal on macOS and Windows.

### App: burst grouping's manual checks are still open

From `burst-grouping`'s implementation: GUI automation is unavailable on this
development machine, so several behaviors were never exercised by a human.
The Sony half of the band and badge check passed on Windows on 2026-09-28
with an ILCE-7M5 folder: band visible, the `position/count` badge on every
member and tracking the position, the gap filled, and the band
distinguishable from `.cell.current` and `.cell.failed` (`.failed` was
produced with an ARW truncated to its first 2 KB).

#### TODO

- [ ] Verify the burst band and count badge by hand on a real Leica burst
      folder and note the group sizes: band visible, every member cell
      shows its `position/size` (a two-digit one like `12/15` clear of the
      file name), unchanged by the selection, the gap above a
      non-first member is filled, and the band is distinguishable from
      `.cell.current` and `.cell.failed`. Files: `crates/app/ui/src/burst.ts`,
      `crates/app/ui/src/strip.ts`, `crates/app/ui/style.css`.
- [x] `Alt+ArrowUp` / `Alt+ArrowDown` (`burstFramePrevious` /
      `burstFrameNext`) reach the app on Windows and step through bursts
      (Windows 11, v0.2.0, 2026-09-22).
- [ ] In `mise run tauri:dev`, confirm `Alt+ArrowUp` / `Alt+ArrowDown`
      reach the app on macOS, i.e. the webview does not swallow them, and
      step through a burst stopping at its ends. Files:
      `crates/app/src/shortcuts.rs`, `crates/app/ui/src/main.ts`.
- [x] `Shift+x` reject-rest and its one-step undo exercised by hand on a
      real burst folder (Windows 11, v0.2.0, 2026-09-22).
- [x] On Windows, the strip scrollbar is thin and dark and the thumbnail
      right edge and the burst bracket are not clipped (Windows 11, v0.2.0,
      2026-09-22).
- [ ] On macOS, confirm the strip looks unchanged (160px wide). Files:
      `crates/app/ui/style.css`.

### App: Windows real-device check of the merged folder-tree, scan-wait and strip-scroll work

Merged since the base of the 2026-09-28 Windows GUI check run and not run by
hand on any platform: `tree-live-watch` (#517, #520, #523),
`trash-rejected-from-tree` (#518, #521, #526, #529, #530), `wait-for-scan`
(#519, #522, #525) and `strip-keep-scroll-on-rescan` (#528). The last one
fixes the scroll jump-back and flicker seen on Windows on 2026-09-28 while
copying 100 ARWs into an open folder. Files: `crates/app/src/treewatch.rs`,
`crates/app/src/trash.rs`, `crates/app/ui/src/folders.ts`,
`crates/app/ui/src/idle.ts`, `crates/app/ui/src/strip.ts`,
`crates/app/ui/src/main.ts`. #518's check of the relabeled File menu item is
not listed: that item was removed by `file-menu-folder-items` (#537). The
tree trash checks come from
`docs/plans/_archived/20260928-trash-rejected-from-tree/learnings.md`
Steps 2–4 and the bodies of #521, #526 and #529; the undo / redo checks are in
`### App: the manual GUI checks for Undo and Redo of Move Rejected to Trash are still open`.

#### TODO

- [ ] Tree watch on Windows: with a folder expanded, create, delete and
      rename a subfolder in Explorer and confirm the tree follows; confirm
      renaming a folder from the app still works with the watch on.
- [ ] Tree trash items on Windows (#521): a non-root folder offers
      `Move Rejected to Trash…` and
      `Move Rejected to Trash, Including Subfolders…`; the home root and a
      volume root offer only the first; the recursive item trashes the
      rejects of the folder and every visible subfolder (dot and hidden
      folders skipped); when the open folder is a target, the strip
      refreshes; when it is not, only the status line and the error list
      change.
- [ ] Tree multi-selection on Windows (#526): `Ctrl+click` toggles a folder
      in and out of the selection (the open folder can be toggled off, and
      the selection can go empty); `Shift+click` selects the range from the
      anchor without selecting the rows' text; a plain click selects only
      the clicked folder; opening a folder any other way (Enter, drop,
      `File > Open Folder…`, the reopen at launch) resets the selection to
      it; selected rows draw `.selected` distinct from `.current` /
      `.cursor`; the container has `aria-multiselectable` and each row
      `aria-selected` following the selection; collapsing a parent or a
      `tree-changed` re-list drops folders no longer drawn; right-clicking a
      selected folder acts on the whole selection, and the menu offers only
      `Move Rejected in N Folders to Trash…` and its `Including Subfolders`
      twin (the latter hidden when any selected folder is a root);
      right-clicking an unselected folder selects it alone first.
- [ ] Trash confirmation dialog on Windows (#529): `#trash-dialog` lists one
      row per folder with its reject count (the verbatim `\\?\` prefix
      stripped), hides zero-reject folders behind a summary line, shows the
      total count and the space freed, and opens with `Move to Trash`
      focused so Enter runs it; Escape and `Cancel` close it with nothing
      moved; a folder that cannot be read (for example one removed after the
      right-click, or a sidecar that does not parse) appears in the failure
      list with its error and is excluded from the run, and when every
      folder has zero rejects but one failed, the dialog still opens with
      `Move to Trash` disabled; a folder with zero rejects and no failure
      writes `No rejected files in …` to the status line with no dialog; the
      dialog, the sequence dialog and the settings modal never open on top
      of each other (`Ctrl+,` and the tree's `Sequence JPEG Timestamps…` do
      nothing while it is up); confirming shows `Moved N files to the Trash`
      and the failures, if any, in the error list.
- [ ] Scan-wait on Windows: press Move Rejected to Trash and a rename while
      a large folder scans; confirm the status line says what is waiting and
      each runs when the scan ends. The Clear Cache hold is covered by
      checks (4) and (5) of "the Clear Cache button's manual GUI
      verification is still open".
- [ ] Strip scroll on Windows: repeat the 100-ARW copy into an open folder
      and confirm the strip's scroll position holds without flicker.

### App: the manual GUI checks for Undo and Redo of Move Rejected to Trash are still open

`undo-trash-rejected` merged as #540, #547, #549 and #551: a Move Rejected to
Trash run is one undo unit; `Edit > Undo` / `Ctrl+Z` restores it through
`trash_rejected_undo`, and `Edit > Redo` / `Ctrl+Shift+Z` moves the restored
files to the Trash again through `trash_rejected_redo`, without the dialog.
The backend side was verified against the real Recycle Bin with throwaway
tests; the GUI side has not been run at all, since an agent session cannot
drive the app. The resume-selection check that used to sit here is in
`### App: the manual GUI checks for the resume landing's selection are still open`
(#553); the macOS undo check is in
`### App: the manual GUI checks for Move Rejected to Trash are still open`.
Files: `crates/app/src/trash.rs`, `crates/app/src/commands.rs`,
`crates/app/ui/src/undo.ts`, `crates/app/ui/src/trash.ts`,
`crates/app/ui/src/main.ts`. Source:
`docs/plans/_archived/20260928-undo-trash-rejected/learnings.md` Steps 3
and 4 (#549, #551).

#### TODO

- [ ] On Windows, trash the rejects in the open folder, then `Ctrl+Z`: the
      files come back, the strip shows them again with the reject flag (read
      from the sidecars), the status line says
      `Restored N files from the Trash`, and the folder tree's count follows
      on its next re-list.
- [ ] The same from the tree for a folder that is not open: only the status
      line changes.
- [ ] A run over several selected folders is undone by one `Ctrl+Z`.
- [ ] A conflict: copy a trashed RAW back by hand before the undo; the
      status line says `…, N failed`, the error list shows
      `could not restore from the Trash: already exists at the original location`,
      and the RAW's sidecars stay in the Trash, reported as
      `left in the Trash: its RAW could not be restored`.
- [ ] An emptied Recycle Bin: every file is reported
      `not in the Trash (emptied or restored by hand)` and the entry leaves
      the undo history.
- [ ] After a restore, the index picks the reject judgments back up on the
      `resync` (a following `Ctrl+Z` undoes the judgment batch beneath, not
      the trash run again).
- [ ] `Ctrl+Z` (files back), `Ctrl+Shift+Z` (moved to the Trash again with
      no dialog, the strip refreshed, status `Moved N files to the Trash`),
      then `Ctrl+Z` again (back again).
- [ ] The same undo / redo / undo from the tree with no folder open.
- [ ] A judgment made after the undo clears the redo: `Ctrl+Shift+Z` does
      nothing.

### App: the scan loading feedback's manual check is still open

The first scan pass now shows a thin progress bar above the strip bar
(`#scan-progress`) and every strip cell without a thumbnail pulses
(`.cell img:not([src])`); a failed cell does not. Neither has been seen on
a real machine yet. Files: `crates/app/ui/src/main.ts`,
`crates/app/ui/style.css`, `crates/app/ui/index.html`.

#### TODO

- [ ] On Windows, open a folder of 500+ files whose index is cold (or after
      Clear Cache) and confirm the bar appears before the first thumbnail,
      fills, and disappears at the end of the first pass, and that empty
      cells pulse until their thumbnail lands while failed cells do not.

### App: face/eye-aware focus check for culling

Face/eye-aware detection and scoring (`crates/core/src/faces.rs`, `crates/core/src/sharpness.rs`) now runs at scan time: sharpness is scored on the Sony eye-AF frame, else on the AF point, else on the eyes of a detected face when there is no trusted AF point, else on the sharpest tile; an AF point off the face no longer scores a bystander's eyes (see `docs/plans/_archived/20260922-face-aware-sharpness/` and `docs/plans/20260924-face-catch-state/`). Without a trusted AF point the whole-preview search now runs at a 640x448 input on a DCT-scaled decode, which raised small-face recall at about 1.7x the earlier per-file cost; closed eyes were found to need a second model (see `docs/plans/_archived/20261005-face-detection-recall-cost/`), which now judges the shown file's eyes on demand, outside the scan, for the meta pane's `Eyes` row (see `docs/plans/_archived/20261007-closed-eyes-detection/`). What remains:

#### TODO

- [x] MakerNote check (exiftool 13.59, 2026-09-22): Sony α7 V ARW writes
      `AFTracking` + `FocusFrameSize` + `FocusLocation` but no per-face list;
      `AFAreaMode` is enciphered; Leica M11-P DNG writes only
      `FocusDistance`. The scan now uses the Sony AF frame and skips detection
      when face tracking was engaged
      (`docs/plans/20260922-sony-eye-af-window/`).
- [x] Measured on Linux WSL2 (not the Mac): detection latency on real
      α7 V ARW / M11-P DNG previews and `riffle-cli scan` before/after adding
      detection and the AF-frame skip; see `docs/humans/performance.md` "Face
      detection cost".
- [x] Optionally, detect closed eyes. YuNet's five landmarks carry no
      eyelid points, so it needed a second model. Four were surveyed on
      2026-10-07 against 504 hand-labeled faces (`model-survey.md`,
      `eyes-truth.md`); none fits the scan's second pass. Adopted: MediaPipe
      Face Landmarker v2 (Apache-2.0, 4.9 MB ONNX converted from Google's
      TFLite, `crates/core/src/eyes.rs`), judging closed eyes by the eye
      aspect ratio (closed iff at most 0.137) of one face, the one nearest a
      trusted AF point, else the largest face at or above 0.8, at or above a
      60 px face side, on demand for the shown file only (`eyes_of`), shown
      as the meta pane's `Eyes: Closed (NN%)` / `Open (NN%)` row. Cost:
      `riffle-app` 47.5 -> 52.5 MB, `riffle-cli` 24.6 -> 29.6 MB; ~35 ms
      per face (p95 41 ms), the whole call 80-92 ms on an ARW with an AF
      point and 140-156 ms on a DNG without one. Accuracy: face AUC 0.974,
      precision 0.94 / recall 0.86 at the threshold. See
      `docs/plans/_archived/20261007-closed-eyes-detection/` and
      `docs/humans/performance.md` "Closed-eyes judgment on demand
      (Windows 11)".
- [ ] Judge every face at or above the 60 px floor, not only the AF-nearest
      (or largest) one, so a bystander's blink shows too. Each face is one
      more model run (~35 ms; ~1.8 faces per file on the 2134-ARW folder,
      more in groups). Needs either a row per face or
      a worst-of summary (`Eyes: Closed (1 of 3)`), a pattern the meta pane
      does not have, and a check of the accuracy on non-subject faces, which
      `eyes-truth.md` did not label. Files: `crates/app/src/commands.rs`
      (`read_eyes`), `crates/core/src/eyes.rs` (`judged_face`),
      `crates/app/ui/src/meta.ts`.
- [ ] Use the model's face presence output (`Identity_1`, a logit the
      `eyes` module ignores) to drop false face detections before judging
      the eyes. Needs a threshold calibrated on labeled faces: the `x`
      labels of `docs/plans/_archived/20261007-closed-eyes-detection/eyes-truth.md`
      (no eye to judge, including false detections) can seed it. Files:
      `crates/core/src/eyes.rs`, `docs/agents/tract-onnx-inference.md`.
- [ ] Mark closed eyes in the strip. Pass 2 now stores the EAR for the face
      the cue meshes and the filter menu has an `Eyes` section (the
      `mesh-eyes-index` work), but no tile shows a closed-eyes mark yet; the
      stored `eyes` state (`open` / `closed` / `unknown`) is on `Focus`, so
      the remaining work is the mark itself. Files:
      `crates/app/ui/src/strip.ts`, `crates/app/ui/src/eyes.ts`.
- [x] Estimate the head pose of the judged face. MediaPipe's face geometry
      pipeline (perspective unprojection at its 63 deg camera, weighted
      Procrustes onto the canonical face) is ported to
      `crates/core/src/pose.rs` and runs on the points of the `eyes_of`
      model run, shown as the meta pane's `Head pose` row (yaw, pitch,
      roll; right, up and clockwise positive). On demand only, nothing in
      the index, so there is no strip filter for it. Cost: 8.2 µs per face,
      no change to the `eyes_of` total. Accuracy on 159 labeled faces: the
      sign right on 94% (yaw), 95% (pitch), 15 of 18 (roll); the yaw class
      on 68%, the gap at the frontal / oblique boundary. See
      `docs/plans/20261007-head-pose/` and `docs/humans/performance.md`
      "Closed-eyes judgment on demand (Windows 11)".
- [ ] Derive the good-photo rule in the backend, next to `candidate` in
      `crates/app/src/index.rs`, so the MCP `get_view` tool can carry it.
      The rule lives only in `crates/app/ui/src/focus.ts` (`photoTier`) and
      `get_view` says nothing about it. Done when the index derives the tier
      from the stored columns with the thresholds in one place (the
      frontend reads it instead of recomputing), `get_view` carries it for
      the current file, and the boundary tests of `focus.test.ts` move with
      the rule. Files: `crates/app/src/index.rs`, `crates/app/src/mcp.rs`,
      `crates/app/ui/src/focus.ts`, `crates/app/ui/src/companion.ts`.
- [ ] Measure the good-photo mark's precision on labels. Decision B rests
      on the user's stars and the mark rate; no labeled set was made. Done
      when 100-150 faced AF frames drawn stratified around the cuts and at
      random are labeled blind (`ok`, `miss` with `blur` / `focus` /
      `closed`, `turned`, `unsure`; the agent first, the user's review
      final) and the precision and coverage of the mark at its cuts are
      written with a Wilson interval, with the focus candidate as the
      baseline (the procedure is in `docs/plans/20261008-burst-keep-score/plan.md` Step 5). Files: the plan
      folder of that work, `crates/app/ui/src/focus.ts` if the cuts move. Include some rated frames with eye_focus between 0.772 and 0.99 (0.90
      cut) that clear the other cuts: GOOD_EYE_FOCUS 0.90 has no held-out
      check, because batch 1 has none in that range.
- [ ] Re-check the good-photo cuts after the closed-eyes and head-pose label
      reviews (the items in the sections below). If the user's review
      moves `EYES_CLOSED_EAR` in `crates/core/src/eyes.rs` or changes the
      pose labels' frontal / oblique boundary, re-read `GOOD_EYE_EAR` (and
      `EYES_CLOSED_EAR` / `EYES_WIDE_OPEN_EAR` in
      `crates/app/ui/src/focus.ts`, which mirror the core value and anchor
      the openness) and `GOOD_MAX_YAW` on the rated frames of
      `D:\photos\samples\ARW\good-mark-2026-10-09\`. Files:
      `crates/app/ui/src/focus.ts`, `crates/core/src/eyes.rs`.
- [x] Give the no-AF path a good-photo mark once it has a focus cue. Done
      in `docs/plans/20261010-af-eye-good-bad-cascade/` Step 2: pass 2 runs
      the cue, the EAR, the pose, the eye offset and the edge gap on the
      largest confident face of the whole preview when a frame has no
      trusted AF point (`FACES_VERSION` 9), and `Focus` carries the cue
      without a point, so the same cuts mark it. The check of those cuts on
      starred no-AF frames is the next item.
- [ ] Check the good-photo cuts on starred frames with no trusted AF point.
      The cuts in `crates/app/ui/src/focus.ts` (`GOOD_EYE_FOCUS`,
      `GOOD_EYE_EAR`, `GOOD_MAX_YAW`, `GOOD_MAX_PITCH`, `MAX_EYE_OFFSET`,
      `MIN_EDGE_GAP`) were fitted on AF faces only; since
      `docs/plans/20261010-af-eye-good-bad-cascade/` Step 2 they also judge
      the largest confident face of a frame with no AF point. Run
      `riffle-cli features` over the Leica folders with `.dop` picks
      (`D:\photos\2026\2026-09-05` and the other DNG folders of the
      `20261008-burst-keep-score` data set), apply the same cuts to the
      `noaf` rows in a script, and report the share of faced frames marked
      good, the share of `.dop` picks among them against the base rate, and
      10 marked and 10 unmarked picks for the user to look at. Done when the
      numbers are recorded in a plan's `learnings.md` and the cuts are either
      kept or a separate no-AF cut is proposed with them. Files:
      `crates/app/ui/src/focus.ts`.
- [x] Measure the AF eye in-focus probability over each eye's eyelid
      region from the face mesh instead of the window between the eyes.
      The scan's second pass runs MediaPipe Face Landmarker v2 on the face
      nearest the AF point, scores each eye's contour bounding box (margin
      0.5 of its longer side) with a refitted logistic and takes the sharper
      eye; when neither eye's region counts (under 24 px or without a clear edge) it scores the window as
      before (61% of the training frames, 60% of the 2134-ARW folder's
      faces). The head pose was measured as an eye rule and did not beat the
      sharper eye, so it is not used. Held-out AUC 0.754 -> 0.800, precision
      / coverage 89.1% / 95.3% -> 88.6% / 95.9%; pass 2 on the 2134-ARW
      folder 12.9 -> 21.5 s at 24 threads (+66%). `FACES_VERSION` 6. See
      `docs/plans/_archived/20261008-mesh-eye-focus/` and
      `docs/humans/performance.md` "Focus candidate pass".
- [x] Measure whether de-rotating the face crop by YuNet's eye line (the
      MediaPipe pipeline's roll step) fits the face mesh better. It does
      not, in any variant (always, dead bands of 10 / 20 deg, a YuNet
      eye-distance guard): always-rotate turns 7 off meshes on and 15 on
      meshes off on 131 readable labeled frames and drops the held-out AF
      eye AUC 0.800 -> 0.783; the share of AF frames with `eye_offset` over
      0.10 stays 28.7% on six folders. Where the rotation moves a fit,
      YuNet's two eye points are not on two eyes. The residual misfits are
      side faces (78% of the off meshes, weighted), then cut, rotated past
      90 deg, seen from above and not-a-face. Nothing in the scan or the
      index changed. See `docs/plans/20261010-mesh-roll/` (Decision,
      `compare.md`, `misfit-truth.md`).
- [ ] Make `eye_offset` reliable where YuNet's eye distance is under 0.20
      of the face box side, for `20261008-burst-keep-score`, which owns the
      `MAX_EYE_OFFSET` 0.10 cut. `docs/plans/20261010-mesh-roll/` found that,
      weighted to its six folders, about 27% of the frames over 0.10 have a
      mesh off the face and about 65% an on-face mesh compared against YuNet
      eye points that are off the eyes (profiles whose two points sit
      together on one eye or the nose); frames under 0.20 eye distance are
      33.5% of the frames but 66% of those over 0.10. Done when the cut is
      skipped below that eye distance or `eye_offset` is replaced by a check
      of the mesh against its own geometry, and the good-photo mark's share
      and the rated frames of `D:\photos\samples\ARW\good-mark-2026-10-09\`
      are re-read. Files: `crates/core/src/candidate.rs`
      (`mesh_eye_offset`), `crates/app/ui/src/focus.ts` (`MAX_EYE_OFFSET`).
- [ ] Distrust the mesh values on side faces, the dominant mesh misfit
      (78% of the off meshes, weighted, in
      `docs/plans/20261010-mesh-roll/misfit-truth.md`). Done when a rule
      (e.g. YuNet eye distance under 0.10 of the box side, or the mesh's
      |yaw| past a bound) leaves the EAR, the AF eye cue and the pose of
      such a face unknown instead of a plausible-looking wrong value, checked
      on the misfit truth set and on the AF-eye, closed-eyes and head-pose
      labeled sets. Files: `crates/core/src/candidate.rs`,
      `crates/core/src/eyes.rs`.
- [ ] Faces rolled past 90 deg (a baby lying head-down,
      `D:\photos\2026\2026-07-11\_DSC2638.ARW`) get a mesh fitted as if
      upright, and the eye line cannot de-rotate them: `faces.rs` orders
      YuNet's eye points by image x, so the eye-line roll folds into -90..+90
      deg. They need another roll source (a mesh pass on rotated crops,
      MediaPipe's tracking loop). Not planned; see
      `docs/plans/20261010-mesh-roll/` (Decision). Files:
      `crates/core/src/faces.rs`, `crates/core/src/eyes.rs`.
- [ ] Review the mesh misfit labels: open the sheets
      `D:\Photos\tests\2026-10-09-mesh-roll\sheets\s00.png` to `s15.png`
      and compare each tile to its row of
      `docs/plans/20261010-mesh-roll/misfit-truth.md` (`before` / `after`
      `on` or `off` with its kind, and `yunet ok` / `yunet off`). The
      Decision's shares rest on the agent's labels; if labels change,
      re-run `compare.py` in that plan folder and update `compare.md` and
      the Decision if a number moves.
- [ ] Repoint the MCP `get_view` tool's `eyes` answer to the stored values.
      Pass 2 now stores the EAR and the pose of the face it meshes (the
      `mesh-eyes-index` work: `eyes_ear`, `eyes`, `eyes_closed` and `pose` on
      `Focus` / `FaceReady`, `docs/plans/_archived/20261008-mesh-eyes-index/plan.md`),
      but `ViewApi.eyes` in `main.ts` still reads `eyesCache.get`, so
      `get_view` answers only for a file already shown (see
      `docs/plans/_archived/20261008-mcp-eyes-pose/plan.md`). Answer from the
      stored values when present, falling back to `eyesCache`, and keep the
      `eyes` key absent from `get_view` until a file is judged. Done when
      `get_view` answers from the stored values and its existing test (key
      absent before and after the JSON round trip) still passes. Files:
      `crates/app/ui/src/main.ts`, `crates/app/ui/src/companion.ts`.
- [ ] Skip the second model run in `eyes_of` for a file whose scan already
      stored the EAR and pose (the `mesh-eyes-index` work stores them, and
      the meta pane reads them). `eyes_of` still runs for every shown file
      because the mesh overlay (`drawFaceMesh`) needs the 478 points, and it
      is the fallback for files with no stored value (face under 60 px, or
      a file pass 2 has not reached). Reusing the scan's mesh needs the points stored or a path that
      runs the model only while `f` is on. Files: `crates/app/src/commands.rs`
      (`read_eyes`), `crates/app/src/index.rs`, `crates/core/src/eyes.rs`.
- [ ] Suggest the sharpest-eye frame within a burst group. Measured and not
      shipped (`docs/plans/20261008-burst-keep-score/`, Decision 1, `results.md` and `fit.md`): the user
      picks within a burst by timing, not by a technical feature; the
      burst's length, not sharpness or the eyes, predicts a kept scene
      (79% held out at 15 or more frames, which the burst band's count
      already shows, against about 56% for the best technical rule); and
      the non-picks are mostly technically fine. Revisit only with a new
      signal; the per-frame good-photo mark above covers "is this a miss".
- [ ] Spot-check whether the sharpness ranking within a burst changes now
      that Sony frames with face tracking are scored on the camera's AF frame
      instead of YuNet's eye midpoint. Needs a per-file score output
      (`riffle-cli scan` prints none). Files: `crates/core/src/sharpness.rs`,
      `crates/core/src/scan.rs`, `crates/cli/src/main.rs`.
- [ ] Crop detection can false-positive on printed faces or logos near the AF
      point (e.g. `_DSC3632`: a shirt logo scored 0.67), yielding a false
      candidate (or a false not-a-candidate) when the logo is the face nearest
      the AF point. Consider a size or aspect filter on crop boxes. Files:
      `crates/core/src/faces.rs` (`detect_around`),
      `crates/core/src/candidate.rs`.
- [ ] AF on a person in the background gives a sharp face and a false focus
      candidate: the cue says the face nearest the AF point is sharp, not that
      it is the subject. Files: `crates/core/src/candidate.rs`.
- [ ] Back-of-head and upturned faces find no face in the crop, so their
      focus candidate state is unknown (white), even when the camera tracked
      them. Files: `crates/core/src/candidate.rs`, `crates/core/src/faces.rs`.

Related: `crates/core/src/sharpness.rs`, `crates/core/src/faces.rs`, `crates/core/src/arw.rs`, `crates/app/src/index.rs`, `crates/app/ui/src/burst.ts`.

### App: real-device check of the face boxes on the whole-image detection path

The face-detection-recall-cost work moved the whole-image face search (files with no trusted AF point, such as every Leica M file) to a 640x448 YuNet input on a DCT-scaled decode, and bumped `FACES_VERSION` to 5. `riffle-cli detect` / `candidates`, the unit tests and the `riffle-cli faces` PNGs cover the detector and the coordinate mapping back to stored preview pixels. The app GUI (the `f` mark drawing `faces_of` boxes after the second pass re-runs) was never exercised. Plan: `docs/plans/_archived/20261005-face-detection-recall-cost/plan.md`.

Files: `crates/core/src/faces.rs` (`detect_whole_upright`, `scaled_to_stored`), `crates/app/src/commands.rs` (`faces_of`), `crates/app/src/index.rs` (`FACES_VERSION`).

#### TODO

- [ ] On Windows or macOS, open `D:\photos\2026\2026-02-01` in the app after updating and let the second pass re-run (`FACES_VERSION` 5). Press `f` on `L1005161.DNG`: seven face boxes should be drawn (all seven people, as `D:\Photos\tests\2026-10-06-face-recall\step3\png\L1005161.png` shows), on the stored preview's scale (boxes on the faces, not shrunk to the top-left 3/8). Press `f` on `L1005233.DNG` (portrait): no boxes.

### App: real-device check of the meta pane's `Eyes` row, and the review of the closed-eyes labels

`closed-eyes-detection` (`docs/plans/_archived/20261007-closed-eyes-detection/plan.md`) added `eyes_of`, which judges the shown file's eyes with MediaPipe Face Landmarker v2, and the meta pane's `Eyes` row (since `eyes-open-probability`, `docs/plans/_archived/20261007-eyes-open-probability/plan.md`, shown as `Eyes open: NN%`, the open probability `1 - p`, and since `burst-keep-score` Step 4b as `Open · NN` / `Closed · 0`, the openness 0-100 from the EAR). CI covers the unit tests (`eyes.test.ts`, `meta.test.ts`, the core `eyes` tests), and the `#[ignore]`d `times_eyes_of_on_real_files` timed the backend; the app itself was never run with it, so the preview-to-row time in `docs/humans/performance.md` "Closed-eyes judgment on demand (Windows 11)" is missing. Steps 1 and 3 were ticked on the automated criteria. The truth set the threshold (0.137) and the AUCs come from was labeled by the agent, and the user's review of its `closed` faces is still open.

Files: `crates/app/src/commands.rs` (`eyes_of`, `read_eyes`), `crates/app/ui/src/main.ts`, `crates/app/ui/src/eyes.ts`, `crates/app/ui/src/meta.ts`, `crates/core/src/eyes.rs`, `docs/plans/_archived/20261007-closed-eyes-detection/eyes-truth.md`.

#### TODO

- [ ] On Windows 11, in a `mise run dev` build with Settings > `Timing logs` on, open `D:\photos\2026\2026-09-19` and wait for the scan to finish. Select `_DSC1889.ARW` or `_DSC1890.ARW` (labeled closed): the Analysis group shows `Eyes  Closed · 0` after `AF eye in focus`. Select `_DSC1894.ARW`: `Eyes  Open · NN`. Select `_DSC1897.ARW` (no face judged): no `Eyes` row.
- [ ] Open `D:\photos\2026\2026-02-01`, let the second pass re-run (`FACES_VERSION` 9) and select `L1005161.DNG` (closed) and `L1005155.DNG` (open): the `Eyes` row shows the value pass 2 stored for the largest confident face (L1005161 `Closed · 0`, L1005155 `Open · NN`), and the strip icon and the filter's `AF eye` state follow the same face.
- [ ] Hold the page key through 30 files of the ARW folder: no row of a previous file stays on a later one, and the preview keeps pace with no added stall against the previous build.
- [ ] With the folder idle, read the `eyes total=... read=... decode=... detect=... model=... ipc=...` lines in `Riffle.log` for a few ARWs with an AF point: the row should appear within ~150 ms of the preview. Add the numbers (and the preview-to-row time) to `docs/humans/performance.md` "Closed-eyes judgment on demand (Windows 11)" and `performance.ja.md`. Since `face-mesh-overlay` (`docs/plans/_archived/20261007-face-mesh-overlay/plan.md`) the response also carries 478 mesh points (~8-10 KB of JSON): confirm the stages still add up to about `total` and `ipc=` stays within a few ms of the figure before that change (not measured; `crates/app/src/commands.rs` `read_eyes`, `crates/app/ui/src/main.ts` `requestEyes`).
- [ ] Review the faces labeled closed: for each file in the "Faces with a closed eye" list of `docs/plans/_archived/20261007-closed-eyes-detection/eyes-truth.md`, open its tile `D:\Photos\tests\2026-10-07-closed-eyes\tiles\<stem>.png` (or the crops under `arw\` / `dng\`) and confirm that the eye marked `c` shows no iris; note any that are open. If labels change, re-run `riffle-cli eyes` on `labeled-paths.txt` and `scratch\auc.py` there, and re-fit the threshold and the slope in `crates/core/src/eyes.rs` if the best F1 moves.

### App: real-device check of the face mesh overlay on the judged face

`face-mesh-overlay` (`docs/plans/_archived/20261007-face-mesh-overlay/plan.md`) returns the 478 face-mesh points from `eyes_of` and drew MediaPipe's `FACEMESH_TESSELATION` (1322 edges) over the judged face while `f` is on; on real files the tessellation was seen to bury the face. `face-mesh-parts-overlay` (`docs/plans/20261007-face-mesh-parts-overlay/plan.md`) replaced it with the face parts outline (`FACEMESH_CONTOURS` + `FACEMESH_NOSE`, 149 edges, `FACE_OUTLINE_EDGES`) and, when the `Eyes` judgment is open, the two iris rings (`FACEMESH_IRISES`, 8 edges, `FACE_IRIS_EDGES`) with a dot at each center (`FACE_IRIS_CENTERS`, points 468 and 473). CI covers the tables' shape, `meshEdges`'s open / closed rule, the `meshPoints` scaling and centering, the core point mapping and the `Eyes` fixtures. The canvas drawing itself (`drawFaceMesh`) and its line widths (3 px outline at 0.8 black, then 1.5 px `FACE_MARK_COLOR`, chosen without a GUI again) and the iris dot size (`FACE_MARK_EYE_RADIUS`) were never seen on a real file. The step was ticked on the automated criteria.

Files: `crates/app/ui/src/main.ts` (`drawFaceMesh`, `FACE_MESH_OUTLINE_WIDTH`, `FACE_MESH_LINE_WIDTH`), `crates/app/ui/src/facemesh.ts` (`FACE_OUTLINE_EDGES`, `FACE_IRIS_EDGES`, `FACE_IRIS_CENTERS`, `meshEdges`).

#### TODO

- [ ] On Windows or macOS, with `f` on, show (1) an upright ARW with an open-eyed face of 60 px or more: the cyan outline of the face parts and both iris rings with their center dots sit on the judged face a moment after the `Eyes` row appears; (2) a portrait ARW (orientation 6 or 8) with such a face: the same, rotated with the image and scaling with the window; (3) a file labeled closed (`D:\photos\2026\2026-09-19\_DSC1889.ARW` / `_DSC1890.ARW`, per the closed-eyes todo above): the outline only, no iris ring and no iris dot; (4) a file whose only face is below 60 px on the preview: nothing drawn and no error in `Riffle.log`. Tune `FACE_MESH_OUTLINE_WIDTH` / `FACE_MESH_LINE_WIDTH` (or give the iris dots their own radius) if the outline competes with the face box or the rings collapse onto the dots.

### App: real-device check of the meta pane's `Head pose` row

`head-pose` (`docs/plans/_archived/20261007-head-pose/plan.md`) ports MediaPipe's face geometry to `crates/core/src/pose.rs` and returns the yaw, pitch and roll of the judged face from `eyes_of` (`EyesJudgment.pose`). The meta pane's Analysis group shows it as a `Head pose` row after `Eyes`. CI covers the unit tests (`pose.rs`, `meta.test.ts`, the `EyesJudgment` serialization), and the core solve was timed at about 8 µs per face. The app itself was never run with the row. The Step 3 checkbox was ticked on the automated criteria.

Files: `crates/app/src/commands.rs` (`read_eyes`, `EyesJudgment`), `crates/app/ui/src/eyes.ts`, `crates/app/ui/src/meta.ts` (`poseValue`), `crates/core/src/pose.rs`.

#### TODO

- [ ] On Windows, in a `mise run dev` build with Settings > `Timing logs` on, open `D:\photos\2026\2026-09-19`. Select a frontal ARW: the Analysis group shows `Head pose  yaw N°, pitch N°, roll N°` after `Eyes`, with |yaw| small. Select an ARW labeled oblique in `docs/plans/_archived/20261007-head-pose/pose-truth.md`: the yaw sign matches the labeled direction (right positive, up positive for pitch, clockwise positive for roll).
- [ ] Open `D:\photos\2026\2026-02-01`, let the second pass re-run (`FACES_VERSION` 9) and select a DNG with a judged face: the `Head pose` row shows the pose pass 2 stored for the largest confident face.
- [ ] Select a file with no judged face (for example `_DSC1897.ARW`): neither `Eyes` nor `Head pose` shows.
- [ ] Read the `eyes total=` lines in `Riffle.log` for a few ARWs: the total is not noticeably longer than before (the solve adds about 8 µs).

### App: the review of the head-pose labels is still open

`head-pose` (`docs/plans/_archived/20261007-head-pose/plan.md`) measured the angles against 190 labels (159 readable) that the agent made from the face crops: yaw class and direction, pitch, roll, or `x`. The user's review is pending, and the plan's Decision rests on these labels. Step 2 was ticked on the automated criteria (the labels exist, the measurement and Decision are written, `mise run ci` passes).

Files: `docs/plans/_archived/20261007-head-pose/pose-truth.md`, `docs/plans/_archived/20261007-head-pose/pose-results.md`.

#### TODO

- [ ] Also check hand check 16 of `docs/plans/_archived/20261008-burst-keep-score/results.md`
      (`D:\photos\2026\2026-07-05\_DSC2445.ARW`): a profile looking up behind a ball
      reads pitch -68 deg. Decide whether `crates/core/src/pose.rs` should call that
      unknown or whether the label stands.
- [ ] On Windows, open the sheets `D:\Photos\tests\2026-10-07-head-pose\sheets\s00.png` to `s16.png` (4 tiles across, row by row in the `#` order of the `pose-truth.md` table; crops in `crops\`) and compare each tile to its row. Note any label to change. If labels change, re-run `aggregate.py` (in `D:\Photos\tests\2026-10-07-head-pose\`) and update `pose-results.md` and the plan's Decision if a number moves.

### Core: the no-AF-point path decodes the preview up to three times

#### Background

On a file with no trusted AF point, pass 2 decodes the preview up to three times: the DCT-scaled RGB decode for the whole-image face search, a full-size RGB decode for the face mesh and the eye regions of the judged face (since `docs/plans/20261010-af-eye-good-bad-cascade/` Step 2, only when a confident face is found), then a full-size grayscale decode in `sharpness::score_preview`. The face-detection-recall-cost work cut the first decode to 3/8. Step 2 measured on one thread (`riffle-cli features`, `D:\photos\2026\2026-02-01` and `2026-09-05`, Leica DNG): a faced no-AF file went from about 94 ms to about 122 ms of analysis, a file with no face did not change; the numbers are in that plan's `learnings.md`. Scoring from the full-size RGB decode (its luma) would drop the grayscale decode on faced files, but it changes the score's pixels and its stored values, so it needs a `FACES_VERSION` bump. See also `docs/plans/_archived/20261005-face-detection-recall-cost/learnings.md` (Step 3 measurements).

Files: `crates/core/src/scan.rs` (`extract_analysis_unless`, `whole_image_cue`, `score`), `crates/core/src/sharpness.rs` (`score_preview`).

#### TODO

- [ ] Score a faced no-AF-point file from the full-size RGB decode the cue already made (its `candidate::luma`) instead of `score_preview`'s own grayscale decode. Measure the per-file saving with `riffle-cli features` on one thread over `D:\photos\2026\2026-02-01` and `2026-09-05` before and after, check that the score's ranking within bursts holds on those folders (Spearman per burst against the old score), and bump `FACES_VERSION`. Files: `crates/core/src/scan.rs`, `crates/core/src/sharpness.rs`, `crates/app/src/index.rs`.

### Agents: confirm `pr-runner` waits for a child's hand-back on a real `/pr` run

#### Background

The `long-wait-timeouts` fix is confirmed on 240 `pr-runner` and 225 `merger`
transcripts: every `wait-pr-actionable.sh` / `wait-post-merge-runs.sh` call
passed `timeout: 600000`, and the runs the harness still moved to the
background were `TaskStop`ped and re-run in the foreground. What remained was
`pr-runner` polling (`wait-pr-actionable.sh`, `sleep`, `until` / `while`,
`git ls-remote`, `gh pr view`) while a child it had dispatched was still
running. The `pr-runner-child-handback` plan added a rule to
`.claude/agents/pr-runner.md` that the runner ends its turn after an `Agent`
dispatch and waits for the child's hand-back. It is verified only statically.

#### TODO

- [ ] On the next real `/pr` run whose PR hits a conflict, a CI failure or a
      review round, read the `pr-runner` subagent transcript (the subagent
      `.jsonl` files under the Claude Code project directory for that
      session) and confirm that after each `Agent` dispatch the runner ended
      its turn, the next event is the "[Subagent hand-back]" message, and
      there is no `wait-pr-actionable.sh`, `sleep`, `until` / `while`,
      `git ls-remote` / `git fetch` or `gh pr view` call in between. If not
      clean, adjust `.claude/agents/pr-runner.md`.

### Agents: confirm `merger` runs its step 6 commands as separate, verbatim calls on a real `/pr` run

#### Background

The `merger-separate-sync-calls` plan added a rule to `.claude/agents/merger.md` step 6: `mise run git:main` and `./tools/git/delete_merged_branches.sh` run as two separate Bash calls, each exactly as written in its code block (no pipes, no `2>&1`, no `tail`, no `;` / `&&` chaining, no `cd` prefix). The rule exists because the chained or piped forms match neither allow rule in `.claude/settings.json`, so they fall to the auto mode classifier, which can deny the sandbox bypass as a Safety Bypass. The merger then stops with `post_merge_sync_failed` after a merge that already succeeded (PR #657). CI covers only the Markdown formatting. A prose rule cannot be unit-tested, so it has never been exercised by a real unattended merger run. Plan: `docs/plans/_archived/20261003-merger-separate-sync-calls/plan.md`

Files: `.claude/agents/merger.md`

#### TODO

- [ ] On the next real `/pr` run that reaches the merger, read the `merger` subagent transcript (the subagent `.jsonl` files under the Claude Code project directory for that session). Confirm that step 6 issued `mise run git:main` and `./tools/git/delete_merged_branches.sh` as two separate Bash calls, each with exactly the text in its code block and `dangerouslyDisableSandbox: true`, and that the run did not end with `post_merge_sync_failed`. If a pipe, redirect or chain appears, or a denial recurs, strengthen the wording in `.claude/agents/merger.md` step 6.

### Agents: confirm `pr-runner` runs its skill scripts verbatim as standalone calls on a real `/pr` run

#### Background

The `pr-runner-verbatim-script-calls` plan extended `.claude/agents/pr-runner.md` §4 so that every script under `.claude/skills/**/scripts` it runs, the `wait-pr-actionable` call above all, is run exactly as written in its code block as a standalone Bash call: no pipes, no `tail`, no redirects, no `; echo exit=$?`, no `${PIPESTATUS[...]}`, no `;` / `&&` chaining. The rule exists because of PR #671, where `bash .claude/skills/pr/scripts/wait-pr-actionable.sh 671 | tail -12; echo exit=${PIPESTATUS[0]}` missed the `Bash(bash .claude/skills/pr/scripts/*)` allow rule and waited about 62 minutes before it ran. The script then printed `ACTION=ready` at about 150 s, but the call did not return until the 600 s timeout, most likely because a lingering child held the pipe open. A bare re-run returned in 2 s. CI covers only the Markdown formatting. A prose rule cannot be unit-tested, so it has never been exercised by a real unattended pr-runner run. Plan: `docs/plans/_archived/20261004-pr-runner-verbatim-script-calls/plan.md`

Files: `.claude/agents/pr-runner.md`

#### TODO

- [ ] On the next real `/pr` run, read the `pr-runner` subagent transcript (the subagent `.jsonl` files under the Claude Code project directory for that session). Confirm that every `.claude/skills/**/scripts` call, especially `wait-pr-actionable.sh`, has exactly the text in its code block with no pipe, `tail`, redirect, `; echo exit=$?`, `${PIPESTATUS[...]}` or chaining, and that none waited long before starting or was moved to the background after the script finished. If a decorated call appears, strengthen the wording in `.claude/agents/pr-runner.md` §4.

### Agents: `merger.md`'s post-merge paragraph calls backgrounding "impossible"

#### Background

The `agent-foreground-rationale` plan replaced the inaccurate "a subagent exits the moment its turn ends" rationale in the agent files. It left `merger.md`'s post-merge paragraph alone to keep the change surgical. That paragraph still says backgrounding is "impossible for the same reason as step 1". With the corrected reason, backgrounding is ruled out (the foreground run keeps the exit code observable to branch on), not impossible. Plan: `docs/plans/_archived/20261007-agent-foreground-rationale/plan.md`

Files: `.claude/agents/merger.md`

#### TODO

- [ ] Reword the post-merge paragraph so it no longer calls backgrounding "impossible". Its wording should agree with step 1 and with `pr-runner.md` §4.

### App: SIGMA fp L strip thumbnails have over ten times the pixels of other bodies'

SIGMA fp L DNGs have no strip JPEG at or above `PREVIEW_MIN_WIDTH` (1600)
below the 9520x6328 full-size one, so the preview tier falls back to the
full-size JPEG. The strip does not decode that per thumbnail: at scan time
`thumbnail_jpeg` decodes the preview tier at 2/8 scale and re-encodes it,
the index caches the result, and the strip's `thumbnail` command reads the
cache. But 2/8 of 9520x6328 is about 2380x1582 (about 3.8 MP), more than
ten times the pixels of the ~404x270 thumbnail of other bodies, so each
strip cell costs more to decode and to ship over IPC. Each scan also
decodes the ~60 MP JPEG (at 2/8 scale) once per file. OM System / Olympus
ORFs have the same cost on a smaller scale: their only JPEG is 3200x2400, so
the fixed 2/8 gives 800x600 thumbnails, about four times the pixels of the
~404x270 ones (`crates/core/src/orf.rs`; decided in
`docs/plans/_archived/20260930-olympus-orf/plan.md` to leave `scan.rs`
alone for now). Files: `crates/core/src/decode.rs` (`thumbnail_jpeg`), `crates/core/src/scan.rs`,
`crates/app/src/index.rs`, `crates/app/src/commands.rs` (`thumbnail`),
`crates/core/src/arw.rs` (`PREVIEW_MIN_WIDTH`, tier selection).

#### TODO

- [ ] Measure strip thumbnail load time on a SIGMA fp L folder (per-cell
      decode and IPC).
- [ ] Candidate fix, not decided: cap the thumbnail size, e.g. pick a
      larger DCT scale-down (`d.scale(n)`) when the preview is large, or
      resize to the normal thumbnail width.

### App: SIGMA fp L main view appears late on Linux

Since #383 and #385, the main view for SIGMA fp L DNGs displays on Linux
(WebKitGTK), but noticeably later than for other files. SIGMA fp L has no
mid-size embedded JPEG, so the `preview` payload is the 9520x6328 (about
60 MP, about 28 MB) full-size JPEG. On Linux the decode worker must pass
`createImageBitmap` resize options to fit it under `PREVIEW_PIXEL_LIMIT`
(6 MP), because WebKitGTK draws transferred bitmaps of about 6.87 MP or more
transparent. A MiniBrowser run on a synthetic 60 MP JPEG (not the app) took
about 200-400 ms per resized decode, on top of the IPC of the ~28 MB
payload; the real app is unmeasured. Same root cause as "App: SIGMA fp L
strip thumbnails have over ten times the pixels of other bodies'" (strip
side), and the Linux/SIGMA-specific case of "App: unmeasured end-to-end
per-page latency".
Files: `crates/app/ui/src/worker.ts` (resize on decode),
`crates/app/src/commands.rs` (`PREVIEW_PIXEL_LIMIT`).

#### TODO

- [ ] Measure first on the real app on Linux with `Timing logs` on: the
      `page invoke=… decode=… total=… keypressToPixels=…` line splits the
      IPC (invoke) from the decode.
- [ ] Candidate fixes, none decided: prefetch and decode the neighbouring
      pages ahead; or have the backend downscale and cache a mid-size JPEG
      for files lacking one, which would serve the main view and, as a side
      effect, shrink the thumbnail noted in the SIGMA fp L strip item.

### Core: Nikon DX-crop and non-standard-crop AF point mapping is unverified

`nef::parse`'s Nikon `AFInfo2` AF point is read as top-left coordinates in
the `AFImageWidth` / `AFImageHeight` frame, assumed to already match the
JPEG's crop. No DX-crop NEF sample was available to confirm this: exiftool's
`Nikon.pm` hints that some DX-mode results are reported in FX (full-frame)
coordinates, in which case the point would land off by the crop factor when
`AFImageWidth` stays the FX size while the JPEG is the DX crop. Found in the
Step 5 sample survey of
`docs/plans/_archived/20260929-canon-nikon-raw/plan.md`. Files:
`crates/core/src/nef.rs`.

#### TODO

- [ ] Find a Nikon DX-crop NEF sample (or a full-frame Z body shot in DX
      crop mode) and confirm whether the AF point position and
      `AFImageWidth` / `AFImageHeight` are already in DX coordinates or
      need scaling by the crop factor; fix `nef.rs` if the latter.

### Core: widen camera support from public sample RAW files

The user no longer owns the Sigma fp L or BF and cannot shoot new
samples, so public sample files are the verification source for more
cameras. Two kinds exist: raw.pixls.us (CC0; a few files per camera,
often flat test scenes that are weak for AF-point checks; CC0 means a
small file could be committed as a fixture, but tests prefer synthetic
bytes as in `crates/core/src/arw.rs`) and review-site sample galleries
(real scenes, good for AF checks, but not redistributable, so local
verification only). Riffle reads ARW, DNG, NEF, CR3, RAF and ORF today (README.md
"RAW formats"). The work splits into tiers, cheapest first:

1. More DNG-writing cameras (Pentax, Ricoh GR, other Leica bodies,
   phones). DNG reading already exists, so only preview/EXIF
   extraction needs verifying on samples.
2. AF point from MakerNotes that exiftool already decodes (Canon
   `AFInfo`, Nikon `AFInfo2`, Fujifilm `FocusPixel`, Olympus
   `AFPointSelected`, Panasonic `AFPointPosition`). Tag meaning and
   coordinate system are known; samples only confirm. This presupposes
   the RAW container is readable (tier 3), except for bodies that
   write DNG.
3. New RAW containers (CR3 = ISOBMFF, NEF, RAF, ...). Each needs a new
   parser next to `crates/core/src/arw.rs`; implementation outweighs
   verification.

AF data exiftool does not decode needs inference from many off-center
samples, as done for the Sigma BF `0x0147` in
`docs/plans/_archived/20260924-sigma-bf-af-point/plan.md`. Files:
`crates/core/src/arw.rs`, `crates/core/src/reader.rs`, `README.md`.

#### TODO

- [ ] Tier 1: verify preview/EXIF extraction on public DNG samples from
      Pentax, Ricoh GR, other Leica bodies, and phones; add each
      working body to the `docs/humans/cameras.md` table.
- [ ] Tier 2: read the AF point from the exiftool-decoded MakerNote
      tags above, confirming on samples, for bodies whose container
      Riffle can already read. Canon `AFInfo2` (EOS bodies) and Nikon
      `AFInfo2` versions `03xx` / `04xx` (Z bodies) are done
      (`crates/core/src/cr3.rs`, `crates/core/src/nef.rs`). Left: the
      Nikon DSLRs' `AFInfo2` `0100` / `0101` (D850, D500), whose AF point
      is a grid point name rather than a position, and a sample with an
      AF position from the Nikon Z 8 and the Canon EOS R6 (their
      raw.pixls.us samples carry none). Fujifilm `FocusPixel` on the RAF
      bodies is done too (`crates/core/src/raf.rs`, in the embedded JPEG's
      frame); an autofocus sample from the X-T3 and the GFX 100 is still
      missing (theirs are manual focus). Next: the OM
      System / Olympus ORF bodies (`crates/core/src/orf.rs`): CameraSettings
      0x030a `AFTargetInfo` on the OM bodies (a 640x480 frame with the focus
      and selected areas) and 0x0305 `AFPointSelected` (percentages) on the
      Olympus bodies. Every raw.pixls.us sample puts the point near the
      center, so the origin and orientation of the frame are unconfirmed; an
      off-center landscape and a portrait sample are needed (see the Step 2
      survey in `docs/plans/_archived/20260930-olympus-orf/learnings.md`).
- [ ] Tier 3: decide per container whether a new parser is worth it,
      given the samples available. CR3, NEF, RAF and ORF are done
      (`crates/core/src/cr3.rs`, `crates/core/src/nef.rs`,
      `crates/core/src/raf.rs`, `crates/core/src/orf.rs`); the rest remain.
- [ ] Check the Sigma BF AF point's open assumptions against public BF
      samples: portrait orientation, manual-focus behavior, and the
      1000x667 scale (see the `SIGMA_BF_AF_GRID_W` doc comment in
      `crates/core/src/arw.rs` and the archived plan's
      [Trade-offs and risks](docs/plans/_archived/20260924-sigma-bf-af-point/plan.md#trade-offs-and-risks)).

### Core: a RAF folder scan costs ~222ms per file

#### Background

The `fujifilm-raf` Step 1 survey (`docs/plans/_archived/20260930-fujifilm-raf/learnings.md`) ran `riffle-cli scan` over 46 raw.pixls.us RAFs: 221.8ms per file mean on one thread (290.2ms p95), 36 files/s on 24 threads. The embedded JPEG is 4416x2944 / 4000x3000 (against ARW's 1616x1080 preview), it is decoded whole for the sharpness score, and the face search runs on the whole image on frames with no AF point. The Fujifilm `FocusPixel` is now read (Tier 2 of "Core: widen camera support from public sample RAW files"), so the face search is skipped on autofocus frames while manual-focus frames still search the whole image; a scaled decode for the score would be a separate change. Files: `crates/core/src/scan.rs`, `crates/core/src/sharpness.rs`, `crates/core/src/raf.rs`.

#### TODO

- [ ] Measure the scan on a real RAF folder after the AF point is read, and decide whether the sharpness score should work on a scaled decode of the RAF JPEG.

### Core: M-RAW RAF files are unverified

#### Background

No raw.pixls.us sample of the listed Fujifilm bodies is an M-RAW (multi-image) RAF: the M-RAW header words at `0x48` / `0x4c` are zero on all 46 samples of the `fujifilm-raf` Step 1 survey (`docs/plans/_archived/20260930-fujifilm-raf/learnings.md`). `raf::parse` reads the one embedded JPEG and ignores the M-RAW header, which ExifTool suggests is enough, but no such file has been opened. Files: `crates/core/src/raf.rs`.

#### TODO

- [ ] Open an M-RAW RAF (from a body that writes one) and confirm the preview, the 1:1 view and the EXIF rows.

The `raf-older-bodies` sweep of 228 RAFs from 87 bodies (`docs/plans/_archived/20260930-raf-older-bodies/learnings.md`) found no M-RAW either: `0x48` is zero on all of them.

### Docs: Sony AF point, face tracking and portrait orientation are unconfirmed for lack of samples

#### Background

The `sony-arw-coverage` verification (`docs/plans/_archived/20260930-sony-arw-coverage/plan.md`) marks these `–` in `docs/humans/cameras.md` because the raw.pixls.us samples don't show them:

- Every α9 III and α7CR sample is manual focus (`FocusMode` 0, point at the exact center, `FocusFrameSize` invalid). `trusted_focus` drops the point as designed, so both bodies show `–` for AF point and AF frame size.
- Face tracking: the raw.pixls.us samples hold no `AFTracking` 1. The local samples later checked in `sony-face-tracking-docs` (`docs/plans/_archived/20261001-sony-face-tracking-docs/plan.md`) record it on the α7 IV, α7R V, α7S III, α6700 and ZV-E1, so those show `✓`. The α1, α9 III, α7C II and α7CR stay `–` because no sample of them recorded it.
- No sample is a portrait frame (Orientation 1 on all 66 files), so orientation is unverified on every listed Sony body.

Files: `docs/humans/cameras.md`.

#### TODO

- [ ] Find an AF-C sample of a person from the α9 III and α7CR (portrait orientation if possible), and turn their `–` for AF point / AF frame size into `✓` if the point lands on the subject.
- [ ] Confirm orientation on a portrait ARW sample and record the result in `docs/humans/cameras.md`.

### App: Claude Desktop's MCP connection form is unverified

Whether Claude Desktop accepts a direct `url` entry for a local Streamable HTTP
server was not checked by hand (no Claude Desktop available during
implementation); the settings window currently shows the `npx -y mcp-remote
http://127.0.0.1:41917/mcp` fallback form. Files: `crates/app/ui/src/mcp.ts`,
`crates/app/ui/src/mcp.test.ts`.

#### TODO

- [ ] Verify by hand whether Claude Desktop accepts a direct `url` entry for a
      local Streamable HTTP server and, if so, replace the `mcp-remote`
      example.

### App: the settings window's Copy buttons are unverified across webviews

`navigator.clipboard.writeText` for the MCP settings tab's Copy buttons was not
checked in a running app on Linux (WebKitGTK) or macOS. The same call is used
by the folder tree's `Copy Path` / `Copy Folder Name` context-menu items. A
manual run on Windows (WebView2) on 2026-09-28 passed for the MCP tab's Copy
button and both tree items. Files: `crates/app/ui/src/settings.ts`,
`crates/app/ui/src/main.ts`.

#### TODO

- [ ] Verify the settings window's Copy buttons in the Linux (WebKitGTK) and
      macOS webviews, and confirm the select-text fallback fires when the
      write is refused.
- [ ] Verify the folder tree's `Copy Path` / `Copy Folder Name` items the same
      way on Linux and macOS; if a webview refuses the write, switch to
      `tauri-plugin-clipboard-manager` with a
      `clipboard-manager:allow-write-text` capability.

### App: the MCP `get_view` tool is unverified against a running app

Calling `get_view` from an MCP client (e.g. Claude Code) and checking it
returns the open folder's state was not done by hand; no GUI session was
available during implementation. Files: `crates/app/src/mcp.rs`,
`crates/app/ui/src/main.ts`, `crates/app/ui/src/companion.ts`.

#### TODO

- [ ] Verify by hand that `get_view` returns the open folder's state
      (current file, selection, burst, mode) in a running app.

### App: the MCP `get_photo` / `get_preview` tools are unverified against a running app

Checking that `get_photo` returns the index row and shooting settings, and
that `get_preview` shows the image in an MCP client, was not done against a
running app with a real ARW / DNG folder (no GUI session or sample file was
available). Files: `crates/app/src/mcp.rs`, `crates/core/src/decode.rs`.

#### TODO

- [ ] Verify by hand that `get_photo` and `get_preview` work end to end
      against a running app with a real ARW / DNG folder.

### App: the MCP `show_photo` / `select_photos` / `set_view` tools are unverified against a running app

Whether these tools move the strip, the selection, and the view mode of a
running app from an MCP client was not checked (no GUI session was
available). Files: `crates/app/src/mcp.rs`, `crates/app/ui/src/main.ts`,
`crates/app/ui/src/companion.ts`.

#### TODO

- [ ] Verify by hand that `show_photo`, `select_photos` and `set_view` drive
      a running app's strip, selection and view mode from an MCP client.

### App: `set_judgment` byte-identity with a key press is unverified

Whether a `set_judgment` call from an MCP client writes the XMP / `.dop` with
the same bytes a key press produces, and that `Cmd+Z` undoes it, was not
checked by hand (no GUI session was available); the code path is shared by
construction with the key-press path. Files: `crates/app/src/mcp.rs`,
`crates/app/ui/src/main.ts`, `crates/app/ui/src/companion.ts`.

#### TODO

- [ ] Verify by hand that `set_judgment` from an MCP client writes the same
      XMP / `.dop` bytes as a key press, and that `Cmd+Z` undoes it.

### App: the backend `folder_entries` read is the main cost of the remaining `refreshEntries`

Now that the `scan-done` refresh is skipped when neither the scan nor the reconcile changed anything (docs/plans/_archived/20260926-scan-done-refresh-skip/plan.md), the one refresh that still runs (the open-time one) is dominated by the backend `folder_entries` read: 321 ms cold on 2134 rows in the user's Windows debug-build measurement. Files: `crates/app/src/index.rs` (`folder_entries` / `AppIndexReader`), `crates/app/ui/src/main.ts` (`refreshEntries`).

docs/plans/_archived/20260928-strip-keep-scroll-on-rescan/plan.md made the frontend `set_files=true` share of the `refresh entries:` timing line cheaper (no cell teardown and reload on a kept-scroll update), but this item is about the backend read, which is untouched.

#### TODO

- [ ] Investigate why the cold `folder_entries` read costs 321 ms on 2134 rows and whether it can be reduced (indexing, query shape, or caching), verified by a measurement with `Timing logs` on before/after.

### App: the manual GUI checks for the resume landing's selection are still open

Step 1 of `docs/plans/_archived/20260928-resume-selection/plan.md` (merged as
#535) collapsed the selection to the landed file when a pending resume
resolves. The user found the bug on a real Windows machine; the manual check
the step specified was not run, since the GUI cannot be driven from an agent
session. Files: `crates/app/ui/src/selection.ts`, `crates/app/ui/src/main.ts`.

#### TODO

- [ ] On Windows, in folder A focus a file other than the first, open folder
      B, then return to A: the status line shows no `selected` suffix, only
      the landed cell has `.selected`, and a rating key changes only that
      file.
- [ ] The same on macOS.
- [ ] The same with a judgment filter that hides the remembered file, so the
      landing falls on a neighbor: exactly that neighbor is selected.
- [ ] The same with a folder that has no index cache (the `catch` branch of
      `refreshEntries` in `main.ts`).

### App: measure whether a `notify` watch blocks deleting a folder on Windows

`docs/plans/_archived/20260928-tree-live-watch/plan.md` (Step 1) measured only
rename on Windows with `notify` 8.2: a watch pins the watched folder's
ancestors against rename, not the folder itself. Delete was never measured,
so `docs/humans/usage.md` and the plan's Trade-offs section say "rename" only.

#### TODO

- [ ] Using the same scratch-binary approach as the Step 1 rename
      measurement, measure on Windows whether deleting a watched folder
      itself, or a folder with a watched descendant, succeeds with `notify`
      8.2. Record the result next to the rename measurement and update
      `docs/humans/usage.md` and `docs/agents/tauri-app.md` to match (or confirm
      they need no change).

### App: real-device checks of Rewrite Sidecars from Index… and Delete Sidecars…

#### Background

The `folder-sidecars` feature added `Rewrite Sidecars from Index…` and
`Delete Sidecars…` to the folder tree's right-click menu (single folder
only), both behind the shared `#sidecar-dialog`. Rewrite marks the folder's
index rows dirty and drains them through the sidecar writer, patching only
the judgment fields of the current format's sidecars; delete moves those
sidecars to the OS trash as one `trash::Runs` run, clears the rows'
judgments, and is undone / redone through `trash_rejected_undo` /
`trash_rejected_redo`. Rust tests (a Lightroom-style `crs:` XMP and a
PhotoLab-style `.dop` patched with the foreign content intact, the sidecar
collection, a sidecar-only run's restore and redo), Vitest and `mise run ci`
cover it; the GUI, Lightroom, PhotoLab and a real Trash were never
exercised. See `docs/plans/_archived/20261005-folder-sidecars/plan.md` and
its `learnings.md`. Files: `crates/app/src/foldersidecars.rs`,
`crates/app/src/trash.rs` (`redo`), `crates/app/src/index.rs`
(`rows_of`, `mark_dirty`, `clear_judgments`), `crates/app/ui/src/sidecars.ts`,
`crates/app/ui/src/context.ts`, `crates/app/ui/src/main.ts`
(`rewriteSidecarsIn`, `deleteSidecarsIn`, `trashed`),
`crates/app/ui/src/trash.ts`, `crates/app/ui/src/undo.ts`.

#### TODO

- [ ] On Windows, on a copy under `D:\Photos\tests\<date>-folder-sidecars\`
      of RAWs with a Lightroom-written `.xmp` (with develop settings) and a
      PhotoLab-written `.dop` (with corrections), open the folder in Riffle
      under `XMP and .dop`, rate, flag and label a few files, then
      right-click the folder and choose `Rewrite Sidecars from Index…`.
      Expect the item after the `Move Rejected to Trash…` group and before
      `Sequence JPEG Timestamps…`, and absent with several folders
      selected; the dialog titled `Rewrite Sidecars from Index` shows the
      judged / unjudged / skipped rows and `Rewrite the XMP and .dop
      sidecars of N files in <name>?` with a primary `Rewrite`; `Rewrite`
      disables both buttons and shows the running note, then the status
      line reads `Rewrote the sidecars of N files in <name>` and the strip
      is unchanged. In Lightroom (`Metadata > Read Metadata from File`) and
      PhotoLab the judgments show and the develop settings / corrections
      survived; a file with a judgment and no sidecar got a new one, an
      unjudged file without one got none.
- [ ] On Windows, on that copy, choose `Delete Sidecars…`. Expect the dialog
      titled `Delete Sidecars` listing `N XMP sidecars (size)` and `N .dop
      sidecars (size)`, ending `Move N sidecars (size) of <name> to the
      Trash?`, with a red `Move to Trash` and an outline `Cancel`; running
      it moves the files to the Recycle Bin, the status line reads `Moved N
      sidecars to the Trash`, and the open folder's stars, flags and labels
      disappear at once. `Edit > Undo` restores them (`Restored N sidecars
      from the Trash`, the judgments come back after the rescan), and
      `Edit > Redo` moves them again and clears the strip. Repeat the undo
      after opening another folder.
- [ ] On Windows, press each item during a scan (switch to a terminal and
      straight back to start one, or a long first scan): the status line
      shows `Rewrite Sidecars from Index: waiting for the scan to finish` /
      `Delete Sidecars: waiting for the scan to finish` and the dialog opens
      when the scan ends, with no `a scan is running` refusal from the
      backend.
- [ ] On Windows, the refusals: `Rewrite Sidecars from Index…` on a folder
      never opened shows `No judgments indexed for <name>; open the folder
      first`; `Delete Sidecars…` on a folder never opened works; on a folder
      with no sidecar of the current format, delete shows `No XMP sidecars in
      <name>` (naming the format); on a JPEG-only folder both show the
      `applies to RAW files only` refusal; none of them opens a dialog.

### App: a JPEG folder's preview decodes 15-20x longer than an ARW's

Backend measurement (docs/plans/_archived/20260927-jpeg-view-only/learnings.md,
Step 2): a 21.5 MP JPEG's `read_preview` + full `decode_rgb` took ~197 ms
and a 14.8 MP one ~141 ms, versus ~9-15 ms for an ARW/DNG preview. The
plan's remedy (a DCT-scaled, capped re-encode) was measured and found
slower than shipping the whole file, so `preview` sends the whole JPEG as
is.

#### TODO

- [ ] Find a payload that skips the full decode/encode round trip, e.g. a
      DCT-scaled decode (4/8 or 3/8) sent as raw pixels, or reading a
      JPEG's embedded Exif thumbnail first. Verify with `Timing logs` in the
      GUI before choosing. Files: `crates/app/src/commands.rs` (`preview`),
      `crates/core/src/decode.rs`, `crates/app/ui/src/` decode worker.

### App: PhotoLab Uuid lookup on macOS: real-device checks and external volumes

`crates/app/src/photolab.rs` looks up the registered Uuids on macOS too
(`docs/plans/_archived/20261005-photolab-uuids-macos/plan.md`), verified by
hand on macOS (PhotoLab 10, 2026-10-05; see `docs/agents/photolab.md`). Paths
under `/Volumes` get no lookup yet, so a fresh `.dop` there still gets random
Uuids and may become a virtual copy.

#### TODO

- [ ] Real-device check: give a source a virtual copy in PhotoLab and confirm
      the master is the lowest `ZDOPINPUTITEM.Z_PK`.
- [ ] External volumes: open a folder on `/Volumes/X` in PhotoLab, find how it
      records the volume (`ZTYPE`, `ZNAME`, `ZUNIQUEID` in `ZDOPFOLDER`), then
      extend `folder_names` / `query`'s macOS mapping in
      `crates/app/src/photolab.rs`.

### App: PhotoLab after a file or folder rename

From Step 3's check of the PhotoLab Uuid lookup (`crates/app/src/photolab.rs`
queries `Sources` by `FolderId` and `Name`; `crates/core/src/dop.rs` writes
the RAW's `Name` into the `.dop`). `rename_file`
(`crates/app/src/rename.rs`) moves the `.dop` unchanged, so its inner `Name`
still names the old file and PhotoLab's database still holds the old name
and folder.

#### TODO

- [ ] Find out (checked with PhotoLab) whether PhotoLab re-matches a renamed
      RAW with its moved `.dop` or imports it as a new image / virtual copy,
      and either patch the `.dop`'s `Name` on rename or document what to
      expect.

### App: real-device check that an inline rename is refused while its rename is in flight

#### Background

`rename-in-flight-guard` made the folder tree's `startRename` and the strip's
`startRename` refuse to open the inline editor, with a status note, on a row
or cell whose `rename_folder` / `rename_file` invoke is in flight (the tree
reuses `A folder is being renamed; try again in a moment.`, the strip says
`This file is being renamed; try again in a moment.`), and made a pending
mark clear only from the rename that set it. Vitest covers the in-flight
check for a file path and the identity-keyed pending slot, and `mise run ci`
passes. The GUI behavior was never run on a real device. A rename that is
still held behind a scan must still be re-editable as before. Plan:
`docs/plans/_archived/20261007-rename-in-flight-guard/plan.md`. Files:
`crates/app/ui/src/folders.ts`, `crates/app/ui/src/strip.ts`,
`crates/app/ui/src/main.ts`.

#### TODO

- [ ] On macOS, confirm a folder rename in the tree and a file rename in the
      strip while a scan runs, let the scan end, and while `rename_folder` /
      `rename_file` is still running (a large folder or a slow drive) try
      `Rename…` and the slow second click on the same row / cell: the editor
      does not open and the status line shows `A folder is being renamed; try
      again in a moment.` / `This file is being renamed; try again in a
      moment.`; once the rename settles, editing works again. Also re-check
      that a rename still held behind the scan can be re-edited (replace,
      cancel, keep) as before, in both views.
- [ ] On Windows, run the same check as above.

### App: the wait-for-scan manual checks for Move Rejected to Trash and Rename are still open

#### Background

`wait-for-scan` made Move Rejected to Trash (now the folder tree's
`Move Rejected to Trash…` item; the File menu item was removed by
`file-menu-folder-items`, #537), the folder / file
`Rename…` and the settings modal's `Clear Cache` wait for a running scan
instead of refusing with `a scan is running; wait for it to finish`: the
frontend's `IdleGate` holds one pressed operation (a second press replaces
it), the status line names what is waiting, and the held operation runs when
the scan ends. The backend `SCAN_RUNNING` refusals (`crates/app/src/commands.rs`
for trash, `crates/app/src/rename.rs` for renames) remain as the last line of
defense. None of it has been run by hand; the user's Windows setup is where
these checks will
first be run. The trash confirmation is now an in-window HTML dialog
(`trash-rejected-from-tree`, #529), so the archived plan's "closing the
confirm dialog refocuses the window" race no longer applies to trash, only to
Clear Cache's native confirm. The Clear Cache checks live in
`### App: the Clear Cache button's manual GUI verification is still open`
and are not repeated here. Basis: `wait-for-scan`
(`docs/plans/_archived/20260928-wait-for-scan/plan.md`, #519 and #522).
Files: `crates/app/ui/src/idle.ts`, `crates/app/ui/src/main.ts` (`whenIdle`,
`settleIdle`, `trashRejectedIn`, `renameFolder`, `renameFile`, the
`faces-done` `idle.drain()` and the `openDirectory` `idle.discard()`),
`crates/app/ui/src/strip.ts` (`setFiles` carrying the live inline edit across
a rebuild).

#### TODO

- [ ] In an ARW folder, reject a file, switch to a terminal and straight back
      (the `tauri://focus` `resync()` starts a rescan), press
      the folder tree's `Move Rejected to Trash…` on that folder (the File
      menu item was removed by `file-menu-folder-items`, #537): the status
      line briefly shows
      `Move Rejected to Trash: waiting for the scan to finish` and the
      in-window trash dialog (`#trash-dialog`) follows with no error.
- [ ] During a long first scan (thousands of files) press Move Rejected to
      Trash: the status line stays visible for the whole scan and the dialog
      appears when the scan ends.
- [ ] Un-reject a file while the trash waits: it is absent from the dialog's
      counts and is not trashed (the held closure calls
      `trash_rejected_preview` at run time, not press time).
- [ ] Switch folders while an operation waits: it is dropped
      (`idle.discard()` in `openDirectory`) and the status line clears.
- [ ] Rename a folder (tree inline edit) and a file (strip inline edit)
      during a scan: the edit starts at once, the status line shows
      `Rename…: waiting for the scan to finish`, the rename runs when the
      scan ends, and, when the renamed folder is the open one (or contains
      it), the folder reopens at the new path; a file rename keeps the
      cell's marks under the new name; an inline edit still being typed
      survives the strip being rebuilt by the scan's `setFiles` without
      losing typed text or confirming early.
- [ ] Pressing `Move to Trash` in the trash dialog (`trash_rejected_run`
      through `settleIdle`) and confirming a rename never make the backend
      refuse with `a scan is running`; the native-confirm refocus case
      (closing a dialog refocuses `main` and `resync()` starts a scan) is
      Clear Cache's and is covered by that item's check (3).
- [ ] The pending rename display (`pending-rename-display`,
      `docs/plans/_archived/20261006-pending-rename-display/plan.md`), in both the
      folder tree and the strip: a rename confirmed during a scan shows the
      new name muted and italic after the clock icon (hover: `Renames when
      the scan finishes`) at once, and the normal name when it runs; pressing
      Move Rejected to Trash (or another rename) meanwhile, or opening
      another folder, reverts the name with `Rename… was canceled`; a rename
      that fails at the scan's end reverts with its error; re-editing the
      pending cell starts from the pending name, a different name replaces
      it, the original name reverts it with `Rename… was canceled`, and
      `Escape` keeps it; on a labeled strip cell the name stays in the
      label's text color, italic with the clock; the pending row still opens
      its folder and the pending cell is still judged.

### App: real-device check of the File menu separator on Windows

`file-menu-separator` (docs/plans/_archived/20260929-file-menu-separator/plan.md)
fixed the doubled separator in the Windows and Linux File menu by inserting
Settings... and Check for Updates… at `own.len()`, the index right after the
prepended File items, in `app_menu::build`. CI (`mise run ci`) covers the build,
but the rendered menu was never looked at on a real machine, and Step 1 was
ticked on the automated criteria only. Linux follows the same code path but was
not built here.

Files: `crates/app/src/main.rs` (`app_menu::build`).

#### TODO

- [ ] On Windows, run the app and open the File menu: it reads Open Folder,
      Reload Folder, ─, Settings..., Check for Updates…, ─, Close Window, Quit,
      with a single separator between each group (no doubled separator).
- [ ] On Linux, open the File menu and confirm the same order and the single
      separators.

### App: real-device visual checks for the shadcn Neutral dark restyle are still open

#### Background

The `ui-design-tokens` feature moved the whole frontend onto shadcn/ui's Neutral dark tokens and shared component classes, and gave the settings modal a side navigation. CI and the unit tests cover class names, `nextTab` (vertical) and `theme.ts`. Nobody has looked at the result on a real machine: every step's checkbox was ticked on the automated criteria only, so the Windows passes for Steps 1 to 6 and the macOS / Linux passes were never run. The plan is `docs/plans/_archived/20260929-ui-design-tokens/plan.md`; the styling rules are in `docs/agents/ui-styling.md`.

Files: `crates/app/ui/style.css`, `crates/app/ui/index.html`, `crates/app/ui/src/settings.ts`, `crates/app/ui/src/main.ts`, `crates/app/ui/src/icons.ts`, `crates/app/ui/src/strip.ts`, `crates/app/ui/src/folders.ts`, `crates/app/ui/src/theme.ts`, `crates/app/ui/src/tabs.ts`.

#### TODO

- [ ] On Windows (`mise run tauri:dev`), Step 1, Settings: walk every tab (Sidecar with both XMP and `.dop` so `#label-names` shows and hides, Culling, Keyboard Shortcuts with a chip removed and a key captured, Cache, MCP with `Copy`). Expect the modal to read as shadcn Neutral dark: near-black backdrop over the `#0a0a0a` window, `#171717` box, hairline `rgb(255 255 255 / 10%)` borders, `#fafafa` text, outline buttons, `#737373` focus ring. Every control is consistent with the others, the Tab trap cycles through the same controls, and the modal's size and scrolling (a long shortcuts table) are unchanged. Also check that the selected-tab and hover fills, and the chip `×` hover, are told apart (`--muted` and `--accent` are both `#262626`); if the chip `×` hover is invisible, lift it to `--input`.
- [ ] On Windows, Step 2, dialogs: clear the `sidecarFormat` key (or use a fresh profile) to see the first-run dialog and pick a format; open `Move Rejected to Trash` from the folder tree's right-click on a folder with rejects and one without; open `Sequence JPEG Timestamps…` on a JPEG folder and run it. Expect the format dialog on the `#171717` card with three outline choices; the trash dialog with rows on `--muted`, the total, a red destructive `Move to Trash` and an outline `Cancel`; the sequence preview with changed (pick color) and unchanged (`#a1a1a1`) lines, a light primary `Run` and an outline `Cancel`; disabled buttons during a run at half opacity; all four dialogs (with Settings) look like one family.
- [ ] On Windows, Step 3, menus / strip bar / cells: open a folder; open the filter and sort menus and toggle a few items; right-click a cell for the context menu; click a cell and shift-click another; watch a fresh folder's placeholders fill in; rename a file inline. Expect hover on `#262626`; a checked item shows the check mark in its leading column with no fill, and it disappears when unchecked; labels stay aligned, nothing wraps and the filter menu still fits above the strip bar; the `AF eye` heading, the star rows (unlit stars dim), the `#filter-exif` groups and the separators are visible on `#171717` (if a separator vanishes, raise `.menu-separator` to `--input`); the context menu's `.shortcut` column is `#a1a1a1` and a radio item shows its check; the filter / sort toggles are outline buttons, the lit filter toggle is `#e5e5e5`, and the sort toggle is half-opacity in a JPEG folder; the focused cell has the light `--primary` border on `#262626`, the other selected cells the `#737373` border on the same fill, and the two read as different; the rejected dimming, burst band, badges and failed red are as before; placeholders are `#262626`; the inline rename field is `#0a0a0a` with a `#737373` outline; the strip and menu scrollbars are thin and visible.
- [ ] On Windows, Step 4, folder tree: expand a few roots, hover rows, select two folders and open one, move the keyboard cursor while the tree has focus, expand a folder with RAW files, find (or make, e.g. an unreadable folder) a failed folder, rename a folder inline, and drag a folder over the window. Expect `#fafafa` rows with rounded `#262626` hover; the selected folders and the open one share `#262626`, the open one bold and the selected ones normal weight; the keyboard cursor shows a `#737373` ring only while the tree has focus; the count is a pill badge (its fill merges into a hovered / selected / open row, so only the number shows there); a failed folder's name is `#ff6467`; a long name on the open row ellipsizes without wrapping or pushing the count out; the inline rename field is `#0a0a0a` with a `#737373` outline; the drop outline is `#737373`; the tree scrollbar is thin and visible.
- [ ] On Windows, Step 5, viewer / meta pane / empty states: start with no folder open, then open an empty folder and a folder whose files are all filtered out; open a fresh folder and watch it scan; make a sidecar write fail (mark a sidecar read-only and rate the file) and dismiss the error; read the meta pane of a RAW file; toggle 1:1 zoom; open Compare with three frames. Expect the empty hints (`Drop a folder…`, `This folder has no RAW or JPEG files.`, `No files match the current filter.`) centered in `#a1a1a1`; the `scanning N / M` then `analyzing N / M` notes, `1:1` and `Compare · N frames` in `#a1a1a1`; the error row in `#ff6467` (not the old amber) with a ghost `×` that hovers on `#262626` and dismisses; the meta pane's EXIF / Maker note / Analysis headings and keys in `#a1a1a1`, values `#fafafa`, a thin scrollbar; Compare's label bars `#171717` with `#fafafa` text, frames with a faint hairline border, and the active frame showing a light `#e5e5e5` inner ring.
- [ ] On Windows, Step 6, settings side-nav (a debug build so Debug shows, a 1280x720 window): open Settings. Expect the modal wider and taller (`min(860px, 90vw)` x `min(720px, 80vh)`) and fitting; the left nav listing Sidecar, Culling, Keyboard Shortcuts, Cache, MCP and Debug, each with its 16px icon, `#a1a1a1` at rest, hovering on `#262626`, the active one bold `#fafafa` on `#262626`; all six items fit without the nav scrolling and "Keyboard Shortcuts" reads acceptably (one line preferred); the nav's right hairline is visible (if not, raise `.sidenav`'s border to `--input`); `ArrowUp` / `ArrowDown` move and activate with wrapping, `Home` / `End` go to the first / last, `ArrowLeft` / `ArrowRight` do nothing, the focused item shows the `#737373` ring, Tab moves into the content, Shift+Tab back, Escape closes; the shortcuts table scrolls inside the content pane (thin scrollbar) while the header and nav stay put, and a status error shows under the content pane.
- [ ] On macOS (WKWebView), repeat the six Windows passes above (Settings, first-run / trash / sequence dialogs, menus / strip bar / cells, folder tree, empty states / meta pane / status lines / Compare, settings side-nav). Also confirm that `scrollbar-color` is honored or harmlessly ignored, that native checkboxes / radios take `accent-color` (Safari 15.4+; an older engine shows the UA control), and that `:focus-visible` rings appear.
- [ ] On Linux (WebKitGTK), repeat the six Windows passes above and the same three extra confirmations (`scrollbar-color`, `accent-color` on checkboxes / radios with WebKitGTK 2.36+, `:focus-visible` rings).

### App: real-device check of an ORF folder in the strip, sidecars and Move Rejected to Trash

`orf.rs` and the `.orf` arm of `scan::is_raw_file` / `reader::parse_raw`
were verified by unit tests and `riffle-cli bench` / `scan` / `focusbox` on
the raw.pixls.us samples in `D:\photos\samples\ORF\` (never committed), but
the GUI was never exercised on them. See
`docs/plans/_archived/20260930-olympus-orf/plan.md` and its `learnings.md`.
Files: `crates/core/src/orf.rs`, `crates/core/src/scan.rs`,
`crates/core/src/reader.rs`.

#### TODO

- [ ] On Windows, open `D:\photos\samples\ORF\` in the app and check that
      (1) every `.ORF` appears in the strip with a thumbnail, and the folder
      tree's RAW count includes them; (2) the preview and the 1:1 view show
      the 3200x2400 embedded JPEG, the E-M1 Mark II's rotated frames
      (Orientation 8, e.g. `E-M1MarkII_olympus_om_d_e_m1_mark_ii_01.orf`)
      upright; (3) the meta pane shows the EXIF rows (camera without
      trailing spaces, lens, exposure, capture time); (4) a star, a flag and
      a color label write an XMP and a `.dop` next to the file; (5)
      rejecting a file and running `Move Rejected to Trash…` moves the ORF
      and its sidecars, and Undo restores them.

### App: real-device checks for the flex / grid gap layout (no-margin-layout) are still open

#### Background

The `no-margin-layout` feature replaced every layout `margin` in `crates/app/ui/style.css` with flex / grid `gap` and padding, added a reset block, wrapped the meta pane's groups in `div.group` and the shortcut chips in `div.keys`, and made `.folder` a three-column grid. CI, the greps and the unit tests cover the code. Nobody has looked at the result on a real machine: the Step 1 checkbox was ticked on the automated criteria only, so the Windows pass and the macOS / Linux passes were never run. The `div.keys` wrapper (instead of `display: flex` on the `td`) was chosen by reasoning, and the settings-panel spacing before / after values in that feature's `learnings.md` are Inferred from the stylesheet, not measured. The plan is `docs/plans/_archived/20260930-no-margin-layout/plan.md`; the layout rules are in `docs/agents/ui-styling.md`.

Files: `crates/app/ui/style.css`, `crates/app/ui/src/main.ts`, `crates/app/ui/src/settings.ts`, `docs/agents/ui-styling.md`.

#### TODO

- [ ] On Windows (`mise run tauri:dev`), measure in devtools the settings spacing on a pre-change build first (the old `p`-to-`p` and label-to-`#label-names` distances), then on the current `main`, and record it. Expect an even 8px rhythm in the settings panels. Then walk the pass. Dialogs: the first-run format dialog (clear `sidecarFormat` or use a fresh profile; the error line under the note when a write fails), `Move Rejected to Trash…` on a folder with rejects and one without (rows, the empty note, the failed list, the total, the button row), and `Sequence JPEG Timestamps…` on a JPEG folder (source / output lines, rebuild note, rows, count, the running note during a run), all spaced as before. Settings: Sidecar with XMP and `.dop` so `#label-names` shows and hides; Culling; Keyboard Shortcuts with a chip removed and a key captured on a row with enough keys to wrap (chips, the `Press a key...` prompt and `+` vertically centered against the label and Reset cells, 0.2rem / 0.3rem gaps, a one-line row keeping its height, which validates the `div.keys` choice); Cache with the clear note visible during a scan; MCP with the endpoint and both example blocks. Strip bar: the `N / M` counter at the right edge and the toggles at the left, also with `· K selected`; the filter / sort / context menus open and their separators still bleed edge to edge. Folder tree: a long name ellipsizing before its badge, the badge at the right edge, the open folder bold, a collapsed folder without a badge, an inline rename filling the name column without the row jumping, the keyboard cursor ring. Meta pane: the EXIF / Maker note / Analysis headings 0.75rem below the previous block and 0.125rem above their lists; the status notes (scan progress, `1:1`, Compare, an error with its `×`) unchanged. Empty states: no folder, no files, filtered out.
- [ ] On macOS (WKWebView), repeat the Windows pass above, and confirm that flex `gap` renders (the guide's floor: Safari 14.1 / macOS 11).
- [ ] On Linux (WebKitGTK), repeat the Windows pass above, and confirm that flex `gap` renders (the guide's floor: WebKitGTK 2.32).
- [ ] On macOS and Linux, confirm that Settings > Sidecar's `#label-names > label` rows (`grid-template-columns: subgrid`, Chromium 117+ / Safari 16+, see the subgrid bullet in `docs/agents/ui-styling.md` "CSS features") still line up; if not, replace the subgrid with explicit columns.

### App: real-device check of focus after deleting the focused file from outside the app

#### Background

The `todo-five-small-items` feature (Step 3) made `resync` fall back to the deleted focused file's neighbour instead of the first file (`anchorAfterFilter` takes an optional `previous` list, and `refilter` passes it from `resync`). Vitest cases in `crates/app/ui/src/filter.test.ts` cover the picker (next neighbour, previous neighbour, a filtered-out neighbour, consecutive deletions), and `mise run ci` passes. The flow through a real window, where the file is deleted in Explorer / Finder and the app regains focus, was never exercised because the GUI cannot be driven from the implementing session. Plan: `docs/plans/_archived/20260929-todo-five-small-items/plan.md`.

Files: `crates/app/ui/src/main.ts` (`resync`, `refilter`), `crates/app/ui/src/filter.ts` (`anchorAfterFilter`).

#### TODO

- [ ] On Windows, open a folder of RAW files, focus a file in the middle of the strip, delete it in Explorer, and switch back to the app. Expected: the strip stays on the deleted file's next file (the previous one if it was the last), not the first file, and the preview shows that file. Repeat with a filter active whose next neighbour is filtered out. Expected: focus lands on the next passing file.
- [ ] On macOS, the same steps with Finder.

### App: real-device check of the focus-rescan throttle (todo-five-more-items Step 2)

#### Background

The `todo-five-more-items` feature made an idle focus rescan end at once and throttled repeated focus events. Vitest cases for `focusRescanDue` and `mise run ci` cover it. The GUI behaviour was never exercised: the implementation agent had no GUI session. See `docs/plans/_archived/20260930-todo-five-more-items/plan.md` (Step 2).

Files: `crates/app/ui/src/main.ts`, `crates/app/src/commands.rs`.

#### TODO

- [ ] On Windows, with `Timing logs` on, open a folder and let its scan and faces pass finish. Wait more than 5 s, then switch to another app and back to Riffle. Expect `Riffle.log` to show `scan prepare: … todo=0` with no `scan extract` or `scan faces` line after it, and the status line's `scanning` never appears. Then alt-tab away and back twice within 5 s: only the first focus logs a `scan list` line.

### App: real-device check of Expand All / Collapse All in the folder tree (todo-five-more-items Step 3)

#### Background

The `todo-five-more-items` feature added `Expand All` / `Collapse All` to the folder tree's context menu. Vitest cases in `tree.test.ts` / `context.test.ts` and `mise run ci` cover the logic. The real menu on a real photo archive was never exercised, and neither was the watcher count or the ancestor-rename interaction. See `docs/plans/_archived/20260930-todo-five-more-items/plan.md` (Step 3) and the `notify` ancestor-pinning item in `docs/agents/tauri-app.md`.

Files: `crates/app/ui/src/folders.ts`, `crates/app/ui/src/tree.ts`, `crates/app/ui/src/main.ts`.

#### TODO

- [ ] On Windows, right-click a folder with two or more levels of subfolders (e.g. under `D:\photos`) and choose `Expand All`: every level should appear. Then `Collapse All` on the same folder: all levels fold back with the clicked folder still open. Then `Expand All` on a folder containing a subfolder that cannot be listed (e.g. read permission denied): the unreadable folder shows its error (status line, row marked failed) and the rest still expand.
- [ ] On Windows, measure how many `tree-changed` watches a real `Expand All` on a photo archive makes (the number of expanded drawn folders; via `Timing logs` or a debugger on `set_tree_watches`). Also check that renaming an ancestor of the expanded folders still works.

### App: real-device check of the 16-language Lightroom label preset dropdowns (lightroom-label-presets-all-languages)

#### Background

The `lightroom-label-presets-all-languages` feature grew the Lightroom color label preset list from English and Japanese to 16 (de, es, fr, it, ko, nb, nl, pl, pt, ru, sv, th, zh-Hans, zh-Hant added) and made `defaultPreset` preselect a script-coded preset (`zh-Hans` / `zh-Hant`) through `Intl.Locale.maximize()`. `cargo test -p riffle-core`, the `firstrun.test.ts` cases and `mise run ci` cover it, but neither the dropdown contents nor the preselection were run in the GUI. See `docs/plans/_archived/20261005-lightroom-label-presets-all-languages/plan.md`.

Files: `crates/app/ui/src/main.ts`, `crates/app/ui/index.html`, `crates/app/ui/src/settings.ts`, `crates/app/ui/src/firstrun.ts`, `crates/core/i18n/*.json`.

#### TODO

- [ ] On Windows, with a fresh settings store, pick `Lightroom (XMP)` and open the first-launch language dropdown: it lists 16 entries in the order English, Deutsch, Español, Français, Italiano, 日本語, 한국어, Norsk bokmål, Nederlands, Polski, Português (Brasil), Русский, Svenska, ไทย, 简体中文, 繁體中文, and the preselection matches the OS locale (for example `ja` on a Japanese system). Choosing another entry fills the five label names.
- [ ] On Windows, open the settings window's label-name language dropdown: it lists the same 16 entries in the same order, and choosing one fills the five names. (This could also be checked on macOS.)

### App: real-device checks for trash undo / redo after a folder rename (todo-five-more-items Step 5)

#### Background

The `todo-five-more-items` feature rebased the trash runs and their undo / redo entries when a folder is renamed. On Windows / Linux, restore recreates the missing old folders, restores from the Trash, moves the file to the new place and removes only the folders it created; on macOS, `restore_recorded` renames straight to the new destination. `trash.rs` tests (`Runs::rename_dir`, `restore` with a destination, `relocate`), the `undo.test.ts` case and `mise run ci` cover it. No real Recycle Bin or macOS Trash was exercised. See `docs/plans/_archived/20260930-todo-five-more-items/plan.md` (Step 5).

Files: `crates/app/src/trash.rs` (`restore_recorded`), `crates/app/src/commands.rs` (`restore_run`), `crates/app/src/rename.rs`, `crates/app/ui/src/main.ts`.

#### TODO

- [ ] On Windows, reject two files in a copy under `D:\photos\samples\X` (ideally one with both sidecar formats), run `Move Rejected to Trash…`, rename the folder in the tree (`Rename…`), then `Undo`. Expect the files and their sidecars back in the renamed folder, the strip (reopened under the new path) showing them with their reject marks, and no empty old-name folder left behind. Repeat with a recursive run over a parent whose subfolder is renamed. Then `Redo` should move them to the Recycle Bin again.
- [ ] On macOS, repeat the undo / redo after a rename (`restore_recorded` renames the Trash file straight to the new destination, with no `relocate`). Expect the files back in the renamed folder and no old-name folder appearing.

### App: real-device check of the scan priority and on-screen-first order on a large RAW folder

`scan-priority` (docs/plans/_archived/20260930-scan-priority/plan.md) moved the
sharpness score into the second pass, ran the scan workers below normal OS
priority (first pass `BELOW_NORMAL` / `QOS_CLASS_UTILITY`, analysis pass
`LOWEST` / `QOS_CLASS_BACKGROUND`) and added `set_scan_focus`, which makes both
passes take the current file and the strip's visible range first. CI covers
the build, the unit tests (queue order, cancel, delivery once, the priority
read back inside a worker on Windows and Linux) and the Linux / macOS
compilation. Never exercised on a real machine: the effect of the priority on
paging, the on-screen-first order on a cold large folder, the queue's own
overhead in wall time, and the macOS QoS branch. Steps 4 and 6 were ticked on
the automated criteria only.
Files: `crates/core/src/scan.rs` (`for_each_path`, `Priority`, `WorkQueue`),
`crates/app/src/index.rs` (`run_scan`, `run_faces_scan`),
`crates/app/ui/src/main.ts` (`sendScanFocus`), `crates/app/ui/src/scanfocus.ts`,
`crates/app/ui/src/strip.ts` (`visibleRange`), `docs/humans/performance.md` ("Which
pass carries which cost").

#### TODO

- [ ] On Windows, clear the cache (settings modal, `Clear Cache`), open a large RAW folder (a few thousand files) in a development build with the settings modal's `Timing logs` on, and page through it with the arrow keys while `scanning N / M` and then `analyzing N / M` run. Compare the `page invoke=.. decode=.. total=.. keypressToPixels=..` lines in `Riffle.log` with the same run on the previous release (and with paging after the scan ends). Expect that during the passes they are not worse and are closer to the idle value. Note the wall time of the `scan extract` / `scan faces` summary lines of both runs, to see what the lowered priority costs while paging. No `ran at normal priority` warning may appear in the log.
- [ ] On Windows, clear the cache, open the same kind of folder in a development build with `Timing logs` on, and at once jump to the middle of the strip (drag the scrollbar, then click a cell). Expect the cells around the current file to get their thumbnails during `scanning N / M`, and then their focus marks during `analyzing N / M`, before the cells at the folder's start do (by eye, or from the `scan-progress` / `faces-progress` `ready` lists in the webview devtools). Note the wall time of the `scan extract` / `scan faces` summary lines against the previous TODO's run of the same folder, so the queue's own overhead shows.
- [ ] On macOS, repeat the paging run once. Expect the analysis pass (`QOS_CLASS_BACKGROUND`, which also throttles disk IO) not to crawl; if it does, move it to `QOS_CLASS_UTILITY` and update the priority entry in `docs/agents/tauri-app.md`.
- [ ] Done when: the measured page-latency and `scan extract` / `scan faces` numbers, with their conditions, are in `docs/humans/performance.md` ("Which pass carries which cost"), and the README wording on paging is revisited from that result.

### App: real-device check that big-endian DNGs open after the EXTRACTOR_VERSION bump

`big-endian-dng` (`docs/plans/_archived/20261002-big-endian-dng/plan.md`) made `arw::parse` follow the TIFF header's byte order, so the `MM` DNGs of Pentax, Samsung, Ricoh, Leica M (Typ 240 / 246) and iPhone no longer fail with `not a little-endian TIFF/ARW`. It also bumped `EXTRACTOR_VERSION` from 11 to 12, so an index that cached those parse errors should re-extract the rows. The unit tests, `riffle-cli check` on the sample tree (34 of the 52 `MM` files open at every stage) and `mise run ci` cover the parser. The Riffle app was never opened on such a folder, so the re-extraction of cached failed rows and the thumbnails, preview and 1:1 view for these files were never exercised. The step was ticked on the automated criteria only.

Files: `crates/core/src/arw.rs`, `crates/app/src/index.rs` (`EXTRACTOR_VERSION`).

#### TODO

- [ ] On Windows, with an index written by `EXTRACTOR_VERSION` 11 (open `D:\Photos\samples\DNG\` in a build from before the bump, then in this build), open a folder containing the Pentax K-5 II, iPhone 12 Pro and Leica M (Typ 240) samples. Expected: the rows that cached `not a little-endian TIFF/ARW` are re-extracted, and the thumbnails, the preview and the 1:1 view show.

### App: real-device check of the 1:1 view on older Sony ARW bodies (no full-size JPEG)

The `sony-arw-preview-full` work (`docs/plans/_archived/20261005-sony-arw-preview-full/plan.md`) made `arw::parse` fall back to the preview as `full` when the largest other JPEG is smaller, so the α9 II, α7R IV, α7R IVA, α7C, α6400, α6600 and ZV-E10 no longer take the 160x120 IFD1 thumbnail for the 1:1 view. A synthetic-TIFF unit test, `riffle-cli info` / `bench` on the samples (`full` at the preview's offset / length; decodes of 1616x1080, 1440x1080 on the 4:3 α7C sample and 1920x1080 on the ZV-E10 read from the JPEG SOF) and `mise run ci` cover the core path. The running app's 1:1 view (`read_focus_crop` via `reader::read_full`) was never exercised on these files. The step was ticked on the automated criteria only. Files: `crates/core/src/arw.rs`, `crates/core/src/reader.rs`.

#### TODO

- [ ] On Windows (or any desktop build), open `D:\photos\samples\ARW` (or a folder with e.g. `ILCE-6400_DSC00087.ARW` / `ZV-E10_DSC00002.ARW`), select the file and open the 1:1 focus check. Expected: a sharp crop of the 1616x1080 preview (1920x1080 on the ZV-E10) around the AF point, not a blurry upscaled 160x120 thumbnail.

### Agents: bare `python` / `node` hang in Git Bash on the Windows host

#### Background

During `merge-settings-drop-hook-exclusions` (`docs/plans/_archived/20261002-merge-settings-drop-hook-exclusions/learnings.md`), a bare `python` or `node` in Git Bash on the Windows host hung instead of failing; running them through `mise x --` worked. Any agent session on this host that runs a script needing them hits the same hang, before any guide would be consulted.

Files: `CLAUDE.md` (Development) or a new `docs/agents/dev-environment.md`.

#### TODO

- [ ] Check that the note is not already in the user-level memory or instructions, then add it: preferably a short line under `CLAUDE.md` Development ("On Windows, run python / node through `mise x --`; a bare invocation hangs in Git Bash"), otherwise a new `docs/agents/dev-environment.md`. Done when the note exists in the chosen place.

---

### App: real-device checks for the folder tree's Refresh item

#### Background

The error-row-exif-tree-refresh feature (Step 2) added a `Refresh` item to the folder tree's right-click menu. It re-lists a folder with the same `list_subfolders` + `setChildren` as `toggle`, keeps the folders open under it, and re-sends the watch set so a watch that failed is retried. `context.test.ts` covers the menu groups. The two checks below were never run on a real machine: they need a folder whose watch cannot be set (a network share) and a folder that vanishes without its parent's watch removing the row. Plan: docs/plans/_archived/20261002-error-row-exif-tree-refresh/plan.md.

Files: `crates/app/ui/src/folders.ts` (`refresh`), `crates/app/ui/src/context.ts`.

#### TODO

- [ ] On a network share (or any folder whose watch `set_tree_watches` cannot set), expand a folder, create a subfolder in it from another machine or the OS file manager, then right-click the folder in Riffle's tree and choose `Refresh`. Expected: the new subfolder appears and the RAW count updates, with the folders already open under it still open.
- [ ] On the same kind of folder, delete or unmount a folder shown in the tree (without its parent's watch removing the row, e.g. on a share), then choose `Refresh` on it. Expected: the row is marked failed (its tooltip shows the error), it collapses, and the error appears in the status line.

### App: real-device check that a folder open, a cache clear and a focus return each start exactly one scan

#### Background

The `picker-focus-double-scan` feature wrapped `openFolder`'s whole chain (the `pick_folder` invoke through `openDirectory`) in `settleIdle`. A `tauri://focus` rescan that arrives while the folder picker is open is now deferred, so it no longer calls `scan_folder` for the previous folder. `scan_folder` also logs `scan superseded: dir={dir} scan_id={scan_id} by={latest}` at info level when its id is superseded. `mise run ci` (vitest, Rust tests) passes. It does not cover the race itself, which is Tauri IPC ordering between two `invoke`s and a window event. No GUI session ever exercised it, so the check below was never run. See `docs/plans/_archived/20261003-picker-focus-double-scan/plan.md` (Step 1).

Files: `crates/app/ui/src/main.ts` (`openFolder`, the `tauri://focus` listener), `crates/app/src/commands.rs` (`scan_folder`).

#### TODO

- [ ] On Windows, run `mise run tauri:release:devtools` with timing logs on. With folder A open and its scan older than 5 s, choose File > Open Folder and pick folder B. Expect exactly one `scan list` / `scan prepare` / `scan extract` set, for B, and no `scan superseded` line.
- [ ] On Windows, repeat with Settings > Clear Cache. Expect one set, for the reopen.
- [ ] On Windows, alt-tab away from the app and back. Expect one set.

### App: real-device check of Refresh on the open folder also reloading the strip

#### Background

The refresh-reloads-open-folder feature made the folder tree's right-click `Refresh` on the open folder also run `resync("refresh")`, the same rescan as `File > Reload Folder` (`CmdOrCtrl+R`). It logs a dedicated `trigger=refresh`, so `Riffle.log` tells the menu item from `CmdOrCtrl+R`. `refresh.test.ts`, `trash.test.ts` and `mise run ci` cover the trigger line and the same-folder decision. The behavior on a folder whose watch does not fire was never exercised on a real machine. Plan: docs/plans/_archived/20261006-refresh-reloads-open-folder/plan.md.

Files: `crates/app/ui/src/main.ts` (the `refreshFolder` branch), `crates/app/ui/src/refresh.ts` (`RescanTrigger`).

#### TODO

- [ ] On Windows, open a folder on a share (or any volume) whose watch does not fire, add a RAW to it and edit a sidecar outside Riffle, then right-click the open folder in the tree and choose `Refresh`. Expected: the new file appears in the strip, the edited judgment is picked up, the tree's RAW count updates, and `Riffle.log` shows `rescan: trigger=refresh`.
- [ ] On Windows, choose `Refresh` on the open folder during a running scan. Expected: `Riffle.log` shows `rescan deferred: trigger=refresh`, and the rescan runs when the scan ends.
- [ ] On Windows, choose `Refresh` on a folder that is not open, including its parent or a subfolder. Expected: the strip is left untouched and only the tree re-lists.

### App: a redo of Delete Sidecars… leaves a closed folder's rows judged

#### Background

The folder-sidecars feature added `Delete Sidecars…`. `trash_rejected_redo` moves the sidecars to the Trash again but writes no index row. When the target folder is open, the frontend's resync clears the judgments. When another folder is open, the rows keep the judgments that the undo's rescan restored, until the folder is next opened and its reconcile clears them. A `Rewrite Sidecars from Index…` run in between would mint the sidecars again. Fixing it needs the run to remember each sidecar's RAW, for example in `trash::Trashed`. Plan: `docs/plans/_archived/20261005-folder-sidecars/plan.md`. Files: `crates/app/src/commands.rs` (`trash_rejected_redo`), `crates/app/src/foldersidecars.rs`, `crates/app/src/trash.rs`, `crates/app/ui/src/main.ts` (`trashed`).

#### TODO

- [ ] Record each sidecar's RAW in the trash run (for example in `trash::Trashed`), so that a redo clears the index rows of a closed folder.
- [ ] Add a test covering delete, undo, then redo with another folder open.

### App: Rewrite Sidecars from Index… patches an oversize sidecar

#### Background

The folder-sidecars feature added `Rewrite Sidecars from Index…`. `rewrite_sidecars_run` hands every listed row to the writer, including a file whose sidecar is larger than `MAX_SIDECAR_BYTES`. A folder open deliberately keeps such a file away from the writer through `reconcile_sidecars_of`, because its contents were never read. This was left as is because the user asked for the index to be pushed out. Plan: `docs/plans/_archived/20261005-folder-sidecars/plan.md`. Files: `crates/app/src/foldersidecars.rs`, `crates/app/src/commands.rs` (`MAX_SIDECAR_BYTES`, `reconcile_sidecars_of`).

#### TODO

- [ ] Decide whether the rewrite should skip oversize sidecars like a folder open does.
- [ ] If it should, skip them and count them in the dialog's "skipped" figure.

### App: real-device checks for the main window being created hidden (startup window flash)

#### Background

The startup-window-flash feature sets `"visible": false` on the main window in `crates/app/tauri.conf.json`. `tauri-plugin-window-state` then restores the saved size and position before the window is shown. `crates/app/src/main.rs` also calls `show()` in the `setup` closure as a fallback. That fallback covers two cases:
- A saved `visible: false`, for example from quitting while minimized on macOS.
- A failed `set_position` / `set_size` / `set_fullscreen` that aborts `restore_state` before its own `show()`.

CI covers only the automated criteria (`mise run ci`). No GUI session was available, so none of the checks below was ever run, and the step's checkbox was ticked on the automated criteria alone. Plan: `docs/plans/_archived/20261010-startup-window-flash/plan.md`.

Files: `crates/app/tauri.conf.json`, `crates/app/src/main.rs`.

#### TODO

- [ ] On Windows, on a built binary (`pnpm tauri build`; the `pnpm tauri dev` binary is acceptable, so note which was used):
  1. With a saved `.window-state.json` (app config dir) at a non-default size and position, launch. The window appears directly at the saved size and position, with no 1280x800 frame visible first.
  2. Delete `.window-state.json` and launch. The window appears at 1280x800, OS-placed, and focused.
  3. Edit `.window-state.json` so `main.visible` is `false` and launch. The window still appears (the fallback `show()` path).
  4. With the app running (once normal, once minimized), launch it again. The existing window is unminimized and focused, and no second window appears.
  5. Quit maximized and relaunch. It comes back maximized with no normal-size frame first.
  6. Note whether a white (unpainted WebView) frame shows before the dark UI paints. If it does and bothers the user, the follow-up is `"backgroundColor": "#0a0a0a"` on the window entry in `tauri.conf.json`.
- [ ] On macOS, minimize the window, quit with Cmd+Q, and relaunch. The window appears. This is the real-world source of a saved `visible: false`.
