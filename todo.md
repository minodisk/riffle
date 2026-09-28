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

### Docs: record the practice of verifying platform-workaround thresholds on the production code path

`fix(app): lower the Linux preview pixel limit to 6 MP` was a follow-up to
#383, whose original 12 MP limit was based on a secondhand claim ("12.3 MP
drew, 16 MP did not") never measured on the shipped code path; the user's
manual check still showed a blank main view. A MiniBrowser bisect (worker
`createImageBitmap` with resize options, the bitmap transferred, `drawImage`
to a canvas, the center pixel read, results POSTed to a local Python HTTP
server) found the real threshold was ~6.87 MP by pixel count, shape-independent.

#### TODO

- [ ] Decide whether this belongs as a note in the `docs/agents/tauri-app.md`
      WebKitGTK "draws a large transferred `ImageBitmap` transparent" Hit item
      (with the MiniBrowser + POST-logging harness as the method), or as a
      broader practice elsewhere, and write it into the chosen guide: verify a
      platform workaround's numeric threshold on the exact production code
      path before shipping it, instead of taking a quoted/secondhand number.

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

### Docs: write a guide for RAW metadata parsing (`docs/agents/raw-metadata-parsing.md`)

Leica DNG support found several non-obvious facts in `crates/core/src/{arw,reader}.rs`'s MakerNote/TIFF parsing that cost time in Steps 1 and 3: the Sony gate skips only when the note lacks `SONY` *and* `Make` is present and non-Sony, so non-Sony test fixtures must set `Make`; a `SubIFDs` entry with `count == 1` stores the IFD offset inline, not an offset to an array; the Leica MakerNote is `LEICA\0` + `02 00` then a little-endian IFD at note offset 8, with `FocusDistance` at tag 0x0304 (LONG, millimeters); `ApertureValue` (APEX) converts via `2^(AV/2)`; and a "non-Sony note is skipped" test needs a valid empty IFD in the fixture, not a `0xffff` sentinel count. Sony's α7 V MakerNote also stores `FocusFrameSize` (tag 0x2037) as `UNDEFINED[6]` (type 7, count 6), not `SHORT[3]` — exiftool only reinterprets it as `int16u[3]`. A reader that only accepts `SHORT[3]` returns `None` on real files (verified on `_DSC3590.ARW`); the parser now accepts both encodings.

#### TODO

