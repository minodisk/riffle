# todo

## Cross-cutting / other

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

### Docs: the "End to end, keypress to pixels" section of docs/performance.md has a stale menu path

The "End to end, keypress to pixels" section of docs/performance.md still says `Debug > Timing logs`, but the toggle lives in the settings window (`crates/app/ui/settings.html`, `debug-timing`). Found while documenting page-latency-timing's Step 3; out of scope for that step.

#### TODO

- [ ] Update the "End to end, keypress to pixels" section in docs/performance.md to say the `Timing logs` toggle is in the settings window, matching the wording used in "Measuring on your own folder" and "Per-page preview read".

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
- [ ] Verify the `Some`/`None` accelerator behavior on Linux (only
      reasoned from muda 0.19.3's sources so far, never run; Windows passed).

### App: the Clear Cache button's manual GUI verification is still open

From `clear-cache-stuck-guard`'s implementation: the button's guard used to
get stuck (never re-enabling after the first scan), so none of its GUI
behavior has been run by a human. GUI automation is unavailable on this
Mac and a native confirm dialog cannot be driven by an agent. A later manual
run on Windows passed (1) and showed the dialog in (2) immediately; the
Cancel half of (2) and checks (3)-(7) were blocked by a Clear Cache refusal
bug, since fixed (`focus-rescan-main-window`), and are still to be run. Files:
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
      note visible without any press, and both clear when the scan ends,
      and the size figure updates when the scan ends;
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

#### TODO

- [ ] Consider letting a stored override win over a default belonging to
      another action, so an old override does not silently disappear when a
      new action claims its key. Files: `crates/app/src/shortcuts.rs`.

### App: face/eye-aware focus check for culling

Face/eye-aware detection and scoring (`crates/core/src/faces.rs`, `crates/core/src/sharpness.rs`) now runs at scan time: sharpness is scored on the eyes when a face is found and the AF point is missing or off the face, else on the AF point, else on the sharpest tile (see `docs/plans/_archived/20260922-face-aware-sharpness/`). What remains:

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
- [ ] Store the face region in the SQLite index and suggest the sharpest-eye
      frame within a burst group.
- [ ] Spot-check whether the sharpness ranking within a burst changes now
      that Sony frames with face tracking are scored on the camera's AF frame
      instead of YuNet's eye midpoint. Needs a per-file score output
      (`riffle-cli scan` prints none). Files: `crates/core/src/sharpness.rs`,
      `crates/core/src/scan.rs`, `crates/cli/src/main.rs`.

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

### App: SIGMA fp L strip may decode the full-size JPEG per thumbnail

SIGMA fp L DNGs have no strip JPEG between 640x480 and the 9520x6328
full-size one, so the preview tier falls back to the full-size JPEG (about
28 MB) and the strip decodes it per thumbnail. Worth checking strip load
time on SIGMA fp L folders. Files: `crates/core/src/arw.rs`
(`PREVIEW_MIN_WIDTH`, tier selection).

#### TODO

- [ ] Measure strip scroll/thumbnail load time on a SIGMA fp L folder; if
      slow, consider downscaling the full-size JPEG for the preview tier
      instead of using it as-is.

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
strip may decode the full-size JPEG per thumbnail" (strip side), and the
Linux/SIGMA-specific case of "App: unmeasured end-to-end per-page latency".
Files: `crates/app/ui/src/worker.ts` (resize on decode),
`crates/app/src/commands.rs` (`PREVIEW_PIXEL_LIMIT`).

#### TODO

- [ ] Measure first on the real app on Linux with `Timing logs` on: the
      `page invoke=… decode=… total=… keypressToPixels=…` line splits the
      IPC (invoke) from the decode.
- [ ] Candidate fixes, none decided: prefetch and decode the neighbouring
      pages ahead; or have the backend downscale and cache a mid-size JPEG
      for files lacking one, the fix already noted in the SIGMA fp L strip
      item, which would serve both the strip and the main view.

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
