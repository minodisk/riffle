# Using Riffle

The detailed behavior of Riffle. For a quick start, see [README.md](../README.md).

## Features

Open a folder from the picker, or drop a folder or any file in it onto the
window. Riffle lists the ARW and DNG files in it and pages through their
embedded previews, rotated by each file's Orientation. With no folder open the
viewer shows a prompt in its center; click it to open the folder picker.

- **Folders**: the left pane is a folder tree rooted at the home folder and
  the mounted volumes (`/Volumes/*` on macOS, the drive letters on Windows,
  `/mnt/*`, `/media/*/*` and `/run/media/*/*` on Linux; the list of these
  root volumes is read once at launch, so one mounted afterward does not
  appear until the app restarts). A folder's arrow lists its subfolders,
  again on every expand, so a subfolder created since shows up; an expanded
  folder shows how many RAW files it holds itself. Clicking a folder's name
  opens it. `File > Open Folder…`, the `open` key (or the empty-state hint)
  and a drop still open anything the tree does not reach. The open folder
  is highlighted, and the tree expands down to it whenever a folder opens,
  however it was opened; one on a volume the tree does not list (a network
  share, say) adds that volume to the top level for the session.
- **Filmstrip**: thumbnails run along the bottom, under the viewer and the
  folder tree, follow paging and show the file you click. The mouse wheel
  scrolls it sideways. Its header bar holds the `N / M` counter and the
  filter and sort menus.
  Several files can be selected: `Cmd+click` (`Ctrl+click` on Windows and
  Linux) adds or removes one file without changing the file shown (the shown
  file itself always stays selected), `Shift+click` selects every file from the
  last clicked one (the anchor) to the clicked one and shows the clicked file,
  and `Shift+ArrowLeft` / `Shift+ArrowRight` grow or shrink that range one file at
  a time. A plain click, or any key that moves to another file (arrows, burst
  jumps), collapses the selection to the new file; a plain arrow at either end
  of the strip, which moves nowhere, keeps it. A right-click on a cell outside
  the selection collapses it to that cell; one inside keeps it, so the context
  menu items act on the whole selection. Files the filter hides leave the
  selection.
- **Panels**: `F7` hides and shows the left pane (the folder tree), `F8` the
  right pane (the metadata), `F6` the filmstrip, and `Tab` both side panes at
  once (hiding both when either is shown, as Lightroom does), so the viewer
  can take the whole height a landscape frame needs. The filmstrip's header
  bar goes with it, so the filter and sort menus and the `N / M` counter are
  hidden while the strip is. Which panels are hidden is remembered across
  restarts.
- **Focus mark**: `f` draws a crosshair at the camera's recorded focus point,
  inside a rectangle of the AF frame the camera used on Sony bodies that
  record it (hidden by default). A body that records only the point, such as
  the SIGMA BF, shows the crosshair alone; cameras that record none, such as
  the M11-P, and manual-focus shots show no mark (see
  [What the camera records](./cameras.md)). The mark's color is the focus
  candidate state: green for a focus candidate, where the face nearest the
  AF point has sharp eyes (its eye sharpness, the Laplacian variance of the
  preview between the eyes, is at least 80); orange when a face is near the
  AF point but its eyes are not sharp; and white when Riffle does not know
  (no AF point, manual focus, no face near the point, or not computed yet).
  The camera's face tracking no longer colors the mark: a Sony eye-AF frame
  is judged by the faces Riffle detects like any other. The state is computed
  in a second pass that starts right after the thumbnails and metadata of the
  folder are in, so the marks turn from white to green or orange while the
  status shows `focus N / M`. On the 500 hand-labeled α7 V frames it was
  checked on, 93% of the candidates were in focus and 80% of the in-focus
  frames were candidates. It is a cue, not a verdict: AF on a person in the
  background gives a sharp face and a false candidate, and the back of a
  head or an upturned face finds no face and stays white. The mark also
  draws the faces Riffle detects near the AF point (anywhere on the preview
  when there is no AF point) as a cyan box with a dot between the eyes. They
  appear a moment after `f`, because the detection runs when the frame is
  shown and is kept only for the session. The 1:1 view and Compare draw no
  faces.
- **1:1 focus check**: `z` shows the full-resolution image at one pixel per
  screen pixel, centered on the focus point (or the frame center without one).
  Paging while zoomed stays zoomed and moves to the next file's focus point.
  There is no panning or free zoom.
