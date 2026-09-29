# Learnings: heif-cr3-message

## Step 1

- The local HDR PQ samples (`R8.CR3`, `R5m2.CR3`, not committed) carry `HEVC`
  and `free` as the first track's sample entry sub-boxes (read at the usual
  82-byte offset into the `CRAW` entry); the other two `CRAW` tracks carry
  `CMP1` / `CDI1` / `free` and the fourth track is `CTMD` with no sub-boxes.
  No `hvcC` or `GRID` showed up at that level, so the `HEVC` box alone is the
  signal. `PRVW`'s data starts with `00 00`, not the JPEG SOI. The normal
  samples (R5, R6 Mark II) carry `JPEG` / `free` on the first track.
- `jpeg_track` was split into `sample_entry` (the sub-boxes and the `stbl`
  boxes) and `jpeg_track` (the offset and length of a JPEG track), so the
  track loop in `parse` can see both `JPEG` and `HEVC` in one walk.
- `riffle-cli info` prints no error (it only calls `read_metadata`); the
  `candidates` subcommand goes through `read_preview` and printed
  `R8.CR3  error: HDR PQ (HEIF) CR3: its HEVC preview is not supported yet`
  (and the same for `R5m2.CR3`), with the other 13 CR3 samples decoding.
  `info` on the two HDR PQ files still shows the capture time and the AF
  point, so `read_metadata` is intact.
- The meta pane does not read the index's EXIF: it calls the `metadata`
  command, which runs `reader::read_metadata` on the file, so an error row
  still gets its EXIF rows there.
- Tooling trap: a Python heredoc passed through the Bash tool turned the
  escape text `\0` / `\x01` inside a byte-string literal into real NUL and
  0x01 bytes when writing a Rust source file (even with `'EOF'`). Build such
  bytes with `bytes([92])` or edit with the Edit tool.

## Deferred issues (todo candidates)

- **Pending manual check** (GUI, Windows, on `D:\photos\samples\CR3\R8.CR3`
  and `R5m2.CR3`): open the folder in the app and check that (1) the strip
  cells of the two files are the failed color and show "HDR PQ (HEIF) CR3:
  its HEVC preview is not supported yet" over the image box, with the same
  text as the cell's tooltip; (2) the viewer's note reads
  `<path>: HDR PQ (HEIF) CR3: its HEVC preview is not supported yet`; (3) the
  meta pane shows the EXIF rows (camera, lens, exposure, capture time); (4) a
  star, a flag and a color label each write a sidecar next to the file. A
  folder scanned before the change should re-extract once (the
  `EXTRACTOR_VERSION` bump to 8) and pick up the new text. The step's
  checkbox was ticked on the automated criteria (unit tests, `mise run ci`,
  and `riffle-cli candidates` on the two samples); this GUI check was not
  done in the implementation session.
