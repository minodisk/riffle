# Using Riffle

The detailed behaviour of Riffle. For a quick start, see [README.md](../README.md).

## Features

Open a folder from the picker, or drop a folder or any file in it onto the
window. Riffle lists the ARW and DNG files in it and pages through their
embedded previews, rotated by each file's Orientation. With no folder open the
viewer shows a prompt in its centre; click it to open the folder picker.

- **Filmstrip**: thumbnails run down the left edge, follow paging and show the
  file you click. The `N / M` counter sits under it.
  Several files can be selected: `Cmd+click` (`Ctrl+click` on Windows and
  Linux) adds or removes one file without changing the file shown (the shown
  file itself always stays selected), `Shift+click` selects every file from the
  last clicked one (the anchor) to the clicked one and shows the clicked file,
  and `Shift+ArrowUp` / `Shift+ArrowDown` grow or shrink that range one file at
  a time. A plain click, or any key that moves to another file (arrows, burst
  jumps), collapses the selection to the new file; a plain arrow at either end
  of the strip, which moves nowhere, keeps it. A right-click on a cell outside
  the selection collapses it to that cell; one inside keeps it, so the context
  menu items act on the whole selection. Files the filter hides leave the
  selection.
- **Focus mark**: `f` draws a crosshair at the camera's recorded focus point
  (hidden by default; cameras that record none, such as the M11-P, show none).
- **1:1 focus check**: `z` shows the full-resolution image at one pixel per
  screen pixel, centred on the focus point (or the frame centre without one).
  Paging while zoomed stays zoomed and moves to the next file's focus point.
  There is no panning or free zoom.
- **Grayscale preview**: holding `g` shows the viewed image in grayscale to
  judge composition; releasing it restores colour. It is momentary and
  display-only: nothing is written or remembered.
- **Compare**: `v` lays 2–4 selected files out in the viewer. With only one
  file selected, it instead puts that file beside the highest-scoring
  frame in its burst (the same file appears twice when it is already the
  highest-scoring one). Each frame is labelled with its file name and score, and
  the highest-scoring frame is outlined as `BEST`. Click a frame to make it
  `ACTIVE`; a star, pick, reject or label then applies only to that frame,
  regardless of the filmstrip selection. Press `v` again to return.
- **Judgements**: stars, a pick / reject flag and a colour label, shown on
  the strip cell (the label tints the file-name band along the cell's bottom
  edge) and written to a sidecar; see
  [Ratings and sidecars](#ratings-and-sidecars).
  With several files selected, a judgement sets the same value, decided from
  the shown file, on every selected file; fields the judgement does not touch
  keep each file's own value (a star key leaves each label as it was, and a
  pick replaces a reject on every selected file but leaves each file's stars
  as they were). Hidden files
  are never judged.
- **Meta pane**: camera, lens, shutter, aperture, ISO and focal length. When a
  lens reports no f-number (the M11-P with an M-mount lens), the aperture is the
  camera's estimate, marked `(est.)`; Leica files add the focus distance.
- **Filter menu**: narrows the strip by pick flag, stars, colour label,
  orientation (`Portrait` / `Landscape`), camera, lens, aperture, shutter speed,
  ISO and focal length (grouped into ranges such as `24–35 mm`). The colour
  label group lists the seven colours and `No label`; a label outside those
  seven colours matches no colour item (nor `No label`). The orientation is
  decided by the file's EXIF Orientation — a quarter turn is portrait, so a
  frame the camera did not tag as rotated counts as landscape.
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
- **Move Rejected to Trash…**: `File > Move Rejected to Trash…` moves every
  file of the open folder marked as a reject to the OS Trash, together with the
  sidecars sitting next to it — both `.xmp` and `.ARW.dop` when both are there,
  no matter which format is currently selected. It asks first, showing how many files
  it is about to move, and `Cancel` leaves the folder untouched. Nothing is
  deleted: the file and its sidecars all go to the Trash, so restoring them
  brings back the stars, the flag and the colour label. Whatever could not be
  moved is listed as an error and stays in the folder.
- **Undo**: `Edit > Undo` (`CmdOrCtrl+Z`) restores the rating, flag and colour
  label the last judged file had before, writes that to its sidecar and returns
  to the file (unless the filter now hides it, which the status line says).
  Repeated presses walk further back. A judgement on several selected files
  is undone as one. A reject-rest (`Shift+x`) is undone as
  one, restoring every frame it rejected and staying on the current file. The history belongs to the open folder
  and is cleared when another folder opens or the sidecar format changes.
- **Redo**: `Edit > Redo` (`CmdOrCtrl+Shift+Z`) re-applies the most recently
  undone judgement, the same way round. The redo history is forgotten as soon
  as you judge a file again, and like the undo history it is cleared when
  another folder opens or the sidecar format changes.
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
  it is not skipped twice. A judgement that changes more than one selected
  file does not advance.
