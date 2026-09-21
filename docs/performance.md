# Performance

## Sony α7 V ARW (Apple Silicon Mac, n=20)

| Step | Target | Measured (median) |
|------|--------|-------------------|
| 1616x1080 preview extraction | 10ms | 4.4ms |
| JpgFromRaw full decode | 300ms | 84ms |
| 512px partial decode at the focus point | 50ms | 17.5ms |

## Leica M11-P DNG (Apple Silicon Mac, n=32)

Measured with release builds over 32 distinct real M11-P DNGs
(60.8MB to 78.0MB, 72.0MB mean), with the page cache warm (the files had been
read just before). Each DNG embeds a 2112x1408 preview and a 9504x6320 1:1
JPEG. The M11-P writes no `FocusLocation`, so the 512px crop is taken at the
image centre (row ~3160), the fallback the app uses.

`riffle-cli bench` (two runs, medians):

| Step | Target | Measured (median) |
|------|--------|-------------------|
| 2112x1408 preview decode | 10ms | 7.5-8.0ms (α7 V 1616x1080: 4.4ms) |
| 9504x6320 1:1 JPEG full decode | 300ms | 105ms (p95 125-130ms) |
| 512px partial decode at the centre | 50ms | 16.4-16.5ms (p95 ~20ms, max 22.7ms) |

The centre crop sits well inside the 50ms budget on its own; this is the CLI
decode only, not keypress to pixels.

`riffle-cli scan` over the folder:

| Threads | Total | Throughput | Per file on a worker (mean / p95) |
|---------|-------|------------|-----------------------------------|
| 1 | 0.39s | 83 files/s | 12.1ms / 14.1ms |
| 12 | 0.05s | 596 files/s | 17.3ms / 24.2ms |

Thumbnails (528x352): 30,231 bytes per file on average (967,419 bytes over
32 files), against ~19KB on the α7 V.

## Per-page preview read

The per-page cost on the Rust side only, on an Apple Silicon Mac.
**It excludes the IPC hop and `createImageBitmap`**. The app now logs the
end-to-end number per page turn (a `page …` line in `Riffle.log`; see
"Measuring on your own folder" below), but it has not been measured yet. Reading the whole 48MB ARW (`std::fs::read` plus `arw::parse`) is
compared with reading a bounded 1MiB prefix (`reader::read_preview`), which is
where the metadata and the embedded preview live:

| Folder | n | whole file | bounded prefix (Riffle) |
|--------|---|---------------------|------------------------|
| 5000 symlinks to one ARW, warm page cache | 300 | mean 6.6ms / p95 7.7ms | mean 0.06ms / p95 0.13ms |
| 20 distinct 48MB copies, first read | 20 | mean 16.0ms / p95 30.0ms | mean 5.1ms / p95 7.8ms |

Both columns were measured together, so they compare like with like. The symlink folder is 5000 symlinks to the same file and
the copy folder is 20 `cp` copies, read in file-name
order by a fresh process. The "first read" column cannot be guaranteed cold.

The whole-file read dominates, which is what the bounded read removes.

## Folder scan throughput

`riffle-cli scan` runs the folder extraction (bounded read, metadata parse,
404x270 thumbnail) over a folder on a dedicated rayon pool, with no database.
On a 12-core Apple Silicon Mac:

| Folder | threads | total | files/s |
|--------|---------|-------|---------|
| 5000 symlinks to one ARW, warm page cache | 1 | 34.9s | 143 |
| 5000 symlinks to one ARW, warm page cache | 8 | 5.35s | 935 |
| 5000 symlinks to one ARW, warm page cache | 12 | 4.46s | 1121 |
| 100 distinct 48MB copies, freshly written, page cache not guaranteed cold | 4 | 0.44s | 230 |
| 100 distinct 48MB copies, freshly written, page cache not guaranteed cold | 12 | 0.19s | 529 |

The thumbnails come to 19232 bytes each, i.e. ~96MB for 5000 files.
Extrapolating the freshly-written-copies column to 5000 files gives 9.5s at 12
threads and 21.7s at 4, but whether that holds on a real, cold-read folder is
not something these numbers establish. A single thread would not
make it (34.9s). More threads than cores does not help: 16 threads was flat
against 12 and doubled the per-file p95. On real cold reads this
extrapolation does not hold; see "Real folders on Windows" below.

