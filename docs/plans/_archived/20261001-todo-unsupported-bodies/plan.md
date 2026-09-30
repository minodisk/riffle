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

# Drop unsupported bodies from todo.md

## Purpose

`docs/cameras.md` now states the support rule: a body released before 2010,
or whose embedded JPEG is under 1280 px on the long edge, is not supported
(its files may open, but no work is done for it). `todo.md` still names such
bodies in three places: the FinePix trailing-spaces item is framed around
the pre-2010 FinePix E550, the pre-2012 NEF item names the D3 / D40 / D70 /
D90 (annotated as unsupported), and the ORF real-device check names the
XZ-10 and the 2008 E-30 as its rotated frames. After this work every item in
`todo.md` is about a listed body, so nothing on the list asks for work on a
body Riffle does not support, and nothing about a supported body is lost.

The sweep behind this plan (a direct Exif IFD0 `Make` / `Model` read of the
3,568 files under `D:\Photos\samples\`, and `riffle-cli info` for the ORF
orientations) found:

- Padded `Model` on Fujifilm: FinePix E500, E550, E900, F700, S100FS, S3Pro,
  S5000, S5200, S5500, S5600, S5Pro, S7000, S9500, S9600 (all unsupported)
  and the **FinePix SL1000** (2013, listed in `docs/cameras.md` at
  2048x1536), which pads both `Make` (`FUJIFILM               `) and
  `Model` (`FinePix SL1000         `) on
  `D:\Photos\samples\RAF\FinePix_SL1000_RAW_file_from_Fijifilm_Finepix_SL1000.RAF`.
  Every other listed FinePix / X / GFX body writes both tags unpadded.
- Every listed OM System / Olympus body pads both tags too, which
  `crates/core/src/orf.rs` already trims locally.
- Nikon: only the D1 and D100 pad (unsupported). Sony: only the
  DSLR-A100 to A900 pad `Make` (unsupported). No listed Sony / Canon /
  Nikon / SIGMA / Leica body pads.
- ORF samples from a listed body with a non-1 orientation:
  `E-M1MarkII_olympus_om_d_e_m1_mark_ii_{01,07,09,34,82}.orf`
  (Orientation 8). The XZ-10 is not listed and the E-30 is from 2008.
- No other `todo.md` item, TODO line or example has an unsupported body as
  its subject. The "widen camera support" item's Tier 1 targets (Pentax,
  Ricoh GR, other Leica bodies, phones) are categories, not bodies, and are
  left as they are.

## Steps

- [x] Step 1: Rewrite the three `todo.md` passages that name unsupported bodies
  - Done when: `todo.md` names no body that is pre-2010 or unlisted for a
    tiny preview; the three passages below read as specified; nothing else
    in `todo.md` changes; `mise run ci` passes; `learnings.md` records the
    sample-sweep numbers above (which bodies pad, which listed body does).
  - Implementation approach:
    - **(1) `### Core: old FinePix bodies show Exif `Model` with trailing
      spaces`**. Rename the heading and rewrite the Background around the
      listed SL1000; keep the TODO line's substance. Replacement for the
      whole section:

      ```markdown
      ### Core: the FinePix SL1000 shows Exif `Make` and `Model` with trailing spaces

      #### Background

      The FinePix SL1000 writes the Exif `Make` and `Model` padded with trailing spaces (`FUJIFILM               ` / `FinePix SL1000         ` on its raw.pixls.us sample, `D:\photos\samples\RAF\FinePix_SL1000_RAW_file_from_Fijifilm_Finepix_SL1000.RAF`), and the meta pane shows them as is; every other listed FinePix and X-series body writes both unpadded. Found by the `raf-older-bodies` sweep (`docs/plans/_archived/20260930-raf-older-bodies/learnings.md`). The listed OM System / Olympus bodies pad both tags the same way, which `crates/core/src/orf.rs` trims locally. A trim in the shared Exif reader is the candidate fix, which would cover every format that goes through it and let `orf.rs` drop its own. Files: `crates/core/src/exif.rs` (shared by `crates/core/src/jpeg.rs`, `nef.rs`, `cr3.rs`, `raf.rs`, `orf.rs`).

      #### TODO

      - [ ] Trim trailing spaces from the Exif `Model` (and `Make`) in `crates/core/src/exif.rs`, with a test on a padded value, and check the meta pane on the SL1000 RAF.
      ```

      If `main` has already changed this section (the `clear-three-todos`
      plan implements the same trim), rewrite whatever remains of it so it
      names only the SL1000, not an unsupported body.

    - **(2) `### Core: pre-2012 NEFs fall back to the full-size JPEG for the
      preview`**. Keep the heading, the D7000 and the 2010-2012 framing;
      delete the D3 / D40 / D70 / D90 clause. The first sentence becomes:

      ```markdown
      NEFs from bodies of about 2010 to 2012 (the D7000 on the raw.pixls.us
      samples) carry one JPEG SubIFD, so `nef::parse` uses the
      full-size JpgFromRaw as the preview too: each page turn decodes a
      ```

      (the rest of the paragraph and the TODO are unchanged).

    - **(3) `### App: real-device check of an ORF folder in the strip,
      sidecars and Move Rejected to Trash`**. In the TODO line, replace
      `the XZ-10 and E-30 frames upright` with `the E-M1 Mark II's rotated
      frames (Orientation 8, e.g.
      `E-M1MarkII_olympus_om_d_e_m1_mark_ii_01.orf`) upright`, re-wrapping
      the surrounding lines to the item's existing width.

    - Touch nothing else in `todo.md`. `docs/cameras.md`, `README.md`,
      `README.ja.md` and `docs/agents/**` are out of scope.

## Trade-offs and risks

- **Overlap with `docs/plans/20261001-clear-three-todos/plan.md` Step 1**,
  which implements the FinePix trim and edits the same section. Decided:
  this small docs PR goes first; that plan's implementer works from the
  renamed section on `main`.
- **Heading rename.** "old FinePix bodies" is false once the item is about
  the 2013 SL1000, so the heading is renamed.

## Progress

- (2026-10-01) Step 1 complete. The FinePix section followed the fallback path because `main` already had the trim.
