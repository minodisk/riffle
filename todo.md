# todo

## Cross-cutting / other

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
      and record AUC / precision / coverage. Files: `crates/cli/src/main.rs`,
      `crates/core/src/candidate.rs`.

### Docs: the "Focus candidate pass" numbers in docs/performance.md are missing the app's own scan/faces log lines

Step 3 of `docs/plans/_archived/20260925-focus-candidate/plan.md` asked for the
`scan extract` / `scan faces` log lines of one app open of the 2134-file
Sony folder to be recorded in `docs/performance.md` "Focus candidate pass",
alongside the `riffle-cli scan` / `riffle-cli candidates` numbers already
there. The implementing agent could not drive the GUI, so only the CLI
numbers are recorded; `docs/performance.md` says so in a note under the
section.

#### TODO

- [ ] Open the 2134-file Sony folder once in the app and record its
      `scan extract` / `scan faces` log lines in `docs/performance.md`
      "Focus candidate pass", next to the existing CLI numbers.

### App: unmeasured end-to-end per-page latency

End-to-end per-page latency (IPC + `createImageBitmap`) is unmeasured, since the GUI could not be driven from this development machine. Only the Rust-side file-read cost was measured; see "Per-page preview read" in docs/performance.md.

#### TODO

- [ ] Measure keypress-to-pixels per page turn on real hardware and add the numbers to "Per-page preview read" in docs/performance.md. The instrumentation now exists: with `Timing logs` on, the app logs a `page invoke=… decode=… total=… keypressToPixels=…` line per page turn to `Riffle.log`, and "Measuring on your own folder" in docs/performance.md spells out the procedure. Only running the measurement and filling in the numbers is left.

### App: a scan can be started twice after a cache clear / focus rescan

In the Windows real-folder measurement, after a cache clear `scan_id` N was superseded by N+1 with no `scan extract` line for N, and two focus rescans once fired at the same instant. It reproduced on 2026-09-22 (Windows 11, v0.2.0): `scan_id=3` and `4` started in the same second at 04:39:40. This may be one bug or two. Files: `crates/app/src/commands.rs` (`scan_folder`), `crates/app/src/watch.rs`, the settings-window clear-cache path.

#### TODO

- [ ] Find why two scans start and make the second one not fire (or coalesce it), verified by the log showing one `scan extract` per trigger.

### App: cold first scan on an internal SSD is far slower than the extrapolation

A real cold first scan on Windows 11 (internal SSD, 22 threads, Sony ARW) costs ~16-19ms per file, ~82-97s extrapolated to 5000 files against the 30s target; see "Real folders on Windows" in docs/performance.md. Excluding the folder from Defender did not help, and a warm-cache scan runs at ~1ms per file, so neither Defender nor CPU is the cause. The cause is unknown. A later data point (Windows 11, v0.2.0, 2026-09-22): a cold first scan after an index schema change took 25.5s on 3045 Sony ARW (~8.4ms/file, ~42s extrapolated to 5000), against the earlier 16-19ms/file; it is unknown whether the OS cache was cold for that run.

#### TODO

- [ ] Run `riffle-cli scan` on a cold real folder on Windows at thread counts 1 / 4 / 8 / 22 (cold each run) to separate IO concurrency from per-file cost, and compare the bounded 1MiB read against reading the whole file.

### App: `open entries` is called again after a scan's follow-up rescan

In the Windows real-folder measurement, the log showed two `open entries` lines (56ms and 76ms on 2677 files) for one folder open. A later run (Windows 11, v0.2.0, 2026-09-22) showed a plain folder open logs one `open entries` (seen at 03:15 and 04:00); the extra call appears after scan-done when a follow-up rescan runs (`scan_id=2` immediately after the cold scan finished at 04:39:37).

docs/plans/_archived/20260928-strip-keep-scroll-on-rescan/plan.md made the follow-up refresh invisible in the UI (no scroll snap, no thumbnail reload), but did not remove the extra `folder_entries` read or its timing cost; this item is still about that.

