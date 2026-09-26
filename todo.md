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
`crates/app/ui/src/main.ts`.

#### TODO

- [ ] On macOS, verify: the confirmation names the
      right count (and the singular for one file) with `Move to Trash` /
      `Cancel`; Cancel leaves the folder untouched; confirming moves the RAW
      plus its `.xmp` and `.ARW.dop` to the Trash and the strip updates to the
      next passing file; the zero-reject, no-folder and mid-scan cases each
      write their message to `#status`; a "Put Back" from the Trash restores
      the file with its judgment, and if no "Put Back" entry exists, that
      dragging it out does.
- [ ] Verify the same flow on Windows and Linux (the `trash` crate's other
      backends have never been run here).

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
(v0.2.0) while clicking the Edit items worked. The new keys have not been
run by hand on any platform yet.

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
- [ ] On Windows (`mise run tauri:dev`), verify: one `Ctrl+Z` press undoes
      exactly once and one `Ctrl+Shift+Z` redoes exactly once; the Edit menu
      shows the accelerators; rebinding `undo`/`redo` updates the Edit menu
      label in place (no `menu item undo not found` warning in the log) and
      kills the old key; both Edit items work via mouse click. Files: same
      as above, plus `crates/app/src/commands.rs` (`update_keymap`).
- [ ] On the dev machine, verify Select All: pressing `Cmd/Ctrl+A` once
      with the strip focused selects every file (the current file stays
      current); pressing it with the folder tree focused does nothing to
      the strip; pressing it inside a settings text input selects that
      input's text. Check `Edit > Select All` shows the accelerator and
      that rebinding `selectAll` updates it, including in place on
      Windows. Files: same as above (`crates/app/src/main.rs`
      (`app_menu`), `crates/app/src/shortcuts.rs`,
      `crates/app/src/commands.rs`), plus `crates/app/ui/src/main.ts`,
      `crates/app/ui/src/settings.ts`.
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
`tauri://focus` listener).

#### TODO

- [ ] Run `mise run tauri:release:devtools` and check, in order: (1) after
      a folder's scan finishes, Settings > Cache shows the button enabled
      and the note hidden; (2) pressing Clear Cache shows the confirmation
      dialog immediately, and Cancel leaves the size and `#settings-status`
      unchanged with the button re-enabled; (3) Clear Cache then Clear
      keeps the button disabled until the size drops and the main window
      rescans, and never refuses with `a scan is running` (closing the
      confirmation refocuses `main`, whose focus listener skips `resync()`
      while the modal is open); (4) while a scan is running (or opening a large
      folder with the settings modal already open), the button is disabled
      and the note visible without any press, and both clear when the scan
      ends, and the size figure updates when the scan ends;
      (5) opening the settings modal during a large folder's prepare phase
      (before the first `scanning N / M` line) shows the button already
      disabled; (6) an error path, if reachable, writes the refusal to
      `#settings-status` and it stays until the next press; (7) note
      whether the very first press after opening the settings modal ever
      does nothing, and if so record the window focus state at that
      moment; (8) the native confirmation looks right over the settings
      modal on macOS and Windows.

### App: a file picked up mid-copy may be scanned from a partial read

The folder watcher (`crates/app/src/watch.rs`, `crates/app/src/commands.rs`) can fire a rescan while a file is still being copied into the open folder, extracting a partial preview and writing that size/mtime into the index; the copy's completion later fires another event and `reconcile` re-extracts. The plan accepted this as expected behavior but it was never observed either way.

#### TODO

- [ ] Verify by hand (copy a large ARW/DNG into an open, watched folder) whether a partial mid-copy read ever produces a visibly wrong thumbnail/rating before the follow-up event corrects it, and whether any guard is warranted.

### App: the manual GUI check of `scan-progress` `ready` is outstanding

Steps 2 and 3 of `docs/plans/_archived/20260921-scan-progress-ready-paths/plan.md`
each specified a check in `mise run tauri:dev` that no agent session could run,
since the GUI cannot be driven from one. Files: `crates/app/ui/src/strip.ts`,
`crates/app/ui/src/main.ts`, `crates/app/src/commands.rs`.

#### TODO

- [ ] On a folder with a cold index, confirm thumbnails fill in while the scan
      runs rather than only at `scan-done`, and that devtools shows no burst of
      `thumbnail` invokes per `scan-progress` beyond the newly ready cells.
      Note the observed payload sizes, which were reasoned rather than measured.
- [ ] On a large folder, confirm the un-throttled `scan-done` `refresh()` does
      not visibly starve the IPC channel: at most `MAX_IN_FLIGHT` invokes for
      the still-missing visible cells, issued once.

### App: burst grouping's manual checks are still open

From `burst-grouping`'s implementation: GUI automation is unavailable on this
development machine, so several behaviors were never exercised by a human.

#### TODO

- [ ] Verify the burst band and count badge by hand on a real Sony and Leica
      burst folder and note the group sizes: band visible, badge shows on
      the first displayed cell of a burst, the badge switches to
      `position/size` and follows the current selection, the gap above a
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

### App: an unreadable folder in the tree looks like an unexpanded one

A `list_subfolders` failure collapses the row again and reports through
`setStatus`; the row itself carries no error mark, so once the status line
changes, an unreadable folder is indistinguishable from one that was simply
never expanded.

#### TODO

- [ ] Mark the row itself on error. Files: `crates/app/ui/src/folders.ts`
      (`toggle`).

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

