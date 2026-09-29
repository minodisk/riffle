# Learnings: canon-nikon-raw

## Step 1: shared Exif reader

- The IFD0 + Exif IFD reading moved from `jpeg.rs` to the crate-private
  `crates/core/src/exif.rs` (`mod exif;` in `lib.rs`, not `pub`). It exposes
  `read(tiff, ifd0)` for a single TIFF and `read_ifd0` / `read_exif_ifd` for
  a container that keeps the two IFDs in separate TIFFs (CR3's `CMT1` /
  `CMT2`); `read_ifd0` returns the orientation and the Exif IFD offset.
  `read_exif_ifd` clears `estimated_f_number` itself, so feed all the Exif
  IFD entries in one call.
- The tag and type constants (`TAG_*`, `TYPE_*`) and `ascii` / `rational` /
  `integer` are `pub(crate)` in `exif.rs`. `jpeg.rs`'s tests reach them with
  `use crate::exif::*;` (the only change to that test module besides
  visibility), so their bodies are unchanged.
- The synthetic TIFF writer stays in `jpeg::tests`: `W(little_endian)`, the
  `Field` type and all of `W`'s methods (`u16`, `u32`, `ascii`, `short`,
  `long`, `rational`, `srational`, `tiff`) are now `pub(crate)`, alongside
  `plain_jpeg` and `with_exif`. `W::tiff` always appends an 0x8769 pointer to
  IFD0; a caller that wants a lone IFD (e.g. CR3's `CMT2`) can pass its
  entries as `ifd0` and ignore that pointer, as `exif::tests` does. NEF's
  SubIFDs / MakerNote will need a more general builder or hand-built bytes.