- [ ] When the next feature touches the MakerNote/TIFF parsing in `crates/core/src/{arw,reader}.rs` (another maker's MakerNote, or a new synthetic-TIFF fixture), create `docs/agents/raw-metadata-parsing.md` capturing the points above, linking `docs/plans/_archived/20260918-leica-dng-support/learnings.md` for the underlying measurements instead of duplicating them.
- [ ] Also cover the `FocusFrameSize` `UNDEFINED[6]`-vs-`SHORT[3]` quirk, linking `docs/plans/_archived/20260922-sony-eye-af-window/learnings.md` (Step 1) alongside the Leica one.
- [ ] Also cover the count-1 `SHORT` TIFF-entry padding quirk (the unused high 16 bits of the 4-byte value field can carry nonzero per-file garbage, as seen on SIGMA fp L DNGs) and that `integer()` in `crates/core/src/arw.rs` now masks it, linking `docs/plans/_archived/20260924-tiff-short-padding/learnings.md` (Step 1) alongside the Leica and Sony eye-AF ones.
- [ ] Also cover the Sigma MakerNote conventions found while adding the Sigma BF AF point: a MakerNote entry whose `count` fits inside the entry (`<= 4` for a `SHORT[2]`, generally `<= header_len`) stores its value inline rather than as an offset into `buf`, so the offset/range check must come after that case; and `Make` differs by body within one vendor (Sigma BF writes `Sigma`/`Sigma BF`, Sigma fp L writes `SIGMA`/`SIGMA fp L`), so a vendor gate needs a case-insensitive prefix on `Make` plus an exact match on `Model`. Link `docs/plans/_archived/20260924-sigma-bf-af-point/learnings.md` (Step 1) alongside the Leica, Sony eye-AF, and TIFF-short-padding ones.

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
of the menu item's visible behavior has been run by a human. One point is
specifically unverified: whether the Trash's "Put Back" entry is actually
created — the command pins
`DeleteMethod::NsFileManager` to avoid the Finder route's Automation
permission, and the `trash` crate documents that on some macOS systems files
moved that way get no "Put Back" entry (trash-rs#14); dragging them out of the
Trash still restores them. (The menu item's icon is now a bundled template
PNG, confirmed tinting with the menu appearance — see `menu-icon-template`'s
learnings.) Files: `crates/app/src/main.rs` (`app_menu`),
`crates/app/src/commands.rs` (`trash_rejected`, `trash_context`),
`crates/app/src/trash.rs`, `crates/app/ui/src/trash.ts`,
`crates/app/ui/src/main.ts`. A manual run on Windows on 2026-09-28
(`mise run tauri:dev`) passed: the confirmation shows the count and the
button labels, Cancel changes nothing, confirming moves the RAW, its `.xmp`
and `.ARW.dop` to the Recycle Bin with the right original location, the
strip moves to the next file, restoring all three shows the file still as a
reject, and a folder with 0 rejects gets a message.

#### TODO

- [ ] On macOS, verify: the confirmation names the
      right count (and the singular for one file) with `Move to Trash` /
      `Cancel`; Cancel leaves the folder untouched; confirming moves the RAW
      plus its `.xmp` and `.ARW.dop` to the Trash and the strip updates to the
      next passing file; the zero-reject and no-folder cases each write their
      message to `#status`, and a press during a scan is held until the scan
      ends (`crates/app/ui/src/idle.ts`); a "Put Back" from the Trash restores
      the file with its judgment, and if no "Put Back" entry exists, that
      dragging it out does.
- [ ] Verify the same flow on Linux (the `trash` crate's freedesktop backend
      has never been run here).

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

### App: an old shortcut override for `previous`/`next` can silently conflict with new burst defaults

A stored `shortcuts` override that still binds `arrowleft` / `arrowright` to
`previous` / `next` (older defaults) now conflicts with the new burst
navigation defaults, and the whole override for that action is ignored at
load (found while implementing burst-grouping Step 2, where the
`a_store_from_the_old_defaults_still_loads` test had to drop `arrowleft`).
It recurred when lightroom-layout Step 1 rotated the defaults: an override
that binds `arrowup` / `arrowdown` to `previous` / `next` now collides with
the burst actions' new defaults, and every other key in that override (e.g.
`w` / `a` / `h` / `k`) is dropped with it.

#### TODO

- [ ] Consider letting a stored override win over a default belonging to
      another action, so an old override does not silently disappear when a
      new action claims its key. Files: `crates/app/src/shortcuts.rs`.

### App: a horizontal strip's scrollbar can grow `#film` and shrink the viewer without a resize event

With classic (non-overlay) scrollbars, e.g. on Windows or macOS set to
"always show scroll bars", `#strip`'s horizontal scrollbar appears once the
files outgrow the width and makes `#film` taller, shrinking `#viewer`
without a window `resize` event, so the canvas is not redrawn until the
next `draw()`. `scrollbar-gutter: stable` does not cover the block axis.

#### TODO

- [ ] Consider `overflow-x: scroll`, a fixed `#strip` height, or a
      `ResizeObserver` on `#viewer`. Files: `crates/app/ui/style.css`
      (`#strip`), `crates/app/ui/src/main.ts` (the `resize` handler).

### App: the folder tree does not reveal a differently-cased open path

A folder opened with a path whose case differs from the listing's (possible
on macOS / Windows, e.g. a typed or dropped path) is not revealed past the
first mismatching level, since `tree.ts` compares paths case-sensitively
apart from the drive letter.

#### TODO

