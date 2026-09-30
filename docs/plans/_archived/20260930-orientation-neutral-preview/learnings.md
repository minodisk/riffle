# Learnings: orientation-neutral preview JPEGs

## Step 1

- The choke point is `reader::neutral`, which both `read_preview` and
  `read_full` map their result through; `jpeg::neutralize_orientation`
  rewrites the IFD0 Orientation (SHORT, count 1) to 1 in place, in the TIFF's
  own byte order, through a new `Tiff::little_endian()` accessor. Anything
  else (no Exif, a cut or malformed segment, a LONG Orientation, no entry) is
  a no-op.
- The GFX 100 sample's embedded JPEG (`GFX_100_fujifilm_gfx_100_13.raf`)
  holds Orientation as a little-endian SHORT, count 1, value 8, in IFD0, so the
  rewrite applies to it.
- The invariant test `every_format_hands_out_jpegs_without_an_exif_rotation`
  was checked to fail with the neutralizer commented out (the ARW preview
  reported 8).
- Deviation from the plan: no HEVC stream can be synthesized here (hpvcd is a
  decoder only and ships no test vectors), so the HDR PQ CR3 case is a
  separate `#[ignore]` test,
  `an_hdr_pq_cr3_hands_out_jpegs_without_an_exif_rotation`, driven by
  `RIFFLE_HEVC_CR3` like `hevc::tests::decodes_the_preview_of_a_real_file`.
  It passed locally against `D:\photos\samples\CR3\R8.CR3`. The HEVC output
  is a fresh mozjpeg JPEG with no Exif, so it takes the no-Exif arm.
- The CLI never dumps the embedded JPEG bytes as is (it decodes, rotates by
  `Arw.orientation` and re-encodes), so no CLI doc needed a change.

## Deferred issues (todo candidates)

- Pending manual check (Step 1; the checkbox was ticked on the automated
  criteria). On Windows, `mise run tauri:dev` (or a release build): open
  `D:\Photos\samples\RAF\` and select `GFX_100_fujifilm_gfx_100_13.raf`
  (GFX 100, Orientation 8). Expected: upright in the main preview, the zoom
  crop and the compare view, the same as in the filmstrip. Then open an ARW
  folder and a JPEG-only folder (e.g. `D:\photos\samples\ARW`,
  `D:\photos\samples\JPG`) and check portrait shots still display upright as
  before. Related files: `crates/core/src/reader.rs`, `crates/core/src/jpeg.rs`.
  Completion criteria: all three views are upright for the GFX 100 RAF and
  the ARW and JPEG-only folders show no regression; the "(Hit)" entry in
  `docs/agents/tauri-app.md` then stands as is, and is corrected if not.