#### TODO

- [ ] Find why a follow-up rescan starts right after a cold scan finishes and whether its `open entries` is needed (`crates/app/src/commands.rs` `scan_folder`, `crates/app/ui/src/main.ts`); remove it or document why it is needed.

### App: a deep-row focus point still exceeds the 50ms budget

A focus point in a deep row of the unrotated JPEG measured 58-65ms keypress to pixels (n=2, before the #56 and #60 fixes); see "The 1:1 focus check path" in docs/performance.md. Options are prefetching the neighboring files' crops (Phase 4's ring buffer) or a DCT-scaled placeholder; nothing is chosen.

#### TODO

- [ ] Retake the deep-row measurement post-fix, then decide between crop prefetch and a DCT-scaled placeholder.

### App: the silent update path is unverified end-to-end

The Updating paragraph in `docs/usage.md` describes a background download and install on launch and the "Check for Updates…" menu item, but nothing has confirmed on a real build that an installed copy detects a newer release, installs it silently, and launches as the new version next time. The background flow is verified on Windows 11 (2026-09-22): 0.1.10 downloaded 0.2.0, installed it on quit, and launched as 0.2.0.

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
opt-in `set_icon_as_template` API, against muda `dev`.

#### TODO

- [ ] Once muda#413 (or equivalent) ships in a muda release that Tauri
      resolves under its `muda = "^0.19"` (or a later Tauri bump), **and**
      Tauri exposes the template flag for menu items, drop the
      `[patch.crates-io]` entry from the workspace `Cargo.toml` and switch to
      the upstream opt-in API.
- [ ] Re-verify in the running app that all bundled menu PNGs still tint
      correctly with the menu appearance after the switch.

Related: `Cargo.toml`, `docs/agents/tauri-app.md`,
`tools/macos/export-menu-icons.swift`.

### App: real-device checks for the View menu

`view-menu-panes` (docs/plans/_archived/20260930-view-menu-panes/plan.md) cut the native
`View` menu down to three `CheckMenuItem`s (`Folders`, `Metadata`,
`Filmstrip`) whose checks follow the shown panes, with the pane toggles moved
to modifier defaults (`Ctrl+Alt+Arrow` on Windows / Linux, `Alt+Cmd+Arrow` on
macOS) so they show an accelerator. CI covers the build and the tests, but the
macOS `cfg` branch of `app_menu::build` / `refresh` was only reviewed by
reading, and the Windows GUI checks (the menu, the checks, the accelerators)
were never run: the step was ticked on the automated criteria only. Reverting
muda's native toggle on click relies on muda toggling the check before it
sends the event, which was confirmed only by reading its source.
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
      (screen rotation). If it does, note it in `docs/usage.md` as a driver
      setting to turn off (not a blocker; do not change the default).
- [ ] On macOS, confirm the three items sit above `Enter Full Screen` with a
      separator between, show `Alt+Cmd+ArrowLeft` / `ArrowRight` /
      `ArrowDown`, and have checks matching the panes the last session left.
- [ ] On macOS, press each of the three keys once: the pane toggles exactly
      once (no double fire from keydown plus the accelerator) and the check
      follows. Click `View > Filmstrip` twice: hides, shows, check right each
      time. With Settings open, click `View > Folders`: nothing changes and
      the check stays.
- [ ] On macOS, rebind `toggleStrip` to `ctrl+alt+s`: the item shows it (the
      menu is rebuilt), the checks survive the rebuild, `Ctrl+Alt+S` toggles
      once, and `Reset` restores `Alt+Cmd+ArrowDown`.

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
      sharpness bar and the file name), unchanged by the selection, the gap above a
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

### App: a large folder gives no visible loading feedback beyond the status line

Opening a 500-JPEG folder on Windows on 2026-09-28, the only sign of
progress was the small bottom-left status text (`JPEG folder: view only`,
`scanning 223 / 500`), so it was unclear whether loading had started.
Files: `crates/app/ui/src/strip.ts`, `crates/app/ui/src/main.ts`,
`crates/app/ui/style.css`.