What this cannot measure: a real 5000-distinct-file folder. Each
freshly-written-copies folder here is 100 `cp` copies scanned once, nothing is guaranteed cold, and the copies were
still likely warm in the page cache right after being written; the folders
were also scanned in the order they were written, which flatters the higher
thread counts. On a card reader or slow external disk the scan is disk-bound
regardless.

### Real folders on Windows

Measured on 2026-09-21 with the release app on Windows 11 Home, reading
real folders of Sony ARWs from a DRAM-less QLC SATA SSD (Crucial BX500 4TB)
with 22 extraction threads.
The numbers come from the `scan prepare` and `scan extract` lines of
`Riffle.log` (see "Measuring on your own folder" below). Each run used a
different folder:

| Run | files | prepare | extract | total | per file | files/s |
|-----|-------|---------|---------|-------|----------|---------|
| Cold first scan, Defender real-time protection on | 2677 | 1427ms | 42509ms | ~43.9s | ~16ms | ~61 |
| Cold first scan, folder excluded from Defender | 3401 | 639ms | 65605ms | ~66.2s | ~19ms | ~51 |
| First scan, warm page cache (index cleared, files read moments before) | 2134 | 47ms | 2084ms | ~2.1s | ~1ms | ~1000 |

Both cold runs finished with 0 errors. Excluding the folder from Defender did
not make the scan faster, so Defender is not the cause; the warm run shows
the CPU side costs ~1ms per file, so cold IO dominates. Extrapolated to 5000
files, a cold first scan takes ~82-97s. At a 1MiB bounded read per file and
~16ms per file, the effective throughput works out to ~63MB/s (derived, not
measured).

A thread-count sweep pins the cause on the drive. Measured on 2026-09-22 on
the same Windows 11 machine (i7-13700) with a release `riffle-cli scan`
cross-built for `x86_64-pc-windows-gnu`; each run used a different real
folder of Sony ARWs never read since boot, on the same drive, and all runs
finished with 0 errors:

| threads | files | total | files/s | per file on a worker (mean / p95) |
|---------|-------|-------|---------|-----------------------------------|
| 1 | 1337 | 41.08s | 33 | 30.7ms / 58.2ms |
| 4 | 1415 | 27.34s | 52 | 77.0ms / 103.5ms |
| 8 | 1520 | 29.47s | 52 | 154.9ms / 196.8ms |
| 22 | 1545 | 22.90s | 67 | 324.2ms / 369.5ms |

Throughput plateaus at ~50-67 files/s from 4 threads on, while the per-file
time on a worker roughly doubles with each doubling of threads: the workers
queue on one shared resource, the drive. A DRAM-less QLC SATA SSD is slow at
cold random reads: with no DRAM for the mapping table it needs extra NAND
reads, QLC read latency is high (worse for data written long ago), and SATA
has a single command queue. On one thread a file costs ~30ms, of which an
estimated ~10ms is CPU (from the warm run) and ~20ms is waiting on the drive.
The cause is the drive, not Riffle's code, and more threads help little.

The boot NVMe drive in the same machine (Crucial P5, TLC with DRAM) was not
measured, so there is no NVMe-vs-SATA comparison. The expected workaround,
not a measurement, is to keep the folders being culled on an NVMe drive,
internal or in an external USB NVMe enclosure with UASP; even over 5Gbps USB
it should be several times faster.

