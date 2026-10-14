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
