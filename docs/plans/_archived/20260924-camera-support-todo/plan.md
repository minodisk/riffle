<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Record the public-sample camera support idea in todo.md

## Purpose

The user no longer owns the Sigma fp L or BF and cannot shoot new samples, so
publicly available sample RAW files are the only way to widen camera support
and to check the open assumptions behind the Sigma BF AF point. Capture that
idea, the sample sources, and the cost tiers in `todo.md` so the work is not
lost. Docs only; no code changes.

## Steps

- [x] Step 1: Add a "Core: widen camera support from public sample RAW files" item to `todo.md`
  - Done when:
    - `todo.md` gains the item below at the end of `## Cross-cutting / other`
      (after "App: SIGMA fp L strip may decode the full-size JPEG per
      thumbnail"), in the file's existing style (`### <Area>: <title>`, a
      background paragraph, `#### TODO` with checkbox bullets, prose wrapped
      at about 78 columns)
    - The relative links resolve (`lychee --offline --include-fragments`
      inside `mise run ci`)
    - `mise run ci` passes
  - Implementation approach:
    - Insert exactly this Markdown, preceded by one blank line:

      ```markdown
      ### Core: widen camera support from public sample RAW files

      The user no longer owns the Sigma fp L or BF and cannot shoot new
      samples, so public sample files are the verification source for more
      cameras. Two kinds exist: raw.pixls.us (CC0; a few files per camera,
      often flat test scenes that are weak for AF-point checks; CC0 means a
      small file could be committed as a fixture, but tests prefer synthetic
      bytes as in `crates/core/src/arw.rs`) and review-site sample galleries
      (real scenes, good for AF checks, but not redistributable, so local
      verification only). Riffle reads only ARW and DNG today (README.md
      "RAW formats and cameras"). The work splits into tiers, cheapest first:

      1. More DNG-writing cameras (Pentax, Ricoh GR, other Leica bodies,
         phones). DNG reading already exists, so only preview/EXIF
         extraction needs verifying on samples.
      2. AF point from MakerNotes that exiftool already decodes (Canon
         `AFInfo`, Nikon `AFInfo2`, Fujifilm `FocusPixel`, Olympus
         `AFPointSelected`, Panasonic `AFPointPosition`). Tag meaning and
         coordinate system are known; samples only confirm. This presupposes
         the RAW container is readable (tier 3), except for bodies that
         write DNG.
      3. New RAW containers (CR3 = ISOBMFF, NEF, RAF, ...). Each needs a new
         parser next to `crates/core/src/arw.rs`; implementation outweighs
         verification.

      AF data exiftool does not decode needs inference from many off-center
      samples, as done for the Sigma BF `0x0147` in
      `docs/plans/_archived/20260924-sigma-bf-af-point/plan.md`. Files:
      `crates/core/src/arw.rs`, `crates/core/src/reader.rs`, `README.md`.

      #### TODO

      - [ ] Tier 1: verify preview/EXIF extraction on public DNG samples from
            Pentax, Ricoh GR, other Leica bodies, and phones; add each
            working body to the README "RAW formats and cameras" list.
      - [ ] Tier 2: read the AF point from the exiftool-decoded MakerNote
            tags above, confirming on samples, for bodies whose container
            Riffle can already read.
      - [ ] Tier 3: decide per container (CR3, NEF, RAF, ...) whether a new
            parser is worth it, given the samples available.
      - [ ] Check the Sigma BF AF point's open assumptions against public BF
            samples: portrait orientation, manual-focus behavior, and the
            1000x667 scale (see the `SIGMA_BF_AF_GRID_W` doc comment in
            `crates/core/src/arw.rs` and the archived plan's
            [Trade-offs and risks](docs/plans/_archived/20260924-sigma-bf-af-point/plan.md#trade-offs-and-risks)).
      ```

    - Do not touch any other item in `todo.md`.
    - `crates/core/src/reader.rs` is listed because the existing "write a
      guide for RAW metadata parsing" item names `crates/core/src/{arw,reader}.rs`
      as the MakerNote/TIFF parsing files; confirm the path exists before
      committing (drop it if not).

## Trade-offs and risks

- **One item vs. a sibling for the Sigma BF assumptions.** Taken (user
  approved): a bullet in the same item, since the verification source (public
  BF samples) is the same.
- **Whether to cite the external raw.pixls.us URL.** Taken: name the site
  without a link.
- **Tier 3 wording.** "Riffle reads only ARW and DNG today" matches the
  README list at the time of writing; if the README changes before this PR
  merges, adjust the sentence.

## Progress

- (2026-09-24) Step 1 complete
