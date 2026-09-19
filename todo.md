# todo

## Tooling / CI

### Tooling: no Markdown link checker in CI

`README.md`'s anchor link to `#running-the-app` is not checked by anything; `mise run ci` has no Markdown link checker.

#### TODO

- [ ] Add a Markdown link checker to `mise run ci` (or otherwise verify README anchors) covering `mise.toml` and `README.md`.

### Tooling: `create-pr.sh` hard-codes a label that doesn't exist in this repo

`.claude/skills/pr/scripts/create-pr.sh` hard-codes `--label ai-coauthored`. That label does not exist in this repository (it's a convention carried over from the repo these skills were ported from), so the first PR creation in a fresh clone fails with `could not add label: 'ai-coauthored' not found` until something creates the label and retries.

#### TODO

- [ ] Either create the `ai-coauthored` label deliberately as part of repo setup, or remove the hard-coded `--label` flag from `.claude/skills/pr/scripts/create-pr.sh`.

### Tooling: `merger`'s branch cleanup can delete the next step's freshly cut branch

A duplicate `MERGED` notification for a step can arrive after the next step's
branch has already been cut. `tools/git/delete_merged_branches.sh`, re-run on the
second arrival, then deletes that next branch out from under a running
`implementer` (seen during `ratings-xmp-sidecars` Step 4; the commit survived on
a detached HEAD and the branch was recreated by hand). The caller worked around
it from Step 4 on by telling `merger` to skip cleanup for step branches within a
plan.

#### TODO

- [ ] De-duplicate the repeated `MERGED` notification, or make
  `delete_merged_branches.sh` idempotent against a repeat, or have
  `.claude/skills/develop/SKILL.md`'s merge section default to skipping cleanup
  for in-plan step branches.

## Cross-cutting / other

### App: a shortcut override skipped under the current format is dropped on the next rebind

`update_keymap` (`crates/app/src/commands.rs`) saves `Keymap::overrides()` of
the resolved keymap, so an override skipped because it collides with the
current sidecar format's default (e.g. `reject: ["6"]` under XMP, which
collides with the `red` label default) is dropped from the stored `shortcuts`
value the next time the user rebinds anything, even though it would apply
under `.dop`.

#### TODO

- [ ] Persist an override that is only inactive under the current format,
      rather than round-tripping through the resolved keymap.

### App: unmeasured end-to-end per-page latency

End-to-end per-page latency (IPC + `createImageBitmap`) is unmeasured, since the GUI could not be driven from this development machine. Only the Rust-side file-read cost was measured; see the Phase 4 baseline table in the README.

#### TODO

- [ ] Add a timing readout in the status line, or a Rust-side benchmark that includes the IPC hop, before Phase 4 tuning work begins.

### App: `riffle-cli info`/`focusbox`/`bench` read whole files instead of the bounded prefix

`info` / `focusbox` / `bench` in `crates/cli/src/main.rs` still call `std::fs::read(path)` for the whole file rather than `reader::read_head`'s bounded prefix, unlike the scan path and unlike `crop` (which moved to `reader::read_full`, a ranged read, in Phase 5 Step 1). This may be inherent: these subcommands need the full-size `JpgFromRaw`, which sits past the 1 MiB prefix.

#### TODO

- [ ] Decide whether these subcommands can move to `reader::read_head`/`reader::read_full` with a whole-file fallback, or whether they inherently need the whole file and this is not worth changing.

### App: `focus_crop`'s header has no full-JPEG size, so the zoom placeholder's scale is approximate

The `focus_crop` payload header (`crates/app/src/commands.rs`, `crop_payload`) has a reserved 4-byte word but does not carry the full JPEG's width/height. The frontend's placeholder scale in `drawZoom()` (`crates/app/ui/src/main.ts`) falls back to the index row's `FocusLocation` sensor width, which is exact only when the JpgFromRaw size equals the sensor size.

#### TODO

- [ ] Carry the full JPEG width/height in the `focus_crop` header's reserved word and use it in `drawZoom()` instead of the sensor-width approximation.

### App: no rescan when new files appear in an already-open folder

`scan_folder` in `crates/app/src/commands.rs` reconciles the index only when a folder is opened, so a file added to an already-open folder is not picked up until the folder is reopened.

#### TODO

- [ ] Add a folder watcher, or a rescan on window refocus, to catch new files without a reopen.

### App: the SQLite index is never pruned or `VACUUM`ed

The database in `crates/app/src/index.rs` never evicts rows for folders that are not reopened, and deleting rows does not shrink the file without `VACUUM`. Measured at ~20.8KB per row (104,177,664 bytes for 5000 rows), so it grows without bound.