- **Clear Cache**: the `Cache` tab of `Riffle > Settings...` shows how much disk
  the folder index takes, and `Clear Cache` empties it after a confirmation. It
  removes the cached thumbnails and metadata of every folder ever opened; your
  judgements are not touched, because they live in the sidecars. The size is in
  the same units your file manager uses (decimal on macOS, binary on Windows
  and Linux). A scan has to finish before the cache can be cleared: while one is
  running the button is unavailable, with a note saying so, and it becomes
  available again by itself when the scan ends. The size figure refreshes each
  time a scan ends.
- **Sharpness cue**: a thin bar up the left edge of each strip cell shows how
  sharp the frame is next to its neighbours on the strip; the sharpest frame of
  a run is marked in the pick colour. The score is computed from the embedded
  preview around the AF focus point when the camera recorded one, and
  otherwise (manual focus, Leica DNG) as the sharpest region of the frame, so
  it ranks a burst rather
  than judging a frame on its own, and it does not replace the 1:1 focus check.
  The meta pane shows the raw score.
- **Bursts**: frames shot within 1 s of the previous frame form a burst. The
  grouping follows capture order whatever the chosen sort, and Leica files,
  which record no sub-second time, are grouped by whole seconds. A tinted
  band behind the strip cells joins the frames of a burst of two or more. The
  first cell of the band shows the burst's size (`7`), and the current cell
  shows its position in the burst instead (`3/7`). `ArrowRight` jumps to the first frame of the next burst and
  `ArrowLeft` to the first frame of the current one, or of the previous one
  when already there. `Alt+ArrowUp` / `Alt+ArrowDown` (`Option` on macOS) move
  one frame at a time within the current burst and stop at its first and last
  displayed frame. `Shift+x` rejects every other frame of the current burst,
  including frames the filter hides, and one `Edit > Undo` restores them all.

## Keys

| Key | Action |
|-----|--------|
| `ArrowUp` | previous file |
| `ArrowDown` | next file |
| `Shift+ArrowUp` | extend the selection to the previous file (or shrink it back towards the anchor) |
| `Shift+ArrowDown` | extend the selection to the next file (or shrink it back towards the anchor) |
| `ArrowLeft` | first frame of the current burst, or of the previous burst when already on it |
| `ArrowRight` | first frame of the next burst |
| `Alt+ArrowUp` | previous frame in the current burst (stops at its first frame) |
| `Alt+ArrowDown` | next frame in the current burst (stops at its last frame) |
| `Cmd+O` / `Ctrl+O` | open a folder (`File > Open Folder…`) |
| `Shift+Cmd+O` / `Ctrl+Shift+O` | open the folder in DxO PhotoLab (`File > Open in DxO PhotoLab`) |
| `f` | toggle the focus mark |
| `z` | toggle the 1:1 focus check |
| `g` (hold) | grayscale preview |
| `v` | toggle comparison of selected files / the current file with its burst's highest-scoring frame |
| `1`-`5` | rate the current file that many stars |
| `x` | reject the current file (replaces a pick, keeps the stars) |
| `Shift+x` | reject every other frame of the current burst, including frames the filter hides (replaces their picks) |
| `p` | pick the current file (replaces a reject, keeps the stars) |
| `u` | un-reject or un-pick the current file |
| `0` | clear the stars |
| `c` | clear every flag of the current file: stars, reject, pick and colour label |
| `CmdOrCtrl+Z` | undo the last judgement (the `Edit > Undo` accelerator; not rebindable) |
| `CmdOrCtrl+Shift+Z` | redo the last undone judgement (the `Edit > Redo` accelerator; not rebindable) |

Pressing the key of the label the file already has clears it; the stars, the
flag and `0` leave the label alone, while `c` clears it along with
everything else.

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

## Ratings and sidecars

The RAW file is never written. Judgements go into a sidecar next to it, in one
of two formats chosen in `Riffle > Settings...`:

- **XMP** (default): `FOO.ARW` gets `FOO.xmp`, holding `xmp:Rating` (`0`-`5`),
  the pick / reject flag as `xmpDM:good` (`True` for a pick, `False` for a
  reject, absent for neither), and the colour label as both
  `photoshop:LabelColor` and `xmp:Label`, the way Lightroom writes them
  (`Red`, `Yellow`, `Green`, `Blue`, `Purple`).
- **DxO PhotoLab**: `FOO.ARW` gets `FOO.ARW.dop`, holding the stars, the
  pick / reject flag and the `ColorLabel` line (`Red`, `Orange`, `Yellow`,
  `Green`, `Blue`, `Pink`, `Purple`), which PhotoLab 10 reads.

Clearing a label removes `photoshop:LabelColor` and `xmp:Label`, or the
`ColorLabel` line; no label is the field being absent. The label is kept as
the exact string the sidecar holds: a name from the other tool's vocabulary is
written back unchanged and shown in its colour, and any other name (say, a
custom Lightroom label) is shown grey.

A sidecar written by another tool is edited in place: only the rating, the
flag and the label change, and everything else — develop settings, keywords —
is kept byte for byte. Clearing a file that has no sidecar creates none.

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

To build from source, see [CONTRIBUTING.md](../CONTRIBUTING.md).