#### TODO

- [ ] Consider a progress bar in the strip or the main view during the scan
      and/or a loading indicator in cells with no thumbnail yet; decide and
      implement.

### App: face/eye-aware focus check for culling

Face/eye-aware detection and scoring (`crates/core/src/faces.rs`, `crates/core/src/sharpness.rs`) now runs at scan time: sharpness is scored on the Sony eye-AF frame, else on the AF point, else on the eyes of a detected face when there is no trusted AF point, else on the sharpest tile; an AF point off the face no longer scores a bystander's eyes (see `docs/plans/_archived/20260922-face-aware-sharpness/` and `docs/plans/20260924-face-catch-state/`). What remains:

#### TODO

- [x] MakerNote check (exiftool 13.59, 2026-09-22): Sony α7 V ARW writes
      `AFTracking` + `FocusFrameSize` + `FocusLocation` but no per-face list;
      `AFAreaMode` is enciphered; Leica M11-P DNG writes only
      `FocusDistance`. The scan now uses the Sony AF frame and skips detection
      when face tracking was engaged
      (`docs/plans/20260922-sony-eye-af-window/`).
- [x] Measured on Linux WSL2 (not the Mac): detection latency on real
      α7 V ARW / M11-P DNG previews and `riffle-cli scan` before/after adding
      detection and the AF-frame skip; see `docs/performance.md` "Face
      detection cost".
- [ ] Improve small-face recall: the detector found no face in 29 of 36
      sampled M11-P DNGs and 2 of 7 people in a group frame
      (`L1005161.DNG`).
- [ ] Decode the preview for detection at a DCT-scaled size instead of a
      full-size RGB decode, to cut the per-file cost on the detector path.
- [ ] Optionally, detect closed eyes from the landmarks.
- [ ] Suggest the sharpest-eye frame within a burst group.
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

Related: `crates/core/src/sharpness.rs`, `crates/core/src/faces.rs`, `crates/core/src/arw.rs`, `crates/app/src/index.rs`, `crates/app/ui/src/sharpness.ts`, `crates/app/ui/src/burst.ts`.

### Merge skill: jq reserved words as variable names in skill scripts

jq 1.6 rejects `$label` (`label` is a jq keyword) with `syntax error, unexpected
label, expecting IDENT`; jq 1.7+ accepts it, so CI (which runs jq 1.7.x) never
catches it. This caused `.claude/skills/merge/scripts/wait-post-merge-runs.sh`
to exit 3 on jq 1.6 and made the merger report `run_list_failed` on successful
merges (fixed in the `fix-jq-label-keyword` plan by renaming `$label` to
`$verdict`). Other jq keywords (`def`, `as`, `if`, `reduce`, `foreach`, `try`,
`import`, `include`, `and`, `or`, `not`, ...) would fail the same way in any
other skill script. No guide currently covers skill-script (shell/jq)
conventions.

#### TODO

- [ ] Either create a guide for skill-script (shell/jq) conventions that
      includes a note to avoid jq keywords as variable names, or judge it not
      worth a guide and close this with no action

### Agents: confirm the long-wait timeout fix on a real `/pr` or `/merge` run

The `long-wait-timeouts` plan added explicit `timeout: 600000` to every
`wait-pr-actionable.sh` / `wait-post-merge-runs.sh` call site and a rule for
`merger` / `pr-runner` to stop improvising `sleep`/`until` polling and to
`TaskStop` their own lingering background tasks before handing back. The fix
is verified only statically (docs and script comments); no real run has
confirmed it works.

#### TODO

