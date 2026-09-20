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

### App: no rescan when new files appear in an already-open folder

`scan_folder` in `crates/app/src/commands.rs` reconciles the index only when a folder is opened, so a file added to an already-open folder is not picked up until the folder is reopened.

#### TODO

- [ ] Add a folder watcher, or a rescan on window refocus, to catch new files without a reopen.

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

### App: rejected files cannot be cleared out from the app

Culling ends with the rejects still in the folder; removing them means going to
another tool.

#### TODO

- [ ] Add a menu item that moves every rejected file of the open folder, with
  its sidecars, to the OS trash (or a chosen folder), after a confirmation
  showing the count.

### App: sidecar read and write errors for the same file show as two separate entries

The sticky error area added for sidecar problems
(`crates/app/ui/src/errors.ts`, `crates/app/ui/src/main.ts`) keys a read
error (from `reconcile_sidecars_of`/`scan_folder`) by the sidecar path and a
write error (from the `sidecar-error` event) by the RAW path, so a single
file that fails both to read on open and to write afterward shows two
entries in the pane instead of one being superseded by the other.

#### TODO

- [ ] Decide on a shared key (e.g. the RAW path) for sidecar read and write
  errors so the two can supersede each other instead of coexisting.
