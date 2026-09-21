# todo

## Cross-cutting / other

### App: unmeasured end-to-end per-page latency

End-to-end per-page latency (IPC + `createImageBitmap`) is unmeasured, since the GUI could not be driven from this development machine. Only the Rust-side file-read cost was measured; see "Per-page preview read" in docs/performance.md.

#### TODO

- [ ] Measure keypress-to-pixels per page turn on real hardware and add the numbers to "Per-page preview read" in docs/performance.md. The instrumentation now exists: with `Timing logs` on, the app logs a `page invoke=… decode=… total=… keypressToPixels=…` line per page turn to `Riffle.log`, and "Measuring on your own folder" in docs/performance.md spells out the procedure. Only running the measurement and filling in the numbers is left.

### Docs: the "End to end, keypress to pixels" section of docs/performance.md has a stale menu path

The "End to end, keypress to pixels" section of docs/performance.md still says `Debug > Timing logs`, but the toggle lives in the settings window (`crates/app/ui/settings.html`, `debug-timing`). Found while documenting page-latency-timing's Step 3; out of scope for that step.

#### TODO

- [ ] Update the "End to end, keypress to pixels" section in docs/performance.md to say the `Timing logs` toggle is in the settings window, matching the wording used in "Measuring on your own folder" and "Per-page preview read".

### App: a scan can be started twice after a cache clear / focus rescan

In the Windows real-folder measurement, after a cache clear `scan_id` N was superseded by N+1 with no `scan extract` line for N, and two focus rescans once fired at the same instant. This may be one bug or two. Files: `crates/app/src/commands.rs` (`scan_folder`), `crates/app/src/watch.rs`, the settings-window clear-cache path.

#### TODO

- [ ] Find why two scans start and make the second one not fire (or coalesce it), verified by the log showing one `scan extract` per trigger.

### App: `open entries` is called twice per folder open

In the Windows real-folder measurement, the log shows two `open entries` lines (56ms and 76ms on 2677 files) for one folder open.

#### TODO

- [ ] Find the second caller (frontend `crates/app/ui/src/main.ts` / backend `crates/app/src/commands.rs`) and remove the redundant call, or document why both are needed.

### App: a deep-row focus point still exceeds the 50ms budget

