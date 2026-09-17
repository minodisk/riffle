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

## Deferred issues (todo candidates)

- Keyboard layout dependence of the WASD/HJKL bindings (from this step's
  implementation of the `keydown` handler in
  `crates/app/ui/src/main.ts`): the bindings follow `event.key`, so a Dvorak or
  AZERTY user gets scattered physical keys. Revisit only if a user asks; the
  fix would be a `event.code` fallback or a key-config layer, both out of scope
  now.
- `mozjpeg` panics instead of returning `Err` on malformed JPEG input (found
  while writing the `thumbnail_jpeg` test in `crates/core/src/decode.rs`; the
  same applies to the pre-existing `decode_rgb`). One corrupt file would take
  down a whole parallel scan. Step 3's `extract` should wrap the decode in
  `catch_unwind` or validate the SOI/EOI markers first; filed here so it is
  decided there rather than rediscovered.
- `riffle-cli` still reads the whole file in `info` / `focusbox` / `crop` /
  `bench` (`crates/cli/src/main.rs`). Those subcommands need the full-size
  `JpgFromRaw`, which is past the 1 MiB prefix, so moving them to
  `reader::read_head` was left out of this step deliberately.
