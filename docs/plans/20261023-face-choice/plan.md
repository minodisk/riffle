<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../humans/usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Face choice: does the face `eye_focus` judges hurt its discard prediction?

## Purpose

On the fresh blind "OK to discard?" labels of the model-search confirmation
([modelsearch.md, "Results of the confirmation"](../_archived/20261020-model-search/modelsearch.md)),
the plain `eye_focus` cue was the strongest discard signal: weighted AUC
**0.785** [0.723, 0.844], against 0.684 for the learned winner `G0 ⊕ ef`.
Improving `eye_focus` itself is therefore the most promising lever for
Riffle's discard suggestion (the AI's goal: suggest frames to discard, the
human picks from what remains; the label is "OK to discard?", read with
`RR`, `PL`, removed share and AUC under HT weights `1/π`, stratified and
folder-cluster bootstrap, 1,000 draws, seed 0).

The user's hypothesis, in their words: "Today `eye_focus` takes the face
nearest the AF point. Would the results change if we chose the sharpest
face, or the largest face? My hypothesis is that the face chosen differs
from the one a human looks at, and that this hurts the score."

This plan is an **offline evaluation only**: it recomputes `eye_focus` for
every detected face of the dump frames, scores a small pre-registered set
of face-choice rules against the two blind label rounds, and ends with a
Decision naming the follow-up. **No app behavior changes here**; a shipping
change is a later plan, and any winning rule needs a **fresh blind
confirmation round** first (the user agreed the next labeling round would
combine that confirmation with more labels from folders not yet sampled).

## Decisions (by the user, at planning time)

- **Cut source: option A.** Recompute the per-face dump on all 27 dump
  folders (32,459 frames), so the per-folder bottom-q cuts and `PL` are
  exactly discard.md's. (Rejected: only the 1,200 labeled frames, with
  sample-weighted quantiles and a `PL` resting on about 86 sampled picks.)
- **Label read: per round, round 2 primary.** Each round keeps its own
  population, `π` and strata as its harness defined them; round 2 (the
  model-search confirmation) is the primary read and round 1 the
  replication. (Rejected: pooling the 1,200 frames with round x stratum
  cells and approximate weights.)

### What is known (investigated 2026-10-21, persisted 2026-10-23)