- [ ] Match children case-insensitively on case-insensitive platforms.
      Files: `crates/app/ui/src/tree.ts` (`ancestorsWithin`),
      `crates/app/ui/src/folders.ts` (`reveal`).

### App: the folder tree's roots don't pick up a volume mounted after launch

`folder_roots` is invoked once at launch (`loadRoots`) and never again, so a
card or drive mounted afterward (a new `/Volumes/*`, `/media/*/*` or drive
letter) does not appear in the tree until the app restarts.

#### TODO

- [ ] Re-invoke `folder_roots` and `addRoots` on window focus or on each
      `reveal`. Files: `crates/app/ui/src/folders.ts` (`loadRoots`),
      `docs/usage.md`.

### App: a folder reachable from two tree roots is expanded and highlighted twice

A folder reachable from two roots (e.g. on Windows, the home root
`C:\Users\me` and `C:\` > `Users` > `me`, since folder listing keeps both; or
any root under `/` that `rootOf` adds) shares one `TreeNode`, keyed by path
alone. Expanding either row expands both, and the subtree and the `.current`
highlight are drawn twice.

#### TODO

- [ ] Key `expanded` (and the node map itself) by the row's root-plus-path,
      or stop listing a root's own path as a child of another root. Files:
      `crates/app/ui/src/tree.ts` (`TreeNode`, `Tree.nodes`).

### App: Windows real-device check of the merged folder-tree, scan-wait and strip-scroll work

Merged since the base of the 2026-09-28 Windows GUI check run and not run by
hand on any platform: `tree-live-watch` (#517, #520, #523),
`trash-rejected-from-tree` (#518, #521, #526, #529, #530), `wait-for-scan`
(#519, #522, #525) and `strip-keep-scroll-on-rescan` (#528). The last one
fixes the scroll jump-back and flicker seen on Windows on 2026-09-28 while
copying 100 ARWs into an open folder. Files: `crates/app/src/treewatch.rs`,
`crates/app/src/trash.rs`, `crates/app/ui/src/folders.ts`,
`crates/app/ui/src/idle.ts`, `crates/app/ui/src/strip.ts`,
`crates/app/ui/src/main.ts`.

#### TODO

- [ ] Tree watch on Windows: with a folder expanded, create, delete and
      rename a subfolder in Explorer and confirm the tree follows; confirm
      renaming a folder from the app still works with the watch on.
- [ ] Tree trash on Windows: right-click a folder, a multi-selection, and a
      folder with subfolders, then Move Rejected to Trash; confirm the
      dialog shows per-folder counts and the space freed, Cancel changes
      nothing, and confirming moves the rejects of every listed folder.
- [ ] Scan-wait on Windows: press Move Rejected to Trash and a rename while
      a large folder scans; confirm the status line says what is waiting and
      each runs when the scan ends. The Clear Cache hold is covered by
      checks (4) and (5) of "the Clear Cache button's manual GUI
      verification is still open".
- [ ] Strip scroll on Windows: repeat the 100-ARW copy into an open folder
      and confirm the strip's scroll position holds without flicker.

### App: Windows real-device check of the in-flight resume-selection and undo-trash work

Not on `main` yet: `resume-selection` (landing on a folder's remembered
file left the first file in the selection too, showing `2 selected`) and
`undo-trash-rejected` (Undo of a Move Rejected to Trash). Files:
`crates/app/ui/src/resume.ts`, `crates/app/ui/src/main.ts`,
`crates/app/src/trash.rs`.

#### TODO

- [ ] Once `resume-selection` has merged, reopen a folder with a remembered
      file on Windows and confirm only that file is selected.
- [ ] Once `undo-trash-rejected` has merged, trash the rejects on Windows,
      then Undo, and confirm the files come back with their judgment.

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

### App: every main-window focus runs resync() and holds scanRunning until faces-done

The `tauri://focus` listener in `crates/app/ui/src/main.ts` calls `resync()`
on every focus; even with 0 files to process, `scanRunning` stays true until
`faces-done`. `wait-for-scan` removed the refusals this caused, but the
rescan itself is still unthrottled. Seen on Windows on 2026-09-28.

#### TODO

- [ ] Throttle the focus rescan, and clear `scanRunning` immediately when
      the scan has 0 files to process. Files: `crates/app/ui/src/main.ts`,
      `crates/app/src/commands.rs` (`scan_folder`).

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

### Docs: write a guide for tract-onnx inference (`docs/agents/tract-onnx-inference.md`)

`crates/core/src/faces.rs` (face-aware-sharpness feature) worked out several tract 0.23 / ONNX pitfalls that no existing guide covers: `with_ignore_value_info` / `with_ignore_output_shapes` for models whose fixed-size shape annotations don't match a different input size, finding outputs by outlet label (`model.outlet_label`) rather than ONNX node name, the `Arc<TypedRunnableModel>` / `try_as_plain_ram` API, trimming with `default-features = false`, feeding an upright input and mapping detections back to stored (possibly rotated) coordinates, and sharing a built model across rayon workers via `OnceLock`.

#### TODO

- [ ] Write `docs/agents/tract-onnx-inference.md` covering the points above.
- [ ] Link it from `CLAUDE.md` or `docs/agents/`.

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
decodes the ~60 MP JPEG (at 2/8 scale) once per file. Files:
`crates/core/src/decode.rs` (`thumbnail_jpeg`), `crates/core/src/scan.rs`,
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

### Core: widen camera support from public sample RAW files

The user no longer owns the Sigma fp L or BF and cannot shoot new
samples, so public sample files are the verification source for more
cameras. Two kinds exist: raw.pixls.us (CC0; a few files per camera,
often flat test scenes that are weak for AF-point checks; CC0 means a
small file could be committed as a fixture, but tests prefer synthetic
bytes as in `crates/core/src/arw.rs`) and review-site sample galleries
(real scenes, good for AF checks, but not redistributable, so local
verification only). Riffle reads only ARW and DNG today (README.md
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
      Riffle can already read.
- [ ] Tier 3: decide per container (CR3, NEF, RAF, ...) whether a new
      parser is worth it, given the samples available.
- [ ] Check the Sigma BF AF point's open assumptions against public BF
      samples: portrait orientation, manual-focus behavior, and the
      1000x667 scale (see the `SIGMA_BF_AF_GRID_W` doc comment in
      `crates/core/src/arw.rs` and the archived plan's
      [Trade-offs and risks](docs/plans/_archived/20260924-sigma-bf-af-point/plan.md#trade-offs-and-risks)).

### Docs: docs/usage.md still names cameras in the Focus mark, Sharpness cue and Bursts bullets

`docs/usage.md`'s Focus mark, Sharpness cue and Bursts bullets still name
specific cameras (Sony, SIGMA BF, M11-P, Leica), unlike the now camera-neutral
README bullets. The camera-differences-doc plan kept this on purpose (only
adding links to `docs/cameras.md` there), but if the READMEs'
camera-neutral wording should extend to this detailed doc, reword those
bullets too.

#### TODO

- [ ] Reword the Focus mark, Sharpness cue and Bursts bullets in
      `docs/usage.md` to describe behavior by what the camera records rather
      than by camera name, matching the README's approach.

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

### App: `a_dirty_row_is_written_to_both_sidecars_after_a_switch_to_both` is flaky on Windows

The post-merge CI run for #492 (a docs-only change,
https://github.com/minodisk/riffle/actions/runs/36330791050) failed on
`test (windows-latest)` with this test panicking at
`crates/app/src/commands.rs:3869` on `Os { code: 2, kind: NotFound }`
("The system cannot find the file specified."). The same test passed on the
PR's own CI and on a rerun of the failed job, so it is intermittent. The
cause has not been investigated; one guess is the test reading a sidecar
before the coalescing writer thread has written it. Files: `crates/app/src/commands.rs` (the test),
`crates/app/src/sidecar.rs`.

#### TODO

- [ ] Find what the test reads at `commands.rs:3869` before it exists on
      Windows and make the test wait for the sidecar writer deterministically
      instead of racing it.

### App: the folder listing payload carries always-null Maker note fields

The folder listing's `Exif` struct is also serialized for every `IndexedFile`
(`index.rs` rebuilds it from cached columns with `..Shot::default()`), so
`focus_mode`, `af_tracking`, `af_area`, `drive`, `stabilization`,
`exposure_mode`, `metering`, `creative_style`, `dro` and `raw_type` travel
there as `null` for every file, since the index does not cache them.

#### TODO

- [ ] Move the Maker note formatting out of `Exif` into a separate struct used
      only by `read_metadata`, or skip serializing `None` fields. Files:
      `crates/app/src/exif.rs`, `crates/app/src/index.rs`,
      `crates/app/ui/src/exif.ts`.

### App: the backend `folder_entries` read is the main cost of the remaining `refreshEntries`

Now that the `scan-done` refresh is skipped when neither the scan nor the reconcile changed anything (docs/plans/_archived/20260926-scan-done-refresh-skip/plan.md), the one refresh that still runs (the open-time one) is dominated by the backend `folder_entries` read: 321 ms cold on 2134 rows in the user's Windows debug-build measurement. Files: `crates/app/src/index.rs` (`folder_entries` / `AppIndexReader`), `crates/app/ui/src/main.ts` (`refreshEntries`).

docs/plans/_archived/20260928-strip-keep-scroll-on-rescan/plan.md made the frontend `set_files=true` share of the `refresh entries:` timing line cheaper (no cell teardown and reload on a kept-scroll update), but this item is about the backend read, which is untouched.

#### TODO

- [ ] Investigate why the cold `folder_entries` read costs 321 ms on 2134 rows and whether it can be reduced (indexing, query shape, or caching), verified by a measurement with `Timing logs` on before/after.

### App: `crates/app/src/sequence.rs`'s comment still describes a picker-based flow

The header comment says the sequenced output goes to "a sibling of the
picked folder", but since `docs/plans/_archived/20260928-file-menu-folder-items/plan.md`
removed the File-menu picker path, the folder is now always the one
right-clicked in the tree. Left as-is when the plan closed to keep the
change to the header line the plan named.

#### TODO

- [ ] Update the comment in `crates/app/src/sequence.rs` to describe the
      tree-right-click flow instead of the removed picker.

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

### App: a rename can carry away an `.xmp` shared by two RAWs of the same stem

From Step 3's `file_plan` (`crates/app/src/rename.rs`), which, like
`trash::plan` (`crates/app/src/trash.rs`), takes `a.xmp` as `a.ARW`'s
sidecar even when an `a.DNG` in the same folder uses the same `a.xmp`.
Renaming `a.ARW` carries the DNG's sidecar away.

#### TODO

- [ ] Make a rename (and the trash) leave a `.xmp` another RAW of the same
      stem still uses, or refuse with a message.

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

`wait-for-scan` made `File > Move Rejected to Trash…`, the folder / file
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
      `File > Move Rejected to Trash…`: the status line briefly shows
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

### Agents: Bash-tool heredocs on Windows mangle doubled backslashes

During `trash-rejected-from-tree` Step 4, text holding a doubled backslash
(a Windows path, a verbatim `\\?\` prefix) written through a Bash-tool
heredoc landed with the backslashes halved, even with a quoted delimiter.

#### TODO

- [ ] Add a rule (in `CLAUDE.md` or an agent-tooling note) telling agents
      on Windows to write text holding backslashes (Windows paths,
      verbatim `\\?\` prefixes, regex escapes) with the Write / Edit
      tools, or a script file written by them, rather than a Bash
      heredoc. Confirm the behavior first by writing a doubled-backslash
      string both ways and diffing the results.
