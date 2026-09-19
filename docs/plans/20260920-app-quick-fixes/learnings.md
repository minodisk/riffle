# Learnings

## Step 1

- The full JPEG size lives on `riffle_core::partial::Crop` as
  `image_width`/`image_height`, so `decode_crop` callers get it too. The crop
  header carries it as two `u16`s at offsets 28/30 (a JpgFromRaw never exceeds
  65535 px per axis), and the kind tag moved from 4 (`CROP_KIND_RGBA_V2`) to 5
  (`CROP_KIND_RGBA_V3`); 3 (`CROP_KIND_RGBA_V1`) and 4 were both already used,
  so 5 is the next free value.
- The placeholder still uses the sensor fallback until the current file's crop
  arrives (option a); the rect maths is in `crates/app/ui/src/zoom.ts`.