#### TODO

- [ ] Add a size cap or LRU eviction for the index, with a `VACUUM` step.

### App: the filmstrip re-requests every visible placeholder on each `scan-progress` event

`refresh` in `crates/app/ui/src/strip.ts`, driven from the `scan-progress` handler at ~10/s, re-requests every visible placeholder rather than only the indices the scan has newly passed. It is bounded by the 4-in-flight cap and the visible range, but it is avoidable IPC. Relatedly, the strip discovers whether a file has a thumbnail by invoking `thumbnail` and treating an `Err` as "not yet", rather than reading `has_thumb` from the `folder_entries` map, because keeping that map fresh during a scan would mean the same 10/s full re-read.

#### TODO

- [ ] Use the `done` counter or per-file `has_thumb` state to request only newly available thumbnails, if the strip turns out to be IPC-bound.

### App: filmstrip cell geometry assumes 3:2 thumbnails

`.cell img` in `crates/app/ui/style.css` is a fixed 144x96 box, matching the current 404x270 pipeline. A body with a differently shaped IFD0 preview would letterbox harmlessly (`object-fit: contain`) but waste cell space.

#### TODO

- [ ] Revisit if a camera body with a non-3:2 preview turns up.

### App: a multi-file drop is rejected wholesale, and drag-hover gives no early feedback

The `tauri://drag-drop` handler in `crates/app/ui/src/main.ts` rejects a multi-item drop outright, even when every item shares one parent folder. Separately, the `body.dragging` overlay in `crates/app/ui/style.css` looks the same whether or not the payload will be accepted, although Tauri's `drag-enter` event already carries the paths.

#### TODO

- [ ] Take the common parent folder of a multi-file drop instead of rejecting it.
- [ ] Indicate during drag-hover whether the drop will be accepted.

### App: paging keys follow `event.key`, not the physical layout

The paging key handler in `crates/app/ui/src/main.ts` matches on `event.key`, so on a non-QWERTY layout (Dvorak, AZERTY) WASD and HJKL land on scattered physical keys.

#### TODO

- [ ] Revisit only if a user asks; a fix would be an `event.code` fallback or a key-config layer.

### App: real-folder scan and second-open numbers are still missing

Every Phase 3 performance figure in the README (5.55s first scan, 34.4ms second open, the per-file timings) was measured on 5000 symlinks to one inode, or on freshly `cp`-copied files — never on a real folder of 5000 distinct ARWs on real hardware. Only the user can close this.

#### TODO

- [ ] Measure first-scan and second-open times on a real folder of ~5000 distinct ARW files, and update the README's numbers.

### Core: `xmp_prefix` can rebind a namespace prefix already used for something else

In `crates/core/src/xmp.rs`, if a sidecar binds the `xmp` prefix to some namespace
other than `http://ns.adobe.com/xap/1.0/` and binds no prefix at all to that XMP
namespace, `xmp_prefix` declares `xmlns:xmp` on the `rdf:Description` anyway, which
rebinds the prefix for the tag's other attributes. No real-world producer does this;
handling it would need a generated prefix. Noted while implementing Phase 6 Step 2.

#### TODO

- [ ] Detect a colliding `xmp:` binding and fall back to a generated prefix (or the
  `xap:` alternative) instead of overwriting it.

### App: a failed sidecar write is never retried until the folder is reopened

`crates/app/src/sidecar.rs`'s writer leaves a row `dirty` and reports a
`sidecar-error` event on a failed write (e.g. a read-only directory), but
schedules no retry of its own; the write is only attempted again the next time
the folder is opened (Phase 6 decision 3). Acceptable for now, but worth
revisiting if a locked/read-only sidecar target (an SD card, a locked share)
turns out to be common.

#### TODO

- [ ] Consider a bounded retry (e.g. on a timer, or on the next `set_rating`
  for that path) rather than requiring a folder reopen.

### App: an unparseable or oversize sidecar fails silently on folder open

`reconcile_sidecars_of` in `crates/app/src/commands.rs` drops the `Err` for a
sidecar another tool corrupted or that exceeds the read bound, so the file shows
no rating and no reason. Reporting it needs a count or event the frontend can
show (`crates/app/ui/src/main.ts`); out of scope for Phase 6 Step 4.

#### TODO

- [ ] Surface unparseable/oversize sidecars to the status line, e.g. via a count
  in the `scan-done` payload or a dedicated event.

### App: folder open lists the directory twice (ARWs, then sidecars)