### App: the Lightroom 9.5.1 round-trip check is still open

From `lightroom-xmp-flags-labels`'s implementation: the tri-state flag and
color-label read/write for XMP were built and unit-tested against trimmed
copies of Lightroom-shaped fixtures, but the round-trip was never confirmed
against real Lightroom. Files: `crates/core/src/xmp.rs`,
`crates/app/src/sidecar.rs`, `README.md` ("Sidecar formats and software"
checklist).

#### TODO

- [ ] Open `D:\Photos\2026\2026-09-05` under the XMP format and check L1005439
      is picked, L1005438 is rejected, L1005428-L1005432 show purple / blue /
      green / yellow / red, and L1005433-L1005437 show 5..1 stars.

### App/Core: unverified whether a Riffle-written pick/reject shows correctly in Lightroom Classic

Lightroom Classic writes a pick as `xmpDM:good="true"` + `xmpDM:pick="1"`;
Riffle only reads and writes `xmpDM:good`. The user confirmed Lightroom
Classic 2026 read Riffle-written pick flags (`xmpDM:good` only, no
`xmpDM:pick`) correctly on first import, but reject display from a
Riffle-written XMP was not separately checked. Basis: `lightroom-label-names`
plan's "Trade-offs and risks" and the user's session verification. Files:
`crates/core/src/xmp.rs`, `README.md`.

#### TODO

- [ ] Verify a Riffle-written reject (no `xmpDM:pick`) shows correctly as
      rejected in Lightroom Classic; if it does not, decide whether Riffle
      should also write `xmpDM:pick`.

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

### App: decide whether `scan-state` needs a frontend consumer

From `settings-modal` Step 2: the settings modal was the only frontend
listener of `scan-state`, and it now reads `main.ts`'s `scanRunning`
instead, which follows `scan-done`. The backend still emits `scan-state`
under the `Scans` lock, per the rule in `docs/agents/tauri-app.md`, but
nothing listens. Files: `crates/app/src/commands.rs` (the `scan-state`
emits), `docs/agents/tauri-app.md`.

#### TODO

- [ ] Either remove the `scan-state` emits and the doc rule that covers
      them, or give the event a consumer (for instance, have `main.ts`
      follow it instead of deriving `scanRunning` itself). Done when no
      event is emitted without a listener, and `mise run ci` passes.

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
checked in a running app on Linux (WebKitGTK), Windows, or macOS. Files:
`crates/app/ui/src/settings.ts`.

#### TODO

- [ ] Verify the settings window's Copy buttons in the Linux (WebKitGTK) and
      Windows / macOS webviews, and confirm the select-text fallback fires
      when the write is refused.

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

### Core: `en.json`'s Lightroom label preset is unverified against a real Lightroom install

`crates/core/i18n/en.json`'s `lightroom.colorLabels.verified` is
`"Lightroom Classic (English UI)"`, with no version or OS, because the English
names were not checked against a specific install when the file was created
(they are the long-standing `LabelNames::default()`). Basis:
`label-presets-i18n` plan's Step 1 learnings. Files: `crates/core/i18n/en.json`.

#### TODO

- [ ] Confirm the English color label names against a real Lightroom Classic
      install and name the version / OS in `verified`.

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

#### TODO

- [ ] Investigate why the cold `folder_entries` read costs 321 ms on 2134 rows and whether it can be reduced (indexing, query shape, or caching), verified by a measurement with `Timing logs` on before/after.

### App: `File > Sequence JPEG Timestamps…` has no macOS menu icon

Its File-menu neighbours get an icon on macOS, but `tools/macos/export-menu-icons.swift`
cannot run on Windows, so the item was added as a plain `MenuItem` on every
platform. Basis: Step 2 of `docs/plans/_archived/20260926-sequence-jpeg-timestamps/plan.md`
("icon on macOS if the neighbours have one").

#### TODO

- [ ] Add an SF Symbol (e.g. `clock.arrow.circlepath`) to
      `tools/macos/export-menu-icons.swift`, render it on macOS into
      `crates/app/icons/menu/`, and switch the item in
      `crates/app/src/main.rs` to the `IconMenuItem` / `MenuItem` `cfg` split.

### App: open and preview JPEG-only folders in the strip

The strip lists ARW / DNG only, and scan, index, sidecars and the focus cue
all assume RAW. This is needed to let a user check a
`sequence-jpeg-timestamps` `<folder>-sequenced/` output (order and times)
inside Riffle. Needs its own plan: decide which features apply to JPEGs.
Basis: requested by the user during `docs/plans/_archived/20260926-sequence-jpeg-timestamps/plan.md`'s
planning and deferred as out of scope (plan.md "Follow-ups").

#### TODO

- [ ] Decide which existing RAW-oriented features (thumbnails, metadata,
      sidecars, focus cue) apply to a JPEG-only folder, and let such a
      folder open in the strip with thumbnails and the preview.

### App: hand-check the sequence-run output-folder reveal on Windows

The automated tests cover only the `revealAfter` decision in
`crates/app/ui/src/sequence.ts`, not the actual `reveal_folder` opener call
added to `finishSequence` in `crates/app/ui/src/main.ts`. Basis:
`docs/plans/_archived/20260927-sequence-reveal-output/learnings.md`.

#### TODO

- [ ] Confirm on Windows that (1) a run that wrote files opens Explorer with
      `<folder>-sequenced` selected, (2) a cancelled run opens nothing, and
      (3) a folder with no JPEGs opens nothing; merge any fix needed.
