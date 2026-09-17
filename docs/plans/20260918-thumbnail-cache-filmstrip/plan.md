<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Phase 3: index and thumbnail cache, left filmstrip, more paging keys

## Purpose

Phase 2 shows one preview at a time by reading the whole ARW on every page
turn and knows nothing about the folder beyond its file names. Phase 3 gives
the app a persistent picture of the folder: on the first open it extracts,
in parallel, each file's capture time (`DateTimeOriginal` +
`SubSecTimeOriginal`), `FocusLocation`, Orientation and a ~400px thumbnail,
and stores them in a SQLite index in the app's cache directory. A second open
of the same folder comes from the index and takes well under 3 seconds. On top
of that cache the window gets a thumbnail filmstrip down its left edge that
highlights the current file and follows paging, and paging itself gains
ArrowUp/ArrowDown, WASD and HJKL bindings.

Later phases inherit the index: Phase 5 reads `FocusLocation` from it, Phase 6
will want to filter by rating next to it, and Phase 7 groups bursts by capture
time within one second.

Scope guard: no prefetch ring buffer (Phase 4), no 1:1 focus check (Phase 5),
no ratings / XMP (Phase 6), no burst grouping (Phase 7).

## Decisions taken before implementation

Settled with the user; do not reopen.

1. **The strip goes on the left, 160px wide, not the bottom.** More than half
   the user's shots are portrait; on a 16:10 display both orientations are
   height-constrained, so the spare space is left/right. A left strip costs a
   portrait image nothing and a landscape image ~5%; a bottom strip would cost
   ~13% in both. Left rather than right to match macOS navigation sidebars.
2. **The index is SQLite (`rusqlite`, `bundled`), one database in the app
   cache directory, thumbnails stored as JPEG BLOBs in it.** Rationale and
   alternatives in "Trade-offs and risks".
3. **The scan reads a bounded prefix of each file, not the whole file.** The
   IFDs, ExifIFD, MakerNote and the IFD0 preview all sit in the first ~550 KB
   of a 48 MB ARW (measured on the one real file). Reading whole files would be
   240 GB for 5000 files and cannot meet the 30 s target.
4. **`FocusLocation` is parsed by our own TIFF walker**, not `exiftool` and not
   a new EXIF crate. It is Sony MakerNote tag 0x2027 (`int16u[4]`,
   `[sensor_w, sensor_h, x, y]`), reachable through IFD0 -> 0x8769 (ExifIFD)
   -> 0x927c (MakerNote) with offsets absolute to the TIFF start. The parser in
   `arw.rs` already walks IFDs; this is one more hop.
5. **The `preview` command switches to the bounded read too, in Step 2** — the
   planner offered leaving it on `std::fs::read` until Phase 4, and the user
   chose to take the win now. It is a small change on top of the reader Step 2
   builds anyway, and it cuts the per-page floor that the Phase 4 baseline
   measured at mean 19.3 ms / p95 26.6 ms. Consequences, both of which are part
   of Step 2's work rather than afterthoughts: the "Phase 4 baseline" table in
   `README.md` becomes stale and must be re-measured and replaced in the same
   step, and Phase 4's prefetch design now starts from a much cheaper per-page
   read than it was planned against.

## Steps

- [x] Step 1: Add ArrowUp/ArrowDown, WASD and HJKL paging keys
  - Done when:
    - `ArrowUp`, `w`, `a`, `h`, `k` move to the previous file; `ArrowDown`,
      `s`, `d`, `j`, `l` move to the next; `ArrowLeft` / `ArrowRight` / `o`
      keep working as today; all go through the existing `move()` so clamping
      and the sequence counter are unchanged
    - Key events with `metaKey`, `ctrlKey` or `altKey` are ignored by the
      handler (so Cmd+W still closes the window and Cmd+O is not swallowed);
      handled keys still get `preventDefault()`
    - `README.md` "Status" lists the paging keys
    - `mise run ci` passes. **(manual)** The user confirms the new keys page in
      the running app
  - Implementation approach:
    - `crates/app/ui/src/main.ts` `keydown` listener only. Prefer a
      `Map<string, number>` from `event.key` to delta over a longer
      `if`/`else` chain; do not add a key-config layer
    - Collision check against keys claimed by later phases (`1`-`5`, `x`,
      `Space`, `o`): none. Record that in the PR description. Note for later:
      `x` sits between `s` and `d` on the keyboard, so a mistyped reject is
      one key away from paging; not a reason to change anything now