After a cache clear, the log once showed a `scan_id` superseded by the next
one with no `scan extract` line for the first (`todo.md`: "App: a scan can be
started twice after a cache clear / focus rescan").

### Sharpness scoring cost

Scoring sharpness adds a grayscale decode of the embedded preview to each
file's extraction. `riffle-cli scan`, release build, over 1000 symlinks to one
α7 V ARW (1616x1080 preview), warm page cache, on a 12-core Apple Silicon Mac,
runs alternated before and after:

| Threads | Runs | Per file on a worker before (mean / p95) | After (mean / p95) |
|---------|------|------------------------------------------|--------------------|
| 1 | 2 | 7.2-7.5ms / 7.6-7.9ms | 10.4-10.7ms / 10.9-11.2ms |
| 8 | 3 | 8.6-9.1ms / 12.1-13.3ms | 12.3-17.1ms / 15.7-22.0ms |

About +3.2ms per file on one thread (~+45%). The symlinks repeat one file, so
this is CPU cost with no IO variety.

## Opening an indexed folder again

The second open of a fully indexed folder does no extraction: it stats every
file, reconciles the rows and queries them. On the 5000-file folder:

| Step | Target | Measured |
|------|--------|----------|
| First open, full scan (5000 files, 10 threads, 0 errors) | - | 5.55s |
| Second open (stat + reconcile + query, fully indexed) [^1] | 3s | 34.4ms |

Both rows were measured on **5000 symlinks pointing at one real ARW, with a
warm page cache**, so they carry the same caveat as the tables above. The
second open is a stat-and-query number, which the symlinks flatter less than
they flatter a read benchmark, but 5000 lookups of one cached inode's metadata
is still cheaper than 5000 distinct 48MB files' metadata on a card. The first
scan is the same folder and procedure as the scan throughput table above, at
10 threads, a thread count that table does not have a row for; for a real
folder of distinct files, see the Windows numbers below and "Real folders on
Windows" above; on a card reader or a slow external disk the first scan is
disk-bound regardless.

[^1]: Measured with a temporary `#[ignore]`d test that was removed before
committing, so this number is not reproducible from the committed tree.

On a real folder of 2677 Sony ARWs on the same SATA SSD (Windows 11 Home,
release app, 2026-09-21, from `Riffle.log`):

| Step | Target | Measured |
|------|--------|----------|
| Second open after a relaunch (`open list` 34ms + `open entries` 56ms + `scan prepare` 73ms) | 3s | ~163ms |
| Focus rescan of the unchanged folder (`scan prepare`) | - | ~65ms |

The focus rescan is not noticeable, so the watcher's debounce does not need
to grow for it. Two anomalies showed up in the log: each open called
`open entries` twice (56ms and 76ms), and the second call is not counted in
the ~163ms above (`todo.md`: "App: `open entries` is called twice per folder
open"); and two focus rescans once fired at the same instant (`todo.md`:
"App: a scan can be started twice after a cache clear / focus rescan").

### Measuring on your own folder

Riffle logs its own scan timings, so these numbers can be reproduced on any
folder. Clear the cache in `Riffle > Settings...` (the `Cache` tab) so the next
open counts as a first scan, and reopen the folder. Then quit and launch again
to get the second open, and use `Help > Open Log Folder` to find `Riffle.log`.

Every open writes `open list`, `open entries`, `scan list` (reading the
folder), `scan reconcile` (stat-ing the files against the index),
`scan sidecars`, `scan prepare` (the sum of those three) and `scan extract`,
which carries the whole extraction pass with its `files`, `done`, `errors`
and `threads` counts. The first scan's `scan extract` has `files` > 0; `scan
prepare` plus that `scan extract` is the first-scan total.

The second open writes the same lines but with `todo=0`, and its
`scan extract` reads `files=0 done=0` with a near-zero duration — that
`files=0` line, not its absence, is what marks a cached open. It spans three
separate calls, so there is no single number for it: add the `open list`,
`open entries` and `scan prepare` lines of that open. Every line ends in
`in <n>ms` and names its directory, and the timestamps tell the two runs
apart.

Per-page latency is logged the same way. Turn on `Timing logs` in the settings
window (the item only shows in a `mise run tauri:release:devtools` or debug
build), page through a folder with the next/previous keys, and read the
`page invoke=… decode=… total=… keypressToPixels=…` lines in `Riffle.log`.
`invoke` is the `preview` call's round trip (the Rust read plus IPC), `decode`
is the worker's `createImageBitmap` round trip, `total` runs from the request
to the drawn canvas, and `keypressToPixels` from the keypress to the drawn
canvas; it is left out for a page not asked for by a key (a strip click or a
folder open). A page turn overtaken by the next one logs nothing.

## The 1:1 focus check path

The Rust side of one zoom keypress, on one α7 V ARW (Orientation 8,
`JpgFromRaw` 7008x4672 baseline 4:2:2, 5.76MB) on an Apple Silicon Mac. **One real file, warm
page cache, in-process, release build, n=20, medians.** Measured against the
functions the CLI and the app both call.
**The IPC hop and `createImageBitmap` are excluded** — they could not be
measured headlessly; the end-to-end numbers the user measured by hand are in
the next table.

| Step | Median |
|------|--------|
| Ranged read of the 5.76MB `JpgFromRaw` (`reader::read_full`) | 0.7ms |
| Crop at the focus point, 512 / 1024 / 2048 per axis (`partial::decode_focus_crop`, RGBA) | 18.4 / 21.5 / 29.8ms |
| 1024 crop at row 300 / 2336 / 4400 of the 4672-row JPEG (`partial::decode_crop`, RGB) | 10.8 / 27.3 / 44.1ms |

Crop size barely matters; the crop's **row** dominates. `jpeg_skip_scanlines`
on a baseline JPEG still entropy-decodes every skipped row, so a focus point
low in the frame costs four times one at the top, and 44ms leaves nothing of
the 50ms budget for the IPC hop and the bitmap. The payload is raw RGBA (4.2MB
at the 1024 cap) rather than a re-encoded JPEG because re-encoding that crop
with `mozjpeg::Compress`'s defaults measured 49ms at q85 — more
than the decode it follows.

### End to end, keypress to pixels

Measured by hand in the running app. **Conditions**:
optimised build (`mise run tauri:release:devtools`), `Debug > Timing logs` on, **DevTools
open** (a webview can be slower with the inspector attached, so these may be
upper bounds), warm page cache (the folder had been opened before), one real
folder of Sony ARW files. `read` and `decode` come
from the `focus_crop` payload header (`Instant` inside `spawn_blocking`), the
rest from `performance.now()` on the frontend; **`ipc` is derived** as the
invoke elapsed minus `read` minus `decode`, so it is everything else on the
Rust side plus transport, not pure transport.

n=8 crops across 3 `z` presses:

| Measurement | n | Measured |
|-------------|---|----------|
| Keypress → pixels, for the crop the keypress itself asked for | 3 | 39 / 45 / 40ms |
| `read` | 8 | 1.8-2.7ms |
| `decode` | 8 | 21.9-38.7ms |
| `ipc` (derived) | 8 | 2.3-3.7ms |
| `bitmap` | 8 | 0-1ms |
| Total per crop | 8 | 26-45ms |

`decode` dominates. `read`, `ipc` and `bitmap` are noise beside it, so the
raw-RGBA-over-IPC decision (a 4MB payload at the 1024 cap) costs a few
milliseconds: **IPC is not the bottleneck**.

## What the sidecar pass adds to a folder open

Opening a folder also reconciles the XMP sidecars: one listing of the
directory, a `stat` per sidecar found, and a parse of only those whose
`(size, mtime)` changed since the app last saw them.

| Second open of a 5000-file folder | Measured (median) |
|-----------------------------------|-------------------|
| No sidecars in the folder | 31.3ms |
| 5000 sidecars, all with an unchanged stat | 67.4ms |

**These two numbers are not comparable to the 34.4ms above and do not replace
it.** They were measured on a different folder: 5000 one-KB regular files
named `*.ARW`, not symlinks and not real ARWs, on a local APFS disk with a
warm page cache, on an Apple Silicon Mac. The work timed is the same
sequence a folder open does on the Rust side — list, `stat` every file,
`reconcile`, reconcile the sidecars, `entries` — in a temporary `#[ignore]`d
test (release profile, median of 7 runs after 3 warm-up runs) that was
removed before committing, so neither number is reproducible from the
committed tree. What they do establish is the shape of the cost: on this
machine the sidecar pass roughly doubles a second open, adding about 36ms
for 5000 sidecars, and that cost is a listing plus a `stat` each, not a
parse, because an unchanged stat parses nothing. A first open of a folder
full of foreign sidecars pays the parse as well and was not measured.
