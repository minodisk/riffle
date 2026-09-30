# Learnings: drop unsupported bodies from todo.md

## Step 1

- `main` already had the FinePix trim from `clear-three-todos` Step 1
  (`exif::read_ifd0` trims `Make` / `Model`), so the section's fallback
  applied: the heading and Background were rewritten around the SL1000 and
  the remaining TODO became the real-device check on the SL1000 RAF (both
  `Make` and `Model`), instead of the plan's pre-trim replacement text.
- Sample sweep (a direct Exif IFD0 `Make` / `Model` read of the 3,568
  files under `D:\Photos\samples\`, and `riffle-cli info` for the ORF
  orientations):
  - Fujifilm bodies that pad `Model`: FinePix E500, E550, E900, F700,
    S100FS, S3Pro, S5000, S5200, S5500, S5600, S5Pro, S7000, S9500, S9600
    (all unsupported), and the listed FinePix SL1000 (2013, 2048x1536),
    which pads both `Make` (`FUJIFILM               `) and `Model`
    (`FinePix SL1000         `). Every other listed FinePix / X / GFX body
    writes both tags unpadded.
  - Every listed OM System / Olympus body pads both tags
    (the shared `exif::read_ifd0` trims them now; `orf.rs` no longer has a
    local trim).
  - Nikon: only the D1 and D100 pad (unsupported). Sony: only the DSLR-A100
    to A900 pad `Make` (unsupported). No listed Sony / Canon / Nikon /
    SIGMA / Leica body pads.
  - ORF samples of a listed body with a non-1 orientation:
    `E-M1MarkII_olympus_om_d_e_m1_mark_ii_{01,07,09,34,82}.orf`
    (Orientation 8). The XZ-10 is unlisted and the E-30 is from 2008.
- The Python rewrite of `todo.md` had to keep its line endings
  (`newline=''`) so the diff stayed limited to the three passages.