A focus point in a deep row of the unrotated JPEG measured 58-65ms keypress to pixels (n=2, before the #56 and #60 fixes); see "The 1:1 focus check path" in docs/performance.md. Options are prefetching the neighbouring files' crops (Phase 4's ring buffer) or a DCT-scaled placeholder; nothing is chosen.

#### TODO

- [ ] Retake the deep-row measurement post-fix, then decide between crop prefetch and a DCT-scaled placeholder.

### App: the silent update path is unverified end-to-end

The Updating paragraph in `docs/usage.md` describes a background download and install on launch and the "Check for Updates…" menu item, but nothing has confirmed on a real build that an installed copy detects a newer release, installs it silently, and launches as the new version next time.

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
of the menu item's visible behaviour has been run by a human. One point is
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
      the file with its judgement, and if no "Put Back" entry exists, that
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
- [ ] Verify the `Some`/`None` accelerator behaviour on Linux (only
      reasoned from muda 0.19.3's sources so far, never run; Windows passed).

### App: Open in DxO PhotoLab always fails on Windows

Found during the Windows manual GUI check run: `open_in_photolab` only reads
`/Applications` for `DXOPhotoLab<N>.app`, so on Windows "Open in DxO
PhotoLab" always fails with `Could not open PhotoLab: ... (os error 3)`
(path not found) while the menu item is still shown. Files:
`crates/app/src/commands.rs` (`open_in_photolab`), `crates/app/src/main.rs`.

#### TODO

- [ ] Decide between a Windows / Linux PhotoLab lookup and hiding or
      disabling the item on those platforms, then implement it.

### App: Clear Cache is refused on Windows with no scan running

Found during the Windows manual GUI check run: pressing Clear in the Clear
Cache confirm dialog, with no scan running beforehand, is refused with `a
scan is running; wait for it to finish`. Likely cause (unverified):
`crates/app/ui/src/main.ts` subscribes
`window.__TAURI__.event.listen("tauri://focus", resync)` globally, so the
settings window regaining focus when the dialog closes triggers a
main-window rescan, and `clear_index`'s second `scanning()` check (after the
sidecar flush) refuses. Proposed fix: scope the focus listener to the main
window. Files: `crates/app/ui/src/main.ts`, `crates/app/src/commands.rs`
(`clear_index`).

#### TODO

- [ ] Confirm the cause, fix it, and add a regression test if feasible.

### App: the Clear Cache button's manual GUI verification is still open

From `clear-cache-stuck-guard`'s implementation: the button's guard used to
get stuck (never re-enabling after the first scan), so none of its GUI
behaviour has been run by a human. GUI automation is unavailable on this
Mac and a native confirm dialog cannot be driven by an agent. A later manual
run on Windows passed (1) and showed the dialog in (2) immediately; the
Cancel half of (2) and checks (3)-(7) are blocked by the Clear Cache
refusal bug above. Files:
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
development machine, so several behaviours were never exercised by a human.

#### TODO

- [ ] Verify Step 1's burst bracket and `· N / M in burst` counter by hand on a
      real Sony and Leica burst folder and note the group sizes. Files:
      `crates/app/ui/src/burst.ts`, `crates/app/ui/style.css`.
- [ ] Exercise `Shift+x` reject-rest and its one-step undo by hand on a real
      burst folder. Files: `crates/app/ui/src/main.ts`.

### App: an old shortcut override for `previous`/`next` can silently conflict with new burst defaults

A stored `shortcuts` override that still binds `arrowleft` / `arrowright` to
`previous` / `next` (older defaults) now conflicts with the new burst
navigation defaults, and the whole override for that action is ignored at
load (found while implementing burst-grouping Step 2, where the
`a_store_from_the_old_defaults_still_loads` test had to drop `arrowleft`).

#### TODO

- [ ] Consider letting a stored override win over a default belonging to
      another action, so an old override does not silently disappear when a
      new action claims its key. Files: `crates/app/src/shortcuts.rs`.

### App: face/eye-aware focus check for culling

The sharpness score in `crates/core/src/sharpness.rs` (`score_preview`, `trusted_focus`, `tile_max`) measures a `WINDOW` around the Sony `FocusLocation` or, without one, the sharpest tile of the embedded preview, so a portrait focused on the background, the nose or the ear rather than the eye still scores high. The stored `files.sharpness` column in `crates/app/src/index.rs`, the strip's relative cue (`crates/app/ui/src/sharpness.ts`) and the burst group (`crates/app/ui/src/burst.ts`) inherit that blind spot. Idea: detect faces/eyes on the embedded preview at scan time with a lightweight detector (such as YuNet or BlazeFace via ONNX, e.g. the `ort` crate), store the region in the SQLite index, score sharpness on the eye/face region, and suggest the sharpest-eye frame within a burst group.

#### TODO

- [ ] First check whether Sony ARW and Leica DNG MakerNotes record face/eye-AF
      detection positions (Sony's MakerNote parsing lives in
      `crates/core/src/arw.rs`, which already reads `FocusLocation` and
      `FocusMode`); if they do, no inference is needed for those bodies.
- [ ] Prototype a lightweight detector and measure the per-image cost at scan
      time and the added bundle size (runtime plus model); note the numbers in
      `docs/performance.md` or the plan's learnings.
- [ ] Fall back to the current AF-point / tile scoring when no face is found
      (landscapes, animals), keeping the `files.sharpness` semantics for such
      files.
- [ ] Optionally, detect closed eyes from the landmarks.

Related: `crates/core/src/sharpness.rs`, `crates/core/src/arw.rs`, `crates/app/src/index.rs`, `crates/app/ui/src/sharpness.ts`, `crates/app/ui/src/burst.ts`.

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