`scan_folder` calls `list_arw_in` and then a separate `list_sidecars_in` pass over
the same directory (`crates/app/src/commands.rs`). One listing that returns both
ARW and sidecar entries would halve that part of a folder open. Not done in Phase
6 Step 4 because `list_arw_in` is shared with callers that do not want sidecars.

#### TODO

- [ ] Fold sidecar discovery into a single directory listing shared with ARW
  discovery, without changing behaviour for callers that only want ARWs.

### App: a `sidecar-error` event can be missed under key-mashing

A `sidecar-error` event (`crates/app/ui/src/main.ts`) overwrites whatever note is
in the status line and is cleared by the next page turn. Phase 6 Step 5's "shows
its message once in the status line" is met literally, but `note`/`setStatus` is a
single transient slot, so an error raised while the user is mashing rating/paging
keys can go unseen. A dedicated, sticky error area would be a UI change beyond
that step.

#### TODO

- [ ] Give sidecar errors a sticky, dismissible display distinct from the
  transient status note.
- [ ] When a `sidecar-error` fires after an optimistic rating keypress,
  revert the "has sidecar" flag it set (`crates/app/ui/src/main.ts`) rather
  than leaving it showing a sidecar that was never written.

### Docs: consider a `docs/agents/core.md` guide for `crates/core`