- [ ] Watch `merger` and `pr-runner` through their waits on a real `/pr` or
      `/merge` run and confirm they pass `timeout: 600000` (no "moved to the
      background" message), do not improvise polling if a command is
      backgrounded anyway, and `TaskStop` any background task of their own
      before handing back, so `ListAgents` shows them completed with nothing
      running. If not clean, adjust
      `.claude/agents/{merger,pr-runner}.md`.

### Release: the draft-then-publish release flow is unverified on a real release

The `draft-release-publish` plan made release-please create the release as a
draft and added a `publish` job that publishes it after every build job
succeeded, so `releases/latest/download/latest.json` never 404s mid-build. It
is verified only by actionlint; no real release has run through it. Files:
`.github/workflows/release.yml`, `release-please-config.json`.

#### TODO

- [ ] On the next `chore(main): release x.y.z` merge, run the "Verification on
      the next release" checklist in
      `docs/plans/_archived/20260923-draft-release-publish/learnings.md`
      (draft while building, tag exists at once, no duplicate release, published
      as Latest with a four-platform `latest.json`, and the next release PR has
      the right changelog base).

### Core: an explicit `Orientation = 1` line for landscape `.dop` files is unverified in PhotoLab

From `dop-orientation`'s implementation: PhotoLab 10 was verified to display
correctly with `Orientation = 8` (portrait) written into a Riffle-made `.dop`
sidecar, but a landscape file's explicit `Orientation = 1` line has not been
checked in isolation in PhotoLab. Files: `crates/core/src/dop.rs` (`template`,
`Doc::insert_orientation`).

#### TODO

- [ ] Open a Riffle-made `.dop` sidecar for a landscape (EXIF Orientation 1)
      RAW in PhotoLab 10 and confirm it displays upright with the explicit
      `Orientation = 1,` line present.

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

### Core: pre-2012 NEFs fall back to the full-size JPEG for the preview

NEFs from bodies older than about 2012 (D3, D40, D70, D90, D7000 on the
raw.pixls.us samples) carry one JPEG SubIFD, so `nef::parse` uses the
full-size JpgFromRaw as the preview too: each page turn decodes a
3000-5000 px JPEG and the fixed 2/8 thumbnail scale gives large thumbnails,
the same issue recorded above for the SIGMA fp L. Found in the Step 2
sample survey of `docs/plans/20260929-canon-nikon-raw/plan.md`. Files:
`crates/core/src/nef.rs`, `crates/core/src/decode.rs` (`thumbnail_jpeg`),
`crates/core/src/scan.rs`.

#### TODO

- [ ] Decide whether older NEFs are worth a smaller preview (the MakerNote
      `PreviewIFD` JPEG is 570x375, below `PREVIEW_MIN_WIDTH`) or a scaled
      decode of the full JPEG, and fold it into the SIGMA fp L thumbnail
      fix if that lands first.

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
"RAW formats and cameras"). The work splits into tiers, cheapest first:

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
      working body to the README "RAW formats and cameras" list.
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

### Core: older Sony ARW bodies take the 160x120 thumbnail as the full-size JPEG

#### Background

The `sony-arw-coverage` sample verification (`docs/plans/_archived/20260930-sony-arw-coverage/plan.md`) found that the α9 II, α7R IV, α7R IVA, α7C, α6400, α6600 and ZV-E10 write no full-size JPEG. Their IFD chain holds only IFD0 (the 1616x1080 preview; 1920x1080 on the ZV-E10) and IFD1 (a 160x120 thumbnail), and the SubIFD is the raw data. `arw::parse` picks the largest JPEG in the IFD chain and the SubIFDs as `full`, which on these files is the IFD1 thumbnail. The `bench` full decode takes 0.2 ms, and `read_focus_crop` (the 1:1 view) crops an upscaled 160x120 image. The recent bodies add IFD2, the full-size JPEG, and are listed in the README. These seven bodies are left off the README until this is decided. Files: `crates/core/src/arw.rs` (`parse`), `crates/core/src/reader.rs` (`read_full`), `crates/app/src/commands.rs` (`read_focus_crop`), `README.md`, `README.ja.md`, `docs/cameras.md`, `docs/raw-formats.md`.

#### TODO

