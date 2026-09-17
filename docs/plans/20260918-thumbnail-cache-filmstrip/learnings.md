# Learnings

## Step 1: paging keys

- Letter keys are matched case-insensitively (`event.key.toLowerCase()`), so
  Shift+J pages like `j`. `o` goes through the same lower-cased key, so Shift+O
  now opens the folder picker too; that is consistent rather than a regression.
- `metaKey` / `ctrlKey` / `altKey` events return before any handling, so Cmd+W
  and Cmd+O reach the system. `shiftKey` is deliberately not excluded, since
  that is what makes the case-insensitive match useful.
- `event.key` is the produced character, not the physical key, so on a
  non-QWERTY layout WASD/HJKL land on different physical keys. Accepted for
  now; switching to `event.code` would break HJKL for anyone who chose a layout
  deliberately, and nobody has asked. Noted so it is not rediscovered.
- Not verified in the running app: osascript assistive access is denied on this
  machine, so the GUI could not be driven. Only `mise run ci` (which
  type-checks `main.ts`) was run; the user confirms the keys by hand.
- Collision check against keys claimed by later phases (`1`-`5`, `x`, `Space`,
  `o`): none.

## Step 2: bounded read, metadata parsing, thumbnails

- The MakerNote route works exactly as planned on the real file. `info` on
  `~/Downloads/_DSC6978.ARW` prints `focus: 7008 4672 3613 1732`,
  `capture_time: 2026:09:13 09:23:33`, `subsec: 122`, and
  `exiftool -s3 -FocusLocation -DateTimeOriginal -SubSecTimeOriginal` prints
  the same three values. Orientation 8 (`Rotate 270 CW`) also matches.
  `exiftool` is only the reference now; the shell-out is gone from the CLI.
- The MakerNote on this body has no `SONY ...` header, as expected. The
  header-ed form is covered by a fixture only.
- Layout of the real file: preview at 204962 + 337198 = ends at 542160
  (~542 KB), JpgFromRaw at 544768. So `HEAD_LIMIT = 1 MiB` covers the whole
  metadata + preview region with ~1.9x margin and stops well before the full
  JPEG, which the bounded path never wants.
- `SubSecTimeOriginal` is 4 bytes including the NUL, so it rides *inside* the
  entry rather than at an offset. The fixture exercises both paths (inline
  subsec, out-of-line `DateTimeOriginal`).
- Out-of-range offsets are errors, not silent `None`s: a truncated prefix must
  not be able to produce a wrong answer, and `reader::read_preview` turns that
  error into the whole-file fallback. Absent ExifIFD / MakerNote / 0x2027 stay
  normal `None`s.
- The fallback only fires when the prefix actually hit the limit
  (`head.len() == HEAD_LIMIT`). Re-reading a file we already read in full would
  cost 48 MB for nothing, and a genuinely previewless file would pay it on
  every page turn.
- Measurements (Apple Silicon, release build, `~/Downloads/_DSC6978.ARW`):
  - `thumbnail_jpeg` at quality 80: **mean 7.6 ms, median 7.0 ms** per file,
    **median output 19232 bytes** (n=50). 5000 files is ~19 MB of BLOBs and,
    at 7 ms each, ~35 s of single-threaded CPU, i.e. a few seconds over the
    rayon pool Step 3 adds.
  - Per-page read, whole file vs bounded prefix: 6.6 -> 0.06 ms mean on the
    warm 5000-symlink folder, 16.0 -> 5.1 ms mean on 20 fresh `cp` copies.
    Both columns re-measured in the same session; the README table carries
    them.
  - `purge` needs root here, so the "first read" numbers cannot be guaranteed
    cold. Same limitation as the Phase 2 baseline, and it is stated in the
    README.
- `mozjpeg` **panics** (it does not return `Err`) when handed bytes that are
  not a JPEG: a `thumbnail_jpeg(b"not a jpeg")` test failed on
  `assert!(...is_err())` with no panic message. The test was dropped; the
  matter belongs to Step 3, which runs over thousands of files.
