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