- [ ] Decide what 1:1 shows without a full-size JPEG (refuse, fall back to the 1616x1080 preview, or decode the raw data), and implement it with a synthetic-TIFF unit test next to the existing ones in `arw.rs`.
- [ ] Add the seven bodies to `README.md`, `README.ja.md` and `docs/cameras.md`. Per learnings.md, the samples show AF point present on all; `FocusFrameSize` absent (`–`); sub-second present on the α9 II, α7C and ZV-E10 only; `AFTracking` 2 on the α9 II, α6400 and ZV-E10, else 0.

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

### Docs: Sony AF point, face tracking and portrait orientation are unconfirmed for lack of samples

#### Background

The `sony-arw-coverage` verification (`docs/plans/_archived/20260930-sony-arw-coverage/plan.md`) marks these `–` in `docs/cameras.md` because the raw.pixls.us samples don't show them:

- Every α9 III and α7CR sample is manual focus (`FocusMode` 0, point at the exact center, `FocusFrameSize` invalid). `trusted_focus` drops the point as designed, so both bodies show `–` for AF point and AF frame size.
- No sample records `AFTracking` 1 (face tracking) on any body.
- No sample is a portrait frame (Orientation 1 on all 66 files), so orientation is unverified on every listed Sony body.

Files: `docs/cameras.md`.

#### TODO

- [ ] Find an AF-C sample of a person from the α9 III and α7CR (portrait orientation if possible), and turn their `–` for AF point / AF frame size into `✓` if the point lands on the subject.
- [ ] Find a sample that records `AFTracking` 1 on any listed Sony body, and mark Face tracking `✓` for it.
- [ ] Confirm orientation on a portrait ARW sample and record the result in `docs/cameras.md`.

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

### App: the first-launch dialog does not ask for Lightroom's UI language

The first-launch dialog only asks for the sidecar format. The `xmp:Label`
names start as the English preset (`Red`, `Yellow`, ...) and can only be
changed later under Settings > Lightroom language. A user whose Lightroom
Classic runs in another language (e.g. Japanese, whose default set is
`レッド`, `イエロー`, ...) who picks XMP or Both gets every Riffle-written color
label shown as a white, unmatched label, with nothing pointing at the cause.
Found during the 2026-09-28 manual Lightroom checks (`lightroom-verified`).
Files: `crates/app/ui/src/main.ts` (`showFormatDialog`),
`crates/app/ui/index.html`, `crates/app/ui/src/settings.ts`,
`crates/core/i18n/`.

#### TODO

- [ ] When XMP or Both is chosen in the first-launch dialog, require choosing
      Lightroom's UI language (one of the `crates/core/i18n/` presets) before
      the dialog closes, and save that preset's names as the label names.

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

### App: Open in Terminal from the folder tree's context menu

A folder-menu item that opens a terminal in the folder. The terminal has to be
chosen per platform (Windows Terminal or `cmd` on Windows, Terminal.app on
macOS); on Linux there is no standard terminal, so the choice is ambiguous and
needs a fallback or a setting. Basis: deferred in
`docs/plans/20260927-folder-menu-copy/plan.md` (Purpose). Files:
`crates/app/src/folders.rs`, `crates/app/ui/src/context.ts`,
`crates/app/ui/src/main.ts`.

#### TODO

- [ ] Decide the terminal per platform (and the Linux fallback), then add an
      `Open in Terminal` item to the folder context menu backed by a command
      in `crates/app/src/folders.rs`.

### App: Refresh from the folder tree's context menu for a folder whose watch failed

An expanded folder follows the disk through its own watcher
(`docs/plans/20260928-tree-live-watch/plan.md`), and the open folder's
re-index already has the focus rescan and `File > Reload Folder`. What is left
is a folder whose watch could not be set (a network share, a permission
refusal): `set_tree_watches` only `log::warn!`s it, so its subfolders and RAW
count go stale until it is collapsed and expanded again, with no visible sign.
A manual Refresh item re-lists it. Basis: narrowed from the Refresh / Rescan
item deferred in `docs/plans/20260927-folder-menu-copy/plan.md` (Purpose), as
decided in `docs/plans/20260928-tree-live-watch/plan.md`. Files:
`crates/app/ui/src/folders.ts`, `crates/app/ui/src/context.ts`,
`crates/app/src/treewatch.rs`.

