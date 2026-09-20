# Riffle

A culling app for Sony ARW and Leica DNG files: look through them fast, apply
ratings, and mark picks and rejects. Nothing else — no developing, no editing.
It never runs a RAW decoder (LibRaw / rawler); everything comes from the JPEGs
already embedded in the ARW or DNG.

## Compatibility

### RAW formats

- ARW
  - [x] Sony α7 V
- DNG
  - [x] Leica M11-P
  - [ ] Sigma BF
  - [ ] Sigma fp L

### Sidecar formats

Which tools read the sidecars Riffle writes; see
[Ratings and sidecars](#ratings-and-sidecars).

- XMP
  - [ ] Adobe Lightroom Classic
  - [ ] Capture One
- DOP
  - [x] DxO PhotoLab 10

### Requesting support

Requests for any camera, RAW format or developing software are welcome;
[open an issue](https://github.com/minodisk/riffle/issues). Since Riffle only
reads the JPEGs embedded in a RAW, attach a sample file from the camera (or
link to one) so they can be checked. One file is enough to start; a landscape
and a portrait shot together also let the rotation and the focus mark be
checked.

## Features

Open a folder from the picker, or drop a folder or any file in it onto the
window. Riffle lists the ARW and DNG files in it and pages through their
embedded previews, rotated by each file's Orientation. With no folder open the
viewer shows a prompt in its centre; click it to open the folder picker.

- **Filmstrip**: thumbnails run down the left edge, follow paging and show the
  file you click. The `N / M` counter sits under it.
- **Focus mark**: `f` draws a crosshair at the camera's recorded focus point
  (hidden by default; cameras that record none, such as the M11-P, show none).
- **1:1 focus check**: `z` shows the full-resolution image at one pixel per
  screen pixel, centred on the focus point (or the frame centre without one).
  Paging while zoomed stays zoomed and moves to the next file's focus point.
  There is no panning or free zoom.
- **Judgements**: stars, reject, (with `.dop`) pick and a colour label, shown on
  the strip cell (the label tints the file-name band along the cell's bottom
  edge) and written to a sidecar; see
  [Ratings and sidecars](#ratings-and-sidecars).
- **Meta pane**: camera, lens, shutter, aperture, ISO and focal length. When a
  lens reports no f-number (the M11-P with an M-mount lens), the aperture is the
  camera's estimate, marked `(est.)`; Leica files add the focus distance.
- **Filter menu**: narrows the strip by pick flag, stars, colour label, camera,
  lens, aperture, shutter speed, ISO and focal length (grouped into ranges such
  as `24–35 mm`). The colour label group lists the seven colours and `No label`;
  a label outside those seven colours matches no colour item (nor `No label`).
  The EXIF groups list only values present in the folder. Checks within a group
  are OR-ed, groups are AND-ed, and `Reset` clears them all. A judgement that drops
  the current file out of the filter hides it at once and moves to the next
  passing file after it, else the last one before it, else the empty view.
- **Sort menu**: orders the strip by file name, capture time or rating;
  paging and `n / N` follow the chosen order. Capture time breaks ties by
  sub-second then file name, and files without a capture time come last.
  Rating puts higher stars first, then unrated files, then rejects, with file
  name breaking ties.
  The order settles while a folder is scanned for the first time, and the
  choice is remembered across launches.
- **Open Folder…**: `File > Open Folder…` opens the folder picker, the same as
  the `open` key.
- **Open in DxO PhotoLab**: `File > Open in DxO PhotoLab` hands the open
  folder to the newest PhotoLab in `/Applications`. Both File menu items are
  rebindable: each shows the first key of its action that can be an
  accelerator, and none when the action holds no such key.
- **Reload Folder**: the open folder keeps up with the disk on its own: a file
  copied in or deleted outside Riffle shows up in the strip about a second
  later. On volumes that send no change notifications, such as network shares,
  bring the window back to the front or choose `File > Reload Folder`
  (`CmdOrCtrl+R`) to do the same. Only what changed is read again; the stars,
  flags and colour labels already given, the current file and the strip's
  position all stay as they were.
- **Undo**: `Edit > Undo` (`CmdOrCtrl+Z`) restores the rating, flag and colour
  label the last judged file had before, writes that to its sidecar and returns
  to the file (unless the filter now hides it, which the status line says).
  Repeated presses walk further back. The history belongs to the open folder
  and is cleared when another folder opens or the sidecar format changes. There
  is no redo.
- **Open Log Folder**: `Help > Open Log Folder` reveals the folder holding
  `Riffle.log`, the app's log file (capped at 1 MB; on rotation the previous
  contents are discarded, not kept as a separate file). It lives in the app
  log directory:
  `%LOCALAPPDATA%\com.minodisk.riffle\logs\` on Windows,
  `~/Library/Logs/com.minodisk.riffle/` on macOS and
  `~/.local/share/com.minodisk.riffle/logs/` on Linux.
- **Auto-advance**: when `Auto-advance after a star, reject or pick` is on in
  `Riffle > Settings...` (off by default), `1`-`5`, reject and pick move to the
  next file once they change the current one; pressing the value the file
  already has does not. The last file stays selected. A file the judgement
  drops out of the active filter already hands the cursor to the next file, so
  it is not skipped twice.
- **Clear Cache**: the `Cache` tab of `Riffle > Settings...` shows how much disk
  the folder index takes, and `Clear Cache` empties it after a confirmation. It
  removes the cached thumbnails and metadata of every folder ever opened; your
  judgements are not touched, because they live in the sidecars. The size is in
  the same units your file manager uses (decimal on macOS, binary on Windows
  and Linux), and a scan has to finish before the cache can be cleared.
- **Sharpness cue**: a thin bar up the left edge of each strip cell shows how
  sharp the frame is next to its neighbours on the strip; the sharpest frame of
  a run is marked in the pick colour. The score is computed around the focus
  point (or the centre) from the embedded preview, so it ranks a burst rather
  than judging a frame on its own, and it does not replace the 1:1 focus check.
  The meta pane shows the raw score.

### Keys

| Key | Action |
|-----|--------|
| `ArrowUp` | previous file |
| `ArrowDown` | next file |
| `Cmd+O` / `Ctrl+O` | open a folder (`File > Open Folder…`) |
| `Shift+Cmd+O` / `Ctrl+Shift+O` | open the folder in DxO PhotoLab (`File > Open in DxO PhotoLab`) |
| `f` | toggle the focus mark |
| `z` | toggle the 1:1 focus check |
| `1`-`5` | rate the current file that many stars |
| `x` | reject the current file (replaces a pick) |
| `p` | pick the current file (`.dop` only; replaces a reject, keeps the stars) |
| `u` | un-reject or un-pick the current file |
| `0` | clear the rating or the reject (a pick stays) |
| `CmdOrCtrl+Z` | undo the last judgement (the `Edit > Undo` accelerator; not rebindable) |

Pressing the key of the label the file already has clears it; the stars, the
flag and `0` leave the label alone.

| Label | Key |
|-------|-----|
| red | `Ctrl+Alt+1` |
| orange | `Ctrl+Alt+2` |
| yellow | `Ctrl+Alt+3` |
| green | `Ctrl+Alt+4` |
| blue | `Ctrl+Alt+5` |
| pink | `Ctrl+Alt+6` |
| purple | `Ctrl+Alt+7` |
| clear the label | `Ctrl+Alt+0` |

Keys can be changed from `Riffle > Settings...` (`CmdOrCtrl+,`):
click a row's `+` and press a key to add it (`Escape` cancels), or click the
`×` on a key to remove it. The last key of an action cannot be removed (use
Reset). A key already used by another action is refused, and a modifier pressed alone is ignored. Any combination of
Ctrl, Alt (Option), Shift and Cmd (Windows / Super) can be bound; it is stored
as `ctrl+alt+shift+meta+` with only the modifiers held, and the key named from
its physical key, so `ctrl+alt+1` stays `1` though Option changes the typed
character on macOS. Shift counts, so Shift+J is a different key from J.
Combinations the system or the app's menu already use (`Cmd+Q`, `Cmd+Z`,
`Cmd+Tab`, `Ctrl+C` on Windows, any Windows-key combination, ...) are refused.
The two File menu accelerators are the exception: they follow their own
action's keys, so unlike `Cmd+Z` and `Cmd+,` they can be rebound, and the
combination an action leaves behind is free for another action.
`Reset all` restores the defaults.

### Ratings and sidecars

The RAW file is never written. Judgements go into a sidecar next to it, in one
of two formats chosen in `Riffle > Settings...`:

- **XMP** (default): `FOO.ARW` gets `FOO.xmp`, holding `xmp:Rating` — `0`-`5`,
  or `-1` for a reject — and `xmp:Label`, Lightroom's colour label (`Red`,
  `Yellow`, `Green`, `Blue`, `Purple`). XMP has no pick.
- **DxO PhotoLab**: `FOO.ARW` gets `FOO.ARW.dop`, holding the stars, the
  pick / reject flag and the `ColorLabel` line (`Red`, `Orange`, `Yellow`,
  `Green`, `Blue`, `Pink`, `Purple`), which PhotoLab 10 reads.

Clearing a label removes `xmp:Label` or the `ColorLabel` line; no label is the
field being absent. The label is kept as the exact string the sidecar holds: a
name from the other tool's vocabulary is written back unchanged and shown in
its colour, and any other name (say, a custom Lightroom label) is shown grey.

A sidecar written by another tool is edited in place: only the rating, the
label (and, for `.dop`, the flag) change, and everything else — develop
settings, keywords — is kept byte for byte. Clearing a file that has no sidecar
creates none.

Writes happen in the background and are atomic, so a crash never leaves a
half-written sidecar, and quitting finishes any pending write. A judgement that
could not be written (say, on a locked card) is retried a few times over the
next half minute, and if it still fails it is kept and retried the next time
the folder is opened; the error is shown at the bottom of the right pane until
you dismiss it, and so is any sidecar the app cannot read when a folder opens
(damaged, or larger than 4 MiB). Sidecars edited by another tool are picked up the next
time the folder is opened; when both changed, the other tool's edit wins.
Switching the format keeps unwritten judgements and writes them in the new
format; the other format's files are left alone.

The folder index is a cache, but it also holds unwritten judgements, so a new
index schema migrates the previous ones in place instead of discarding them;
only a version it cannot migrate is dropped and rebuilt from the sidecars.

Reading `-1` back is up to the other tool: exiftool documents it as
"rejected", Adobe Bridge and darktable use it, and Lightroom Classic is
reported to read it as a reject on import.

## Installing

Download the installer for your OS from the latest release on the
[Releases page](https://github.com/minodisk/riffle/releases):

| OS | File |
|----|------|
| macOS, Apple Silicon | `Riffle_<version>_aarch64.dmg` |
| macOS, Intel | `Riffle_<version>_x64.dmg` |
| Windows | `Riffle_<version>_x64-setup.exe` (or `Riffle_<version>_x64_en-US.msi`) |
| Linux | `Riffle_<version>_amd64.AppImage` (or `Riffle_<version>_amd64.deb` / `Riffle-<version>-1.x86_64.rpm`) |

The builds are not OS-signed (no Apple notarization, no Authenticode), so the
first launch needs one extra step:

- **macOS**: Gatekeeper blocks the first launch. Right-click `Riffle.app` →
  Open, or System Settings → Privacy & Security → Open Anyway, or run
  `xattr -d com.apple.quarantine /Applications/Riffle.app`.
- **Windows**: SmartScreen warns. More info → Run anyway.
- **Linux**: nothing extra.

Updating: the app checks for a newer release on launch and, when there is one,
downloads and installs it quietly in the background; the new version is used the
next time Riffle launches. On Windows the download still happens quietly in the
background, and the installer runs when Riffle quits, so the next launch is the
new version. **Check for Updates…** in the app menu (the File menu
on Windows and Linux) runs the same check by hand and reports the outcome. The update itself is signed with the
project's updater key and verified before it is installed. On Linux only the
AppImage updates itself; a `.deb` / `.rpm` install is updated by installing the
newer package.

To build from source, see [CONTRIBUTING.md](./CONTRIBUTING.md).

## Performance

### Sony α7 V ARW (Apple Silicon Mac, n=20)

| Step | Target | Measured (median) |
|------|--------|-------------------|
| 1616x1080 preview extraction | 10ms | 4.4ms |
| JpgFromRaw full decode | 300ms | 84ms |
| 512px partial decode at the focus point | 50ms | 17.5ms |

### Leica M11-P DNG (Apple Silicon Mac, n=32)

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

### Per-page preview read

The per-page cost on the Rust side only, on an Apple Silicon Mac.
**It excludes the IPC hop and `createImageBitmap`**, which could not be
measured. Reading the whole 48MB ARW (`std::fs::read` plus `arw::parse`) is
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

### Folder scan throughput

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
threads and 21.7s at 4; the CPU cost leaves ~25s of headroom against the
30-second target, but whether that headroom survives on a real, cold-read
folder is not something these numbers establish. A single thread would not
make it (34.9s). More threads than cores does not help: 16 threads was flat
against 12 and doubled the per-file p95.

What this cannot measure: a real 5000-distinct-file folder. Each
freshly-written-copies folder here is 100 `cp` copies scanned once, nothing is guaranteed cold, and the copies were
still likely warm in the page cache right after being written; the folders
were also scanned in the order they were written, which flatters the higher
thread counts. On a card reader or slow external disk the scan is disk-bound
regardless.

#### Sharpness scoring cost

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

### Opening an indexed folder again

The second open of a fully indexed folder does no extraction: it stats every
file, reconciles the rows and queries them. On the 5000-file folder:

| Step | Target | Measured |
|------|--------|----------|
| First open, full scan (5000 files, 10 threads, 0 errors) | 30s | 5.55s |
| Second open (stat + reconcile + query, fully indexed) [^1] | 3s | 34.4ms |

Both rows were measured on **5000 symlinks pointing at one real ARW, with a
warm page cache**, so they carry the same caveat as the tables above. The
second open is a stat-and-query number, which the symlinks flatter less than
they flatter a read benchmark, but 5000 lookups of one cached inode's metadata
is still cheaper than 5000 distinct 48MB files' metadata on a card. The first
scan is the same folder and procedure as the scan throughput table above, at
10 threads, a thread count that table does not have a row for; and the real
number on a real folder of 5000 distinct files has never been measured by
anyone; on a card reader or a slow external disk the first scan is disk-bound
regardless.

[^1]: Measured with a temporary `#[ignore]`d test that was removed before
committing, so this number is not reproducible from the committed tree.

#### Measuring on your own folder

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

### The 1:1 focus check path

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

#### End to end, keypress to pixels

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

### What the sidecar pass adds to a folder open

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
