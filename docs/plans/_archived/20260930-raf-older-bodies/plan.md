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

# Widen the documented Fujifilm RAF compatibility

## Purpose

Fujifilm RAF support shipped with 22 verified bodies (plan:
`docs/plans/_archived/20260930-fujifilm-raf/plan.md`). A later sweep over 228
RAFs from 87 bodies in `D:\photos\samples\RAF\` (local only, never committed)
found that every file parses from the 1 MiB prefix and decodes its preview
and 1:1 view on current `main`, without any code change. Restricted to the
bodies with at least one CC0 raw.pixls.us sample (the only samples Riffle
counts as verification), 57 bodies are new relative to the README list.

Most of them are older X-series and FinePix bodies whose embedded JPEG is
1920x1280 or smaller, so their 1:1 view is a small preview rather than a
useful pixel check. The docs should say both things: the bodies open, and
what the 1:1 view is worth on them. Users of an X-T30, GFX 50R or GFX 50S
(4416x2944 / 4000x3000 JPEG, the same as the listed bodies) get a plain
README entry; the rest get a `docs/cameras.md` row that shows the embedded
JPEG size, and one README sentence.

Decided with the user (2026-09-30): README lists only X-T30, GFX 50R and
GFX 50S plus one sentence; the JPEG sizes go in a Fujifilm-only table in
`docs/cameras.md`; `docs/raw-formats.md` is qualified too; the 8 bodies with
only CC BY-NC-SA samples are left out.

## Steps

- [x] Step 1: Update the README compatibility lists, `docs/cameras.md`, `docs/raw-formats.md` and `todo.md` for the 57 CC0-verified RAF bodies
  - Done when:
    - `README.md` "RAW formats and cameras" RAF list gains `Fujifilm X-T30`,
      `Fujifilm GFX 50R` and `Fujifilm GFX 50S` (placed next to their
      siblings: X-T30 after X-T30 II, GFX 50R / GFX 50S after GFX50S II),
      and the RAF paragraph (currently "A RAF holds one embedded JPEG,
      4416x2944 on the X bodies and 4000x3000 on the GFX bodies ...") is
      reworded so it is true of the listed bodies, plus one sentence that
      older X-series and FinePix bodies (naming a few, e.g. X-T1 to X-T20,
      X-Pro1 / X-Pro2, X100S to X100F, the FinePix line) also open but
      their 1:1 view is limited to the embedded preview, 1920x1280 or
      smaller, and points at `docs/cameras.md` for the per-body size.
    - `README.ja.md` gets the same list entries and the same sentence in
      Japanese, in the same PR.
    - `docs/cameras.md` gains one row per new body with a CC0 sample (57
      rows, list below), with AF point `✓` / `–` as measured on the CC0
      samples only, AF frame size `–`, face tracking `–`, sub-second `–`
      (none of the 57 records it), and the embedded JPEG size visible per
      body in a Fujifilm-only table grouped by size. The existing Fujifilm
      paragraph under the table is extended: the AF point `–` bodies are the
      manual-focus samples (X-A2, X-E3, X100F, XF10, GFX 50S, FinePix S3Pro,
      S100FS, S6000fd) plus the bodies that write no `FocusPixel` at all
      (FinePix S2Pro, S5Pro, DBP for GX680), and the 1:1 view on the
      small-JPEG bodies is that JPEG.
    - The 8 bodies whose only samples are CC BY-NC-SA 4.0 (FinePix E550,
      E900, F600EXR, F700, F900EXR, HS10 HS11, HS20EXR, S5600) appear
      nowhere.
    - `todo.md` gains one section for the follow-up found during the sweep:
      the old FinePix bodies write Exif `Model` with trailing spaces (e.g.
      `FinePix E550   `), which the meta pane shows as is; a trim in
      `crates/core/src/exif.rs` (shared by `jpeg.rs`, `nef.rs`, `cr3.rs`,
      `raf.rs`) is the candidate, not done here. The existing "Core: M-RAW
      RAF files are unverified" section stays open; add one line to it that
      the 228-file sweep of 87 bodies found no M-RAW either (`0x48` zero on
      all).
    - `docs/raw-formats.md`: the two places that say "4416x2944 on the X
      bodies" (the Preview JPEG row of the format table and the "RAF: the
      Exif rides inside the embedded JPEG" subsection) are qualified to the
      recent X bodies, with the older ones at 1920x1280 or below.
    - `mise run ci` passes (lychee checks the links).
    - The samples and the sweep harness stay out of the repository; this
      plan folder's `learnings.md` records the per-body measurements table
      (body, CC0 sample file names, JPEG size, AF, sub-second) so the rows
      can be audited later.
  - Implementation approach:
    - Source of truth for each row: the sweep's `raf.tsv` (session
      scratchpad
      `C:\Users\daisu\AppData\Local\Temp\claude\C--Users-daisu--herdr-worktrees-riffle-worktree-clear-forest-819a\faa0d4fa-ea32-4b1e-99f9-1f3beee9e536\scratchpad\raf.tsv`,
      columns `prev_wh`, `focus`, `subsec`) joined with
      `D:\photos\samples\MANIFEST-pixls.tsv` on `file` (strip the `RAF/`
      prefix), keeping only rows whose `license` is "Creative Commons 0 -
      Public Domain". If the scratchpad is gone, re-run `riffle-cli`
      (`scan` / `focusbox`) over the CC0 files, or the harness under
      `scratchpad\rafeval\`. Do not take AF from `MANIFEST-faces.tsv` /
      `MANIFEST-edge.tsv` samples (Photography Blog, CC BY-SA,
      mirrorlesscomparison): they are not CC0. The measured CC0 result is:
      - 4416x2944, AF ✓: X-T30
      - 4000x3000: GFX 50R ✓; GFX 50S – (MF samples)
      - 1920x1280, AF ✓: X-A1, X-A3, X-A5, X-A7, X-A10, X-E1, X-E2, X-E2S,
        X-H1, X-M1, X-Pro1, X-Pro2, X-T1, X-T2, X-T10, X-T20, X-T100,
        X-T200, X100S, X100T, X70
      - 1920x1280, AF –: X-A2, X-E3, X100F, XF10 (all MF on the CC0 samples)
      - 2176x1448, AF ✓: FinePix X100
      - 2048x1536, AF ✓: X10, X20, X30, XF1, XQ1, XQ2, X-S1, FinePix
        F550EXR, F770EXR, HS30EXR, HS33EXR, HS50EXR, S1, S200EXR, SL1000
      - 1600x1200, AF ✓: FinePix S5200, S6500fd, S9500, S9600;
        AF –: FinePix S100FS, S6000fd (MF)
      - 1280x960, AF ✓: FinePix S5000, S5500, S7000
      - 1440x960, AF –: FinePix S3Pro (MF), FinePix S2Pro, FinePix S5Pro
        (no `FocusPixel`)
      - 1344x960, AF –: DBP for GX680
    - Keep the main table's five columns unchanged and add the 57 rows after
      the GFX50S II row, in the README's order (X-T30, GFX 50R, GFX 50S, then
      the older X bodies, then the FinePix bodies, then DBP for GX680). Put
      the JPEG sizes in a short second table under the Fujifilm paragraph
      ("Embedded JPEG size on the Fujifilm bodies", one row per size class
      listing the bodies).
    - Body names: use Fujifilm's marketing names consistent with the existing
      rows (`Fujifilm X-T30`, `Fujifilm GFX 50R`, `Fujifilm GFX 50S`,
      `Fujifilm X100F`, `Fujifilm FinePix X100`, `Fujifilm FinePix S2 Pro` /
      `S3 Pro` / `S5 Pro`, `Fujifilm FinePix HS30EXR`, `Fujifilm DBP for
      GX680`). The Exif `Model` strings differ in spacing (`FinePix S2Pro`,
      trailing spaces); do not copy them verbatim.
    - The `todo.md` follow-up is a doc note only; no code, and `arw.rs` /
      `cr3.rs` / `nef.rs` / `raf.rs` are not touched.
    - `docs/performance.md` describes the measured 46-file survey and names
      it, so it stays as is.

## Trade-offs and risks

- **Where to show the embedded JPEG size in `docs/cameras.md`.** A new column
  on the main table would need a value for every Sony / Canon / Nikon / OM
  row and would mean two things (on ARW / NEF / CR3 the preview and the
  full-size JPEG differ). A Fujifilm-only table grouped by size is cheap and
  honest; chosen.
- **README list scope.** Listing all 57 in the README would make the RAF list
  79 bodies long and present a 1280x960 1:1 view as equal to the 4416x2944
  ones; not taken.
- **AF `–` on X-E3, X100F and XF10.** On CC0 samples these are manual focus
  samples only, so their AF column is `–` ("unconfirmed"), even though
  non-CC0 samples of the same bodies show `FocusPixel` working. Consistent
  with how the X-T3 / GFX 100 rows were handled.
- **The 8 CC BY-NC-SA-only bodies** are excluded on license grounds only;
  they parsed and decoded fine. If a CC0 sample appears later they can be
  added by the same rule.
- **Verification is on a scratch harness + `riffle-cli`, not the app
  window.** Same as the original RAF plan's rows.

## Progress

- (2026-09-30) Step 1 complete
