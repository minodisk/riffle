# Riffle

A culling app for Sony ARW files: look through them fast, apply ratings, and
mark picks and rejects. Nothing else — no developing, no editing. It never runs
a RAW decoder (LibRaw / rawler); everything comes from the JPEGs already
embedded in the ARW.

## Status

Phase 0.5, Phase 1 (the CLI benchmark), Phase 2 (the app skeleton), Phase 3
(the folder index, the filmstrip and the focus mark), Phase 5 (the 1:1 focus
check) and Phase 6 (ratings, the reject flag and XMP sidecars) are done.

The app opens a folder — through the picker or by dropping a folder, or a
single file (any existing file resolves to its parent folder), onto the
window — lists the ARW files in it, shows one embedded preview on
a `<canvas>` (decoded in a worker, rotated by the ARW's Orientation), and pages
through them. A thumbnail filmstrip runs down the left edge: it is virtualised,
highlights the current file, scrolls to follow paging, and a click on a cell
shows that file. When the index has a `FocusLocation` for the current file, a
focus mark can be drawn over the preview: a crosshair, since the tag records a
point rather than an AF rectangle. It is hidden by default, `f` toggles it, and
it is placed in unrotated sensor coordinates and rotated with the
image. `Space` toggles a 1:1 focus check. `1`-`5`, `x`, `u` and `0` record a
judgement, shown on the strip cell and in the meta pane's sidecar section and
written to an XMP sidecar next to the RAW. The canvas shows only the image; the
`N / M` counter sits in the strip pane, under the filmstrip.
Still missing: no prefetch, no filtering by rating.
See [Running the app](#running-the-app).

Releases are published on the
[Releases page](https://github.com/minodisk/riffle/releases); see
[Installing](#installing); the first one is
[v0.1.0](https://github.com/minodisk/riffle/releases/tag/v0.1.0). Updating an
installed build to a newer release is **awaiting the user's confirmation**: it
needs a second release to check.

Keys:

| Key | Action |
|-----|--------|
| `ArrowLeft`, `ArrowUp`, `w`, `a`, `h`, `k` | previous file |
| `ArrowRight`, `ArrowDown`, `s`, `d`, `j`, `l` | next file |
| `o` | open a folder |
| `f` | toggle the focus mark |
| `Space` | toggle the 1:1 focus check |
| `1`-`5` | rate the current file that many stars |
| `x` | reject the current file (sticky, not a toggle) |
| `u` | un-reject the current file (does nothing unless it is rejected) |
| `0` | clear the rating or the reject |

### The 1:1 focus check

`Space` toggles a third tier on top of the 400px thumbnails and the 1616x1080
preview: a crop of the full-resolution `JpgFromRaw`, partially decoded out of
the ARW with a ranged read, drawn at one JPEG pixel per device pixel. The crop
is centred on the camera's `FocusLocation` (mapped from sensor coordinates onto
the full JPEG), and on a file without one — manual focus — on the centre of the
frame. It is cut in unrotated coordinates and carried by the same canvas
rotation as the preview, so a portrait file comes out upright. On a file with a
`FocusLocation`, the preview bitmap is drawn around the crop at the same scale,
so the frame stays in context while the crop is decoded; on a file without one,
the canvas stays blank until the crop arrives. Paging while zoomed stays zoomed
and moves to the next file's focus point. There is no panning and no free zoom
level; the crop
is capped at 1024 device pixels per axis and travels over the IPC boundary as
raw RGBA.

Keys held with Cmd/Ctrl/Alt are left to the system. Letter keys are matched
lower-cased, so Shift+J pages like `j`.

### The index

On the first open of a folder, every ARW in it is extracted in parallel
(capture time, `SubSecTimeOriginal`, `FocusLocation`, Orientation and a 404x270
thumbnail, from a bounded 1MiB prefix plus a ranged read of the preview
itself) into a SQLite database. The status
line shows `scanning N / M` while that runs; the first preview does not wait
for it. The database lives in the app cache directory:

- **macOS**: `~/Library/Caches/com.minodisk.riffle/index.sqlite`
- **Windows**: `%LOCALAPPDATA%\com.minodisk.riffle\index.sqlite`

One table holds one row per file path, keyed by the absolute path, with the
metadata above and the thumbnail as a JPEG BLOB. A row is valid for a file iff
its stored `size` and `mtime_ns` still match the file's current `stat`;
anything else is re-extracted. Since the path is the key, renaming a folder
re-scans it and leaves the old rows behind — and deleting rows does not shrink
the database file without `VACUUM`, which nothing runs yet. On the 5000-symlink
folder used for the measurements below, the database came to 104,177,664 bytes
(~20.8KB per row, mostly thumbnail); since every row there is a byte-identical
thumbnail of the same file, a real folder of distinct frames will not be
exactly this. It is a cache:
deleting the file costs one more scan.

The CLI from Phase 1:

```sh
cargo build --release
./target/release/riffle-cli info     <file.ARW>            # where the embedded JPEGs are
./target/release/riffle-cli focusbox <file.ARW> <out.png>  # draw the focus box on the preview
./target/release/riffle-cli crop     <file.ARW> <out.png> [size]  # partially decode the focus point at 1:1
./target/release/riffle-cli bench    <file.ARW>...         # measure decode speed
./target/release/riffle-cli scan     <dir> [threads]       # extract a whole folder in parallel
```

### Ratings and XMP sidecars

`1`-`5` set a star rating, `x` marks a reject, `u` un-rejects and `0` clears
either. The RAW file is never written. The judgement goes into a standard XMP
sidecar next to it — `FOO.ARW` gets `FOO.xmp` (an existing sidecar differing
only in case, say `FOO.XMP`, is used instead of a second file being created) —
as a single property, `xmp:Rating`, holding `0`-`5` or `-1` for a reject.
Nothing else is written: no colour label, no pick flag, no private namespace.

The judgement is shown in two places, with the same glyphs and colours: the
strip cell's badge (yellow stars, or a red `✕` on a dimmed cell for a reject)
and the last section of the meta pane, set apart from the EXIF rows by a rule.
That section is headed by the sidecar's name, with `(not created)` when the
file is known to have none, and holds a `Rating` row (stars, `✕`, or `–` when
unrated). The header shows the name the app would write, so a foreign
`FOO.XMP` reads as `FOO.xmp`; the writer still patches `FOO.XMP`. Whether a
sidecar exists is as the app last read or wrote it, not a live stat; a rating
key marks it as existing at once.

A sidecar the app created is a small RDF/XML template. A sidecar another
tool wrote is **patched in place, never regenerated**: the `xmp:Rating` value
is replaced byte for byte (in either legal shape, the attribute
`xmp:Rating="3"` or the element `<xmp:Rating>3</xmp:Rating>`, under whichever
prefix is bound to the XMP namespace), or one attribute is inserted into the
first `rdf:Description` when the property is absent. Everything else in the
file — a Lightroom sidecar's kilobytes of `crs:` develop settings, keywords,
history — stays byte-identical, which a test asserts. A sidecar that is not
parseable XML, or that has no `rdf:Description`, is left alone: writing to it
fails and is reported through the `sidecar-error` event, but reading it on
folder open fails silently (no event, no log), leaving its row as it was.

Clearing on a file that has no sidecar writes nothing rather than creating an
empty one, so a folder is not littered with 5000 sidecars for files that were
never rated. Clearing on a file that does have one writes `xmp:Rating="0"`, so
a foreign sidecar's stars are actually cleared.

**The sidecar is the source of truth; the index is a cache with a write-ahead
role.** The rating lives in the sidecar for other tools to read; the SQLite
index keeps a copy so a folder opens without parsing 5000 files, plus a
`dirty` flag marking a judgement that has not reached its sidecar yet. Each
row also stores the `(size, mtime)` of the sidecar as the app last read or
wrote it. On every folder open, each file is reconciled by six rules, and then
every row still dirty after that pass is handed to the writer as a separate
step:

1. sidecar present, its `(size, mtime)` differs from the stored pair, the row
   is not dirty — parse it and take its rating (an external edit wins; this is
   also the first open of a folder Lightroom has rated, where there is no row
   at all);
2. sidecar present, differs, and the row **is** dirty — the sidecar still
   wins and `dirty` is cleared. A sidecar that changed under us is read, never
   silently overwritten; the cost is one unwritten keypress, on one file, and
   it is visible on screen;
3. sidecar present, `(size, mtime)` unchanged — nothing is parsed (the fast
   path for 5000 files);
4. sidecar absent and the row is dirty — write it now (the crash-recovery
   path);
5. sidecar absent, the row is not dirty, but a stat was stored — the sidecar
   was deleted outside the app, so the rating is cleared too (the truth is
   gone);
6. sidecar absent and nothing was ever stored — nothing.

A sidecar past `MAX_SIDECAR_BYTES` (4 MiB) is a seventh case outside these
rules, but only on the folder-open pass: it is neither parsed nor handed to
the writer as a dirty row, since a rating that was never read back cannot be
patched in without clobbering unread content. A rating set while the folder
is already open still goes straight through `sidecar::write`, which reads and
patches the sidecar regardless of its size.

An external edit made while the folder is open is not noticed; there is no
watcher, so it is picked up on the next open.

Writing is asynchronous and coalesced. A keypress updates the screen first,
then records the rating in the index with `dirty = 1` and hands the file to a
single writer thread, which waits **300 ms after the last change to that
file** before writing — mashing `1`, `2`, `3` on one file produces one write,
containing `3`. The write goes to `FOO.xmp.riffle-tmp` in the same directory,
is `fsync`ed and then renamed over `FOO.xmp`, which is atomic on APFS (the
only filesystem this has been run on). Windows is unverified here, but
`std::fs::rename` is reported to go through `MoveFileEx` with the replace
flag, which offers the same guarantee on NTFS. A crash mid-write therefore
leaves the previous, well-formed sidecar plus a stray temp file, never a
truncated one. On quit, the writer is drained synchronously (bounded at about
two seconds), so a normal Cmd+Q loses nothing.

What a kill during the debounce window costs: `kill -9` or a crash in those
300 ms loses the sidecar write but **not** the judgement, unless the sidecar
changed under us in the meantime, in which case rule 2 applies and the
sidecar wins instead. Otherwise the row stays dirty, and the next time that
folder is opened every dirty row is handed to the writer, which writes it.
Only a death between the keypress and the index write (a few milliseconds)
loses the judgement itself. A sidecar directory that cannot be written (a
locked card, a read-only share) fails the write itself — `File::create` on
the temp file errors before any rename is attempted — leaves the row dirty,
shows one message in the status line, and is retried on the next open of that
folder. Discarding the index loses only the dirty rows not yet written;
everything else is in the sidecars.

Which tools read `-1` back is a claim about those tools, and this repository
has verified none of it. What is *reported*: exiftool's XMP tag reference
documents `xmp:Rating` as a value from 0 to 5, or -1 for "rejected"; Adobe
Bridge writes `-1` for a rejected file and darktable reads and writes it;
Lightroom Classic does not write pick/reject flags to XMP at all (they are
catalog-only) but is reported to read `-1` as a reject on import. **The user's
downstream tools are Lightroom and DxO PhotoLab, and the user has confirmed
neither.** Whether DxO PhotoLab reads `-1` at all could not be checked here
(the web sources returned 403). If it turns out to ignore it, adding a colour
label alongside is a one-line change in `write_rating`.

### What has been confirmed, and by what

**Confirmed by hand on macOS (Phase 2)**: the folder picker opens and returns,
cancelling is a no-op, the arrow keys page, and a portrait file comes out
upright.

The first run found a bug nothing else had: `pick_folder` was a synchronous
`#[tauri::command]`, which tauri runs inline on the main thread, and
`blocking_pick_folder` then parked that thread — so the dialog appeared and
froze. It is an `async` command awaiting a channel now. Passing `mise run ci`,
`tsc --noEmit` and three rounds of review had not caught it, because it only
goes wrong once the app is actually running.

**Verified without a GUI (Phase 3)**: the focus box's coordinate transform was
checked numerically against `riffle-cli focusbox` on a real Orientation 8 file
— scaling `FocusLocation` onto the unrotated preview and letting the canvas
rotation carry the box puts the box where the CLI's PNG does: (-140, -26)
from the canvas centre against the CLI's (-140, -25). Everything else below is `mise run ci`
(`cargo test`, clippy, `tsc --noEmit`) plus the measurements in the next
sections.

**Verified without a GUI (Phase 6)**: the six sidecar reconciliation rules are
covered by tests against a temp folder, and the cost the sidecar pass adds to
a folder open was measured on the Rust side (see "What the sidecar pass adds
to a folder open" below, with its conditions). Nothing about how a rating
looks or feels in the running app has been verified here.

**Confirmed by hand on macOS (Phase 3)**: the user ran the app on a real
folder and confirmed the `scanning N / M` progress line, that the app stays
responsive while a real folder scans, thumbnails filling in during that scan
with portrait cells upright, the focus box (the shape drawn at the time)
landing on the subject, a fast second open, and both drag-and-drop gestures
(a folder and a single ARW). The Phase 3 paging keys
(`w`/`a`/`s`/`d`/`h`/`j`/`k`/`l`) and `f` toggling the focus mark are implied
by those, but this confirmation predates the switch to a crosshair — the
crosshair rendering itself has not been confirmed by hand. "Fast" is the
user's impression, not a measurement; the real-folder numbers are still
missing (see the Phase 3 sections below).

**Awaiting the user's confirmation (Phase 3)**: not everything in this phase
has been looked at yet. Still unconfirmed: the filmstrip highlight following
every paging key and key auto-repeat, click-to-page, scrolling a 5000-file
strip, and a drag that leaves the window without dropping.

**Verified without a GUI (Phase 5)**: the focus-point arithmetic of the 1:1
check is verified numerically only, on the Rust side that the CLI and the app
share — `riffle-cli crop` on the real Orientation 8 test file prints
`crop 525x512 at (3344,1476) point (269,256)`, and unit tests cover the
sensor→JPEG scaling, the MCU snap, the edge clamping and the centre fallback.
The `focus_crop` payload header and the Orientation 6/8 width/height swap are
covered by unit tests. The frontend's placement is right by construction (the
same `rotate()` branches as the preview, cropping in unrotated coordinates) but
that is an argument, not a check.

**Confirmed by hand on macOS (Phase 5)**: the user ran the app on a real folder
with `Debug > Timing logs` on and confirmed, from the log, that `Space` requests
a crop and it arrives, that dragging the window edge no longer storms the crop
path (one crop after the drag settles, against 30 during it before the fix), and
that the timing instrumentation reads correctly — `keypressToPixels` appears
only on the crop `Space` itself asked for. Those logs are the end-to-end numbers
in "The 1:1 focus check path" below. **The 50ms budget is met for a focus point
in a shallow row and missed for one in a deep row**: 39-45ms from keypress to
pixels, against 58-65ms for a focus point at the right edge of an Orientation 8
file. The deep-row case is not fixed.

**Awaiting the user's confirmation (Phase 5)**: what the 1:1 view *looks like*
has not been reported. Unconfirmed: the crop showing the subject's eye at 1:1
and upright on an Orientation 8 file, `Space` again returning to the preview
with the focus box, paging while zoomed staying zoomed and moving to the next
file's focus point without the old crop appearing over the new file, and the
centre fallback on a manual-focus file (tracked in `todo.md`). The logs confirm
a crop is produced and how long it takes, not that it is the right pixels in the
right place.

**Awaiting the user's confirmation (Phase 6)**: nothing about the rating keys
has been looked at in a running window. Unconfirmed: a rating key changing the
strip badge and the meta pane with no perceptible delay, holding `3` down doing
nothing beyond the first press, `x` then `u` then `4` ending at four stars,
mashing keys while paging never marking the wrong file, and one `.xmp` per
rated file appearing in the folder
within about half a second. Which tools read `xmp:Rating="-1"` back as a
reject is likewise the user's to confirm: **no reader has been confirmed by
the user**, and DxO PhotoLab's behaviour could not be checked here at all.

**Awaiting the user's confirmation (rating display tidy-up)**: none of it has
been looked at in a running window. Unconfirmed: no rating or reject drawn on
the canvas in the fitted or the 1:1 view; `N / M` between the filmstrip and
"Open folder", updating on every page turn and empty with no folder open, with
the strip still scrolling and virtualising as before; the meta pane showing no
position or rating line under the file name and ending with the rule-separated
sidecar section (name, `(not created)` when known absent, and the `Rating`
row); a rating key on a sidecar-less file removing `(not created)` at once
while `0` does not; and the strip cell and the `Rating` row using the same
glyphs and colours. `has_sidecar` itself is covered by a unit test in
`index.rs`.

### Phase 4 baseline

The per-page cost on the Rust side only, on an Apple Silicon Mac.
**It excludes the IPC hop and `createImageBitmap`**, which could not be
measured. Phase 2 read the whole 48MB ARW (`std::fs::read` plus `arw::parse`);
`preview` now reads a bounded 1MiB prefix instead (`reader::read_preview`),
which is where the metadata and the embedded preview live, so both numbers are
given:

| Folder | n | whole file (before) | bounded prefix (after) |
|--------|---|---------------------|------------------------|
| 5000 symlinks to one ARW, warm page cache | 300 | mean 6.6ms / p95 7.7ms | mean 0.06ms / p95 0.13ms |
| 20 distinct 48MB copies, first read | 20 | mean 16.0ms / p95 30.0ms | mean 5.1ms / p95 7.8ms |

Both columns were re-measured together in Phase 3 Step 2, so they compare
like with like; the whole-file numbers are in the same range as the ones
Phase 2 recorded (7.3ms / 19.3ms mean). The symlink folder is 5000 symlinks to the same file and
the copy folder is 20 `cp` copies in a scratch directory, read in file-name
order by a fresh process. `purge` needs root on this machine, so the "first
read" column cannot be guaranteed cold; it is the same procedure the Phase 2
baseline used.

The whole-file read dominated, which is what the bounded read removes. Phase 4
still owns prefetching, but it now starts from this cheaper read.

### Folder scan throughput

`riffle-cli scan` runs the Phase 3 extraction (bounded read, metadata parse,
404x270 thumbnail) over a folder on a dedicated rayon pool, with no database.
On an Apple Silicon Mac with 12 cores:

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
freshly-written-copies folder here is 100 `cp` copies scanned once, `purge`
needs root on this machine so nothing is guaranteed cold, and the copies were
still likely warm in the page cache right after being written; the folders
were also scanned in the order they were written, which flatters the higher
thread counts. The real number can only be measured by the user on a real
folder, and on a card reader or slow external disk the scan is disk-bound
regardless.

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

### The 1:1 focus check path

The Rust side of one `Space` keypress, on `~/Downloads/_DSC6978.ARW` (α7 V,
Orientation 8, `FocusLocation` 7008 4672 3613 1732, `JpgFromRaw` 7008x4672
baseline 4:2:2, 5,761,112 bytes) on an Apple Silicon Mac. **One real file, warm
page cache, in-process, release build, n=20, medians.** Re-measured against the
Step 1 functions the CLI and the app both call, not copied from planning.
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
with `mozjpeg::Compress`'s defaults measured 49ms at q85 during planning — more
than the decode it follows.

#### End to end, keypress to pixels

Measured by the user by hand in the running app, not here. **Conditions**:
optimised build (`mise run app:release`), `Debug > Timing logs` on, **DevTools
open** (a webview can be slower with the inspector attached, so these may be
upper bounds), warm page cache (the folder had been opened before), one real
folder of Sony ARW files, canvas 900x268 CSS pixels. `read` and `decode` come
from the `focus_crop` payload header (`Instant` inside `spawn_blocking`), the
rest from `performance.now()` on the frontend; **`ipc` is derived** as the
invoke elapsed minus `read` minus `decode`, so it is everything else on the
Rust side plus transport, not pure transport.

After the two fixes in #56 (the thumbnail storm) and #60 (the resize debounce),
n=8 crops across 3 `Space` presses:

| Measurement | n | Measured |
|-------------|---|----------|
| Keypress → pixels, for the crop the `Space` itself asked for | 3 | 39 / 45 / 40ms |
| `read` | 8 | 1.8-2.7ms |
| `decode` | 8 | 21.9-38.7ms |
| `ipc` (derived) | 8 | 2.3-3.7ms |
| `bitmap` | 8 | 0-1ms |
| Total per crop | 8 | 26-45ms |

**Before those fixes**, a focus point at the right edge of the screen — which
on an Orientation 8 file is a *deep row* of the unrotated JPEG, the worst case
for `jpeg_skip_scanlines` — measured `decode` 51.0 and 55.2ms, total 58 and
65ms (n=2). These two are pre-fix and n=2, so they are not equivalent to the
post-fix set above; the deep-row cost itself is the `jpeg_skip_scanlines`
behaviour documented in [docs/agents/tauri-app.md](./docs/agents/tauri-app.md)
("A partial decode's cost is set by its row, not its size") and the fixes did
not touch it.

Two numbers that drove the fixes, also pre-fix: the first one or two `Space`
presses after opening a folder cost 1624 and 2182ms, of which 1594.8 and
2154.9ms fell in the derived `ipc` bucket while `read` and `decode` were
normal (#56); and dragging the window edge produced 30 crop decodes in 1494ms
(#60).

`decode` dominates. `read`, `ipc` and `bitmap` are noise beside it, so the
raw-RGBA-over-IPC decision (a 4MB payload at the 1024 cap) costs a few
milliseconds rather than the tens the planning phase feared: **IPC is not the
bottleneck**, and the planned fallback of measuring a JPEG payload instead is
closed.

### What the sidecar pass adds to a folder open (Phase 6)

Opening a folder also reconciles the XMP sidecars: one listing of the
directory, a `stat` per sidecar found, and a parse of only those whose
`(size, mtime)` changed since the app last saw them.

| Second open of a 5000-file folder | Measured (median) |
|-----------------------------------|-------------------|
| No sidecars in the folder | 31.3ms |
| 5000 sidecars, all with an unchanged stat | 67.4ms |

**These two numbers are not comparable to the 34.4ms above and do not replace
it.** They were measured on a different folder: 5000 one-KB regular files
named `*.ARW`, not symlinks and not real ARWs, on the local APFS disk with a
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
shows a line offering to install it. The update itself is signed with the
project's updater key and verified before it is installed. On Linux only the
AppImage updates itself; a `.deb` / `.rpm` install is updated by installing the
newer package. **Awaiting the user's confirmation**: an installed build
detecting and installing a newer release has not been checked yet; it needs a
second release.

## Running the app

Prerequisites:

- **macOS**: Xcode Command Line Tools (`xcode-select --install`). Tauri needs no
  other system dependency there.
- **Windows**: MSVC Build Tools, the WebView2 runtime, and `nasm` (`mozjpeg-sys`
  builds libjpeg-turbo from source and needs it for SIMD on x86).

The Rust toolchain, Node and pnpm all come from `mise install`.

```sh
mise run tauri:dev
```

That is a debug build of the Rust side: quick to compile, slow at runtime. **Any
timing measurement has to come from the optimised build instead**, because the
numbers under "Measurements" below are all optimised ones and a debug build is
not comparable to them:

```sh
mise run tauri:release:devtools
```

Both tasks run `pnpm install` first. `tauri:release:devtools` also passes
`--features devtools`: Tauri only wires the webview's devtools up automatically
in a debug build, so without it there is no console to read the timings from. A
distributable build leaves the feature off.

`mise run tauri:dev` and `mise run tauri:release:devtools` need no signing key. A
`pnpm tauri build` does: it creates the updater artifacts, which are signed
with `TAURI_SIGNING_PRIVATE_KEY` (and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`), so
the build fails without them.

Both tasks also get a **Debug** menu, which a distributable build does not have.
Its `Timing logs` item turns on the 1:1 view's keypress → invoke → bitmap
timings, logged to the console; it starts unchecked on every launch.

To try the distributable build itself, bundle it the way the release workflow
does:

```sh
mise run tauri:release
```

The bundles land under `target/release/bundle/`. It skips the updater
artifacts, which need the release signing key.

## Measurements (Apple Silicon Mac, α7 V ARW, n=20)

| Step | Target | Measured (median) |
|------|--------|-------------------|
| 1616x1080 preview extraction | 10ms | 4.4ms |
| JpgFromRaw full decode | 300ms | 84ms |
| 512px partial decode at the focus point | 50ms | 17.5ms |

## The FocusLocation coordinate system

`FocusLocation` is in **unrotated sensor coordinates**. It maps onto both the
preview and the full JPEG unrotated, so draw the box first and apply the
Orientation rotation afterwards. Verified against a real portrait-orientation
file (Orientation 8 / Rotate 270 CW).
