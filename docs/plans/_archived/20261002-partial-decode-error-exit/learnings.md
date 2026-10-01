# Learnings: partial-decode-error-exit

## Step 1

- A JPEG truncated inside its scan data is not a fatal libjpeg error:
  `jpeg_mem_src` feeds a fake EOI, libjpeg only warns (`Premature end of JPEG
  file`) and the crop comes back with filler rows. The plan's "first half of
  `jpeg(320, 240)`" test input therefore still decoded; the test cuts inside
  the header (`&jpeg[..64]`) instead, which fails as intended. `decode_rgb`
  (the `mozjpeg` crate) does fail on the half-cut, since that crate treats the
  end of input differently; the two are not interchangeable as validators.
- Setting `err.error_exit` on the local after `jpeg_std_error(&mut err)` trips
  rustc's `unused_assignments` (it does not see the read through the raw
  pointer). Writing through the pointer, `(*cinfo.common.err).error_exit = ...`,
  avoids it.
- The `mozjpeg-sys` dependency now names `features = ["unwinding"]`; it was
  already on through the default features, and `Cargo.lock` did not change.
- Real files: `riffle-cli crop` on `D:\Photos\samples\NEF\NIKON_D70_Nikon.nef`
  and `D:\Photos\samples\DNG\CGO3P_YUN00007.dng` returns `Error: libjpeg fatal
  error: Not a JPEG file: starts with 0x3c 0x44` / `... 0x80 0x03` through
  `main`'s `Result` instead of libjpeg's `exit(1)`.
- `riffle-cli check D:\Photos\samples` (release, 24 threads) now runs to its
  summary in **23.0 s** (2332 files, 108 failed), against about 2.5 minutes with
  the `decode_rgb` pre-check. Both repro files are listed under `full` with the
  libjpeg message. The process exits 1 only because `check` bails when any file
  failed.

## Deferred issues (todo candidates)

- **Pending manual check** (desktop app, any platform; the user's): open
  `D:\Photos\samples\NEF\NIKON_D70_Nikon.nef` and
  `D:\Photos\samples\DNG\CGO3P_YUN00007.dng` in the app and switch to the 1:1
  view. Expect the app to stay up and the `focus_crop` error
  (`libjpeg fatal error: Not a JPEG file ...`) to surface however the 1:1 view
  shows a failed crop; note what the UI shows. Step 1 was ticked on the
  automated criteria and the CLI stand-in (`crop` / `check`, which call the
  same `decode_focus_crop`). Files: `crates/app/src/commands.rs`
  (`focus_crop`), `crates/core/src/partial.rs`.
