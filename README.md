# Riffle

A culling app for going through the RAW files you shot, fast, and deciding what
to keep and what to throw away.

- It reads only the JPEG previews embedded in the RAW files, so paging through a
  folder of thousands of shots never keeps you waiting.
- Stars, picks / rejects and color labels are written to sidecar files that
  DxO PhotoLab and tools that read XMP sidecars, such as Lightroom, can read.
  The RAW files themselves are never written.
- There is no developing and no editing. It does one thing: choosing.

For the supported cameras and formats, see [Compatibility](#compatibility).

## Getting started

### Install

Download the file for your OS from the latest release on the
[Releases page](https://github.com/minodisk/riffle/releases):

| OS | File |
|----|------|
| macOS, Apple Silicon | `Riffle_<version>_aarch64.dmg` |
| macOS, Intel | `Riffle_<version>_x64.dmg` |
| Windows | `Riffle_<version>_x64-setup.exe` |
| Linux | `Riffle_<version>_amd64.AppImage` |

The app is not signed by the OS vendors, so the first launch needs your
permission once:

- **macOS**: right-click `Riffle.app` → Open.
- **Windows**: in the SmartScreen warning, More info → Run anyway.

After that, updates are downloaded automatically on launch and take effect the
next time Riffle starts. For the other packages and the details, see
[docs/usage.md](./docs/usage.md#installing).

### First steps

1. Drop a folder onto the window (or open one with `Cmd+O` / `Ctrl+O`).
2. Page through the shots with `↑` `↓`, give stars with `1`-`5` and reject with
   `x`.
3. When the focus is in doubt, press `z` for the 1:1 view and check the focus
   point.

Judgments are saved to the sidecars as you go, so there is nothing to save.
When you are done, they are picked up by DxO PhotoLab and tools that read XMP
sidecars.

## Features

- **Filmstrip**: thumbnails run down the left edge; click one to show it.
  `Cmd+click` / `Ctrl+click` adds or removes one, `Shift+click` and
  `Shift+↑` / `Shift+↓` select a range, and every judgment applies to the
  whole selection.
- **1:1 focus check**: `z` shows the image at 1:1, centered on the focus point.
  Paging keeps the zoom.
- **Compare**: `v` shows 2–4 selected shots together, or the current shot
  beside the sharpest frame in its burst. Click a frame to rate, pick or
  reject only that one.
- **Focus mark**: `f` draws a crosshair at the focus point the camera recorded.
- **Sharpness cue**: a bar beside each thumbnail shows which frame of a burst is
  sharpest, scored on the camera's eye-AF frame when a Sony body tracked a
  face, else on the subject's eyes when a face is found, else around
  the AF point, else the sharpest region.
- **Offline face detection**: faces and eyes are found by the bundled
  [YuNet](https://github.com/opencv/opencv_zoo/tree/main/models/face_detection_yunet)
  model (MIT license), run locally with no network access.
- **Bursts**: frames shot within 1 s of each other share a band and a
  count badge on the strip; `ArrowLeft` / `ArrowRight` jump between bursts,
  `Alt+ArrowUp` / `Alt+ArrowDown` step through the frames of one and stop at
  its ends, and `Shift+x` rejects the rest of one.
- **Shooting info**: camera, lens, shutter speed, aperture, ISO and focal
  length.
- **Filter and sort**: narrow down by stars, label, orientation, lens and more;
  sort by file name, capture time or rating.
- **Move Rejected to Trash**: `File > Move Rejected to Trash…` moves the
  rejected shots to the Trash. Nothing is deleted, so restoring them brings the
  judgments back.
- **Undo / redo**: `Cmd+Z` / `Ctrl+Z` undoes a judgment; add `Shift` to redo.
- **Auto-advance**: when turned on in the settings, each judgment moves on to
  the next shot.

Detailed behavior: [docs/usage.md](./docs/usage.md).

## Keys

| Key | Action |
|-----|--------|
| `↑` / `↓` | previous / next shot |
| `Shift+↑` / `Shift+↓` | extend the selection to the previous / next shot |
| `1`-`5` | give stars |
| `0` | clear the stars |
| `x` | reject (keeps the stars) |
| `p` | pick (keeps the stars) |
| `u` | clear the pick / reject |
| `Ctrl+Alt+1`-`7` | color label (red, orange, yellow, green, blue, pink, purple) |
| `Ctrl+Alt+0` | clear the color label |
| `c` | clear everything: stars, pick / reject and color label |
| `z` | 1:1 focus check |
| `g` (hold) | grayscale preview |
| `v` | compare selected shots / the current shot with its burst's sharpest frame |
| `f` | focus mark |
| `Cmd+O` / `Ctrl+O` | open a folder |

Keys can be changed in `Riffle > Settings...`.

## Working with other software

Judgments are saved in a sidecar file next to the RAW. The format is chosen in
the settings:

- **Lightroom (XMP)** (default): `FOO.ARW` gets `FOO.xmp`, the format Lightroom
  and others read. Picks and rejects are written as `xmpDM:good` and color
  labels as both `photoshop:LabelColor` and `xmp:Label`, the way Lightroom
  writes them. The name written to `xmp:Label` can be set per color in the
  settings.
- **PhotoLab (.dop)**: `FOO.ARW` gets `FOO.ARW.dop`, which holds picks and rejects too.
  `File > Open in DxO PhotoLab` opens the folder straight in PhotoLab.

A sidecar created by other software is edited in place: everything except the
stars, the flag and the label (develop settings, keywords, ...) is left
untouched.

### Lightroom Classic

- Lightroom Classic reads the XMP Riffle wrote when the photos are first
  imported.
- After import it does not re-read a sidecar Riffle changed, not even on
  restart. Right-click the folder, choose `Synchronize Folder...`, check
  `Scan for metadata updates` and click `Synchronize`.
- Lightroom Classic does not write XMP by default. `Ctrl+S`
  (`Metadata > Save Metadata to File`) writes it for the selected photos, or
  turn on `Catalog Settings > Metadata > Automatically write changes into XMP`.
- Color labels are matched by name against Lightroom Classic's color label
  set (`Metadata > Color Label Set > Edit...`). Either rename that set's labels
  to `Red`, `Yellow`, `Green`, `Blue` and `Purple`, or set Riffle's label names
  in the settings to the names in the set (a preset for the Japanese default
  set is provided).

## Compatibility

Anything unchecked has not been verified yet. Reports of how it went for you
are very welcome.

### OS

- macOS
  - [x] 26 (Apple Silicon)
- Windows
  - [x] 11
- Linux
  - [x] Ubuntu

If it works, post in the
[OS works report thread](https://github.com/minodisk/riffle/discussions/286) in
Discussions. If it does not, open an issue from the
[OS issue template](https://github.com/minodisk/riffle/issues/new?template=os.yml).

### RAW formats and cameras

- ARW
  - [x] Sony α7 V
- DNG
  - [x] Leica M11-P

If a camera not on the list works, post in the
[camera works report thread](https://github.com/minodisk/riffle/discussions/287)
in Discussions. If it does not, open an issue from the
[camera issue template](https://github.com/minodisk/riffle/issues/new?template=camera.yml).
A sample file is needed to look into it, so please attach one (or link to it).
One file is enough; a landscape and a portrait shot, if possible, also let the
rotation and the focus mark be checked.

### Sidecar formats and software

- XMP
  - [ ] Adobe Lightroom (Windows; not Lightroom Classic)
  - [ ] Adobe Lightroom Classic (Windows, Japanese UI)
  - [ ] Capture One
- DOP
  - [x] DxO PhotoLab 10

If the stars, flags and color labels given in Riffle show up correctly in the
software, post in the
[software works report thread](https://github.com/minodisk/riffle/discussions/288)
in Discussions. If they do not, open an issue from the
[software issue template](https://github.com/minodisk/riffle/issues/new?template=software.yml).

## For developers

- Building from source: [CONTRIBUTING.md](./CONTRIBUTING.md)
- Performance measurements: [docs/performance.md](./docs/performance.md)