- [x] Step 2: Core: bounded ARW read, capture time + FocusLocation parsing, thumbnail encoding, and `preview` on the bounded read
  - Done when:
    - `riffle_core::arw::Arw` gains `capture_time: Option<String>` (the raw
      `DateTimeOriginal` string, `YYYY:MM:DD HH:MM:SS`),
      `subsec: Option<String>` (raw `SubSecTimeOriginal`) and
      `focus: Option<FocusLocation { sensor_w, sensor_h, x, y }>` (u16s),
      populated by following IFD0 tag 0x8769 to the ExifIFD, then 0x927c to
      the Sony MakerNote IFD and reading 0x2027. Existing fields and the
      existing tests are unchanged
    - A bounded reader (`riffle_core::reader::read_head(path, limit) ->
      Vec<u8>` or equivalent) reads at most `limit` bytes from the start; a
      helper that returns `(Arw, preview_jpeg: Vec<u8>)` uses it, and if the
      preview range lies beyond what was read it does a second ranged read
      (`seek` + `read_exact`) of just that range rather than the whole file
    - **The app's `preview` command uses that helper** instead of
      `std::fs::read` (decision 5). Its wire format, its error strings and the
      frontend are unchanged
    - The "Phase 4 baseline" table in `README.md` is **re-measured and
      replaced** with numbers from the bounded path, keeping the same honesty
      about what the measurement excludes (the IPC hop and `createImageBitmap`)
      and how the folders were built. State the before/after so the size of the
      win is visible
    - `riffle_core::decode::thumbnail_jpeg(preview_jpeg, quality) ->
      Vec<u8>` decodes the 1616x1080 preview with mozjpeg at DCT scale 2/8
      (giving 404x270) and re-encodes it as a baseline JPEG; the output is
      **unrotated**, like the preview tier, and the caller keeps the
      Orientation. Measure and record in `learnings.md`: the ms per file for
      decode+encode and the median output size in bytes
    - `riffle-cli info` prints `capture_time`, `subsec` and `focus`;
      `riffle-cli focusbox` and `bench` use the parsed `FocusLocation` and the
      `exiftool` shell-out is removed. On the real file `info` prints
      `focus: 7008 4672 3613 1732`, `capture_time: 2026:09:13 09:23:33`,
      `subsec: 122`, matching `exiftool` (record the comparison in
      `learnings.md`; commit nothing derived from the file)
    - Tests in `crates/core` with hand-built fixtures (extend the existing
      `tiff()` helper: IFD0 -> ExifIFD with 0x9003 / 0x9291 ASCII values and a
      0x927c blob that is itself a little-endian IFD holding 0x2027; also a
      fixture where the MakerNote is absent, one where it is present but has no
      0x2027, and one where an offset points past the buffer). A thumbnail test
      round-trips a small synthetic image encoded in-test with
      `mozjpeg::Compress` (no image files committed). `mise run ci` passes
  - Implementation approach:
    - The Sony5 MakerNote seen on the real file has **no header** (the IFD
      starts at the tag's data offset and its value offsets are absolute to
      the TIFF start). Older Sony bodies prefix `SONY DSC \0\0\0` /
      `SONY CAM \0\0\0` (12 bytes) before the IFD; detect and skip that prefix,
      still with absolute offsets. Do not attempt the encrypted 0x94xx tags
    - Prefix `limit`: pick a value with margin over the measured ~542 KB end of
      the preview (1 MiB is the obvious choice) and put the number and its
      justification in a code comment; the ranged fallback covers bodies whose
      layout differs. `parse` bails on an IFD past the buffer end, so a
      truncated prefix surfaces as an error, not a wrong answer; on such an
      error the helper falls back to the whole-file read
    - `arw.rs` value reads: `SHORT`/`ASCII` arrays longer than 4 bytes live at
      the entry's offset; the existing `Entry` tuple already carries type and
      count. Bounds-check every offset (this parser will now run over 5000
      files in parallel; a panic on one bad file must not take the scan down)
    - mozjpeg: `Decompress::new_mem` -> `.scale(2)` -> `.rgb()`;
      `Compress::new(ColorSpace::JCS_RGB)`, `set_size`, `set_quality`,
      `start_compress(Vec::new())`, `write_scanlines`, `finish`. Quality ~80;
      note the resulting size. mozjpeg's optimised encode is slower than
      libjpeg-turbo's baseline; if encode dominates, turn off
      `set_optimize_scans` / progressive
    - `riffle_core::partial` and `apply_orientation` are unchanged

