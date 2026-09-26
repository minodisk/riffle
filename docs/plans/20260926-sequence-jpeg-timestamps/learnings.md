# Learnings

## Step 1

- Error types: `riffle-core` mixes `anyhow::Result` (the parsers in
  `arw.rs` / `reader.rs`) and `Result<_, String>` (`scan.rs`, `xmp.rs`). The
  ported TIFF walker and `ExifDateTime` keep lapse's `anyhow`; the
  folder-level API (`collect_jpegs`, `output_dir`, `plan`, `run`) and the
  per-file results return `String`, formatted with `{:#}` so the anyhow
  context chain survives, which is what the app needs to send to the UI.
- `Summary` got a `total` field besides `results` and `canceled`: a canceled
  run omits the files it never started from `results`, so the app needs the
  target count separately for "canceled, N of M written" (progress alone does
  not carry it when the cancel lands before the first file).
- Files whose `DateTimeOriginal` cannot be read have no time to sort by, so
  `plan` puts them last (natural order among themselves). They get no copy in
  the output folder.
- SubSec ordering: `arw.rs` keeps `shot.subsec` as a raw string without any
  numeric interpretation, so the fraction rule lives in
  `sequence::cmp_subsec` (right-pad both with `0` to the same length and
  compare as strings). A SubSec that is not all digits after trimming NUL and
  spaces reads as absent (0) rather than failing the file.
- The natural filename tie-break needs no explicit comparator: `collect_jpegs`
  already yields natural order and `sort_by` is stable.
- Progress is reported under a mutex around the counter so `done` is
  monotonic across the rayon workers.
- The cancel test needs enough files (200) that rayon cannot have started all
  of them before the first progress callback sets the flag.
- On Windows, editing files with Python's text mode rewrites LF as CRLF; the
  repo is LF (`git ls-files --eol`), so normalize after such edits.
- `tempfile::NamedTempFile` creates files as `0600` on Unix, and `persist`
  keeps that mode, so the copies are owner-only there. This follows the plan's
  "copies do not carry the source permissions"; noted in case it surprises
  someone sharing the output folder.

## Step 2

- The run state is simpler than `Scans`: no `pending` / `preparing`, since a
  run is spawned in the same call that mints its id. `begin` holds the lock
  across the spawn, so the task cannot clear its own entry before it is
  stored; it is a plain function over `&Sequences` so the refusal test does
  not need an `AppHandle`.
- The task clears its `running` entry before emitting `sequence-done`, so a
  frontend that starts a new run in answer to `sequence-done` is not refused.
- A folder-level error from `sequence::run` (no JPEGs, an output folder that
  cannot be rebuilt) happens inside the spawned task, after `sequence_run`
  has returned the id, so it arrives as `sequence-done` with one failure
  keyed by the source folder, `total: 0`.
- `sequence_preview` is `sequence::run` with `dry_run`, which gives `changed`
  per file for free; `plan` alone does not know it.
- The app-side preview test builds a minimal JPEG (SOI, an Exif APP1 with
  only the Exif IFD pointer and `DateTimeOriginal`, EOI): the dry run reads
  nothing else, so the core tests' `image`-encoded builder is not needed.
- Watcher check: `watch.rs` watches the open folder with
  `RecursiveMode::NonRecursive`, and the output is a sibling of the picked
  folder, so a run on the open RAW folder fires no `folder-changed`.
- The macOS menu icons are PNGs rendered from SF Symbols by
  `tools/macos/export-menu-icons.swift`, which cannot run on Windows, so the
  new item is a plain `MenuItem` on every platform for now.

## Deferred issues (todo candidates)

- Give `File > Sequence JPEG Timestamps…` a macOS menu icon like its File
  neighbours: add an SF Symbol (e.g. `clock.arrow.circlepath`) to
  `tools/macos/export-menu-icons.swift`, render it on macOS into
  `crates/app/icons/menu/`, and switch the item in `crates/app/src/main.rs`
  to the `IconMenuItem` / `MenuItem` `cfg` split. Basis: Step 2 of this plan
  ("icon on macOS if the neighbours have one"), implemented on Windows where
  the export script cannot run.