- `mozjpeg::Compress` defaults to mozjpeg's scan optimisation.
  `set_optimize_scans(false)` nulls `scan_info`, which is what makes the output
  baseline.
- Not verified in the running app: the GUI could not be driven here
  (osascript assistive access denied). The `preview` change keeps the wire
  format, the error strings and the frontend untouched, and CI passes; the user
  confirms by hand.

## Step 3: parallel extraction and the CLI `scan` benchmark

- The `mozjpeg` panic deferred from Step 2 is real and is handled in `extract`
  with one `catch_unwind(AssertUnwindSafe(...))` around `thumbnail_jpeg`
  (which owns the decode as well as the encode), turning it into a per-file
  `Err` delivered through `on_item`. `AssertUnwindSafe` is honest here: the
  closure borrows only the preview bytes and the path, and nothing it touches
  is observed after the unwind.
- Which inputs panic and which return `Err` is not obvious, which is why an SOI
  check would only give false confidence: 512 zero bytes **panics**, while
  `ff d8` followed by garbage returns a normal `Err`. The test fixture is
  therefore an ARW shell whose preview is zeros, and it does exercise the
  `catch_unwind` path (verified by probing both inputs directly before writing
  the test).
- **No panic hook is installed.** mozjpeg's panic carries no message and prints
  nothing to stderr on this platform (checked with `--nocapture`), so there is
  nothing to suppress; and a process-wide hook to silence one crate's panics
  would hide unrelated bugs. The scan counts errors instead.
- `extract_all` builds its own `rayon::ThreadPoolBuilder` and `install`s on it,
  so the size is the caller's and nothing else in the process shares the pool.
  Cancellation is a check at the top of each item, so files already running
  finish; that is what the test asserts (some items delivered, not all).
- Per-file timing in the CLI: `on_item` cannot see when a file *started*, so
  the benchmark takes the gap between two completions on the same worker
  (`rayon::current_thread_index()`), seeded with the scan start. That is the
  per-file wall time on a busy worker, which is why it rises with the thread
  count while throughput still improves.

### Measurements (Apple Silicon, 12 cores, release build)

(a) 5000 symlinks to `~/Downloads/_DSC6978.ARW`, warm page cache - the
CPU-bound number:

| threads | total | files/s | per file mean / p95 |
|---------|-------|---------|---------------------|
| 1  | 34.9s | 143  | 7.0ms / 7.3ms   |
| 4  | 9.19s | 544  | 7.3ms / 7.6ms   |
| 8  | 5.35s | 935  | 8.6ms / 12.1ms  |
| 10 | 5.08s | 984  | 10.2ms / 14.8ms |
| 12 | 4.46s | 1121 | 10.6ms / 16.9ms |
| 16 | 4.51s | 1108 | 14.4ms / 29.6ms |

(b) 100 distinct `cp` copies (4.5GB) per thread count, each folder scanned
once right after being written - freshly written copies, page cache not
guaranteed cold:

| threads | total | files/s | per file mean / p95 | extrapolated to 5000 |
|---------|-------|---------|---------------------|----------------------|
| 4  | 0.44s | 230 | 17.4ms / 22.2ms | 21.7s |
| 8  | 0.25s | 406 | 19.0ms / 30.7ms | 12.3s |
| 12 | 0.19s | 529 | 21.2ms / 36.4ms | 9.5s  |
| 16 | 0.17s | 601 | 25.3ms / 34.4ms | 8.3s  |

- **The CPU cost leaves ~25s of headroom against the 30s target**: the worst
  extrapolation here (4 threads, freshly written copies) is 21.7s and every
  realistic thread count is 8-12s, against a 4.5s floor from pure CPU. Whether
  that headroom holds depends on the disk, and these numbers do not establish
  that - each (b) folder is a `cp` copy scanned right after being written, so
  the page cache is likely still warm from the write, closer to a warm-head
  read with some write-back cost than to a genuinely cold read. The
  single-threaded 34.9s shows the target is unreachable without the pool.