- [x] Step 3: Core: parallel extraction with rayon, and a CLI `scan` benchmark
  - Done when:
    - `riffle_core::scan` exposes a pure per-file function
      `extract(path) -> Result<Entry, String>` returning
      `{ orientation, capture_time, subsec, focus, thumbnail: Vec<u8> }` (via
      Step 2's bounded read), and `extract_all(paths, threads, on_item, cancel)`
      that runs it over a `rayon` pool of the given size, calls `on_item(index,
      Result<Entry>)` from worker threads as each file finishes, and stops
      scheduling new files once the `AtomicBool` cancel flag is set. Per-file
      errors are delivered through `on_item`, never a panic
    - `riffle-cli scan <dir> [threads]` runs `extract_all` over the ARWs in a
      directory with no database, prints files/s, total time, mean/p95 per file
      and the summed thumbnail bytes. This is the benchmark for the 30 s target
    - Measured and recorded in `learnings.md` and README: (a) 5000 symlinks to
      the real file (warm page cache, CPU-bound number) and (b) 20-100 distinct
      copies made with `cp` in a scratch directory (first read, disk-bound
      number), for a couple of thread counts (e.g. 4, 8, all 12). Extrapolate
      (b) to 5000 files and state whether 30 s holds on the internal SSD. State
      plainly that the real 5000-distinct-file number can only be measured by
      the user on a real folder
    - Tests: `extract_all` over a temp dir of synthetic fixtures (from Step 2)
      delivers every index exactly once and stops early when cancelled.
      `mise run ci` passes
  - Implementation approach:
    - `rayon` in `crates/core/Cargo.toml` (build a dedicated
      `ThreadPoolBuilder` so the app can size it; do not use the global pool,
      which would also be sized by whatever else runs)
    - Thread count default: leave it a parameter; the app decides in Step 4.
      Measure whether more threads than physical cores helps a disk-bound scan
      or just adds contention with paging
    - Assumes Step 2 is merged

- [x] Step 4: App: SQLite index, background scan on folder open, progress in the status line
  - Done when:
    - `crates/app/src/index.rs` opens (creating on first use)
      `<app_cache_dir>/index.sqlite` via
      `tauri::Manager::path().app_cache_dir()` with `PRAGMA user_version` for
      the schema version, `journal_mode=WAL`, and one table
      `files(path TEXT PRIMARY KEY, dir TEXT NOT NULL, size INTEGER,
      mtime_ns INTEGER, orientation INTEGER, capture_time TEXT, subsec TEXT,
      focus_w, focus_h, focus_x, focus_y INTEGER, thumb BLOB, error TEXT)` plus
      an index on `(dir)` and on `(capture_time, subsec)`. A row is valid for a
      file iff `size` and `mtime_ns` match its current `stat`
    - Opening a folder: `list_arw` is unchanged and the first preview still
      appears immediately through the existing `preview` path (which does not
      depend on the index). A new `scan_folder(dir)` command returns at once
      after (1) stat-ing every listed file, (2) deleting rows under `dir` whose
      file is gone, (3) computing the set of files with no valid row, and (4)
      spawning the scan of that set on a blocking task using
      `riffle_core::scan::extract_all`. Results are written in batches (one
      transaction per N files, N around 50) so that quitting mid-scan keeps
      what was done and the next open resumes with only the remainder
    - Progress: the scan emits a Tauri event `scan-progress` with
      `{ dir, done, total }` at most every ~100 ms (and always on the last
      file), then `scan-done` with `{ dir, total, errors }`. The frontend
      listens via `window.__TAURI__.event.listen` (declared in `tauri.d.ts`)
      and appends `scanning 1234 / 5000` to the status line while a scan runs;
      errors are counted, not shown one by one. Paging during the scan works
      (**(manual)** user confirms the app stays responsive while a real folder
      scans)
    - Opening another folder mid-scan sets the previous scan's cancel flag;
      only one scan runs at a time (a `Mutex<Option<ScanHandle>>` in managed
      state); a `scan-progress` event from a cancelled scan carries its `dir`
      so the frontend can ignore it
    - `folder_entries(dir) -> Vec<{ path, orientation, capture_time, subsec,
      focus, has_thumb }>` returns the index rows for a folder in the same
      file-name order as `list_arw`, and `thumbnail(path)` returns the cached
      thumbnail JPEG as raw bytes through `tauri::ipc::Response` with the same
      8-byte header layout as `preview`, using a new kind tag
      `THUMBNAIL_KIND_JPEG_V1 = 2` and the Orientation; a missing row is an
      error string. These are what Step 5 consumes; this step only has to
      expose and test them
    - Tests in `crates/app` against a temp database: upsert then re-open
      returns the row; changing `mtime` invalidates it; a deleted file's row is
      removed on the next `scan_folder` reconciliation; the thumbnail payload
      header encodes kind 2 and the orientation. The scan orchestration test
      uses the synthetic fixtures from Step 2 and a cancel flag that fires
      after the first batch. `mise run ci` passes (note: `rusqlite` with
      `bundled` compiles SQLite from C; measure the cold CI time and raise
      `timeout-minutes` in `.github/workflows/ci.yml` if it gets near 30)
    - Second-open target: `scan_folder` on a fully indexed folder of 5000
      files (stat + reconciliation + query) finishes well under 3 s; measure
      with the 5000-symlink folder and record. **(manual)** The user confirms
      the second open of a real folder is under 3 s
  - Implementation approach:
    - `rusqlite = { version = "0.3x", features = ["bundled"] }` in
      `crates/app/Cargo.toml` (bundled so Windows needs no system SQLite). Do
      not put SQLite in `riffle-core`; the CLI must not grow a database
    - The commands must be `async` and do their IO under
      `tauri::async_runtime::spawn_blocking`, per the Phase 2 deadlock lesson;
      emit events with `tauri::Emitter` from the blocking thread (that is
      safe). `core:default` already covers `event:listen`; verify by running
      `pnpm tauri dev` rather than assuming
    - Thread count for the scan: start from Step 3's measurement; leave a
      couple of cores free so paging is not starved. Record the chosen number
      and why
    - `serde` + `serde::Serialize` for the `folder_entries` rows and event
      payloads (Tauri commands need it); `focus` as four `Option<u16>` or a
      nested struct, either is fine
    - Delete the orphaned `ping` command only if it is in the way; otherwise
      leave it for its todo item

- [x] Step 5: Frontend: left thumbnail filmstrip with current-file highlight and auto-scroll
  - Done when:
    - The layout becomes a 160px-wide scrollable column on the left (`#strip`)
      next to the existing canvas; the canvas' fit calculation uses its own
      `clientWidth`, so the main image loses no height. Portrait thumbnails
      are drawn upright using the Orientation from the `thumbnail` header
    - After `list_arw`, the strip shows one cell per file in file order. A
      cell whose thumbnail is not in the index yet (scan still running, or the
      file errored) shows a neutral placeholder with the file name; when a
      `scan-progress` event arrives, cells in the visible range that are still
      placeholders are re-requested
    - The strip is **virtualised**: only cells within the visible range plus a
      margin have a decoded image; scrolled-out images are released
      (`URL.revokeObjectURL` / `ImageBitmap.close()`), so a 5000-file folder
      does not hold 5000 decoded thumbnails
    - The current file's cell has a visible highlight; on every page turn the
      strip scrolls so the current cell is in view (`scrollIntoView({ block:
      "nearest" })` or manual `scrollTop`), including during key auto-repeat
    - Clicking a cell shows that file (`index = i; show()`); the strip does
      not take keyboard focus away from the window handler
    - `mise run ci` passes. **(manual)** The user confirms on a real folder:
      thumbnails fill in while scanning, portrait ones are upright, the
      highlight follows all paging keys, holding a key keeps the strip in
      sync, clicking a cell pages, and scrolling a 5000-file strip stays
      smooth
  - Implementation approach:
    - Keep the frontend framework-free and bundler-free. If the strip goes in
      its own module (`crates/app/ui/src/strip.ts`), import it as
      `./strip.js` (moduleResolution is `bundler`, nothing enforces the
      extension)
    - Thumbnail delivery: `invoke("thumbnail", { path })` -> `ArrayBuffer` ->
      strip the 8-byte header -> `Blob` -> `URL.createObjectURL` on an `<img>`
      with a CSS `transform: rotate()` for Orientation 6/8. The decode worker
      is not needed for 20 KB images. Cap in-flight thumbnail invokes (a small
      queue, newest-visible first), and drop responses for cells that scrolled
      out, mirroring the `seq` discipline in `requestPreview`
    - Cell geometry: fixed height per cell (e.g. 160px column, 8px gutter,
      cells sized for the 404x270 / 270x404 image at 2x DPR), so
      `scrollTop -> index` is arithmetic and virtualisation is a spacer div
      plus absolutely positioned cells
    - `ArrowUp`/`ArrowDown`/`Space` must not scroll the strip; the window
      handler already calls `preventDefault()` and the strip must not be the
      focused element
    - Assumes Step 4 is merged

- [x] Step 6: Frontend: draw the focus box on the preview
  - Done when:
    - When the index has a `FocusLocation` for the current file, the preview
      canvas draws a rectangle at the focus point over the image; when it does
      not (a manual-focus shot, or the file is not indexed yet), nothing is
      drawn and no error is shown
    - The box is placed in **unrotated sensor coordinates and rotated with the
      image**, not drawn after rotation: scale `(x, y)` from
      `(sensor_w, sensor_h)` to the unrotated preview's pixel size, then apply
      the same Orientation transform the image gets. Verified against the real
      portrait file (Orientation 8), where a box drawn the naive way lands on
      the wrong edge
    - The box size is a fixed fraction of the image's short side (so it reads
      the same on portrait and landscape), stroked in a colour that survives
      both a white jersey and a dark background (e.g. a 2px stroke with a
      contrasting 1px outline). It is drawn on every `show()`, including during
      key auto-repeat, and costs no extra IPC round trip — the coordinates come
      from the `folder_entries` rows Step 4 already returns
    - The box can be toggled off with a key (`f`), defaulting to on; the
      setting is not persisted (no settings store exists yet). Collision check
      against keys claimed by later phases (`1`-`5`, `x`, `Space`, `o`, and the
      paging keys from Step 1): `f` is free — record that in the PR description
    - `mise run ci` passes. **(manual)** The user confirms on a real folder
      that the box sits on the subject's face for both portrait and landscape
      shots
  - Implementation approach:
    - `crates/app/ui/src/main.ts` only; no Rust change. `riffle-cli focusbox`
      already does this transform for its PNG output — port that arithmetic
      rather than re-deriving it, and keep the two in step
    - The frontend needs the focus values per file: hold the `folder_entries`
      rows in a `Map<path, entry>` populated after `list_arw`, and refresh the
      entry for the current file when a `scan-progress` event indicates the
      scan has advanced past it
    - Assumes Step 5 is merged (the entries map is shared with the strip)

