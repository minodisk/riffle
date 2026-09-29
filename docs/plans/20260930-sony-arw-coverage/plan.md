<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Sony ARW camera coverage

## Purpose

`README.md`'s compatibility list confirms only the Sony α7 V for ARW, while
CR3 and NEF list 13 and 12 bodies each. The ARW parser
(`crates/core/src/arw.rs`) should already read most recent Sony bodies, so
verifying them against the public CC0 samples on raw.pixls.us is the cheapest
way to widen the list before announcing Riffle. Per body we must check not
only the preview but the AF point (`FocusLocation` 0x2027), the AF frame
(`FocusFrameSize` 0x2037), face tracking (`AFTracking` 0x2021) and the
sub-second capture time, since Sony MakerNote tags differ by generation.
Bodies that read correctly are added to `README.md` / `README.ja.md` and
`docs/cameras.md`; bodies that fail are fixed in the parser or recorded in
`todo.md`.

## Steps

- [x] Step 1: Verify the raw.pixls.us Sony ARW samples, fix the parser where needed, and list the bodies that pass
  - Done when:
    - Every candidate body below has been opened from a downloaded sample
      with `riffle-cli info` (metadata, `focus`, `focus mode`, `subsec`),
      `riffle-cli bench` (preview and 1:1 decode), `riffle-cli focusbox`
      (the AF point / frame drawn on the preview) and `riffle-cli faces`
      (orientation on a portrait sample, if any), and `riffle-cli scan` has
      run over the whole sample folder with 0 errors for the listed bodies
    - `learnings.md` in this plan folder records, per body: model string,
      preview and full JPEG sizes, whether the 1 MiB prefix was enough for
      metadata and preview, orientation (if a portrait sample exists), the
      meta-pane EXIF rows (capture time, sub-seconds, exposure, F, ISO,
      focal length, lens), `FocusLocation` and whether the drawn point lands
      on the subject in an off-center shot, `FocusFrameSize`, `AFTracking`,
      and the crop-mode behaviour where a crop sample exists (α7 IV APS-C,
      α7R V S35)
    - `README.md` and `README.ja.md` list, under `ARW`, exactly the bodies
      whose whole-file samples opened (preview + 1:1 + metadata), in the same
      PR; `docs/cameras.md` gains one row per listed body with `✓` / `–` for
      AF point, AF frame size, face tracking and sub-second, and a sentence
      under the table for any body-level caveat (as the Nikon Z 8 / Canon R6
      sentence does today)
    - Any body whose preview, metadata or AF point is wrong is either fixed
      in `crates/core/src/arw.rs` with a synthetic-TIFF unit test next to the
      existing ones, or gets a `todo.md` section (basis, files) and is left
      off the README
    - `EXTRACTOR_VERSION` in `crates/app/src/index.rs` is bumped if and only
      if the AF-point reading logic changed
    - `mise run ci` passes
  - Implementation approach (as far as it is known; omit if unknown):
    - Samples: raw.pixls.us's HTML page is JS-rendered, but the Apache
      directory listing `https://raw.pixls.us/data/Sony/<MODEL>/` lists the
      `.ARW` files and `curl -L` downloads them. Download into the OS temp
      dir (precedent: `../_archived/20260929-canon-nikon-raw/learnings.md`),
      never into the repo. exiftool is not installed locally; cross-check
      EXIF rows with an independent TIFF dump (Python) as the CR3 / NEF
      steps did.
    - Model folders present (as of 2026-09-30) and their files:
      - `ILCE-1/`: `A1_full_compressed.ARW`, `A1_full_lossless_compressed.ARW`, `A1_full_uncompressed.ARW`
      - `ILCE-9M2/`: three `SONY_A9II_(ILCE-9M2)_-_*.ARW` (compressed 12/14 bit, uncompressed; URL-encode the parentheses)
      - `ILCE-9M3/`: `DSC00110.ARW` … `DSC00118.ARW`
      - `ILCE-7M4/`: `ILCE-7M4_DSC06673…06681` (full-frame uncompressed / lossless L, M, S / compressed, and APS-C uncompressed / lossless M, S / compressed)
      - `ILCE-7RM4/`: `DSC00395…00398.ARW`; `ILCE-7RM4A/`: `DSC00551.ARW`, `compressed_12_bit.ARW`, `compressed_14_bit.ARW`, `uncompressed_14_bit.ARW`
      - `ILCE-7RM5/`: `7RM5-Lossless{CompressedLarge,CompressedMedium,CompressedSmall,Uncompressed}.ARW`, `7RM5-LossyCompressed.ARW`, and the same four `7RM5-S35-*` crops
      - `ILCE-7SM3/`: `DSC01568.ARW`, `DSC01569.ARW`
      - `ILCE-7C/`: `DSC00018.ARW`, `DSC00107%5b1%5d.ARW`; `ILCE-7CM2/`: `DSC02429…02436.ARW`; `ILCE-7CR/`: `DSC00794…00802.ARW`
      - `ILCE-6400/`: `DSC00087.ARW`, `DSC00475.ARW`; `ILCE-6600/`: `_DSC0268.ARW`; `ILCE-6700/`: `DSC00001.ARW`, `DSC00002.ARW`
      - `ZV-E1/`: `DSC00489.ARW`; `ZV-E10/`: `DSC00002.ARW`
      - Not on raw.pixls.us: α1 II (ILCE-1M2), ZV-E10 II; they stay unlisted.
      The α7 V is already verified from local files and stays as is.
      The user decided (2026-09-30) to verify only these candidates; the
      α7 III, α7R III, α9, FX3 and FX30 folders are out of scope.
    - Name bodies in the README by their marketing name (`Sony α7 IV`,
      `Sony α7R V`, `Sony α6700`, `Sony ZV-E1`, ...) as the α7 V row does;
      record the `Model` string (`ILCE-7M4`, ...) in `learnings.md`.
    - Sony.pm (fetched from GitHub, exiftool not installed): `FocusLocation`
      0x2027 (int16u[4]) and `FocusFrameSize` 0x2037 (int16u[3], stored as
      `UNDEFINED[6]` on real files, see
      `../../agents/raw-metadata-parsing.md`) carry no model `Condition`;
      `FocusMode` 0x201b and `AFTracking` 0x2021 only the `DSC-` condition
      already coded in `arw::excluded_dsc`. So no per-model gate is expected
      in the parser; verify rather than assume. If a new tag is needed,
      follow `../../agents/tauri-app.md` "Sony MakerNote fields: verify each
      tag's type and model Condition in Sony.pm directly".
    - Points to measure specifically (these are where a fix is most likely):
      - Crop-mode samples (α7 IV `APS-C-*`, α7R V `S35-*`): whether
        `FocusLocation`'s `sensor_w` / `sensor_h` match the embedded JPEG's
        aspect and the point lands on the same subject as the full-frame
        shot of the same scene. `sharpness.rs` scales the point by
        `sensor_w` against the preview width, so a full-sensor coordinate on
        a cropped JPEG would land off by the crop factor (the NEF DX-crop
        risk recorded in the CR3/NEF learnings, Step 5).
      - Older bodies (α6400 / α6600 era): whether the MakerNote
        takes the headerless (Sony5) path or the 12-byte `SONY DSC` header
        path of `maker_note_ifd`, and whether 0x2037 / 0x2021 exist at all
        (if absent, `docs/cameras.md` gets `–` for AF frame size / face
        tracking, not a parser change).
      - Lossless-compressed and Medium / Small RAW sizes (α1, α7 IV, α7R V):
        the IFD0 JPEG (0x0201 / 0x0202) and the SubIFD full JPEG still
        resolve, and the full JPEG size matches the reduced RAW size.
      - Whether metadata and the preview lie inside the 1 MiB prefix
        (`HEAD_LIMIT`), as recorded for NEF / CR3; note any body whose
        preview needs the ranged read.
      - Whether `FocusLocation` is present on samples shot in manual focus
        (`focus mode: 0`) and that `trusted_focus` drops it as designed.
    - `docs/cameras.md`: the `Face tracking` column means `AFTracking`
      recorded on the sample; a sample shot without a face will show `0`
      even on a body that supports it. Mark `✓` only when the sample shows
      it, add a note under the table that `–` means "not observed on the
      sample", and say so in `learnings.md`; do not guess from the spec sheet.
    - `docs/raw-formats.md` and `docs/performance.md` change only if a
      body-level caveat (crop coordinates, older header) surfaces; otherwise
      leave them alone.