Phase 6 Step 2 found several non-obvious `quick-xml` 0.42 facts specific to
`crates/core/src/xmp.rs` (only per-event byte offsets, no attribute-level offset,
so the attribute form of `xmp:Rating` is patched by scanning the start tag's own
range; prefix choice goes through `reader.resolver()`; `xmp:Rating="0"` is a legal
value, not absence). Only this feature has touched that code so far, so no guide
exists yet (compare `docs/agents/tauri-app.md`'s Hit/Measured/Inferred format).

#### TODO

- [ ] When the next feature touches `crates/core`'s XML handling, create
  `docs/agents/core.md` capturing these facts, or judge it unnecessary and drop
  this item.

### App: the meta pane's sidecar header shows the predicted name, not an on-disk case variant

The header in `crates/app/ui/src/main.ts` (`sidecarName`) shows the name the
app would write (e.g. `FOO.xmp`), not a foreign on-disk variant such as
`FOO.XMP`, which `list_sidecars_in` (`crates/app/src/commands.rs`)
discovers and the writer patches under its real name. Exposing the real
name needs either a `read_dir` per `folder_entries` call or a new
`xmp_name` column (a `SCHEMA_VERSION` bump). See
`docs/plans/_archived/20260918-rating-display-tidy-up/plan.md`'s "Sidecar name:
predicted versus actual" trade-off for the two alternatives considered.

#### TODO

- [ ] Decide whether the mismatch between the displayed and on-disk
  sidecar name is worth a `read_dir` or a schema change, and implement
  whichever is chosen.

### App: `scan-progress` carries no way to tell what became available

`scan-progress` carries only `{dir, scan_id, done, total}`, so #56 had to throttle the filmstrip refresh to ~1/s rather than re-request only the thumbnails that became available. Carrying the newly-written paths, or a done-index high-water mark, would let the strip ask for exactly what is ready.

#### TODO

- [ ] Extend the `scan-progress` payload so the filmstrip can request only the newly available cells (`crates/app/src/commands.rs`'s `scan-progress` emit, `crates/app/ui/src/strip.ts`, `crates/app/ui/src/main.ts`).

### App: read-only index commands share one `Mutex<Index>` with the scan writer

Read-only index commands still take the same `Mutex<Index>` the scan writer holds; #56 only shortened the critical section (`BATCH` 50 → 10, a 5x increase in transaction count whose throughput cost is unmeasured). A separate read connection would remove the serialisation entirely — WAL is already on.

#### TODO

- [ ] Give the read-only commands their own SQLite read connection in `crates/app/src/index.rs`, and measure what the smaller `BATCH` costs the scan.

### App: a deep-row focus point still exceeds the 50ms budget

A focus point in a deep row of the unrotated JPEG measured 58-65ms keypress to pixels (n=2, before the #56 and #60 fixes); see "The 1:1 focus check path" in the README. Options are prefetching the neighbouring files' crops (Phase 4's ring buffer) or a DCT-scaled placeholder; nothing is chosen.

#### TODO

- [ ] Retake the deep-row measurement post-fix, then decide between crop prefetch and a DCT-scaled placeholder.

### App: the silent update path is unverified end-to-end

`README.md`'s Updating paragraph describes a background download and install on launch and the "Check for Updates…" menu item, but nothing has confirmed on a real build that an installed copy detects a newer release, installs it silently, and launches as the new version next time.

#### TODO

- [ ] Verify the update path once a newer release is out: install the current release, publish the next one, then launch the installed build (and separately, use **Check for Updates…**). Done when the background flow installs the newer release after the signature check and it is used on the next launch, and the menu item reports the up-to-date / installed / already-installed outcomes correctly, on macOS, Windows, and Linux AppImage.

### App: Windows self-update exits the app mid-session

On Windows, `tauri-plugin-updater`'s `install_inner` launches the installer and calls `std::process::exit(0)` because the running exe is locked, so the "installed, used on next launch" behaviour does not hold there: the app quits mid-session. See `crates/app/src/update.rs` and `docs/plans/_archived/20260919-silent-auto-update/learnings.md`.

#### TODO

- [ ] Download the update in the background and install it only on quit (`Update::download`, then `Update::install` from `ExitRequested`), so a running Windows session is never interrupted.

### App: `cancelling_after_the_first_batch_keeps_what_was_written` races on fast runners

The test in `crates/app/src/index.rs` races the scan-cancellation against the scan itself instead of cancelling at a deterministic point. It already failed once on `macos-latest` (PR #58); the fix there (a700d67) only widened the file count from `BATCH * 4` to `BATCH * 40`, which a faster machine can outrun again.

#### TODO

- [ ] Make `cancelling_after_the_first_batch_keeps_what_was_written` in `crates/app/src/index.rs` cancel at a deterministic point (e.g. from the progress callback after the first batch) instead of relying on wall-clock ordering, and confirm it passes repeatedly on all three CI platforms.

### Docs: write a guide for RAW metadata parsing (`docs/agents/raw-metadata-parsing.md`)

Leica DNG support found several non-obvious facts in `crates/core/src/{arw,reader}.rs`'s MakerNote/TIFF parsing that cost time in Steps 1 and 3: the Sony gate skips only when the note lacks `SONY` *and* `Make` is present and non-Sony, so non-Sony test fixtures must set `Make`; a `SubIFDs` entry with `count == 1` stores the IFD offset inline, not an offset to an array; the Leica MakerNote is `LEICA\0` + `02 00` then a little-endian IFD at note offset 8, with `FocusDistance` at tag 0x0304 (LONG, millimetres); `ApertureValue` (APEX) converts via `2^(AV/2)`; and a "non-Sony note is skipped" test needs a valid empty IFD in the fixture, not a `0xffff` sentinel count.

#### TODO

- [ ] When the next feature touches the MakerNote/TIFF parsing in `crates/core/src/{arw,reader}.rs` (another maker's MakerNote, or a new synthetic-TIFF fixture), create `docs/agents/raw-metadata-parsing.md` capturing the points above, linking `docs/plans/_archived/20260918-leica-dng-support/learnings.md` for the underlying measurements instead of duplicating them.

### App: no test harness for `tauri::AppHandle`-taking commands

Review feedback (photolab-dop-sidecar Step 3, Round 1, item 1) asked for a test
that sets a rating between the format swap and `reset_sidecars` in
`switch_sidecar_format`. This was dismissed for that round: the function takes
a real `tauri::AppHandle` backed by `tauri_plugin_store`, and the codebase has
no `tauri::test` mock-app harness. Building one (mock runtime, store plugin
wiring) would let this and other `AppHandle`-taking commands in
`crates/app/src/commands.rs` (e.g. `switch_sidecar_format`, `set_rating`,
`scan_folder`) be unit-tested.

#### TODO

- [ ] Build a `tauri::test` mock-app harness (mock runtime, `tauri_plugin_store`
  wiring) so `AppHandle`-taking commands in `crates/app/src/commands.rs` can be
  unit-tested, then add the deferred `switch_sidecar_format` race test (rating
  set between the format swap and `reset_sidecars`).

### App: no sharpness cue to catch missed focus without the 1:1 view

Spotting a missed focus or camera shake needs `Space` on every file. A relative
sharpness score is most useful for picking the sharpest frame of a burst;
absolute thresholds are unreliable because edge content varies by subject.

#### TODO

- [ ] Compute a sharpness score around the focus point (the frame centre when
  none is recorded), e.g. the variance of the Laplacian, and show it relative
  to neighbouring frames (a strip badge, a sort or a filter).
- [ ] Decide between the embedded preview (cheap, catches only gross misses)
  and a JpgFromRaw crop (20-40ms per file, which a scan-time pass would add to
  the folder open).

### App: rejected files cannot be cleared out from the app

Culling ends with the rejects still in the folder; removing them means going to
another tool.

#### TODO

- [ ] Add a menu item that moves every rejected file of the open folder, with
  its sidecars, to the OS trash (or a chosen folder), after a confirmation
  showing the count.