#### TODO

- [ ] Add a Refresh item to the folder context menu that re-lists the folder's
      subfolders and RAW count (optionally offered only, or marked, when its
      watch failed).

### App: measure whether a `notify` watch blocks deleting a folder on Windows

`docs/plans/_archived/20260928-tree-live-watch/plan.md` (Step 1) measured only
rename on Windows with `notify` 8.2: a watch pins the watched folder's
ancestors against rename, not the folder itself. Delete was never measured,
so `docs/usage.md` and the plan's Trade-offs section say "rename" only.

#### TODO

- [ ] Using the same scratch-binary approach as the Step 1 rename
      measurement, measure on Windows whether deleting a watched folder
      itself, or a folder with a watched descendant, succeeds with `notify`
      8.2. Record the result next to the rename measurement and update
      `docs/usage.md` and `docs/agents/tauri-app.md` to match (or confirm
      they need no change).

### App: expand / collapse all subfolders from the folder tree's context menu

Folder-menu items that expand or collapse every subfolder under the clicked
folder. Basis: deferred in `docs/plans/20260927-folder-menu-copy/plan.md`
(Purpose). Files: `crates/app/ui/src/tree.ts`, `crates/app/ui/src/folders.ts`.

#### TODO

- [ ] Add `Expand All` / `Collapse All` items to the folder context menu.

### App: rewrite or delete a folder's sidecars from the folder tree's context menu

Folder-menu items that rewrite every sidecar in the folder from the index, or
delete them; deleting sits behind a confirmation dialog. Basis: deferred in
`docs/plans/20260927-folder-menu-copy/plan.md` (Purpose). Files:
`crates/app/src/sidecar.rs`, `crates/app/src/commands.rs`,
`crates/core/src/xmp.rs`, `crates/core/src/dop.rs`.

#### TODO

- [ ] Add a sidecar rewrite item and a delete item (behind a confirmation
      dialog) to the folder context menu.

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

### App: PhotoLab's virtual-copy fix only works on Windows

`crates/app/src/photolab.rs`'s database lookup (added in
`docs/plans/_archived/20260928-dop-photolab-uuids/learnings.md`) is
`#[cfg(windows)]`; `registered_uuids` returns `None` on macOS because
PhotoLab's database location there is unknown. A fresh `.dop` for an image
PhotoLab already registered still gets random Uuids on macOS and PhotoLab
still imports it as a virtual copy.

#### TODO

- [ ] Find PhotoLab's database path on macOS and extend
      `crates/app/src/photolab.rs`'s `database_path()` (or equivalent) to
      cover it.

### App: a busy PhotoLab database silently falls back to random Uuids

`crates/app/src/photolab.rs::lookup` opens PhotoLab's database read-only with
a 300 ms `busy_timeout`; if PhotoLab is mid-transaction past that window, the
lookup returns `None` and the fresh `.dop` mints random Uuids, which can still
produce a virtual copy on a registered image. The failure is logged at
`log::debug!`, but the writer does not retry the lookup for an
already-written sidecar.

#### TODO

- [ ] Decide whether to retry the lookup (and re-patch the sidecar) after a
      busy-database miss, or leave it as a rare, logged edge case.

### App: opening a subfolder while its parent folder's rename is in flight

Step 2's round 4 local reviewer noted, outside the reviewed diff, that
clicking a subfolder of the folder being renamed while `rename_folder` is
still in flight can open that subfolder under its old path.

#### TODO

- [ ] Make the tree refuse (or defer) opening a path under a folder whose
      rename has not returned yet, or reopen it under the rebased path once
      it returns.

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

### App: a pending rename waits silently with no visible pending state

#### Background