- **Grayscale preview**: holding `g` shows the viewed image in grayscale to
  judge composition; releasing it restores color. It is momentary and
  display-only: nothing is written or remembered.
- **Compare**: `v` lays 2–4 selected files out in the viewer. With only one
  file selected, it instead puts that file beside the highest-scoring
  frame in its burst (the same file appears twice when it is already the
  highest-scoring one). Each frame is labeled with its file name and score, and
  the highest-scoring frame is outlined as `BEST`. Click a frame to make it
  `ACTIVE`; a star, pick, reject or label then applies only to that frame,
  regardless of the filmstrip selection. Press `v` again to return.
- **Judgments**: stars, a pick / reject flag and a color label, shown on
  the strip cell (the label tints the file-name band along the cell's bottom
  edge) and written to a sidecar; see
  [Ratings and sidecars](#ratings-and-sidecars).
  With several files selected, a judgment sets the same value, decided from
  the shown file, on every selected file; fields the judgment does not touch
  keep each file's own value (a star key leaves each label as it was, and a
  pick replaces a reject on every selected file but leaves each file's stars
  as they were). Hidden files
  are never judged.
- **Meta pane**: grouped by where each value comes from, in three groups.
  **EXIF** lists what the camera wrote in the standard tags (aperture,
  shutter, ISO, focal length, exposure, camera, lens and capture time).
  **Maker note** lists what it wrote in its vendor MakerNote: on Sony, the
  shutter type, the focus mode, the AF area (ILCE, NEX and ZV bodies only;
  not the FX cinema bodies, the compacts, or the A-mount bodies), the AF tracking, such as `Face
  tracking` or `Lock On AF`, the
  drive (the release mode, with the frame number within a burst, as in
  `Continuous, frame 2`), the stabilization, the exposure mode, the metering,
  the creative style (on current bodies the Creative Look, which may show as
  a two-letter code such as `ST`), the DRO and the RAW type; on Leica, the
  focus distance. The focus mode and AF tracking are left out on the older
  RX and HX compacts, where ExifTool reads them as not applying. The Sony
  MakerNote does not record the recognized subject type (human, animal, bird
  or vehicle) or whether the AF sat on an eye or a face, so Riffle does not
  show them, and enciphered values (shutter count, picture profile) are not
  read. When a lens reports no f-number (the M11-P with an M-mount
  lens), the aperture is the camera's estimate, marked `(est.)`. **Analysis**
  holds what Riffle computes itself: the sharpness score, a `Focus` row,
  `Candidate` or `Not a candidate`, with the focus candidate state the focus
  mark is colored by (left out when Riffle does not know: no AF point, manual
  focus, no face near the point, or not computed yet), and the `Eye
  sharpness` the state is decided from (left out when there is none).
- **Filter menu**: narrows the strip by pick flag, stars, color label,
  orientation (`Portrait` / `Landscape`), focus candidate
  (`Focus candidates`, which shows only the files whose focus mark is green
  and fills in as the second pass runs),
  camera, lens, aperture, shutter speed, ISO and focal length (grouped into
  ranges such as `24–35 mm`). The color
  label group lists the seven colors and `No label`; a label outside those
  seven colors matches no color item (nor `No label`). The orientation is
  decided by the file's EXIF Orientation — a quarter turn is portrait, so a
  frame the camera did not tag as rotated counts as landscape.
  The EXIF groups list only values present in the folder. Checks within a group
  are OR-ed, groups are AND-ed, and `Reset` clears them all. A judgment that drops
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
  the `open` key. It is rebindable: the menu shows the first key of `open`
  that can be an accelerator, and none when the action holds no such key.
- **Reload Folder**: the open folder keeps up with the disk on its own: a file
  copied in or deleted outside Riffle shows up in the strip about a second
  later. On volumes that send no change notifications, such as network shares,
  bring the window back to the front or choose `File > Reload Folder`
  (`CmdOrCtrl+R`) to do the same. Only what changed is read again; the stars,
  flags and color labels already given, the current file and the strip's
  position all stay as they were.
- **Move Rejected to Trash…**: `File > Move Rejected to Trash…` moves every
  file of the open folder marked as a reject to the OS Trash, together with the
  sidecars sitting next to it — both `.xmp` and `.ARW.dop` when both are there,
  no matter which format is currently selected. It asks first, showing how many files
  it is about to move, and `Cancel` leaves the folder untouched. Nothing is
  deleted: the file and its sidecars all go to the Trash, so restoring them
  brings back the stars, the flag and the color label. Whatever could not be
  moved is listed as an error and stays in the folder.
