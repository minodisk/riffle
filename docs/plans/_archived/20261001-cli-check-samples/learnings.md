# Learnings: `riffle-cli check`

## Step 1

- `partial::decode_focus_crop` is not pure Rust as the plan assumed: it calls
  libjpeg through `mozjpeg-sys` with `jpeg_std_error`, whose default
  `error_exit` calls `exit(1)`. The first run over `D:\Photos\samples` died
  mid-way with libjpeg's `Empty input file` / `Not a JPEG file: starts with
  0x3c 0x44` on stderr, exit code 1 and no summary (the DNG and NEF folders
  each killed it on their own). `catch_unwind` cannot help, since nothing
  unwinds. The `full` stage now runs the guarded `decode_rgb` on the
  full-size JPEG first and only passes bytes that decode to the partial
  decode; the two files it caught are `NEF\NIKON_D70_Nikon.nef` and
  `DNG\CGO3P_YUN00007.dng` (`full: panic while decoding the JPEG`). The
  extra full decode is why the run takes about 2.5 minutes on 24 threads
  instead of seconds.
- libjpeg warnings from the partial decode (`Corrupt JPEG data: N extraneous
  bytes before marker 0xd7`, `Invalid SOS parameters for sequential JPEG`)
  still go straight to stderr; the failure lines and the summary are on
  stdout.
- Failures are sorted by path only (a stable sort), so a file's stages stay
  in run order (scan, preview, full) instead of the alphabetical order a
  sort on the whole line gives.
- `cargo` is not on the Git Bash PATH here; `mise x -- cargo ...` works.

### Sample run

`cargo run -p riffle-cli --release -- check D:\Photos\samples` (24 threads),
exit code 1:

```
arw: 390 ok, 0 failed
cr3: 109 ok, 0 failed
dng: 223 ok, 1541 failed
jpeg: 13 ok, 0 failed
jpg: 483 ok, 29 failed
nef: 378 ok, 15 failed
orf: 131 ok, 12 failed
raf: 228 ok, 0 failed
3552 files in 10 folder(s), 24 threads, 1597 failed: 153.58s total
Error: 1597 file(s) failed
```

What failed:

- `no embedded preview` (1516 DNGs, mostly phone / drone / scanner DNGs, 10
  old Nikon bodies and 2 COOLSCAN scans, 12 old Olympus compacts): the files
  genuinely carry no preview JPEG; out of scope per the camera support scope
  (pre-2010 or no-preview bodies).
- `not a little-endian TIFF/ARW` (42 DNGs: Pentax, Ricoh GR / GXR, Leica M,
  iPhones, QooCam, Blackmagic): big-endian (`MM`) DNGs, which the DNG path
  (the ARW parser) does not read. A real parser gap, not a sample defect.
- `invalid TIFF byte order` (4 `NIKON_D70s_Issue 247-*.nef`): deliberately
  broken samples from a bug report; genuine sample defects.
- `unexpected end of file` (3 DNGs) and `IFD offset out of range` (1 DNG):
  truncated or malformed samples.
- JPG: test images of decoder corner cases (lossless, CMYK / YCCK, 12-bit,
  missing SOF / SOS, empty, 6x6) that mozjpeg rejects; `panic while decoding
  the JPEG` on `preview`, `panic while encoding the thumbnail` on `scan`.
  Sample defects or unsupported JPEG variants.
- `NEF\NIKON_D70_Nikon.nef` and `DNG\CGO3P_YUN00007.dng`: a corrupt embedded
  JPEG, including the full-size one (see above).
- `Canon_EOS_M50_CanonRaw.cr3` is not in the samples, so the CR3 folder is
  clean.

## Deferred issues (todo candidates)

- **The app's 1:1 view can kill the whole app on a malformed full-size JPEG.**
  `crates/core/src/partial.rs` (`decode_region`, behind `decode_focus_crop`
  and `decode_crop`, the latter with no caller outside the module) sets `cinfo.common.err = jpeg_std_error(&mut err)`
  without overriding `error_exit`, so libjpeg calls `exit(1)` on a fatal
  error instead of returning or panicking. The app's `focus_crop` command
  (`crates/app/src/commands.rs`) would end the process on files such as
  `D:\Photos\samples\NEF\NIKON_D70_Nikon.nef` or
  `D:\Photos\samples\DNG\CGO3P_YUN00007.dng`. Fix by installing an
  `error_exit` that panics (as the `mozjpeg` crate does) and wrapping the
  call in `catch_unwind`, then drop the `decode_rgb` pre-check in
  `crates/cli/src/main.rs` `check_file`. Found by the Step 1 sample run.
  Done when `focus_crop` returns an `Err` on both files without ending the
  process, `riffle-cli check` finishes with a summary without the pre-check,
  and the `docs/agents/tauri-app.md` entry on `partial.rs`'s `exit(1)` says
  the guard is in place.
- **Big-endian DNGs do not open.** 42 samples under `D:\Photos\samples\DNG\`
  (Pentax K-series, Ricoh GR / GXR, Leica M, iPhone, QooCam, Blackmagic) fail
  every stage with `not a little-endian TIFF/ARW`: the DNG path goes through
  `crates/core/src/arw.rs`, which only reads `II` TIFFs. Found by the Step 1
  sample run. Check how `crates/core/src/exif.rs` handles byte order (NEF and
  ORF go through it). Done when `riffle-cli check` on those DNGs no longer
  reports `not a little-endian TIFF/ARW` (each opens or fails for another,
  specific reason), and the finding is in
  `docs/agents/raw-metadata-parsing.md`.
- Files with no embedded preview are unsupported (user decision, 2026-10-01)
  and are to be removed from the samples; no issue is filed for them.