The inline rename edit (folder tree and strip) now always starts immediately,
even during a scan, and a confirmed name is held by `idle.ts`'s `IdleGate`
and runs once the scan ends; the cell keeps showing the old name and only the
status line names what is waiting. Whether the cell should show the pending
new name until the rename actually runs is an open UX question. Basis:
`wait-for-scan` Step 1. Files: `crates/app/ui/src/main.ts`,
`crates/app/ui/src/strip.ts`, `crates/app/ui/src/folders.ts`.

#### TODO

- [ ] Decide whether the folder tree / strip cell should show the pending
      new name while a rename waits for a scan to finish, and implement it
      if so.

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

### App: renaming a folder after a Move Rejected to Trash run leaves the trash run pointing at the old path

A folder renamed (tree `Rename…`) after a `Move Rejected to Trash` run leaves
the recorded run pointing at the old path, so undoing it restores into the
old, now missing, folder (on Windows the Recycle Bin may recreate the
folder). The frontend's rename handler rewrites judgment entries
(`mapJudgments`) but not a trash entry's `dirs`, and the backend's
`TrashRun` is not rebased either. Basis: `undo-trash-rejected` Step 3
implementation. Files: `crates/app/ui/src/main.ts` (rename handler),
`crates/app/src/rename.rs`, `crates/app/src/trash.rs` (`Runs`).

#### TODO

- [ ] Rebase a trash run's recorded paths (and the frontend's trash undo
      entry `dirs`) when the folder they point into is renamed, so undoing
      or redoing the run after a rename still targets the right folder.

### Docs: add a guide for GitHub Actions workflows and `gh` secrets

#### Background

Create `docs/agents/github-actions-workflows.md` (read before writing or
debugging a `.github/workflows/*.yml` that uses `gh`, secrets, or a
dedicated branch as a data store, like `stats`). No guide under
`docs/agents/` covers workflows or secrets, so the next workflow change
would rediscover these points from
`docs/plans/_archived/20260928-promotion-stats/learnings.md`.

#### TODO

- [ ] Create the guide covering: an empty secret is indistinguishable in
      `gh secret list` and shows as an empty `GH_TOKEN:` in the run log (set
      secrets on github.com, or check the value is non-empty before
      `gh secret set`); pushing `.github/workflows/*` needs the `workflow`
      scope (`gh auth refresh -h github.com -s workflow`); `shellcheck` /
      `actionlint` run ad hoc through `mise exec --`;
      `git worktree add --orphan` (git 2.42+, present on `ubuntu-latest`)
      plus `git diff --cached --quiet` for a commit-only-when-changed guard.
- [ ] Link the archived learnings.md above from the guide.

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
- [ ] On Windows, Step 5, viewer / meta pane / empty states: start with no folder open, then open an empty folder and a folder whose files are all filtered out; open a fresh folder and watch it scan; make a sidecar write fail (mark a sidecar read-only and rate the file) and dismiss the error; read the meta pane of a RAW file; toggle 1:1 zoom; open Compare with three frames. Expect the empty hints (`Drop a folder…`, `This folder has no RAW or JPEG files.`, `No files match the current filter.`) centered in `#a1a1a1`; the `scanning N / M` then `focus N / M` notes, `1:1` and `Compare · N frames` in `#a1a1a1`; the error row in `#ff6467` (not the old amber) with a ghost `×` that hovers on `#262626` and dismisses; the meta pane's EXIF / Maker note / Analysis headings and keys in `#a1a1a1`, values `#fafafa`, a thin scrollbar; Compare's label bars `#171717` with `#fafafa` text, frames with a faint hairline border, the best frame keeping its green bar, text and border, and the active frame showing a light `#e5e5e5` inner ring.
- [ ] On Windows, Step 6, settings side-nav (a debug build so Debug shows, a 1280x720 window): open Settings. Expect the modal wider and taller (`min(860px, 90vw)` x `min(720px, 80vh)`) and fitting; the left nav listing Sidecar, Culling, Keyboard Shortcuts, Cache, MCP and Debug, each with its 16px icon, `#a1a1a1` at rest, hovering on `#262626`, the active one bold `#fafafa` on `#262626`; all six items fit without the nav scrolling and "Keyboard Shortcuts" reads acceptably (one line preferred); the nav's right hairline is visible (if not, raise `.sidenav`'s border to `--input`); `ArrowUp` / `ArrowDown` move and activate with wrapping, `Home` / `End` go to the first / last, `ArrowLeft` / `ArrowRight` do nothing, the focused item shows the `#737373` ring, Tab moves into the content, Shift+Tab back, Escape closes; the shortcuts table scrolls inside the content pane (thin scrollbar) while the header and nav stay put, and a status error shows under the content pane.
- [ ] On macOS (WKWebView), repeat the six Windows passes above (Settings, first-run / trash / sequence dialogs, menus / strip bar / cells, folder tree, empty states / meta pane / status lines / Compare, settings side-nav). Also confirm that `scrollbar-color` is honored or harmlessly ignored, that native checkboxes / radios take `accent-color` (Safari 15.4+; an older engine shows the UA control), and that `:focus-visible` rings appear.
- [ ] On Linux (WebKitGTK), repeat the six Windows passes above and the same three extra confirmations (`scrollbar-color`, `accent-color` on checkboxes / radios with WebKitGTK 2.36+, `:focus-visible` rings).

