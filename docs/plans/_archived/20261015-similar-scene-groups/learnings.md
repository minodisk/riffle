# Learnings

## Step 1 (pre-registration)

- Sizing the draw: the candidate pairs and distinct files per cosine x gap
  cell were counted label-free over the 13 ARW pool folders (3304 frames
  after the 24 excluded `2026-08-08` files) with a throwaway script in the
  session scratchpad (importing the archived `pairs.py` `load`), and are
  recorded in `groups.md`. High cosines at long gaps are scarce: cosine
  0.96–1.00 has 0 candidates at 300–1800 s and 1 at > 1800 s; 0.93–0.96 has
  10 pairs over 18 files at 300–1800 s. The per-cell counts were set within
  what exists (0 and 1 for those two cells, 5 for the latter), and the draw
  order takes the scarce cells first so file reuse does not starve them.
- The Clopper-Pearson thresholds in the plan were checked with scipy: 59 of
  59 gives a bound of 0.9505 (58 gives 0.9497); 92 of 93 gives 0.9500.
- Clarifications of the plan text, fixed in the pre-registration (not
  deviations from a decided criterion, but choices the plan left open):
  - Anchors are left out of precision as well as recall (the plan names only
    recall): every rule groups a burst unconditionally, so anchors test the
    definition, not the rule, and would inflate precision.
  - The plan's pairwise "predicted same" (cosine >= t and gap <= W) decides
    for the chained rule. It differs from group membership under chaining
    (two frames can be chained through a third or split by one between), so
    the membership-based precision and recall of the selected chained rule
    are reported beside it, not deciding. For the fallbacks, which have no
    pairwise form, "predicted same" is group membership.
  - The fallbacks run on the same grid and bars; on a recall tie, single
    linkage is taken.
  - A cell that runs out of usable candidates is not refilled from another
    cell (the archived `pairs.py` raised instead); the shortfall is reported.
  - The 185 cross-burst pairs plus 15 anchors give 200 pairs, 105 of them at
    cosine >= 0.90.
  - A reweighted recall (to the folders' proportions) is added as reported,
    not deciding, since the stratified recall overstates the folder-level one.

## Step 2 (draw and labeling page)

- The CLI reads DNG: `scan::is_raw_file` lists `dng`, and `riffle-cli
  features` / `embed` went through both Leica M11-P folders with 0 errors,
  so the app test path was not needed. Command lines (Git Bash, 24 logical
  CPUs, 16 threads), after `mise exec -- cargo build --release -p riffle-cli`
  (2 min 50 s):
  - `riffle-cli features D:/photos/2026/<folder> 16 >
    features/<folder>.tsv 2> features/<folder>.log`, then `ls
    D:/photos/2026/<folder>/Output | sed 's/\.[^.]*$//' | sort >
    features/<folder>.output.txt` (as the archived `gate_dump.sh`):
    `2026-09-05` 411 files in 13.9 s, `2026-08-29-l` 211 in 8.6 s.
  - `riffle-cli embed <app models dir>/siglip2-base-patch32-256/vision_model.onnx
    siglip2 D:/photos/2026/2026-09-05 D:/photos/2026/2026-08-29-l 16
    --near 404 --out embed.tsv`: 622 files, 0 failed, 12.1 s wall.
  - `python -I scene_pairs.py D:/Photos/tests/2026-10-12-rank-app
    D:/Photos/tests/2026-10-15-scene-embed
    D:/Photos/tests/2026-10-15-scene-pairs` with the venv's python: 3 min
    18 s, most of it the 400 previews through exiftool.
- The CLI's `features` columns differ from the app dump's (`cue_side`,
  `af`, `judged_side`, `analysis_ms`, `eyes_ms` added); `pairs.load` reads
  only `file`, `capture_time` and `subsec`, so the two dumps load alike.
- The Leica DNGs carry no sub-second time (`subsec` is `-` on all 622), so
  their capture times and 1 s bursts are at whole-second resolution: two
  frames in consecutive seconds may be up to ~2 s apart and still form a
  burst. Accepted as what `rank::bursts` sees for these files in the app.
- Deviation (preview only, not the draw): exiftool's `-PreviewImage` of
  these DNGs is the 160 x 120 thumbnail of the first SubIFD, so
  `scene_pairs.py` writes the DNG previews first from `-JpgFromRaw` (the
  2112 x 1408 SubIFD2 JPEG), rotated by IFD0 `Orientation` as
  `pairs.previews` does, and then calls `pairs.previews`, which skips the
  files that exist. ARW previews come from `pairs.previews` unchanged.
- The ARW part of the candidate counts reproduces the pre-registration's
  sizing table exactly but for one pair that moved from cosine 0.85–0.90 to
  0.80–0.85 at the 60–300 s gap (18963 / 11043 vs 18962 / 11044): the
  script takes the cosine in float64, the throwaway sizing script took it in
  float32, and one pair sits on the 0.85 edge. The counts per cell are not
  affected.
- The app dump's paths keep the `\\?\` prefix (`//?/D:/Photos/...` after
  `load`); exiftool accepts it, and `pairs.tsv` keeps the paths as `load`
  gives them so Step 3 can join on them directly.
- The labeling page reuses `pairs.PAGE` with three replacements (the
  `localStorage` key `riffle-scene-pairs-v1`, the heading and title) and the
  definition span swapped for the Japanese rendering of the non-sports
  definition. It still shows the folder name under the pair, as the sports
  page did (not the cosine, gap, bin or file name).

## Deferred issues (todo candidates)

- None. The pending labeling hand-off was cancelled when the plan was
  stopped (Decision 1); the unlabeled pairs and page stay in
  `D:\Photos\tests\2026-10-15-scene-pairs\` should Step 3 be resumed.