- [x] Step 7: Open a folder by drag-and-drop
  - Done when:
    - Dropping a folder onto the window opens it exactly as the picker does
      (same path through `list_arw` / `scan_folder` / first preview), and a
      scan already running for another folder is cancelled the same way
    - Dropping something that is not a directory, or several items at once,
      does not open a wrong folder: a single directory is taken; a single
      **file** is accepted by taking its parent directory (dragging one ARW is
      the obvious gesture); anything else is ignored with a status-line message
    - There is visible drop feedback while a drag is over the window (a border
      or overlay), cleared on both drop and leave, including when the drag
      leaves without dropping
    - `mise run ci` passes. **(manual)** The user confirms dropping a folder,
      dropping one ARW, and dragging over and away again
  - Implementation approach:
    - **Tauri intercepts HTML5 drag-and-drop, so `event.dataTransfer.files`
      never carries a usable path.** The paths arrive only through Tauri's own
      `tauri://drag-drop` event (with `tauri://drag-enter` / `drag-over` /
      `drag-leave` for the feedback), listened to the same way the
      `scan-progress` listener is wired in Step 4. Do not spend time on the
      DOM `drop` event
    - Whether the dropped path is a directory is decided in Rust (a new small
      command, or an argument to the existing open path), not by guessing from
      the string in TypeScript
    - `dragDropEnabled` must stay at its default (true) in `tauri.conf.json`;
      note in the PR if it has to be touched
    - Assumes Step 4 is merged; independent of Steps 5 and 6