### App: real-device check of the HDR PQ (HEIF) CR3 message in the strip, viewer, meta pane and sidecars

`mise run ci` and `riffle-cli candidates` confirmed the new HDR PQ (HEIF) CR3
message (unit tests in `cr3.rs` / `reader.rs` / `index.rs`, and a CLI run on
the two local samples), but the GUI itself was never exercised. See
`docs/plans/_archived/20260930-heif-cr3-message/plan.md` (Step 1). Files:
`crates/app/src/index.rs`, `crates/app/ui/src/strip.ts`,
`crates/app/ui/src/main.ts`.

#### TODO

- [ ] On Windows, open the folder containing `D:\photos\samples\CR3\R8.CR3`
      and `R5m2.CR3` (never committed) in the app and check that (1) the
      strip cells of the two files are the failed color and show "HDR PQ
      (HEIF) CR3: its HEVC preview is not supported yet" over the image box,
      with the same text as the cell's tooltip; (2) the viewer's note reads
      `<path>: HDR PQ (HEIF) CR3: its HEVC preview is not supported yet`;
      (3) the meta pane shows the EXIF rows (camera, lens, exposure, capture
      time); (4) a star, a flag and a color label each write a sidecar next
      to the file; (5) if either file was scanned before the
      `EXTRACTOR_VERSION` bump to 8, it re-extracts once and picks up the
      new text.

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

### App: `main.ts` / `trash.ts` path comparisons stay case-sensitive on case-insensitive filesystems

#### Background

The `todo-five-small-items` feature (Step 4) made `relation`, `rebase` and `renameFolder` in `crates/app/ui/src/tree.ts` take `ignoreCase`, and `folders.ts`'s `renamed` passes it. By decision (option (a) in the plan), these callers were left case-sensitive: `crates/app/ui/src/main.ts`'s `renameFolder`, which reopens the folder through `rebase(openDir, path, newPath)`, and `crates/app/ui/src/trash.ts`'s `restoredInto` / `opensTarget`, which call `relation`. On macOS / Windows, an `openDir` that differs in case from the tree key would then not be rebased (the open folder is not reopened after a rename) or matched. This is rare, since both spellings come from the same tree / `list_arw` listings. Plan: `docs/plans/_archived/20260929-todo-five-small-items/plan.md`.

Files: `crates/app/ui/src/main.ts`, `crates/app/ui/src/trash.ts`, `crates/app/ui/src/folders.ts` (the `ignoreCase` flag).

#### TODO

- [ ] Export the platform flag from `folders.ts` (or an equivalent) and pass it to `rebase(openDir, ...)` in `main.ts` and to the `relation` calls in `trash.ts`, with `tree.test.ts` / `trash` test cases for a differently cased path.