- Caveats, stated rather than smoothed over: each (b) folder was scanned once,
  in the order the folders were written, so the later (higher-thread-count)
  folders had a better chance of still being in the page cache - the
  thread-count trend in (b) is weaker evidence than it looks. `purge` needs
  root here, so nothing can be guaranteed cold. And a real 5000-distinct-file
  folder is 240GB of files touched at ~550KB each; **only the user can measure
  that on a real folder**, on the real disk, with real directory layout.
- Expectation from Step 2 confirmed: the work is CPU-bound. 7.0ms per file
  single-threaded matches Step 2's 7.6ms `thumbnail_jpeg` mean almost exactly,
  and the bounded read adds ~0.06ms warm; even in (b) the disk adds ~10ms per
  file, not the seconds a whole-file read would, though (b) cannot be trusted
  as a cold-read number.
- Thumbnail size: 19232 bytes mean, i.e. **96MB of BLOBs for 5000 files** - the
  number Step 4's database has to carry.
- More threads than cores does **not** help: 16 threads was flat against 12
  warm (4.51s vs 4.46s) and doubled the per-file p95 (29.6ms vs 16.9ms). For
  Step 4 the recommendation is **cores - 2 (10 here)**: it costs 10% of scan
  throughput (5.08s vs 4.46s) and keeps two cores free so paging is not
  starved while the scan runs. Nothing beyond `available_parallelism()` should
  ever be used.

## Step 4: SQLite index, background scan, progress in the status line

- **Second open is 34ms, not seconds, but that number is a lower bound.**
  Measured on 5000 **symlinks to one real ARW** (a single inode, warm page
  cache) with a temporary `#[ignore]`d test that was removed before
  committing, so the run is not reproducible as written: `stat` of 5000 files
  42ms, reconciliation 1.3ms, and the full second open
  (stat + reconcile + `entries`) **34.4ms** against a fully populated index.
  5000 lookups of one cached inode's metadata is cheaper than 5000 distinct
  ~48MB files' metadata in a real card folder, so this is not directly
  comparable to a real second-open number; re-running against a real, varied
  folder is needed for that. The first scan of the same folder took 5.55s on
  10 threads with 0 errors, matching Step 3.
- **The database is 104,177,664 bytes for 5000 rows** (~20.8KB per row), right
  on Step 3's 19232-byte thumbnail mean plus SQLite overhead. Worth repeating
  in the README: deleting rows does not shrink the file without `VACUUM`.
- **Thread count: `available_parallelism() - 2` (10 here)**, per Step 3's
  measurement; the reason (10% slower than all cores, two cores left for the
  paging path) is in a comment on `scan_threads` in `crates/app/src/commands.rs`.
- **`rusqlite` with `bundled` is cheap to build here, not "a few minutes".**
  After `cargo clean -p libsqlite3-sys`, rebuilding `libsqlite3-sys` +
  `rusqlite` + `riffle-app` took **6.6s** on this machine. `timeout-minutes: 30`
  in `.github/workflows/ci.yml` was left alone; nothing suggests it is close.
- **`on_item` runs on rayon workers, so nothing in it may panic or poison.**
  `index::lock` takes every mutex with `unwrap_or_else(|e| e.into_inner())`, a
  failed batch write is counted into `errors` instead of unwrapping, and the
  scan's progress throttle is a `Mutex<Option<Instant>>` handled the same way.
  This is the contract Step 3's review wrote down, and it is why the scan code
  has no `unwrap` outside tests.
- **The cancel test could not drive the cancel flag from the progress
  callback**: progress is throttled to ~10/s, so a 200-file scan reports twice.
  The test spawns a watcher thread (`std::thread::scope`) that polls the row
  count and cancels once a batch has landed, then asserts that what was
  persisted equals what was scanned - which is the property the batching
  exists for.
- Design note: `Index::open` **discards a database written by a different
  `user_version`** rather than migrating. It is a cache; rebuilding it costs one
  scan, and migrations would be code nothing has yet needed.