## Trade-offs and risks

- **Single PR vs. split.** The verification, any small parser fix and the
  documentation land in one PR. If the survey uncovers a parser change that
  alters the AF-point mapping (for example crop-mode `FocusLocation`
  rescaling, which also bumps `EXTRACTOR_VERSION` and re-extracts every index
  row), stop and report so the work can be demoted into a `fix(core)` step
  first and a docs step after, so the parser change can be reviewed and
  reverted on its own. Record the split in this file when it happens.
- **Crop-mode samples.** Two readings: (a) the crop JPEG carries a
  `FocusLocation` already in crop coordinates (nothing to do), or (b) it is
  in full-sensor coordinates (the point lands off by ~1.5x). If (b) and the
  fix is not obvious from the sample (Sony.pm gives no crop flag near
  0x2027), record it in `todo.md` and list the body with a caveat sentence
  under the `docs/cameras.md` table rather than blocking the PR.
- **Face tracking column honesty.** Most CC0 samples are still-life shots,
  so `AFTracking` will read 0 on bodies that do support it. The note under
  the table says `–` means "not observed on the sample".
- **Sample scenes are mostly centered.** An AF point that reads
  `(sensor_w/2, sensor_h/2)` proves nothing about the mapping; confirm the
  mapping on at least a few off-center samples (the α9 III, α7CR and α7C II
  folders have several frames each, so an off-center one is likely).

## Progress

- (none yet)