- **Undo**: `Edit > Undo` (the `undo` key, `CmdOrCtrl+Z` by default) restores the rating, flag and color
  label the last judged file had before, writes that to its sidecar and returns
  to the file (unless the filter now hides it, which the status line says).
  Repeated presses walk further back. A judgment on several selected files
  is undone as one. A reject-rest (`Shift+x`) is undone as
  one, restoring every frame it rejected and staying on the current file. The history belongs to the open folder
  and is cleared when another folder opens or the sidecar format changes.
- **Redo**: `Edit > Redo` (`CmdOrCtrl+Shift+Z`) re-applies the most recently
  undone judgment, the same way round. The redo history is forgotten as soon
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
  already has does not. The last file stays selected. A file the judgment
  drops out of the active filter already hands the cursor to the next file, so
  it is not skipped twice. A judgment that changes more than one selected
  file does not advance.
- **Clear Cache**: the `Cache` tab of `Riffle > Settings...` shows how much disk
  the folder index takes, and `Clear Cache` empties it after a confirmation. It
  removes the cached thumbnails and metadata of every folder ever opened; your
  judgments are not touched, because they live in the sidecars. The size is in
  the same units your file manager uses (decimal on macOS, binary on Windows
  and Linux). A scan has to finish before the cache can be cleared: while one is
  running the button is unavailable, with a note saying so, and it becomes
  available again by itself when the scan ends. The size figure refreshes each
  time a scan ends.
- **Sharpness cue**: a thin bar up the left edge of each strip cell shows how
  sharp the frame is next to its neighbors on the strip; the sharpest frame of
  a run is marked in the pick color. The score is computed from the embedded
  preview on the camera's eye-AF frame when a Sony body tracked a face, else
  around the AF focus point when the camera recorded one, even when a face is
  found elsewhere in the frame. Only when there is no AF point (manual focus,
  Leica DNG) does it use the eyes of a detected face, or, failing that, the
  sharpest region of the frame. Faces and eyes are found by the bundled
  [YuNet](https://github.com/opencv/opencv_zoo/tree/main/models/face_detection_yunet)
  model (MIT license, text in `crates/core/models/LICENSE`), run locally with
  no network access. The score ranks a burst rather than judging a frame on
  its own, and it does not replace the 1:1 focus check.
  The meta pane shows the raw score in its Analysis group. See
  [What the camera records](./cameras.md).
- **Bursts**: frames shot within 1 s of the previous frame form a burst. The
  grouping follows capture order whatever the chosen sort, and Leica files,
  which record no sub-second time, are grouped by whole seconds. The grouping
  deliberately follows time rather than the camera's own per-press sequence
  numbering: a burst is one moment, and one moment often spans several
  presses, such as pre-capture frames followed by the full press, or a quick
  re-press. A tinted band behind the strip cells joins the frames of a burst of
  two or more. The
  first cell of the band shows the burst's size (`7`), and the current cell
  shows its position in the burst instead (`3/7`). `ArrowDown` jumps to the first frame of the next burst and
  `ArrowUp` to the first frame of the current one, or of the previous one
  when already there. `Alt+ArrowLeft` / `Alt+ArrowRight` (`Option` on macOS) move
  one frame at a time within the current burst and stop at its first and last
  displayed frame. `Shift+x` rejects every other frame of the current burst,
  including frames the filter hides, and one `Edit > Undo` restores them all.
  See [What the camera records](./cameras.md).

## Keys

| Key | Action |
|-----|--------|
| `ArrowLeft` | previous file |
| `ArrowRight` | next file |
| `Shift+ArrowLeft` | extend the selection to the previous file (or shrink it back towards the anchor) |
| `Shift+ArrowRight` | extend the selection to the next file (or shrink it back towards the anchor) |
| `ArrowUp` | first frame of the current burst, or of the previous burst when already on it |
| `ArrowDown` | first frame of the next burst |
| `Alt+ArrowLeft` | previous frame in the current burst (stops at its first frame) |
| `Alt+ArrowRight` | next frame in the current burst (stops at its last frame) |
| `Cmd+O` / `Ctrl+O` | open a folder (`File > Open Folder…`) |
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
| `c` | clear every flag of the current file: stars, reject, pick and color label |
| `CmdOrCtrl+Z` | undo the last judgment (also `Edit > Undo`, whose accelerator follows this key) |
| `CmdOrCtrl+Shift+Z` | redo the last undone judgment (also `Edit > Redo`, whose accelerator follows this key) |
| `F6` | show / hide the filmstrip |
| `F7` | show / hide the left pane |
| `F8` | show / hide the right pane |
| `Tab` | show / hide both side panes |

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
Combinations the system or the app's menu already use (`Cmd+Q`, `Cmd+,`,
`Cmd+Tab`, `Ctrl+C` on Windows, any Windows-key combination, ...) are refused.
The File menu's `Open Folder…` accelerator and the Edit menu's Undo / Redo
are the exception: they follow their own action's keys, so unlike `Cmd+,` they can be
rebound, and the
combination an action leaves behind is free for another action.
`Reset all` restores the defaults.

