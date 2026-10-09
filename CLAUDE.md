# Riffle

A culling app for Sony ARW, Canon CR3, Nikon NEF, Fujifilm RAF, OM System / Olympus ORF and DNG files. See [README.md](./README.md)
for what it is and the current status.

## Layout

A Cargo workspace: `crates/core` (ARW, DNG, NEF, CR3, RAF and ORF parsing and JPEG decoding, `riffle-core`,
whose `src/xmp.rs` parses and patches XMP sidecar bytes (`xmp:Rating`, the
tri-state pick / reject flag as `xmpDM:good`, and the color label as
`photoshop:LabelColor` and `xmp:Label`) and `src/dop.rs` DxO PhotoLab `.dop`
sidecar bytes (rating, the tri-state pick / reject flag as `ShouldProcess`,
and `ColorLabel`), both sharing the `Flag` enum in `src/lib.rs`, `src/i18n.rs`
the per-language values, one JSON file per language in `crates/core/i18n/`
(so far the Lightroom color label presets) that `build.rs` embeds at build
time, `src/faces.rs` the YuNet face/eye detector, whose ONNX model and license
live in `crates/core/models/`, `src/eyes.rs` the closed-eyes judgment of one
face (the eye aspect ratio of the MediaPipe Face Landmarker v2 face mesh,
its ONNX and `LICENSE-mediapipe` next to YuNet's; the scan runs only its
mesh, `mesh_of`, for the focus candidate cue), `src/pose.rs` the head pose (yaw / pitch / roll) of a face from its face mesh points, a port of MediaPipe's face geometry pipeline whose 33-point Procrustes basis is in `LICENSE-mediapipe`, `src/candidate.rs` the focus candidate cue
(the in-focus probability of the eyes of the face nearest the AF point,
a logistic combination of the Laplacian variance and mean edge width over
each eye's eyelid region from the face mesh, the sharper eye counting, or
over the window between the eyes when neither eye's region counts (under the floor or without a clear edge), and
whether it clears the threshold), `src/jpeg.rs` the Exif reader of a plain
JPEG file (orientation and the standard shooting tags into the same `Shot`
the RAW parsers fill), which `reader` and `scan` dispatch to for a `.jpg` /
`.jpeg`, `src/exif.rs` the crate-private IFD0 + Exif IFD reader `jpeg.rs`,
`nef.rs` and `cr3.rs` share, `src/nef.rs` the Nikon NEF parser (the full-size and
preview JPEGs from the SubIFDs), which `reader` dispatches to for a `.nef`,
`src/cr3.rs` the Canon CR3 parser (the ISOBMFF boxes: Exif from `CMT1` /
`CMT2`, the preview from `PRVW`, the full-size JPEG from its track), which
`reader` dispatches to for a `.cr3`, `src/hevc.rs` the decoder of the HEVC
`PRVW` / `THMB` of a CR3 shot with HDR PQ on (the pure-Rust `hpvcd` crate, then
a PQ-to-sRGB tone map and a JPEG encode), which `reader` hands those images to
so every consumer still gets a JPEG, `src/raf.rs` the Fujifilm RAF parser (the
fixed header's offset of the one embedded JPEG, whose Exif is the file's,
and the AF point from its Fujifilm MakerNote's `FocusPixel`), which `reader` dispatches to for a `.raf`, `src/orf.rs` the OM System /
Olympus ORF parser (IFD0 and the Exif IFD of its `IIRO` TIFF, the preview from
the MakerNote's CameraSettings, with note-relative offsets), which `reader`
dispatches to for a `.orf`, `src/sequence.rs` the JPEG timestamp sequencer ported
from lapse (orders a folder's JPEGs by capture time and writes copies with
unique `DateTimeOriginal` seconds into `<folder>-sequenced/`), and
`src/sharpness.rs` the
sharpness score of the embedded preview, taken on the Sony eye-AF frame
when the camera tracked a face, else around the AF point, else between
the eyes of a detected face, else from the sharpest tile), `crates/cli` (the
benchmark CLI, including the `scan` folder-extraction benchmark), `crates/app`
(the Tauri 2 desktop app, whose `src/index.rs` is the SQLite folder index,
which also keeps each folder's last viewed file in `folders.last_viewed`,
re-extracting rows written by an older `EXTRACTOR_VERSION` and filled in two
passes on the one scan task: `run_scan` (thumbnail, metadata), then
`run_faces_scan`, which fills the `eye_focus` / `sharpness` /
`faces_extractor` columns with the focus candidate cue the `f` focus mark is
colored by (YuNet, then the face mesh on the face nearest the AF point) and
the sharpness score, and the `eyes_ear` / `pose_yaw` / `pose_pitch` /
`pose_roll` columns with the EAR of the more closed eye and the head pose
from that same mesh, and streams them as `faces-progress` /
`faces-done` events; both passes run on worker threads below normal OS
priority (the second pass lower still) and take the on-screen files first
through the shared `ScanFocus` handle,
`src/commands.rs` the Tauri commands, including `set_scan_focus`, which hands
the running scan the current file and the strip's visible range,
`faces_of`, which detects the faces the focus mark
draws on demand through the scan's `detect_around` without touching the index,
and `eyes_of`, which judges with `eyes` whether the eyes of the shown file's
AF face are closed, also on demand, a newer request superseding an older one,
and returns the face mesh points it judged on in stored preview coordinates,
`src/exif.rs` the shooting-settings display formatting shared by the meta pane
and the filter menu, `src/folders.rs` the folder tree's commands (the
home and volume roots, one folder's subfolders and RAW count, and
`reveal_folder`, the right-click item that reveals a folder in the OS file
manager under the per-platform `REVEAL_LABEL`), `src/rename.rs` the
`rename_folder` command (it checks the new name, releases the folder watcher,
renames the folder and carries its index rows to the new path), `src/watch.rs`
the open folder's watcher and `src/treewatch.rs` the folder tree's watchers on
its expanded folders (`set_tree_watches`, per-folder `tree-changed` events),
`src/trash.rs` the
reject collection and the move behind the `trash_rejected_preview` and
`trash_rejected_run` commands (it finds the rejects of a list of folders,
optionally with their subfolders, from the index rows or else the sidecars on
disk, so a folder never opened counts too, counts them and their bytes per
folder for the confirmation dialog, and moves each RAW and its sidecars to
the OS trash, recording every run that moved something in `Runs`, the newest
100, which `trash_rejected_undo` takes one from to restore its files, RAW
before its sidecars, never overwriting, keeping what came back for
`trash_rejected_redo` to move to the Trash again), `src/foldersidecars.rs`
the folder menu's `Rewrite Sidecars from Index…` and `Delete Sidecars…`
commands (`rewrite_sidecars_preview` / `rewrite_sidecars_run` mark the
folder's index rows dirty and push them through the sidecar writer, patching
only the judgment fields of the current format's sidecars, and
`delete_sidecars_preview` / `delete_sidecars_run` move those sidecars to the
OS trash as one `Runs` run and clear the rows' judgments), `src/sequence.rs` the
`Sequence JPEG Timestamps…` commands (`sequence_preview`,
`sequence_run`, `sequence_cancel`) and their `sequence-progress` /
`sequence-done` events, and
`src/photolab.rs` the PhotoLab database lookup that gives a fresh `.dop` the
registered image's Source and master Item Uuids so PhotoLab does not import it
as a virtual copy (see `docs/agents/photolab.md`), and
`src/sidecar.rs` the coalescing sidecar writer thread and `SidecarFormat`, the
XMP, `.dop` or both setting chosen in the settings modal and persisted in the
`sidecarFormat` key of the settings store, next to the configurable
`xmp:Label` names per color persisted in the `labelNames` key, and `src/shortcuts.rs` the keymap:
the default keys and the
user's overrides, persisted in the `shortcuts` key, and `src/mcp.rs` the MCP
companion: the loopback Streamable HTTP server turned on by the `mcpEnabled`
key, its tools, and the bridge that asks the main window over the
`mcp-request` event and the `mcp_reply` command, and `src/diagnostics.rs`
the panic hook, the `log_frontend` command that writes the frontend's
`uncaught-js:` and `invariant:` lines, and the `invariant!` helper, which put
those failures in `Riffle.log` under greppable prefixes (see
`docs/agents/app-log.md`)).

The frontend lives under `crates/app/ui` (TypeScript built by Vite+, configured
in the root `vite.config.ts`; `pnpm exec vp {dev,build,check,fmt,test}`) and is
formatted, linted, type-checked and tested by `mise run ci`; its
`style.css` holds shadcn/ui's Neutral dark tokens and the shared component
classes every control uses (see `docs/agents/ui-styling.md`),
`src/icons.ts` the inlined Lucide icons,
`src/context.ts` builds the items of the strip's HTML right-click menu,
`src/focus.ts` places the `f` focus mark and holds the good-photo rule
(`photoTier`: the AF eyes in focus, the eyes open and the face toward the
camera, from the values pass 2 stored for the AF face), which colors the
mark bright green and puts the strip's face icon on a good frame, and the
eyes' openness the meta pane shows, and
`src/meta.ts` groups the meta pane rows by provenance (EXIF, Maker note
and Analysis, whose rows include the AF eye in-focus probability and the
`Eyes` judgment with its openness and `Head pose`, from the values pass 2 stored for
the AF face, else the `eyes_of` judgment), `src/eyes.ts` caches the `eyes_of` judgments per file with one
in flight at a time, `src/facemesh.ts` holds MediaPipe's face parts
outline and iris edges the focus mark draws over the judged face, `src/filter.ts` decides which files the strip's filter menu
lets through (including its `AF eye` section, whose `Good` item applies the good-photo rule, and its `Eyes` section on the
stored eye state), `src/companion.ts` answers the MCP bridge's
requests over the main window's view state, `src/resume.ts` picks the file a
folder reopens at and coalesces the writes that remember it, `src/idle.ts`
holds an operation pressed during a scan (Move Rejected to Trash, the
renames) until the scan ends, `src/viewonly.ts`
decides from the listed paths whether a folder is JPEG-only and so opens
view-only (no judgment, capture-time order), `src/sidecars.ts` holds the
Rewrite Sidecars from Index / Delete Sidecars dialog's text and its flow,
`src/sequence.ts` holds the
Sequence JPEG Timestamps dialog's text and its flow from the folder tree's right-click
through the preview to the run's end, and `src/uncaught.ts` builds the
`uncaught-js:` line of an uncaught error or unhandled rejection and the flood
guard that caps them per session.

## Language

**Write everything in the repository in English.** This covers documentation
(`README.md`, `CLAUDE.md`, `docs/**`, skill and agent definitions), code
comments, commit messages, PR titles and bodies, and any other text that lands
in the repository. The exceptions are `README.ja.md` and `docs/humans/*.ja.md`,
the Japanese translations of `README.md` and `docs/humans/<name>.md`; their
bodies are Japanese. Keep each pair in sync: a PR that changes `README.md` or
`docs/humans/<name>.md` updates `README.ja.md` or `docs/humans/<name>.ja.md` in
the same PR, and vice versa.

Conversation with the user stays in Japanese; only what gets committed is
English.

## Commits

Conventional Commits, in English. Example:
`feat(cli): add a partial decode benchmark`

## Development

Do all development through the `develop` skill (`/develop`), however small the
change. Do not edit code and open a PR by hand or through `/pr` alone.
