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

# Reject-filter evaluation: re-measure the ranking, its cues and its columns as a bottom filter

## Purpose

Riffle is a culling app; the human culls. What the user wants from the AI /
personal-ranking feature is a **reject filter**: drop the clearly bad photos,
single shots and burst frames alike (bursts are not special), and leave a
candidate set the human picks from. Every evaluation so far, including the
just-finished
[folder-wide-ranking-eval](../_archived/20261015-folder-wide-ranking-eval/plan.md)
(Decision 1, [folderwide.md](../_archived/20261015-folder-wide-ranking-eval/folderwide.md)),
judged the scores on **top-oriented** metrics: frame AUC, R-precision
(precision at k = the folder's pick count), within-burst Top-1. A filter is
judged at the **bottom** under a recall constraint: how many non-picks can be
removed while nearly all picks survive. A cue that does nothing at the top
(closed eyes, an out-of-focus eye, blur, a face turned away: the "obviously
bad" technical cues Decision 1 found redundant) may matter there, because a
single such failure is enough to make a frame a reject even when it says
nothing about which good frame is best.

This plan re-measures the same held-out scores, the same baselines and the
same per-column ablations on a **bottom-oriented, recall-constrained**
criterion, adds simple rule filters and two models trained to find the
bottom, and ends at a recorded Decision 2: which model or rule filters best at
95% / 99% pick recall, which cues contribute at the bottom, whether
Decision 1's "drop genre for the ranking" still holds under this criterion,
and what app change (if any) a follow-up plan should make, stated as a
recommendation only. No app change happens in this plan.

### What is known (investigated 2026-10-16)

- **No bottom-oriented number exists yet.** Nothing in the archived plans
  reports removal at fixed pick recall, a partial AUC at high recall, or the
  picks lost in a bottom share. What is known and bounds the outcome: the
  held-out frame AUC of every variant (0.663–0.678; the full-ROC area is an
  upper bound on how steep the curve can be at any recall), the baselines'
  frame AUC (`eye_focus` 0.579, `photoTier` good 0.546, absolute sharpness
  0.515), and the pick vs explicit reject AUC over the 3 folders that hold
  one. The pre-registration cannot be blind to these and says so.
- **The held-out scores already exist and need no refit.**
  `D:\Photos\tests\2026-10-15-folder-wide-eval\scores.tsv` (32,459 rows,
  `folder`, `file`, then one held-out score column per variant: `W0`,
  `W0+off`, `F0`, `F0+ctx`, `P0`, `F0+ctx +genre cols`, `F0+ctx +genre off`,
  `F0+ctx +c1`, `F0+ctx +c2`, `F0+ctx +c1+c2`, `W0 +genre cols`, `W0 +c1`,
  `W0 +c2`, `W0 +c1+c2`; `nan` for the `W0` family on folder `2026-07-23`,
  which has no scorable burst and is not qualifying). The baselines (random,
  capture order, burst size, absolute sharpness, `eye_focus`, `photoTier`
  good) are rebuilt from the dump in seconds as `folderwide.main` does.
  The Step 3 ablations of the archived plan did **not** save their scores
  (`ablate.py` prints tables only), so the per-column ablations and the
  each-alone models rerun here (356 s on 20 jobs last time).
- **Data.** `D:\Photos\tests\2026-10-12-rank-app\`: 27 folders, 32,459
  frames, 4,419 picks by the gate label rule, 58 explicit rejects in 3
  folders, no `views` rows (the folders are fully culled); 25 qualifying
  folders (>= 50 frames, >= 5 picks); folder pick rates 10–22%. Cue files
  `D:\Photos\tests\2026-10-14-ball-labels\extras_c1.tsv` / `extras_c2.tsv`
  (full coverage). The raw per-frame fields the rule filters need are in
  `gate.load`'s rows (`ef` = `eye_focus`, `eo` = openness 0–100, which is 0
  at or below the app's `EYES_CLOSED_EAR` 0.137, `sharp`, `tier` =
  `photoTier` good, `yaw` / `pitch` / `roll`); the raw EAR is only in
  `features/<folder>.tsv` (`ear` column) if a rule needs it.
- **Harnesses, imported by path, none modified.** All five now live under
  `docs/plans/_archived/`, and **every one of them has stale `sys.path` /
  `GENRES` constants** (todo "Agents: the archived plan scripts' import paths
  broke when their folders moved"). `folderwide.py` worked around the three
  older ones, but since its own archive move its `ARCHIVED = HERE/../_archived`
  resolves to `_archived/_archived/...` and its `GENRES` to
  `docs/crates/core/genres/siglip2.json`. The workaround for this plan: insert
  the five archived folders (`20261012-image-embeddings`,
  `20261014-folder-wide-pick-model`, `20261014-ball-game-ranking-features`,
  `20261014-genre-ranking-offsets`, `20261015-folder-wide-ranking-eval`) on
  `sys.path`, import `gate` first, then `folderwide as fw` (and `ablate`),
  then set `fw.GENRES = folder_gate.GENRES = <repo>/crates/core/genres/siglip2.json`
  before `folder_gate.extend`. What is reused:
  - [gate.py](../_archived/20261012-image-embeddings/gate.py): `load`,
    `build`, `scorable`, `design`, `choose_c`, `fit`, `lofo`, `TECH`, `CS`,
    `table`, `f3`, `pc`.
  - [folder_gate.py](../_archived/20261014-folder-wide-pick-model/folder_gate.py):
    `extend`, `design(d, mask, "full")`, `fit`, `choose_c`, `auc`,
    `signed_rank`, `paired`, `mean_sd`, `values`, `MIN_FRAMES`, `MIN_PICKS`.
  - [ballgate.py](../_archived/20261014-ball-game-ranking-features/ballgate.py):
    `load_extras`.
  - [genre_offsets.py](../_archived/20261014-genre-ranking-offsets/genre_offsets.py):
    `burst_genres`.
  - [folderwide.py](../_archived/20261015-folder-wide-ranking-eval/folderwide.py):
    `point_lofo`, `point_fold`, `append_block`, `collect`, `boot_folders`,
    `ci`, `pm`, the variant name constants, and its baseline construction
    (the `baselines` dict in `main`, to be rebuilt the same way).
  - [ablate.py](../_archived/20261015-folder-wide-ranking-eval/ablate.py):
    `specs`, `task`, `load_scores`, `GROUPS`, `ALONE`, `DROPS`, `BASES`.
  - Python: the venv `%TEMP%\embedvenv` (numpy 2.5.4, scikit-learn 1.9.1,
    scipy 1.18.1, joblib), run as `python -I -B` from outside the data
    directories; check `git status` for stray `__pycache__/` in the archived
    folders afterwards (the loky workers do not inherit
    `dont_write_bytecode`).
- **Out of scope, owned elsewhere (do not touch):** `rank.rs`'s `pairs`,
  `Labeled`, `fit` and the `views` training data; the dlg-genre-weights,
  dlg-folder-best and dlg-likely-pick-icon sessions; other in-flight plans
  (not linked, so their archive moves cannot break lint here). This plan adds
  files only under its own folder and writes data only to
  `D:\Photos\tests\2026-10-16-reject-filter-eval\`.

## Steps

- [x] Step 1: Pre-register the bottom metrics, the thresholds, the variants, the rules, the ablations and the bars in `reject.md`, no script and no score
  - Done when:
    - `reject.md` in this plan folder has a "Pre-registration" section
      merged **before any script exists or any bottom number is computed**
      (the merge commit is the record). Nothing in it changes afterwards; a
      deviation found later goes to `learnings.md` with its reason. The
      section states up front what is already known (the list above: the
      frame AUCs and the pick-vs-reject AUC; no bottom number) and that the
      bars were set by the user before the run.
    - The section fixes, in this order:
      - **Data, labels and negatives**: the 27 folders through `gate.load` /
        `gate.build` / `folder_gate.extend`; picks by the gate label rule;
        **every non-pick is a negative**, with the caveat written out:
        "non-pick" is not "bad". A culled folder's non-picks are the bad
        frames *and* the fine-but-redundant ones (the second-best frame of
        a burst, a duplicate). So the removed-non-pick share is an upper
        bound on "bad removed", and the **pick recall constraint is the
        real safety number**: a filter that keeps 99% of the picks costs the
        human one pick in a hundred whatever it removed. Explicit rejects
        (58, 3 folders) are the only frames known to be bad; they stay
        among the negatives, are too few to train on or to bar on, and are
        reported as one reading aid (the share of the 58 removed at each
        operating point). Bursts as `rank::bursts` (gap 1000 ms); a single
        is a burst of one; bursts play no role in the metric.
      - **Unit and folds**: every frame is a sample; every number is held
        out by folder (the Step 2 scores are leave-one-folder-out already;
        every new fit follows the same 27 folds, `C` by the same inner
        grouped CV). **Qualifying folders**: >= 50 frames and >= 5 picks
        (25 of 27); means, intervals and tests are over them; a sub-table
        over the folders with >= 20 picks is reported because at 5–19 picks
        a 95% target means "lose none".
      - **Primary metric**, per held-out folder, then the unweighted mean
        over the qualifying folders (one folder, one vote) with the
        frame-weighted pooled value beside it:
        1. **Non-pick removal at pick recall r, `RN@r`**, r = 95% and 99%:
           sort the folder's frames by score ascending (ties by the seed-0
           uniform draw the earlier plans use); the folder must keep
           `ceil(r * picks)` picks; remove the longest bottom prefix that
           holds at most `picks - ceil(r * picks)` picks; report the removed
           share of the folder's **non-picks** (`RN@r`, the specificity at
           sensitivity >= r) and of all its **frames** (`RF@r`). This is the
           **oracle** threshold (it uses the held-out folder's labels) and
           is the upper bound. **`RN@95` is the primary number**; `RN@99`
           the second.
        2. **Partial AUC at high pick recall, `pAUC95`**: the
           McClish-standardized partial AUC over pick recall in [0.95, 1],
           computed as `sklearn.metrics.roc_auc_score(~pick, -score,
           max_fpr=0.05)` (the non-picks as the positive class, so that the
           restricted false-positive rate is the pick loss); chance 0.5,
           threshold-free, so it reads the whole high-recall region, not one
           cut.
      - **Label-free thresholds** (what the app could actually apply, since
        the oracle cut needs the folder's own picks), per held-out folder
        without its labels, each reported as the pair (**realized pick
        recall**, `RN`) and as the gap between realized and target recall:
        - `quantile@r`: remove the bottom `q_r` share of the held-out
          folder's frames, `q_r` = the largest share such that the mean
          per-folder pick recall over the **other** qualifying folders'
          held-out scores (the same `scores.tsv` column, so no refit) is
          >= r; scale-free, so it applies to the uncentered `W0` family as
          well as the pointwise logits.
        - `absolute@r`: remove the held-out folder's frames whose score is
          below `t_r`, `t_r` = the score at which the other folders' pooled
          held-out scores keep >= r of their picks; sensitive to score
          scale across folds (each fold is its own model; `W0` has no
          intercept), reported with that caveat.
        **`quantile@95` is the label-free primary.** A variant's "filter
        at 95%" means its `quantile@95` pair unless stated.
      - **Reading aids**: picks lost when removing the bottom X% of the
        folder's frames, X = 10 / 20 / 30 / 50 (the share of the folder's
        picks inside that bottom share; the "what if the user just hides
        the bottom fifth" reading); the explicit-reject removal share at
        `RN@95` / `quantile@95`; the frame AUC from
        [folderwide.md](../_archived/20261015-folder-wide-ranking-eval/folderwide.md)
        beside each row, so the top and the bottom are read side by side.
      - **Scores from `scores.tsv`, no refit** (names as there): `W0` (the
        **reference**: the shipped weights' uncentered score), `W0+off`
        (shipped), `F0`, `F0+ctx`, `P0`, the ten cue variants, and the
        `W0` burst-centered score (what the app displays, rebuilt by
        subtracting the burst mean). **Baselines** rebuilt as `folderwide`
        does: random, capture order both ways, burst size, absolute
        sharpness, `eye_focus` (missing last), `photoTier` good.
      - **Rule filters** (binary rules give one operating point: their
        realized pick recall and `RN`, no curve; graded ones get the full
        treatment), using the app's constants (`GOOD_EYE_FOCUS` 0.9,
        `GOOD_EYE_EAR` 0.25, `EYES_CLOSED_EAR` 0.137, `GOOD_MAX_YAW` 35,
        `GOOD_MAX_PITCH` 45 from `focus.ts` / `rank.rs`), a frame without
        the needed value never removed by that rule:
        - `not good`: remove every frame whose `photoTier` is not good
          (expected to remove most of a folder, faceless frames included;
          listed because it is the one bad / good split the app already
          shows);
        - `eyes closed`: remove frames with `eo == 0` (EAR at or below
          `EYES_CLOSED_EAR`);
        - `eye focus low`: remove frames with an AF face whose `ef` <
          `GOOD_EYE_FOCUS`;
        - `face away`: remove frames with `|yaw| > GOOD_MAX_YAW` or
          `|pitch| > GOOD_MAX_PITCH`;
        - `sharpness bottom q`: remove the folder's bottom q by absolute
          sharpness, q = 10% / 20% (one cut at a fixed q, so a binary
          rule for `RN@r`; the curve is the absolute-sharpness baseline's
          row);
        - `any of`: `eyes closed` or `eye focus low` or `face away` or
          `sharpness bottom 10%`;
        - `rule count`: the graded score "number of the four rules that
          fire" (0–4), descending removal, ties by the seed-0 draw.
      - **Reject-trained variants** (new held-out fits, same folds, 44
        shipped columns + `ln burst size` + absolute `ln sharpness` as
        `F0+ctx`; both run, as decided on 2026-10-16, option (b) in
        "Trade-offs and risks"):
        - `R0` asymmetric pointwise: `F0+ctx`'s weighted logistic with the
          picks' per-folder weight multiplied by 4 (the fit pays four times
          more for a misplaced pick than for a misplaced non-pick, so it
          moves its errors toward the "keep" side); `C` by the inner CV
          maximizing the inner folders' mean `RN@95`;
        - `G0` nonlinear pointwise: `sklearn.ensemble.HistGradientBoostingClassifier`
          on the same columns with the same per-folder class-balanced
          weights, a fixed small setting (`max_iter` 200, `learning_rate`
          0.05, `max_leaf_nodes` 15, `l2_regularization` 1.0, missing
          values native so the indicator columns stay), no tuning; it can
          express the disjunction "closed eyes **or** out of focus **or**
          blur" a linear model cannot;
        - explicit rejects are **not** upweighted in a variant: 58 frames
          in 3 folders cannot move a 27-fold fit and the app's
          `REJECT_WEIGHT` 2 belongs to the `rank.rs` session; said so.
      - **Ablations** (Step 3), on `F0+ctx` and `W0`, the archived plan's
        exact `ablate.specs` (drop each `REASONS` group; drop `|yaw|`,
        `|pitch|`, `|roll|`, `ln burst size`; each group alone, plus c1,
        c2 and the genre labels alone), reported as the change in `RN@95`,
        `RN@99` and `pAUC95` against the unablated `scores.tsv` score with
        the folder bootstrap interval; and on `G0` if it is the best filter
        in Step 2 (the drop-one groups only).
      - **Intervals and tests**: the 1,000-draw bootstrap over the
        qualifying folders (seed 0) of the paired per-folder difference for
        `RN@95`, `RN@99` and `pAUC95`; the paired one-sided Wilcoxon
        signed-rank (`folder_gate.signed_rank`); the count of folders where
        the variant is at or above the comparison on `RN@95`.
      - **Bars** (option B, chosen by the user on 2026-10-16 before any
        score; options A (lenient: G = +2.0, X = 15%) and C (strict:
        G = +5.0, X = 40%) are listed as rejected). Two bars:
        - *Comparison bar* (a variant or rule **filters better** than its
          comparison): mean `RN@95` gain >= **+3.0 points**, >= **16 of 25**
          folders at or above, one-sided Wilcoxon **p < 0.05** on `RN@95`;
          `RN@99` and `pAUC95` reported, not required.
        - *Worth-a-filter bar* (the best filter is **worth an app change**
          at all), on its **label-free** `quantile@95` pair: mean realized
          pick recall >= **93%** (within 2 points of the target) and mean
          `RN` >= **25%**.
        Applied as:
        - *Best filter at 95% / 99%*: the score with the highest mean
          `RN@95` (oracle), with its label-free pair beside it; "wins over
          the shipped weights" when it passes the comparison bar against
          `W0`.
        - *A cue contributes at the bottom* when its drop-one lowers `RN@95`
          with the bootstrap interval entirely below zero (reported per
          column, as Decision 1 did).
        - *Genre keep / drop under this criterion*: Decision 1's "drop"
          **stands** unless `F0+ctx +genre cols`, `F0+ctx +genre off`,
          `W0 +genre cols` or `W0+off` passes the comparison bar against its
          base on `RN@95`; otherwise it is
          **reopened** and recorded as such.
        - *Follow-up*: an app change is recommended only when the best
          filter passes the worth-a-filter bar.
      - **What happens after**: no step of this plan changes the app; the
        Decision names the follow-up plan, if any.
    - `mise run lint` passes.
  - Implementation approach:
    - Model the section on the archived
      [folderwide.md](../_archived/20261015-folder-wide-ranking-eval/folderwide.md)
      pre-registration and keep its vocabulary. Link only `_archived/` or
      stable targets.
    - Before writing, read only headers and file lists (the `scores.tsv`
      header, the features TSV header); compute no bottom metric.
    - Doc-only PR; no Python, no Rust.

- [x] Step 2: `rejectfilter.py`: the bottom metrics on every existing score, the baselines, the rule filters and the reject-trained variants
  - Done when:
    - `rejectfilter.py` in this plan folder (`python -I -B rejectfilter.py
      <app-dump-dir> <ball-labels-dir> <folder-wide-eval-dir> <out-dir>
      [jobs]`) imports the five archived harnesses by path (the `sys.path`
      and `GENRES` workaround above, recorded in `learnings.md`), reads
      Step 2's `scores.tsv`, rebuilds the baselines, builds the rule
      filters, fits `R0` / `G0` held out, computes every pre-registered
      metric, threshold, interval and test, prints Markdown tables, and
      writes `scores_reject.tsv` (`folder`, `file`, one column per new
      score: `R0`, `G0`, the graded rules) to
      `D:\Photos\tests\2026-10-16-reject-filter-eval\`, stdout saved there
      as `rejectfilter.out.md`, progress on stderr.
    - **Reference checks** printed and recorded in `learnings.md`: the
      frame AUC recomputed from `scores.tsv` reproduces `W0` 0.666 and
      `F0+ctx` 0.666 (25 folders); random reads `RN@95` near 5% and
      `pAUC95` near 0.5; `R0` at pick weight 1 reproduces `F0+ctx`. A
      mismatch is investigated first.
    - `reject.md` gains a "Results" section: data counts; the main table
      (every score, baseline, rule and new variant: `RN@95`, `RF@95`,
      `RN@99`, `pAUC95`, `quantile@95` pair, `absolute@95` pair, picks lost
      at bottom 10 / 20 / 30 / 50%, reject share removed, frame AUC from
      the archived results); the >= 20-pick sub-table; the paired-difference
      table (each score minus `W0`; each cue variant minus its base; `R0`
      and `G0` minus `F0+ctx`) with intervals, p and folders above; the
      per-folder table (frames, picks, `W0` and the best filter's `RN@95`
      and `quantile@95` pair); `C` per fold for `R0`; wall time. A
      "Reading" paragraph stating the facts without deciding.
    - `learnings.md` records deviations and the choices the text left open.
  - Implementation approach:
    - Assumes Step 1 merged. Metric-only work on `scores.tsv` runs in
      seconds; `R0` is `fw.point_lofo` with a modified `d["weight"]` (the
      picks' weights times 4, folder by folder) and an inner CV on `RN@95`
      (write a `choose_c` variant beside `folder_gate.choose_c`; do not edit
      it); `G0` is one `HistGradientBoostingClassifier` per fold on
      `folder_gate.design(d, mask, "full")` with `sample_weight =
      d["weight"]`, scored by `decision_function`. Run the fits in the
      background and poll.
    - The label-free thresholds use the other folders' **held-out** scores
      from the same column (no refit), so the 25 qualifying folders each
      get a `q_r` / `t_r` from the other 24; non-qualifying folders are
      excluded from the threshold estimate and from the means.
    - For binary rules and `rule count`, `RN@r` is the point itself when
      the rule's realized recall >= r, else "does not reach r" (the rule
      cannot be relaxed); the table shows the pair.
    - Keep `fw.boot_folders` (fresh `default_rng(0)` per comparison and
      metric) so every comparison is read on the same folder draws.
    - Strip CRLF before pasting into `reject.md`.

- [x] Step 3: `rejectablate.py`: the column and group ablations and each-alone models on the bottom metrics, and Decision 2
  - Done when:
    - `rejectablate.py` in this plan folder (same arguments as
      `rejectfilter.py`) imports the archived `ablate.specs` / `ablate.task`
      unchanged, reruns every pre-registered ablation on `F0+ctx` and `W0`
      (and the drop-one groups on `G0` if Step 2 names it best), writes the
      ablated held-out scores to `scores_ablate.tsv` in the data folder (so
      they are not lost again), and prints: the check that the unablated
      rerun matches `scores.tsv`; the drop-one table per base (change in
      `RN@95`, `RN@99`, `pAUC95` with intervals and "contributes"); the
      each-alone table (head pose, sharpness, AF eye focus, eyes open, good
      photo, burst position, embedding, c1, c2, genre labels alone:
      `RN@95`, `RN@99`, `pAUC95`, and their label-free `quantile@95` pair).
      stdout saved as `rejectablate.out.md`; tables pasted into `reject.md`
      under "Ablations".
    - **Decision 2** is written in this plan with the numbers that decided
      it, against the Step 1 bars:
      1. the best filter at 95% and at 99% pick recall (oracle `RN`, its
         label-free pair and the recall gap, interval, p and folders above
         against `W0`); whether any rule filter or `rule count` is
         competitive with the models, and whether `G0`'s nonlinearity buys
         anything over `F0+ctx` / `R0`;
      2. per technical column group, per head-pose axis, the PCA block,
         burst position, c1, c2 and the genre labels: contributes at the
         bottom or not, beside Decision 1's top-oriented verdict, as a
         table; whether the "obviously bad" cues matter at the bottom
         though they did not at the top;
      3. **genre under this criterion**: Decision 1's "drop for the
         ranking" stands or is reopened, with the numbers; the todo "Core:
         a plan to drop the genre from the personal ranking" is to be
         updated by the wrap-up (not here);
      4. **the follow-up, as a recommendation only**: none when the
         worth-a-filter bar fails; otherwise the shape of the app change a
         next plan should propose (e.g. a "likely reject" filter item or a
         dimming in the strip at the label-free `quantile@95` cut of the
         best filter, which score it would use, whether it needs anything
         from the `rank.rs` session), with the costs stated (the realized
         pick recall is what the user loses).
    - `learnings.md` records run times and deviations; `mise run lint`
      passes.
  - Implementation approach:
    - Assumes Step 2 merged. `ablate.specs(base)` and `ablate.task(...)`
      give the per-fold masked fits as before (about 1,140 fits, 356 s on
      20 jobs); the bottom metrics are then Step 2's functions on the
      collected scores. Import `rejectfilter` by path for the metric code
      rather than duplicating it.
    - A `G0` drop-one is the same column mask on the `G0` fit; skip it when
      `G0` is not the best filter (pre-registered as conditional).
    - Write the Decision only after both tables exist.

## Trade-offs and risks

- **Bar sizes: decided (2026-10-16), option B.** The user chose the middle
  option: comparison gain G = +3.0 points of `RN@95` and worth-a-filter
  floor X = 25% (mean label-free `RN` at realized recall >= 93%), with
  >= 16 of 25 folders and Wilcoxon p < 0.05. Rejected: A (lenient,
  G = +2.0, X = 15%: an app change for a small effect) and C (strict,
  G = +5.0, X = 40%: a useful-but-modest filter declared "none"). What is
  known loosely bounds these: a frame AUC near 0.67 rarely gives a
  specificity above 20–30% at 95% sensitivity for a unimodal score, so the
  25% floor is within reach only if the bottom is easier than the top
  (the "obviously bad" hypothesis) or the nonlinear `G0` helps.
- **Reject-trained variants: decided, option (b), `R0` and `G0`.** `G0` is
  the one variant that can express "any one failure is enough", which is
  the hypothesis about the bottom, and it costs minutes; if it wins, the app
  path is a new model type, not a weight change in `rank.rs`. Rejected: (a)
  `R0` only; (c) an explicit-reject upweighted variant (58 rejects in 3
  folders would read as noise and touch the `REJECT_WEIGHT` semantics the
  `rank.rs` session owns). The pick weight 4 of `R0` is a fixed choice, not
  tuned.
- **Label-free threshold choice.** The quantile rule assumes folders have
  similar bottom shares, which is false across pick rates 10–22%, so its
  realized recall will scatter; the absolute rule assumes score scales
  carry across folds, which holds for pointwise logits better than for the
  intercept-free `W0`. Both are reported; the quantile is primary because
  the app could apply it without calibration. An alternative not taken: a
  threshold learned in-sample on the training folders of each fold (needs
  the per-fold models, which `scores.tsv` does not keep; would require
  refitting every variant for a second-order gain).
- **Per-folder recall at small pick counts.** At 5–19 picks, `RN@95`
  means "lose no pick" and `RN@99` always does; both are stricter than the
  target and make the mean conservative. Reported with the >= 20-pick
  sub-table rather than changing the qualifying rule, so the folder set
  stays the earlier plans'.
- **"Non-pick ≠ bad" cannot be fixed with this data.** No `views` rows and
  58 rejects; a filter's true precision on "bad" is unmeasurable here. The
  Decision therefore speaks of pick safety (recall) and of removed volume,
  never of "rejects found"; the recommendation names this as the thing a
  later in-app measurement (the `views` table, explicit rejects accumulating)
  could close.
- **Multiplicity.** About 33 score rows, 45 comparisons and 60 ablation
  cells are read at p < 0.05 / interval-below-zero; a few false
  positives are expected and a lone marginal pass is read as such.
- **The archived harnesses are frozen and all have stale paths.** This
  plan works around them by import order and constant override, as
  `folderwide.py` did; it does not fix them (that is the todo's call). The
  workaround is recorded in `learnings.md` so a later plan can copy it.
- **Genre reopened vs stands.** If a genre cue passes at the bottom, the
  pending todo's premise changes; this plan only records that and leaves
  the scope question (ranking only / whole feature) where Decision 1 left
  it. If it stands, nothing changes for the todo.
- **Data is one photographer, mostly basketball.** Any "best filter" is for
  this library; a nonlinear model fit on 27 folders may not carry over. The
  recommendation says so.
- **Fail path is a valid outcome.** If no filter passes the worth-a-filter
  bar, the artifacts are `reject.md`, the two scripts, the bottom-metric
  tables, the per-column reading and a recorded "no filter yet"; no code
  changes.

## Decision 2 (best filter, bottom cues, genre under this criterion, follow-up)

Written 2026-10-16 at the end of Step 3, after both the Step 2 results and
the ablation tables existed, against the bars pre-registered in Step 1
(option B). Comparison bar: mean `RN@95` gain >= +3.0 points, >= 16 of the 25
qualifying folders at or above, one-sided Wilcoxon p < 0.05. Worth-a-filter
bar: on the best filter's label-free `quantile@95` pair, mean realized pick
recall >= 93% and mean `RN` >= 25%. Every number is held out (leave one
folder out) and comes from [reject.md](reject.md) "Results" and "Ablations".

**1. Best filter: `G0`, and it is not worth a filter.** `G0` (gradient
boosting on `F0+ctx`'s columns) has the highest mean `RN@95` and `RN@99`:

| Reading | `RN@95` % | `RN@99` % | `pAUC95` | `quantile@95` recall / `RN` (gap) | `quantile@99` recall / `RN` (gap) |
| --- | ---: | ---: | ---: | ---: | ---: |
| `G0` | 22.0 | 11.5 | 0.568 | 94.7 / 16.8 (-0.3) | 99.0 / 5.9 (+0.0) |
| `F0+ctx` | 18.4 | 8.1 | 0.550 | 95.0 / 14.6 (+0.0) | 98.6 / 3.2 (-0.4) |
| `W0` (reference) | 16.6 | 7.1 | 0.544 | 95.0 / 11.5 (+0.0) | 98.9 / 3.0 (-0.1) |

| Comparison | `RN@95`, points | p | Folders at or above | `RN@99`, points | Bars met |
| --- | ---: | ---: | ---: | ---: | ---: |
| `G0` - `W0` | +5.4 [+2.3, +9.0] | 0.0021 | 19 of 25 | +4.3 [+1.4, +7.8] | 3 of 3 |
| `G0` - `F0+ctx` | +3.6 [+0.8, +7.0] | 0.0275 | 17 of 25 | +3.3 [+0.8, +6.8] | 3 of 3 |
| `G0` on the raw weights - `W0` | +1.0 [-1.2, +3.4] | 0.2891 | 13 of 25 | +1.8 [+0.1, +3.7] | 0 of 3 |
| `G0` on the raw weights - `F0+ctx` | -0.9 [-3.5, +1.8] | 0.7546 | 11 of 25 | +0.8 [-1.2, +3.0] | 0 of 3 |
| `R0` - `F0+ctx` | -0.1 [-1.0, +1.0] | 0.8561 | 12 of 25 | -0.3 [-0.9, +0.3] | 0 of 3 |

- **`G0` passes the comparison bar against `W0` (and `F0+ctx`) only with the
  per-fold weight rescale**, a recorded deviation (Step 2 in
  [learnings.md](learnings.md)): its per-folder class-balanced weights are
  rescaled to mean 1 on each fold's training frames, where the plan's text
  passes them as they are (each folder summing to 1). **On the raw weights
  the same setting fails** both comparisons (0 of 3 bars). The rescale was
  decided before the first run for a stated reason (the fixed
  regularization was meant for unit-scale weights), but the pass rests on
  it, so it is read as "`G0` wins under a choice the pre-registration did
  not fix", not as a clean pre-registered win.
- **No filter passes the worth-a-filter bar.** `G0`'s label-free
  `quantile@95` keeps 94.7% of the picks (the 93% recall floor is met) but
  removes **16.8%** of the non-picks, against the **25%** floor. Even its
  oracle cut, which uses the held-out folder's own labels, removes 22.0%,
  below the floor. No other graded score's `quantile@95` removes more than
  15.9% (`F0+ctx +c1+c2`). Per folder, `G0`'s realized recall at
  `quantile@95` falls below 90% in 5 of the 25 folders (80.0% at the
  lowest). At 99% pick recall the best label-free removal is 5.9% (`G0`).
- **No rule filter is competitive.** No binary rule keeps 95% of the picks
  in more than 11 of the 25 folders; scored 0 where they do not, their mean
  `RN@95` is 0.0–3.9%, 12.7–16.6 points below `W0`. The rule points nearest
  to safe (`eyes closed` 92.8% recall / 11.3% `RN`, `sharpness bottom 10%`
  93.4% / 10.3%) remove less than `W0`'s label-free cut (95.0% / 11.5%)
  while losing more picks; the ones that remove a quarter or more lose a
  quarter or more of the picks (`face away` 75.1% / 23.5%, `any of` 59.7% /
  49.6%, `not good` 28.8% / 80.3%). The graded `rule count` reads `RN@95`
  7.9%, within the baselines' range (5.8–9.3%).
- **What `G0`'s nonlinearity buys.** Over `F0+ctx`: +3.6 points of `RN@95`
  rescaled, -0.9 on the raw weights; `R0`'s asymmetric pick weight buys
  nothing (-0.1). So a linear model trained to keep the picks does not move
  the bottom, and the nonlinear one moves it only in the rescaled fit. Its
  drop-one table (2 below) is the one place the "any single failure" cues
  register at the bottom at all.

**2. Column contributions at the bottom** ("yes" = the drop-one lowers
`RN@95` with its bootstrap interval entirely below zero; the `RN@95` change
in points in brackets, with the secondary readings whose interval is below
zero), beside Decision 1's top-oriented verdict (frame AUC / R-precision /
within-burst wAUC):

| Columns | Bottom, `F0+ctx` | Bottom, `W0` | Bottom, `G0` | Alone `RN@95` % (`F0+ctx` / `W0`) | Decision 1, `F0+ctx` | Decision 1, `W0` |
| --- | --- | --- | --- | ---: | --- | --- |
| Look (32 PCA) | no (-4.2) | no (-3.0) | **yes** (-5.6; `RN@99`, `pAUC95` too) | 13.4 / 12.1 | yes / yes / yes | yes / yes / yes |
| Burst position | no (-1.5) | **yes** (-1.9; `pAUC95` too) | **yes** (-2.2) | 12.7 / 12.7 | no / no / yes | yes / no / yes |
| Sharpness | no (+0.2) | no (-0.8; `RN@99`, `pAUC95` below zero) | no (-0.4) | 9.1 / 9.9 | yes / no / no | no / no / no |
| AF eye focus | no (-0.4) | no (-0.6) | no (-0.9; `pAUC95` below zero) | 9.0 / 9.5 | no / no / no | no / no / no |
| Eyes open | no (-0.4) | no (+0.1) | no (-2.0; `RN@99`, `pAUC95` below zero) | 10.0 / 9.7 | yes / yes / no | yes / no / yes |
| Head pose (all) | no (+0.2) | no (+0.3) | no (-1.2) | 6.9 / 6.5 | no / no / no | no / no / no |
| `\|yaw\|` | no (-0.3; `RN@99`, `pAUC95` below zero) | no (+0.1) | no (+0.8) | - | no / no / no | no / no / no |
| `\|pitch\|` | no (+0.2) | no (+0.4) | no (-0.4) | - | no / no / no | no / no / no |
| `\|roll\|` | no (+0.2) | no (-0.1) | no (-0.2) | - | no / no / no | no / no / no |
| Good photo | no (-0.1) | no (-0.4) | no (+0.4) | 9.3 / 9.3 | no / no / no | no / no / no |
| `ln burst size` | no (+0.5) | (not in `W0`) | no (-0.1) | - | no / no / no | (not in `W0`) |
| c1 (added) | no gain (-0.7) | no gain (+0.6) | - | 9.3 / 10.6 | no gain | no gain |
| c2 (added) | no gain (+0.1) | no gain (-0.9) | - | 7.7 / 7.9 | no gain | no gain |
| Genre (added) | no gain (cols -0.7, off +0.8) | no gain (cols +1.0, off +0.8) | - | 7.2 / 7.1 | no gain | no gain |

For the added cues the bracket is the cue variant minus its base on `RN@95`
(none passes the comparison bar); random reads 5.8% `RN@95`.

- **The "obviously bad" cues do not matter more at the bottom than at the
  top**, on the primary metric. Sharpness, AF eye focus, eyes open, head
  pose and good photo contribute to `RN@95` on no base. Alone each removes
  6.5–10.0% of the non-picks at 95%, about what the simple baselines remove,
  and as rules none holds 95% of the picks across folders (1 above). The one
  hint of the hypothesis is in `G0`: eyes open lowers its
  `RN@99` and `pAUC95` when dropped, and AF eye focus lowers its `pAUC95`, which the linear
  models do not show, but not its `RN@95`; with about 100 drop-one cells
  read (11 rows, 3 metrics, 3 bases), this is a weak, unconfirmed signal.
- What carries the bottom is what carries the top: the embedding (Look) and
  burst position. Look's drop is the largest on every base (-3.0 to -5.6
  points), though its interval reaches zero on the two linear bases. Eyes
  open, which contributed at the top on both bases in Decision 1, does not
  contribute at the bottom on either linear base.

**3. Genre under this criterion: Decision 1's "drop for the ranking"
stands.** None of the four genre comparisons passes the comparison bar on
`RN@95` against its base: `F0+ctx +genre cols` -0.7 points [-2.6, +1.2]
(p = 0.6159, 13 of 25, 0 of 3 bars); `F0+ctx +genre off` +0.8 [-0.7, +2.3]
(p = 0.1498, 14 of 25, 0 of 3); `W0 +genre cols` +1.0 [-0.2, +2.0]
(p = 0.0640, 16 of 25, 1 of 3); `W0+off` (shipped) +0.8 [-0.7, +2.5]
(p = 0.3110, 14 of 25, 0 of 3). The genre labels alone remove 7.2% / 7.1%
of the non-picks at 95% (random 5.8%). Nothing here changes the todo "Core:
a plan to drop the genre from the personal ranking" or its scope question
((a) ranking only or (b) the whole feature, as Decision 1 left it); the
wrap-up updates the todo to say this criterion confirms it.

**4. Follow-up: none.** By the pre-registered rule an app change is
recommended only when the best filter passes the worth-a-filter bar, and none
does, so this plan recommends **no app change**: no "likely reject" filter
item, no dimming in the strip, no `G0` model type, no rule filter, and
nothing asked of the `rank.rs` session. What a filter would cost on today's
evidence: at the label-free cut that keeps about 95% of the picks (one pick
in twenty lost on average, more than one in ten in 5 of 25 folders), the
best score hides about a sixth of the non-picks.

For a later attempt, these are what would have to change for the answer to
differ (conditions, not a plan; whether and when to pursue any of them is
the user's call):

- **Negatives that are bad, not merely unpicked.** Every non-pick counted as
  a negative here, so the second-best frame of a burst and a duplicate count
  against a filter as much as a blurred frame; the removed share is an upper
  bound on "bad removed", and the 58 explicit rejects in 3 folders are too
  few to measure precision on. Explicit rejects and `views` rows (viewed and
  left unflagged) accumulating in the app would let a later evaluation
  measure a filter on "bad" and train on it.
- **More data.** 27 folders of one photographer, mostly basketball; a
  nonlinear model fit on them may not carry over, and per-folder recall
  scatters at small pick counts.
- **`G0`'s weight scale fixed in advance.** Its pass depends on a rescaling
  the pre-registration did not fix; a later plan would pre-register the
  weight scale (or tune it in the inner CV) before reading `G0` again.
- **Label-free threshold calibration.** The label-free cut gives up about 5
  points of `RN` against the oracle for `G0` (16.8% against 22.0%) and
  scatters per folder; a calibrated threshold (for example one learned on
  each fold's training folders, the alternative not taken in "Trade-offs and
  risks") could close part of that gap. It cannot pass the bar alone on this
  data, since even the oracle cut is below 25%.

## Progress

- (2026-10-16) Plan written; bar option B and variants option (b) chosen
- (2026-10-16) Step 1 complete
- (2026-10-16) Step 2 complete
- (2026-10-16) Step 3 complete