- [x] Step 8: Documentation and status update
  - Done when:
    - `README.md` "Status" says Phase 3 is done, describes the index (where
      the database lives per OS, what is in it, how invalidation works), the
      strip, the focus box and folder drag-and-drop, and the full key list;
      it carries the Step 3 scan measurements and the Step 4 second-open
      measurement in a table next to the existing
      ones, with the same "what could not be measured here" honesty as the
      Phase 4 baseline; the manual confirmations the user made are recorded as
      such
    - `CLAUDE.md` "Layout" mentions the index module and that the CLI has a
      `scan` benchmark
    - `mise run ci` passes

## Trade-offs and risks

### SQLite versus a binary index file

Taken: **SQLite via `rusqlite` (bundled)**, one database in the app cache
directory, thumbnails as BLOBs.

- For: Phase 6 wants "filter by rating" and Phase 7 wants "order by
  `capture_time, subsec` and window within one second" — both are one SQL
  statement; a binary index would need custom code and a full rewrite on every
  change. Per-row invalidation and mid-scan persistence are transactions
  rather than atomic file swaps. Thumbnails in the same file mean no orphaned
  thumbnail files and one thing to delete
- Against: `bundled` compiles SQLite from C — a few minutes cold in CI
  (macos-latest cache mitigates; measure in Step 4) and MSVC on Windows
  (already required by Tauri). A 5000-file folder is ~100 MB of BLOBs in one
  file; deleting rows does not shrink it without `VACUUM` (acceptable; note in
  README)