- **Today's rule, exactly.** With a trusted AF point
  (`sharpness::trusted_focus`), `candidate::focus_cue_unless` decodes the
  full preview, detects faces **only in a `CATCH_CROP` (480 px) square
  around the AF point** (`faces::detect_around_rgb`, faces below
  `faces::SCORE_THRESHOLD` 0.6 dropped) and judges `candidate::nearest_face`:
  the face whose nearer eye is closest to the AF point, the distance divided
  by the box's long side. A face outside that square is never seen. Without
  a trusted AF point, `scan.rs` detects on the whole preview at a reduced
  decode (`faces::decode_whole` + `detect_whole_upright`, `WHOLE_INPUT`
  640x448) and judges `eyes::judged_face(faces, None)`: the **largest** face
  with score >= `sharpness::FACE_CONFIDENCE` (0.8). (`sharpness::chosen_face`,
  the highest-scoring face >= 0.8, is only the sharpness window's fallback
  without an AF point; it is not the cue's face.) The index stores the
  judged face's values only (`eye_focus`, `eyes_ear`, `pose_*`), so per-face
  values must be recomputed from the embedded previews.
- **Per-face recomputation is possible on the public API without touching
  `faces.rs` / `candidate.rs`**: `reader::read_preview`, `decode::decode_rgb`,
  `faces::detect_around_rgb` (the crop set) and `faces::decode_whole` +
  `detect_whole_upright` (the whole set), then per face
  `candidate::meshes_face` -> `eyes::mesh_of` -> `candidate::eye_focus(gray,
  w, h, face, mesh)` on `candidate::luma`. `EyeFocus` carries `probability`,
  `state`, `scored` and the mesh's `ear` / `pose`. The scan's own cue is
  `scan::extract_analysis(path).cue` (the reproduction reference). Verify
  these names against `main` when Step 2 starts.
- **Concurrent work, now merged.** The plan
  `docs/plans/20261021-face-detection-values` added `nose`, `mouth_left`,
  `mouth_right` to `faces::Face` and carries every face's score and
  landmarks through `faces_of`. It was merged before this plan started, so
  this plan builds on that `Face`. It still edits neither `faces.rs` nor
  `candidate.rs`.
- **Data.** Round 1: `D:\Photos\tests\2026-10-18-discard-labels\`
  (`labels.tsv` 600 rows, 211 keep / 389 discard; `manifest.tsv` with
  `band`, `inclusion_prob`, the stored `eye_focus`; `strata.tsv`; population
  = dump minus the 1,200 strict frames = 31,259; harness
  `docs/plans/_archived/20261018-discard-labels/discardfilter.py`). Round 2:
  `D:\Photos\tests\2026-10-20-model-search\` (`labels.tsv` 600 rows, 248
  keep / 352 discard; `manifest.tsv`, `strata.tsv` with 31 cells; population
  = dump minus the 1,800 earlier-labeled frames = 30,659; harness
  `confirmread.py` / `confirmlabels.py` in the model-search plan folder).
  Round 2 was the model-search confirmation; using it for selection here is
  fine, and is why a winner needs a fresh round. Both rounds' frames are
  rows of the 27-folder dump `D:\Photos\tests\2026-10-12-rank-app\`
  (`features\<folder>.tsv` from `riffle-cli features`, 32,459 frames,
  `embed.tsv`; loaded by the archived `gate.load` via `badfilter.load`).
  The library `D:\Photos\2026` is **read-only**.
- **Known numbers to reproduce**: round 2's `eye_focus` (missing last) row:
  AUC 0.785 [0.723, 0.844] (cluster [0.693, 0.881]), `RR` 5.5 / 12.9 / 12.8 %
  and `PL` 4.44 / 9.14 / 15.59 % at q5 / q10 / q20, removed 4.80 / 9.70 /
  19.64 %, discardable base rate 42.31 % [35.85, 48.91]; round 1's
  `eye_focus` row in `discardfilter.out.md` (AUC 0.777, `PL` 4.44 % at q5,
  9.14 % at q10) and base rate 48.45 % [42.03, 55.11].
- **Harness pieces to import, none modified**: `badfilter` (`read_tsv`,
  `load`, `strat_counts`, `cluster_counts`, `graded_cut`, `aucs`,
  `auc_pair`, `quant`, `iv`, `civ`, `Tee`; it brings `rejectfilter`, `gate`,
  `folder_gate`, `folderwide` and the archived bad-photo `sample`), then
  `discardfilter` (`metrics`, `unweighted`, `counts`, `QS`), `confirmread`
  for the round-2 cell construction (`strata.tsv` cells: the bands plus
  `control` per folder). **Name no script of this plan `sample.py`.**
- **Compute reference**: the app's three passes on the 32,459-frame dump
  took 3,888 s; `riffle-cli candidates` is about 50–90 ms per file on one
  thread before the mesh; the mesh is the per-face cost (roughly 20–40 ms
  each, measure it).
- **Python**: `D:\Photos\tests\.venv-eval\Scripts\python.exe -I -B` on
  script files only, never `python -` heredocs (the Git Bash stub hangs);
  run from the scratchpad; each tool call under 10 minutes, long runs in
  the background per folder. Check `git status` for stray `__pycache__/`.
- **Where the data goes**: `D:\Photos\tests\2026-10-23-face-choice\`.
- **Out of scope (do not touch)**: `rank.rs`'s `pairs`, `Labeled`, `fit`;
  the `views` table and its data; the dlg-genre-weights, dlg-folder-best and
  dlg-likely-pick-icon sessions; no app code changes; `faces.rs` and
  `candidate.rs` are not edited.

### The candidates (pre-registered in Step 1, read in Step 3)

Two face sets per frame: **crop** (today's `detect_around_rgb` around the AF
point; empty without an AF point) and **whole** (`decode_whole` +
`detect_whole_upright`, the no-AF path, run on every frame). Rules, each
giving one `eye_focus` per frame (missing when the rule picks no face;
missing sorts last, as in the model-search read):

- `af` (today, the reference): with an AF point `nearest_face` over the
  crop set; without one the largest whole face with score >= 0.8. **Must
  reproduce the stored `eye_focus` of the labeled frames.**
- `af-whole` (control): with an AF point `nearest_face` over the whole set
  (same fallback). Separates "which detection" from "which face".
- `largest`: the largest box area over the whole set (score >= 0.8, as
  today's no-AF rule).
- `sharpest`: the maximum `eye_focus` over the whole set (score >= 0.8).
- `score`: the highest YuNet score over the whole set.
- `mean`: the mean `eye_focus` over the whole set's faces (score >= 0.8),
  the one aggregate.

Nothing else is searched. The confidence floor for the whole-set rules is
fixed in Step 1 (0.8 proposed, today's no-AF floor).

### What is measured

1. **Face counts first**: the share of labeled frames with 0 / 1 / 2+ faces
   in the whole set (and in the crop set among AF frames), weighted by
   `1/π` and unweighted, per round, and on the whole dump. If 2+ is rare,
   the rules can move little; the text says so before any score.
2. **Per rule**: weighted discard AUC with stratified and cluster
   intervals; `DP` / `RR` / `PL` / removed share at q5 / q10 (q20 as a
   reading) with the per-folder bottom-q cut on the full dump; **paired**
   bootstrap differences against `af` (one count matrix for every rule).
   Overall and restricted to **multi-face frames** (2+ in the whole set).
   Per round, round 2 primary.
3. **The direct test**: on frames where `af` and `largest` choose different
   faces (box IoU < 0.5), the paired AUC of each rule on that subset and the
   cross-tab label x "which rule's face has the lower `eye_focus`"; the same
   for `af` vs `sharpest`.
4. **Reading rule**: the primary read is the paired AUC difference vs `af`
   on all frames of round 2; a rule "beats today" when its 2.5 % quantile is
   above zero and its `RR` at q10 is not worse (paired 97.5 % quantile of
   the `RR` difference <= +5 points). The winner is the beating rule with
   the highest AUC; none beating is "no winner". Round 1, the q5 / q10
   tables, the multi-face subset and the disagreement test are readings
   that do not change the winner.

## Steps

- [x] Step 1: Pre-register the evaluation in `facechoice.md`, before any per-face value exists
  - Done when:
    - `facechoice.md` in this plan folder has a "Pre-registration" section
      merged **before the CLI subcommand exists and before any per-face
      `eye_focus` has been computed or read**; the merge commit is the
      record. It states today's rule accurately for both cases (AF point
      present: `nearest_face` over the 480 px crop; absent: largest whole
      face >= 0.8; `chosen_face` is the sharpness fallback only), with the
      code references above.
    - It fixes: the two face sets and the six rules exactly as above with
      the confidence floor; the missing-last convention; the IoU threshold
      of the disagreement test; the metrics, cuts and bootstraps (reusing
      discard.md's vocabulary); the two user decisions above with the
      rejected options; the reading rule; the statement that round 2 was a
      confirmation sample and that any winner needs a fresh blind round
      (combined with more labels from unsampled folders); the multiplicity
      statement (5 rules x 2 rounds x 3 cuts, read without correction,
      hence one mechanical rule); the population caveat (one photographer,
      mostly basketball).
    - It lists the known numbers to reproduce (the `eye_focus` rows and
      base rates of both rounds, quoted with their sources).
    - `mise run lint` passes.
  - Implementation approach:
    - Doc-only PR. Model the text on modelsearch.md's "Pre-registration of
      the confirmation" and discard.md. No script runs here.

- [ ] Step 2: `riffle-cli facechoice`, the per-face dump, run on the 27 dump folders and checked against the stored `eye_focus`
  - Done when:
    - A new file `crates/cli/src/facechoice.rs` (declared with one `mod`
      line and one dispatch arm plus one usage line in `main.rs`, nothing
      else in `main.rs`) implements `riffle-cli facechoice <dir|file>...
      [threads] [--out <out.tsv>]`: for every RAW file, `read_preview`,
      `trusted_focus`, one full-size `decode_rgb`, the **crop set** (with an
      AF point, `detect_around_rgb`) and the **whole set** (`decode_whole` +
      `detect_whole_upright`), then for every face of both sets
      `meshes_face` -> `mesh_of` -> `eye_focus` on the one `luma`. One
      TSV row per (file, set, face): `folder`, `file`, `af` (0/1), `af_x`,
      `af_y` (stored coordinates, `-` without), `set` (`crop` / `whole`),
      `face` (index), `x`, `y`, `w`, `h`, `score`, `left_eye`, `right_eye`,
      `nearest` (1 on the `nearest_face` of that set), `eye_focus`,
      `state`, `scored` (`L` / `R` / `window`), `ear`, `yaw`, `pitch`,
      `roll`, `mesh_ms`; a frame with no face in a set gets one row with
      `face` `-`; errors go to stderr. Full `repr` precision on the floats.
    - The rows of the **crop set's `nearest` face reproduce `scan::extract_analysis(path).cue.eye_focus`** (same code path, exact or within 1e-9), checked by the subcommand itself on every file (`--check` prints the count of mismatches) and by Step 3 against the two `manifest.tsv`'s stored `eye_focus` columns.
    - Unit tests cover the line formatting and the per-set `nearest` /
      no-face rows, and `cargo test -p riffle-cli` and `mise run lint` pass.
    - The dump exists under `D:\Photos\tests\2026-10-23-face-choice\faces\`
      as `<folder>.tsv` for each of the 27 dump folders (the library
      read-only, run per folder in the background, resumable: a folder whose
      file exists is skipped). `learnings.md` records the wall time per
      folder and the per-face mesh cost, measured on one folder first.
    - **No per-face value is read against `labels.tsv`** in this step; the
      only comparison is the reproduction check against the stored cue.
  - Implementation approach:
    - Assumes Step 1 merged. Follow `detect` / `candidates` in `main.rs`
      for the rayon pool, `raw_paths` and the `[threads]` convention.
    - Keep the floor filters out of the dump: write every face with its
      score and let Step 3 apply `FACE_CONFIDENCE`, so the floor can be
      read without a rerun.
    - CLAUDE.md's `crates/cli` sentence may gain the new subcommand in the
      same PR; otherwise leave CLAUDE.md.

- [ ] Step 3: `facechoiceread.py`, the single pre-registered read, and the Results section
  - Done when:
    - `facechoiceread.py` in this plan folder (`python -I -B
      facechoiceread.py <app-dump-dir> <folder-wide-eval-dir>
      <reject-filter-eval-dir> <discard-labels-dir> <model-search-dir>
      <out-dir>`) loads the dump through `badfilter.load`, both rounds'
      `labels.tsv` / `manifest.tsv` / `strata.tsv`, and the face dump; builds
      the six rules' per-frame `eye_focus`; prints Markdown tables saved as
      `facechoiceread.out.md` in `<out-dir>`; **refuses to run when that
      file exists**, so the read happens once.
    - It prints, in this order: the checks (the `af` rule reproduces the
      stored `eye_focus` of all 1,200 labeled frames and both rounds'
      `eye_focus` reference rows and base rates); the **face-count table**
      (item 1); the rule tables (item 2: AUC and q5 / q10 / q20 rows per
      rule per round, the paired differences vs `af` with stratified and
      cluster intervals, overall and on multi-face frames); the
      **disagreement tables** (item 3, with the subset sizes and how often
      the AF face is smaller than the largest); the **reading rule applied**
      with the winner named or "no winner".
    - `facechoice.md` gains a "Results" section with these tables and a
      "Reading" paragraph stating the facts (including whether multi-face
      frames are frequent enough for the question to matter) without
      deciding anything.
    - `learnings.md` records deviations and the choices the text left open.
    - `mise run lint` passes.
  - Implementation approach:
    - Assumes Step 2 merged and its dump complete. Reuse `discardfilter.metrics` / `counts`, `badfilter.graded_cut` / `strat_counts` / `cluster_counts` / `aucs` and `confirmread`'s round-2 cell construction; one count matrix per round shared by every rule so the differences are paired.
    - Metric-only; seconds to a minute to run.

- [ ] Step 4: Decision
  - Done when:
    - `facechoice.md` gains a "Decision" section naming the follow-up from
      the pre-registered rule: (1) a winning rule -> a plan for a fresh
      blind confirmation round (combined with more labels from folders not
      yet sampled, as agreed) before any app change, with the shipping
      change sketched (which detection and which rule the scan would run,
      and the extra per-face mesh cost measured in Step 2); (2) no winner ->
      nothing changes in the app, and what the face-count and disagreement
      readings say about why; it also states what this evaluation cannot
      tell (selection on reused labels, one photographer).
    - `learnings.md` is complete; `mise run lint` passes.
  - Implementation approach:
    - Doc-only PR, written after Step 3's read; the judgment stays separate
      from the facts, as in the model-search plan.

## Trade-offs and risks

- **Multi-face frames may be rare** (mostly basketball; the AF crop already limits the set). Item 1 is reported first so a null result is read as "the rule cannot matter here" rather than "the rule does not matter".
- **Detection-set confound**: `largest` / `sharpest` / `score` run on the whole-image 640x448 detection, which can miss small faces the 480 px crop finds at 320 px input and vice versa. The `af-whole` control isolates this; the pre-registration says the comparison of interest is `largest` / `sharpest` vs `af-whole` as well as vs `af`.
- **Selection on reused labels**: round 2 confirmed the model search and already produced the 0.785 that motivated this plan; any winner here is a selection, which is why the Decision can only propose a fresh confirmation, never a shipping change.
- **Compute time**: the full dump is likely 1.5–3 hours on 8 threads, run in the background per folder and resumable.
- **`sharpest` costs a mesh per face in the app**; the Decision must weigh the measured per-face cost before proposing it.

## Progress

- (2026-10-23) Step 1 complete