These keys are fixed and cannot be changed:

| Key | Action |
|-----|--------|
| `Escape` | close the filter, sort or right-click menu, or leave Compare; in Settings, cancel adding a key, or close the settings |
| `Tab` / `Shift+Tab` | while the first-launch developing-software dialog or the settings are open, move between the dialog's buttons or the settings' controls (otherwise `Tab` is the side-pane toggle above, which can be rebound) |
| `CmdOrCtrl+R` | reload the folder (`File > Reload Folder`) |
| `CmdOrCtrl+,` | open the settings (`Riffle > Settings...`, `File > Settings...` on Windows and Linux) |
| `ArrowLeft` / `ArrowRight` / `Home` / `End` | in the settings tab strip, the previous / next / first / last tab |

## Ratings and sidecars

The RAW file is never written. Judgments go into a sidecar next to it, in one
of two formats or in both. Riffle asks which on its first launch, before any
folder can be opened, and the choice can be changed later in
`Riffle > Settings...` (`File > Settings...` on Windows and Linux):

- **Lightroom (XMP)**: `FOO.ARW` gets `FOO.xmp`, holding `xmp:Rating` (`0`-`5`),
  the pick / reject flag as `xmpDM:good` (`True` for a pick, `False` for a
  reject, absent for neither), and the color label as both
  `photoshop:LabelColor` and `xmp:Label`, the way Lightroom writes them.
  `xmp:Label` carries the name configured for the color in the settings
  (English by default: `Red`, `Yellow`, `Green`, `Blue`, `Purple`) and
  `photoshop:LabelColor` the lowercase English color (`red`, ...). A sidecar
  whose `xmp:Label` matches a configured name or the English name is shown in
  that color.
- **PhotoLab (.dop)**: `FOO.ARW` gets `FOO.ARW.dop`, holding the stars, the
  pick / reject flag and the `ColorLabel` line (`Red`, `Orange`, `Yellow`,
  `Green`, `Blue`, `Pink`, `Purple`), which PhotoLab 10 reads.
- **Both**: every judgment is written to `FOO.xmp` and to `FOO.ARW.dop`, each
  as above. When a folder opens, the one of the two modified last is read
  back (on a tie, the XMP), so an edit made later in either Lightroom or
  PhotoLab wins. A judgment made without knowing the file's label keeps the
  label of that newest sidecar and writes it to both. The two files are each
  written atomically, but not together: if one write fails, an in-session
  retry writes the judgment into both again; after every failed attempt, the
  sidecar that did get written is remembered, so a folder open during the
  retries or once they are exhausted still writes the judgment into both
  instead of mistaking that earlier write for an external edit.

Clearing a label removes `photoshop:LabelColor` and `xmp:Label`, or the
`ColorLabel` line; no label is the field being absent. The label is kept as
the exact string the sidecar holds: a name from the other tool's vocabulary is
written back unchanged and shown in its color, and any other name (say, a
custom Lightroom label) is shown gray.

A sidecar written by another tool is edited in place: only the rating, the
flag and the label change, and everything else — develop settings, keywords —
is kept byte for byte. Clearing a file that has no sidecar creates none.

Writes happen in the background and are atomic, so a crash never leaves a
half-written sidecar, and quitting finishes any pending write. A judgment that
could not be written (say, on a locked card) is retried a few times over the
next half minute, and if it still fails it is kept and retried the next time
the folder is opened; the error is shown at the bottom of the right pane until
you dismiss it, and so is any sidecar the app cannot read when a folder opens
(damaged, or larger than 4 MiB). Sidecars edited by another tool are picked up the next
time the folder is opened; when both changed, the other tool's edit wins.
Switching the format keeps unwritten judgments and writes them in the new
format (or both); the files of a format no longer selected are left alone,
and switching to Both rewrites nothing, so a file's two sidecars can disagree
until it is judged again.