- Alternative not taken: a `bincode`/hand-rolled index per folder plus
  thumbnail files in a directory. Fewer dependencies, faster cold CI, but every
  later phase re-implements querying, and partial-scan persistence needs care
- Sub-decision taken with it: **thumbnails as BLOBs rather than files served
  by a custom URI scheme**. Files would let the webview load them by URL with
  native lazy loading and no `invoke`; BLOBs keep one file and one transaction,
  and match how `preview` already crosses IPC. Revisit only if the strip turns
  out to be IPC-bound in Step 5

### Where the cache lives and how it is keyed

Taken: `app_cache_dir()` (macOS `~/Library/Caches/com.minodisk.riffle`, Windows
`%LOCALAPPDATA%\com.minodisk.riffle`), rows keyed by absolute path with
`size + mtime_ns` as the validity check, reconciled against the directory
listing on every open.

- Alternative: a `.riffle/` directory inside the photo folder. Travels with the
  folder and survives renames, but photo folders are often on SD cards or
  read-only mounts, and it litters the user's data. Not taken
- Risk: mtime granularity on some filesystems (FAT: 2 s) — size+mtime is
  still the standard choice; content hashing 48 MB files would defeat the
  purpose. Renaming a folder re-scans it; acceptable
- Risk: two app instances writing the same database — WAL mode handles
  concurrent readers and serialises writers; do not design for it beyond that