- `payload` in `commands.rs` grew a `kind` parameter so `preview` and
  `thumbnail` share one envelope; `THUMBNAIL_KIND_JPEG_V1 = 2`.

### Not verified here

- **The progress line has not been seen running.** `pnpm tauri dev` needs the
  GUI and `osascript` assistive access is denied on this machine, so nothing
  drove the window. What *is* established is static: the generated
  `crates/app/gen/schemas/acl-manifests.json` shows `core:default` ->
  `core:event:default` -> `allow-listen`, so the capability is present. **The
  user must confirm in the running app** that `scanning 1234 / 5000` appears and
  counts up, that paging stays responsive during a scan, and that the second
  open of a real folder is fast.

## Step 5: the left filmstrip

- The layout gained a `#main` flex row (`#strip` + `#canvas`) inside the
  existing column, so `draw()` needed no change: it already fits to
  `canvas.clientWidth`, which now excludes the 160px column. `#canvas` needed
  `min-width: 0` next to its existing `min-height: 0`, otherwise the canvas'
  intrinsic width keeps the flex row from shrinking it.
- Orientation is applied with a CSS `transform: rotate()` on the `<img>`, not
  by re-encoding. Every cached thumbnail is 404x270 (DCT scale 2/8 of the
  1616x1080 preview), so one fixed 144x96 image box with `object-fit: contain`
  holds both a flat thumbnail (144x96) and a quarter-turned one (96x144)
  without changing the cell height the virtual list depends on. The empty box
  doubles as the placeholder: an `<img>` with no `src` but a fixed height still
  paints its background.
- Virtualisation is a spacer div (`#strip-inner`, height =
  `files.length * 176`) plus absolutely positioned cells, so `scrollTop ->
  index` is arithmetic. Releasing a cell revokes its object URL **and** clears
  its entry in `requested`, since the bytes are gone with the URL and a cell
  scrolled back in has to ask again.
- A `thumbnail` invoke for a file the scan has not reached is a plain `Err`
  (`no cached thumbnail`), not an empty success, so the strip has to treat a
  rejection as "not yet" rather than a failure: those indices go into
  `missing`, which `scan-progress` clears so the visible placeholders are
  re-requested. That is also why the catch is silent — one status-line error
  per not-yet-scanned file would be 5000 of them.
- Requests are capped at 4 in flight and picked nearest-to-viewport-centre, so
  scrolling fast fills what the user stopped on rather than everything it flew
  past. Responses for cells that scrolled out (or for a folder that was closed,
  via a `generation` counter mirroring `seq` in `requestPreview`) are dropped.
- Not verified here: everything behind the plan's **(manual)** criteria. GUI
  automation is denied on this machine, so thumbnails filling in during a scan,
  portrait cells being upright, the highlight following auto-repeat, click-to-
  page and 5000-file scroll smoothness are all for the user to confirm. What
  was checked statically: `tsc --noEmit` passes, the module is imported as
  `./strip.js`, no `window.__TAURI__` use moved into a worker, and the cell
  arithmetic (176px pitch, 168px cell) matches the CSS.

- Step 6: the focus box transform was verified without a GUI by comparing the
  canvas arithmetic against `riffle-cli focusbox` on the real portrait file
  (Orientation 8). The CLI drew its box centred at (400, 782) in the rotated
  1080x1616 PNG; scaling `FocusLocation` (3613, 1732) of the 7008x4672 sensor
  onto the unrotated 1616x1080 preview gives (833.2, 400.4), i.e. (+25, -140)
  from the image centre, and `context.rotate(-PI/2)` maps a local `(u, v)` to
  `(v, -u)` = (-140, -25) from the canvas centre — the same offset the PNG
  shows. So scaling first and letting the existing rotation carry the box is
  correct, and the two implementations are in step.
- Canvas `rotate(theta)` maps `(u, v)` to `(u cos - v sin, u sin + v cos)`; for
  the Orientation 8 case (`theta = -PI/2`) that is `(v, -u)`, a 90 degree CCW
  turn, which is what `apply_orientation` does to the pixel buffer in the CLI.