The folder index is a cache, but it also holds unwritten judgments, so a new
index schema migrates the previous ones in place instead of discarding them;
only a version it cannot migrate is dropped and rebuilt from the sidecars.

## MCP companion

Riffle can serve an [MCP](https://modelcontextprotocol.io/) (Model Context
Protocol) endpoint, so an MCP client, such as an AI assistant, can follow
along while you cull: see what Riffle shows, look at a small preview, move the
view and record judgments. It is off by default; turn it on with
`Let MCP clients connect` in the `MCP` tab of `Riffle > Settings...`
(`File > Settings...` on Windows and Linux). The choice is remembered across
launches.

- **Endpoint**: `http://127.0.0.1:41917/mcp`, served over Streamable HTTP.
  The tab shows the URL, whether the server is listening, and the error when
  it could not start (say, another program holds port 41917); Riffle itself
  keeps running either way.
- **Local only**: the server listens on the loopback address, so only programs
  on this computer reach it, and it refuses any request carrying an `Origin`
  header, which is what a web page's request would carry. There is no token:
  anything running as you on this computer can connect while it is on.
- **Nothing is deleted**: no tool moves a file to the Trash or deletes one,
  and the RAW files are never written. The only writes are judgments, and they
  go to the sidecars.
- **The main window answers**: every tool reads or changes the state of the
  main window, so a folder has to be open there for most of them to be
  useful, and a call fails if the window does not answer within 5 seconds.

| Tool | What it does |
|------|--------------|
| `get_view` | What Riffle is showing: the open folder, how many photos the filter leaves, the current photo and its position, the selection, the view mode (`normal`, `zoom` or `compare`) and the active compare frame, the sort and whether a filter is on, and the current photo's burst with each frame's sharpness score, stars, flag and label |
| `get_photo` | One photo's stars, flag and label, sharpness score, AF point and frame, manual focus, orientation, capture time and shooting settings (camera, lens, aperture, shutter, ISO, focal length, exposure bias, focus distance); the current photo when no path is given |
| `get_preview` | A small upright JPEG of one photo, scaled from the embedded preview (never the RAW) to a long edge of 1024 pixels, or of the requested size from 256 to 1616 |
| `show_photo` | Show one photo, as clicking it in the filmstrip does |
| `select_photos` | Select a list of photos; the first one is shown |
| `set_view` | Switch to the fitted view, the 1:1 focus check or Compare, as `z` and `v` do; Compare needs two photos to compare, as with `v` |
| `set_judgment` | Set the stars (`0` clears them), the flag (`none`, `pick` or `reject`) and the color label (`Red`, `Orange`, `Yellow`, `Green`, `Blue`, `Pink`, `Purple`, or null to clear) of the given photos, or of the selection (the active frame in Compare) when none are given |

A photo has to be in the open folder, and the tools that show, select or
judge photos refuse one the filter hides. `set_judgment` goes through the
same path as the judgment keys: fields it is not given keep each photo's own
value, the strip updates at once, the change is one step for `Edit > Undo`,
and the sidecar is written as a key press writes it. Auto-advance does not
apply to it. The server's instructions ask the assistant to suggest
judgments and to write them only when you ask.

Any client that speaks MCP over Streamable HTTP connects to the endpoint
above. Two examples, which the `MCP` tab also shows with Copy buttons:

- **Claude Code**:
  `claude mcp add --transport http riffle http://127.0.0.1:41917/mcp`
- **Claude Desktop**: add the following to `claude_desktop_config.json`. It
  relays through `npx -y mcp-remote`, so it needs Node.js; whether Claude
  Desktop takes a direct `url` entry for a local server has not been
  verified.

  ```json
  {
    "mcpServers": {
      "riffle": {
        "command": "npx",
        "args": ["-y", "mcp-remote", "http://127.0.0.1:41917/mcp"]
      }
    }
  }
  ```

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
on Windows and Linux) runs the same check by hand and reports the outcome; when
it finds a new version it offers **Restart Now**, which quits (saving pending
sidecar writes first) and relaunches into the new version, or **Later**, which
keeps the behavior above. The update itself is signed with the
project's updater key and verified before it is installed. On Linux only the
AppImage updates itself; a `.deb` / `.rpm` install is updated by installing the
newer package.

To build from source, see [CONTRIBUTING.md](../CONTRIBUTING.md).