### The 30-second target and the whole-file read

Not reachable with whole-file reads: 5000 x 48 MB = 240 GB, i.e. 48 s at an
unrealistic 5 GB/s, ~100 s at the ~2.5 GB/s the current measurements imply.
With the bounded read the scan moves ~2.7 GB (~1-2 s of IO on the internal
SSD) and becomes CPU-bound at roughly 5000 x ~5 ms / threads — a few seconds.

Decision 5 takes this further than the scan: `preview` moves to the bounded
read in the same step, so the per-page floor drops too. Phase 4 still owns
prefetching and the ring buffer, but it now starts from a cheaper read than
the baseline it was planned against — which is why Step 2 re-measures and
replaces that table rather than leaving a stale number in the README.

Caveat on the target itself: on a card reader or slow external disk the scan
is disk-bound (2.7 GB at 90 MB/s is 30 s by itself). The criterion is stated
for the internal SSD; the README should say so.

### How `FocusLocation` is obtained

Taken: extend our own TIFF walker (tag 0x2027 in the Sony MakerNote IFD;
verified against `exiftool` on the real file). No subprocess, no new crate.

- `exiftool` subprocess: 5000 process spawns (~50-100 ms each) is minutes and
  makes the app depend on a Perl tool the user has to install on Windows. Kept
  only as the reference in `learnings.md`
- `quickexif` / `kamadak-exif`: both would still leave the MakerNote as an
  opaque blob to parse ourselves, so they add a dependency without removing the
  work
- Risk: other Sony bodies may prefix the MakerNote with a 12-byte header
  (`SONY DSC \0\0\0` / `SONY CAM \0\0\0`); Step 2 handles both forms but only
  the header-less form is verified on a real file. `FocusLocation` absent (a
  manual-focus shot) is a normal `None`, not an error

### Progress reporting, and paging or quitting mid-scan

Taken: push via Tauri events, throttled to ~10/s, with the scan's `dir` in the
payload; frontend keeps showing previews through the index-independent
`preview` path, so paging never waits on the scan. Batched transactions make a
quit lose at most one batch. An alternative is polling a `scan_status` command
every 200 ms — simpler permission-wise and no `event` typing, but it keeps a
timer alive and is less direct; not taken unless `event.listen` turns out to be
blocked by capabilities (verify in Step 4).

Risk: the scan saturates the disk while the user pages, raising per-page
latency during the first seconds. Mitigation is the thread-count knob in
Step 4; measure, do not guess.

### Filmstrip memory

5000 decoded 404x270 thumbnails would be ~2 GB of RGBA; hence virtualisation
is an acceptance criterion in Step 5, not an optimisation. The cost is that the
strip is a hand-rolled virtual list in a framework-free frontend. The
alternative — plain `<img loading="lazy">` for every file and trusting the
webview to discard offscreen bitmaps — is simpler but unmeasurable here, and
Windows WebView2 and WKWebView behave differently. Not taken.

### Things this plan deliberately leaves to their todo items

No frontend formatter/linter (the strip adds a few hundred lines of TS that
nothing formats), `docs/agents/tauri-app.md`, and the unmeasured end-to-end
per-page latency. None blocks a step. The per-page latency item becomes more
relevant now that decision 5 changes the read path.

### Manual verification

`osascript` assistive access is denied on the development machine, so every
criterion marked **(manual)** — new keys page, app stays responsive during a
scan, second open under 3 s, thumbnails fill in / are upright / follow paging
/ scroll smoothly on a 5000-file folder — has to be checked by the user in the
running app. CI and the CLI `scan` benchmark cover everything else.

## Progress

- (2026-09-18) Step 1 complete
- (2026-09-18) Step 2 complete
- (2026-09-18) Step 3 complete
- (2026-09-18) Step 4 complete
- (2026-09-18) Step 5 complete
- (2026-09-18) Step 6 complete
- (2026-09-18) Step 7 complete
- (2026-09-18) Step 8 complete