- The box is drawn inside the same `save`/`translate`/`scale(dpr)`/`rotate`
  block as `drawImage`, in the unrotated image's CSS-pixel space, so its stroke
  widths are DPR-correct for free.
- `folder_entries` was re-read on `scan-done` and, during a scan, only when the
  current file's row is still missing. Re-reading every row on each
  `scan-progress` event (~10/s) would be all 5000 rows ten times a second for
  one file's focus point.
- `f` does not collide with any key the phases claim (`1`-`5`, `x`, `Space`,
  `o`, and the paging keys `Arrow*`/WASD/HJKL).
- Not verified here: the plan's **(manual)** criterion, that the box sits on the
  subject's face in both orientations. GUI automation is denied on this machine.
  What was checked is the arithmetic above plus `mise run ci`.

## Deferred issues (todo candidates)

- Keyboard layout dependence of the WASD/HJKL bindings (from this step's
  implementation of the `keydown` handler in
  `crates/app/ui/src/main.ts`): the bindings follow `event.key`, so a Dvorak or
  AZERTY user gets scattered physical keys. Revisit only if a user asks; the
  fix would be a `event.code` fallback or a key-config layer, both out of scope
  now.
- `mozjpeg` panics instead of returning `Err` on malformed JPEG input. Handled
  for the scan in Step 3 (`catch_unwind` in `riffle_core::scan::extract`), but
  **`decode::decode_rgb` is still unguarded** for its other callers
  (`riffle-cli focusbox` / `bench` / `crop` in `crates/cli/src/main.rs`): a
  corrupt file aborts the CLI instead of reporting it. Out of scope for Step 3,
  which only owns the scan path.
- `riffle-cli` still reads the whole file in `info` / `focusbox` / `crop` /
  `bench` (`crates/cli/src/main.rs`). Those subcommands need the full-size
  `JpgFromRaw`, which is past the 1 MiB prefix, so moving them to
  `reader::read_head` was left out of this step deliberately.
- The scan does not re-run when files appear in a folder that is already open
  (from this step's `scan_folder` in `crates/app/src/commands.rs`): the index is
  reconciled only when a folder is opened. A watcher, or a rescan on refocus, is
  out of scope for Phase 3.
- The index file is never pruned or `VACUUM`ed (this step's
  `crates/app/src/index.rs`): rows of folders never opened again keep their
  ~20KB thumbnail forever, and deleted rows do not shrink the file. A size cap
  or an LRU eviction has no phase that owns it yet.
- `ping` in `crates/app/src/main.rs` is still orphaned; this step did not need
  to touch it, so it was left for its existing todo item.
- The strip re-requests every visible placeholder on each `scan-progress`
  event (~10/s) rather than using the `done` counter to ask only for indices
  the scan has passed (`crates/app/ui/src/strip.ts` `refresh`, driven from
  `crates/app/ui/src/main.ts`). Bounded by the 4-in-flight cap and by the
  visible range, but it is avoidable IPC; revisit if the strip turns out to be
  IPC-bound.
- The strip does not use the `folder_entries` rows at all (from this step): it
  discovers "has a thumbnail" by invoking `thumbnail` and seeing it fail.
  Step 6 introduces the entries map for the focus box, and the strip could then
  skip the doomed invokes for files with `has_thumb: false`.
- Cell geometry assumes every thumbnail is 3:2 (`crates/app/ui/style.css`
  `.cell img`). True for the current 404x270 pipeline; a body with a
  differently shaped IFD0 preview would letterbox inside the box (harmless,
  thanks to `object-fit: contain`) but waste cell space.
- The strip still discovers missing thumbnails by a failing `thumbnail` invoke
  rather than reading `has_thumb` from the entries map Step 6 introduced
  (`crates/app/ui/src/strip.ts`, `crates/app/ui/src/main.ts`). Taking it now
  would mean keeping the whole entries map fresh during a scan for the strip's
  sake, which is exactly the 10/s full re-read Step 6 avoided, so the existing
  todo item from Step 5 stands.
