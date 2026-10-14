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

# Folder-wide pick model: a per-photo "will the user pick this" score, gated offline

## Purpose

The personal ranking (`crates/core/src/rank.rs`, shipped by
[image-embeddings Step 7](../_archived/20261012-image-embeddings/plan.md))
is a pairwise model trained only on pick vs not-chosen pairs **within one
burst**. Its features are burst-relative (sharpness as a ratio to the
burst's sharpest frame, position = index within the burst) and its score is
centered on the burst's mean, so a score of `+0.8` in one burst and `+0.8`
in another say nothing about which of the two frames the user would rather
keep. A single (non-burst) shot has no comparison partner: it enters neither
the training pairs nor the ranking (`rank` gives it place 1 of 1, score 0).
The user asked whether the learned score could be used across a folder; the
answer today is no.

This plan builds a separate, per-photo model of **the probability the user
picks this photo**, so that singles and frames of different bursts can be
compared on one scale across a whole folder. The materials already exist:
the SigLIP 2 embedding (`embeddings` table, `crates/core/src/embed.rs`), the
pass-2 technical values (`eye_focus`, EAR, head pose, sharpness), the genre
labels (`crates/core/src/genre.rs`), the picks / rejects (`ratings`) and the
viewed records (`views`).

Cross-burst differences are driven to a large degree by whether the scene
itself was good (the right play, the right light), which an image embedding
may or may not carry across shooting days. The plan therefore starts with a
**pre-registered offline gate** on the same 27 sidecar-labeled ARW folders
(32,459 frames, 4,419 picks) the in-burst ranking was gated on. Only if the
gate passes does anything ship; if it fails, Steps 3–5 are dropped and the
plan records the numbers and ends.

Out of scope (another session owns them; do not touch):

- The in-burst ranking's `pairs`, `Labeled`, `fit` in `rank.rs`, the
  training-data scope (`views` / `Culling Finished`), the refit triggers and
  the 0.5 s viewed threshold
  (`docs/plans/20261014-per-file-viewed-ranking/plan.md`, on
  `origin/feature/per-file-viewed-ranking`, not yet on `main` at planning
  time).
- Per-genre weighting (the dlg-genre-weights session).

Constraints, binding on every step: everything local, RAW files never
modified, a recommendation never changes a judgment; everything behind the
`aiEnabled` setting; the in-burst ranking keeps working unchanged.

### What is known

- **Data.** `D:\Photos\tests\2026-10-12-rank-app\` still holds the app
  dump of the 27 folders in the format the archived `gate.py` reads:
  `features/<folder>.tsv` (+ `.output.txt`), `embed.tsv`, `genre_text.tsv`,
  `genres.tsv` (per-file genre sigmoid scores), `prior_scores.tsv` (the
  in-burst prior's score of every frame) and `index.sqlite`. Labels as
  [gate.md](../_archived/20261012-image-embeddings/gate.md): a pick is a
  `.dop` Pick, a virtual copy in `Output/`, or an XMP Pick with no `.dop`;
  every other frame is not chosen (these folders count as culled); 58
  explicit rejects. Folder pick rates run from about 10% to 25%
  ([burst-keep-score data.md](../_archived/20261008-burst-keep-score/data.md)).
- **Tooling to reuse.** `gate.py` (`load`, `build`, `bursts_of`,
  `tech_row`, `design`, `choose_c`, the Markdown table helpers),
  `rank_prior.py` (how a prior JSON is written and cross-checked against
  Rust), `rank_dump.py` (the app-dump format), all in
  `docs/plans/_archived/20261012-image-embeddings/`. Import them the way
  `rank_prior.py` does (`sys.path.insert(0, <archived folder>)`, `import
  gate`); do not copy them. Python 3.13, numpy / scikit-learn / joblib are
  on the machine (scipy for the Wilcoxon test; check, else implement the
  exact signed-rank test in a few lines); run with `python -I` from this plan
  folder.
- **Prior numbers.** In-burst, held out: weighted pairwise AUC 0.710 with
  the embedding, 0.652 without, first frame 0.650; Top-1 54.9% vs 39.4%.
  Face-free frames: sharpness alone reads AUC 0.579 pick vs non-pick
  (af-eye-good-bad-cascade Decision C). No folder-wide number exists yet.
- **Rust side.** `rank::Frame`, `rank::bursts` (the `burst.ts` grouping)
  and `Index::rank_rows` (`crates/app/src/index.rs`) already deliver every
  input a per-frame model needs; the `embeddings` table needs no change. The
  PCA basis and standardization live privately in `rank.rs`'s `Prior`
  (`crates/core/ranking/siglip2.json`). `AppRank` (`crates/app/src/rank.rs`)
  stores the fit in `ranking.json`, refits through `refit`, and announces
  `ranking-updated`; the frontend reads `ranking_of` in `main.ts`
  `refreshRanks`, sorts with `sort.ts` `orderFiles` (`recommended` = burst
  id, then score) and shows `meta.ts` `rankingRows`.

### Decisions taken at approval (2026-10-14)

The user asked for whichever option gives the more accurate judgment:

- **Significance rule**: a paired one-sided Wilcoxon signed-rank test across
  folders at p < 0.05 (not the SD-gap rule of the earlier gates); the SD gap
  is reported beside it for comparison.
- **Check-1 baseline**: the best of all non-embedding baselines and variants,
  not the without-embedding variant alone.
- **Genre-column variant**: adopted only if it beats the full model by the
  same Wilcoxon test.
- **PCA basis**: shared with `rank.rs` through one `pub fn project` accessor.
- **UI surface** (if the gate passes): a `Likely picks` sort key and a meta
  pane row; reconfirm with the user before Step 5 only if something changes.

## Steps

- [ ] Step 1: Pre-register the folder-wide gate (criterion, data, model family, baselines, metrics) in `gate.md`, with no fit run
  - Done when:
    - `gate.md` in this plan folder has a "Pre-registration" section
      written and merged **before any script exists or any fit runs** (the
      merge commit of this PR is the record, as `ce0ea14` was for the
      in-burst gate). Nothing in that section changes afterwards; a
      deviation found while implementing Step 2 goes to `learnings.md` with
      its reason and does not touch the criterion.
    - The section fixes, in this order:
      - **Data**: the 27 folders of the app dump, labels as gate.md's
        (picks incl. virtual copies; every non-pick not chosen; rejects
        reported separately); bursts as `rank::bursts` (gap 1000 ms);
        singles are bursts of one.
      - **Unit and folds**: every frame with a vector is a sample;
        leave-one-folder-out over the 27 folders; `C` by an inner 5-fold
        grouped-by-folder CV as `gate.py`.
      - **Model family**: a per-frame L2 logistic regression (not
        pairwise), sample weights giving each folder a total weight of 1
        and, within a folder, picks and non-picks each half of it (so a
        folder's pick rate does not become the model). Columns: the 32
        whitened PCA dimensions of the embedding (PCA fit on the training
        folders' frames); the technical columns of `gate.py`'s `TECH` with
        **absolute** `ln sharpness` added beside the burst-relative one;
        the context columns `ln burst size`, position in burst, first
        frame; every column standardized on the training folders. Report
        variants: full; without embedding (technical + context); embedding
        only; context only (`ln burst size`, position); embedding centered
        on the folder's mean vector (does "stands out in this folder" help);
        full + the shipped genre labels as 0 / 1 columns (from
        `genres.tsv` thresholds of `crates/core/genres/siglip2.json`).
      - **Baselines** on the same folds: random; capture order (earlier
        first and later first); burst size (larger first); absolute
        sharpness; `eye_focus` (missing last); `photoTier` good; the
        in-burst prior's score read from `prior_scores.tsv` applied
        folder-wide (the reading aid that motivates this plan: it should
        read near chance across bursts).
      - **Metrics**, each per folder then the mean and the sample SD
        across folders with at least 50 frames and 5 picks (the
        "qualifying folders"):
        1. **Frame AUC within the folder** (pick vs not chosen over all the
           folder's frames).
        2. **Burst AUC** (the cross-burst question): one unit per burst,
           singles included, the unit's score = its highest frame score,
           the unit's label = the burst holds a pick.
        3. **Singles AUC**: frames whose burst has one frame only, where the
           in-burst ranking is silent.
        4. **Lift at the top tenth**: the pick share among the folder's
           top 10% by score divided by the folder's pick rate.
        5. Also reported: the pick vs explicit-reject AUC on the folders
           with rejects, a learning curve at 5 / 10 / 20 / 26 training
           folders, and the fitted weights on every folder grouped as
           `rank.rs`'s `REASONS` groups them.
      - **Criterion** (the gate passes only when all hold for the full
        model, held out, over the qualifying folders):
        1. per-folder frame AUC is greater than that of the best
           non-embedding baseline or variant (the "without embedding"
           variant, context only, burst size, absolute sharpness,
           `eye_focus`, capture order in either direction, whichever has
           the highest mean frame AUC) by a paired one-sided Wilcoxon
           signed-rank test at p < 0.05;
        2. per-folder burst AUC is greater than the burst-size baseline's
           by the same test at p < 0.05;
        3. mean lift at the top tenth >= 2.0.
        For checks 1 and 2, also report the mean gain against the
        baseline's per-folder SD (the rule of the earlier gates), for
        reading only; it does not decide.
      - **Genre variant**: the full + genre model replaces the full model
        only if its per-folder frame AUC beats the full model's by the
        same test at p < 0.05.
      - **What happens on fail**: Steps 3–5 are dropped, Decision 1 below
        records the tables, and the plan ends with the wrap-up only.
    - `mise run lint` passes (lychee on the new links).
  - Implementation approach:
    - Model the section on gate.md's Pre-registration; keep the same
      vocabulary (weighted / unweighted, per-folder spread, chosen variant).
    - Decide and write the exact SQL-free label rule once: the dump's
      `flag` column is the app's own; gate.md's labels add the 427
      `Output/`-only virtual-copy picks. Use gate.md's (through `gate.load`)
      so the numbers are comparable with the in-burst gate; say so.
    - No Python, no Rust in this PR.

- [ ] Step 2: `folder_gate.py`, the results and Decision 1 (ship or not)
  - Done when:
    - `folder_gate.py` in this plan folder (`python -I folder_gate.py
      <app-dump-dir> [jobs]`) imports the archived `gate.py` for loading,
      bursts, technical columns and the table helpers, implements the
      pre-registered model, variants, baselines, metrics and criterion
      exactly, and prints the Markdown tables; its output is saved as
      `gate.out.md` next to the dump and pasted into `gate.md`'s "Results"
      section with a "Reading" paragraph.
    - Every pre-registered table is in `gate.md`: data counts (frames,
      picks, rejects, bursts, singles, per-folder pick rates), the held-out
      table of all models and baselines for the four metrics with per-folder
      SDs, the criterion table with each check's statistic, p-value and met
      / not met, the per-folder table (frame AUC and burst AUC of the full
      model, the without-embedding variant and the burst-size baseline), the
      pick vs reject table, the learning curve, the weights by reason group.
    - **Decision 1** is written in this plan: pass -> Steps 3–5 proceed with
      the full model (or the genre variant if it won its test; note that it
      then depends on `genre.rs` at run time); fail -> Steps 3–5 are struck
      through with "dropped (Decision 1)" and the plan ends.
    - `learnings.md` records any deviation from the pre-registered text and
      why.
  - Implementation approach:
    - Assumes Step 1 merged. The dump at
      `D:\Photos\tests\2026-10-12-rank-app\` is read as is; if it is gone,
      regenerate it with the ignored test `rank_dump_of_real_folders`
      (`crates/app/src/commands.rs`) and `rank_dump.py` as
      [ranking.md](../_archived/20261012-image-embeddings/ranking.md)
      describes, and note the run time.
    - `gate.build` already yields `tech`, `pos`, `emb`, `burst_of`,
      `folder_of`, `pick`, `reject` and the burst spans; add `ln burst size`
      and absolute `ln sharpness` from `rows` / the spans, do not modify
      the archived script.
    - Sample weights: compute per folder from `pick` and `folder_of`.
      scikit-learn `LogisticRegression(C, fit_intercept=True, max_iter=2000)`
      with `sample_weight`; the intercept is per fit, not per folder.
    - Per-folder AUC with `sklearn.metrics.roc_auc_score` is fine; the
      burst unit needs a small helper (max over the span). Wilcoxon with
      `scipy.stats.wilcoxon(..., alternative="greater")` if scipy is
      present.
    - Record wall time and `jobs`; the in-burst gate took about three
      minutes on 12 workers and this one fits fewer models per fold.

- [ ] Step 3 (only if Decision 1 passes): `crates/core/src/pick.rs`, the folder-wide prior and its cross-check
  - Done when:
    - `crates/core/src/pick.rs` (name free; not inside `rank.rs`) holds the
      per-frame model: `PICK_VERSION`, the column layout of Step 1, `rows`
      (the standardized columns of every frame with a vector, burst context
      from `rank::bursts`), `score(frames, weights) -> Vec<Option<Scored>>`
      with `Scored { probability: f64, logit: f64, reasons: Vec<(&str, f64)> }`
      whose reasons group the columns as `rank.rs`'s `REASONS` do (`Look`,
      `Burst context`, `Sharpness`, `AF eye focus`, `Eyes open`, `Head
      pose`, `Good photo`) and sum to the logit minus the intercept, and
      `fit(folders) -> Fit` (Newton with an L2 pull toward the prior's
      weights at the prior's `1 / (2 C)`, as `rank::fit`), taking the same
      labels the in-burst fit takes at that time (see approach).
    - `crates/core/ranking/siglip2-pick.json` (name free) holds the prior:
      `model`, `embedVersion`, `pickVersion`, the imputation means, the
      standardization, the intercept, the weights and `C`, written by
      `pick_prior.py` in this plan folder (modeled on `rank_prior.py`, fit
      on all 27 folders of the dump, with an optional `scores.tsv`). The
      PCA basis is **not** duplicated: `rank.rs` gains one `pub fn
      project(vec: &[f32]) -> [f64; PCA_DIMS]` (the prior's whitened
      projection, which `rows` already computes inline) and `pick.rs` calls
      it, so both models live in the same coordinates and `pick_prior.py`
      must read the basis from `crates/core/ranking/siglip2.json`.
    - Unit tests as `rank.rs` has: the prior matches `SHIPPED.name`,
      `EMBED_VERSION` and `PICK_VERSION`; `rows` fills the context columns
      and imputes; `score` gives a probability in (0, 1) whose reasons sum
      to the logit; a fit without labels returns the prior; a fit leans
      toward what the picks have. An ignored cross-check test in
      `crates/app/src/rank.rs` (next to `the_scores_match_rank_prior_py`)
      scores the dump's `index.sqlite` through `Index::rank_rows` and
      matches `pick_prior.py`'s `scores.tsv` within 1e-4.
    - `cargo test -p riffle-core -p riffle-app` passes; the module doc
      names the plan and says the score never changes a judgment.
  - Implementation approach:
    - Assumes Step 2 passed. Reuse `rank::Frame`, `rank::bursts`,
      `candidate`, `closed_probability`; copy the `tech` column code only
      if the absolute-sharpness column cannot be added without touching
      `rank::tech`'s signature (keep `rank.rs`'s diff to the `project`
      accessor).
    - Labels: `fit` consumes `&[rank::Labeled]`. If the per-file-viewed
      plan's Step 1 is merged, `Labeled` carries `viewed: Vec<bool>` and a
      `Flag::None` frame is a negative when viewed; if not, it carries
      `finished: bool` and the rule is the in-burst one. Either way, read
      the rule from one shared helper (add `rank::negative_weight(flag,
      ...)` if none exists yet, or call the per-file plan's) so the two
      models can never disagree on what a negative is. A reject weighs
      `rank::REJECT_WEIGHT`.
    - Sample weighting in the Rust fit as pre-registered: per folder total
      1, picks and negatives each half.

- [ ] Step 4 (only if Decision 1 passes): the app's pick scores, the stored fit and the refit hook
  - Done when:
    - `crates/app/src/rank.rs`: `Stored` (or a sibling struct in the same
      `ranking.json`) carries the pick model's weights with `PICK_VERSION`,
      falling back to the pick prior at other versions; `refit` fits both
      models from the same `rank_dirs` / `rank_rows` read (one read, two
      fits) and `ranking-updated` fires once; `ranking_of` returns, per
      file, the in-burst `Ranked` **and** the folder-wide
      `Scored` (`pick: Option<Scored>`) so the frontend makes one call, or a
      separate `pick_scores_of(dir)` command if the payload split is
      cleaner (implementer's call; say why in `learnings.md`). `ModelInfo`
      gains the pick fit's size (`frames`, `folders`) next to the bursts.
    - The pick fit runs wherever the in-burst refit runs (the settings'
      `Refit Ranking` and whatever triggers the per-file-viewed plan ships);
      this step adds no trigger of its own.
    - Tests: the stored pick weights round-trip and fall back at a stale
      version; `ranking_of` on a fixture folder returns a probability for a
      file with an embedding and none for one without.
    - `docs/agents/tauri-app.md` gets the "folder-wide pick score" bullet
      beside the ranking bullet (derived on every call, weights in
      `ranking.json`, prior in `crates/core/ranking/`).
  - Implementation approach:
    - Assumes Step 3 merged and, for the trigger set, whatever of the
      per-file-viewed plan is on `main`; do not add or move triggers.
    - Keep `AppRank`'s one `fit` mutex for both fits.
    - Measure and record in `learnings.md` the time of `ranking_of` on
      `D:\photos\2026\2026-09-19` (2134 files) before and after: the pick
      score is one dot product per file, so it should not move.

- [ ] Step 5 (only if Decision 1 passes): the frontend's folder-wide order and row, and the user docs
  - Done when:
    - `sort.ts`: a new `SortKey` (e.g. `"picks"`, label `Likely picks`)
      orders the whole folder by the pick probability, highest first,
      ignoring bursts; files without a score follow in capture order; ties
      by capture order then name. `sort.test.ts` covers it. The sort menu
      (`index.html`, `main.ts` beside `sort-recommended`) shows it only
      while `aiEnabled` is on, and it reads as capture order while off, as
      `recommended` does; the choice is remembered like the other keys.
    - `meta.ts`: the Analysis group gains a `Pick likelihood` row
      (`NN%`) with its own `Reason` (up to three, as `rankingRows`), for a
      file with a score; `meta.test.ts` covers it. `companion.ts` /
      `mcp.rs` `get_view` carry the probability only if the view summary
      already carries the in-burst rank (check; else leave them).
    - No strip icon or filter item in this step.
    - Docs in the same PR: `docs/humans/usage.md` + `usage.ja.md` (a
      **Likely picks** paragraph after **Recommended order**: what it is,
      that it compares across bursts and singles, what it learned from and
      the held-out numbers from Decision 1, that it never changes a flag);
      `README.md` + `README.ja.md` (one bullet); `CLAUDE.md` layout
      (`pick.rs`, `sort.ts`, `meta.ts`, `rank.rs` sentences); `todo.md`
      gains the manual check (open `D:\photos\2026\2026-09-19` with AI
      analysis on, switch to `Likely picks`, confirm the top of the strip
      is pick-dense and the meta row shows a percentage and reasons).
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 4 merged. `ordered()` in `main.ts` already passes a
      `SortFacts` lookup; add `pick?: number` there.
    - Keep `recommended` as is; the two orders answer different
      questions (within a burst vs across the folder).

## Trade-offs and risks

- **Significance rule.** Decided at approval: paired one-sided Wilcoxon
  signed-rank across the qualifying folders at p < 0.05, with the SD-gap
  rule of the earlier gates reported beside it. The rank test tests the
  per-folder differences directly instead of comparing a mean gain with the
  baseline's spread, which is what the earlier rule did and which can miss a
  consistent but small gain. With about 26 folders the test has reasonable
  power; checks 1 and 2 are not corrected for multiplicity because both must
  hold (an intersection test).
- **Baseline for check 1.** Decided: the best of all non-embedding baselines
  and variants, which guards against "the embedding only learned burst size
  or capture order".
- **Burst-size confound.** A long burst almost always holds a pick, and the
  model can learn it; check 2 against the burst-size baseline exists for
  that reason. If the full model passes check 1 but not 2, the embedding
  adds nothing across bursts and the gate fails as pre-registered.
- **Labels are "culled folder" labels, not `views`.** The 27 folders have
  no `views` rows; every non-pick counts as not chosen, as in gate.md. The
  shipped fit (Step 3) will use whatever negative rule the per-file-viewed
  plan ships; the gate's numbers are therefore an upper bound on what the
  app's first fits see. Recorded, not fixable offline.
- **One PCA basis.** Reusing `rank.rs`'s basis through a small `project`
  accessor keeps one 318 KB basis and one coordinate system (a one-function
  diff to `rank.rs`, outside the other session's `pairs` / `Labeled` /
  `fit`). If the other session's PR conflicts on `rank.rs`, the accessor can
  wait for its merge.
- **Merge order with the per-file-viewed plan.** Steps 3–4 read `Labeled`
  and the refit path that plan rewrites. Start Step 3 after its Step 1
  merges when possible; if not, implement against `finished: bool` and
  expect a small follow-up when `viewed` lands. Step 4 adds no trigger so
  the two plans do not fight over them.
- **How the score surfaces.** A sort key plus a meta pane row, the least
  intrusive, matching `recommended`. A filter section (hides frames behind a
  threshold the user cannot see) and folding into `Recommended` (changes what
  it means today) were set aside; a strip badge is deferred until the sort
  has been used.
- **Genre columns.** Reported as a variant; shipped only if they win their
  test. The 27 folders are almost all sports (1478 of 1801 scorable bursts),
  so a genre effect probably cannot show, and shipping them would couple the
  model to `genre.rs`'s thresholds.
- **Transfer.** The prior is one photographer's taste on one genre; a new
  user starts from it until their own picks pull the weights. The
  `Confidence` text should say so, as the in-burst one does.
- **Fail path.** If Decision 1 fails, the artifacts are `gate.md`,
  `folder_gate.py` and this plan with the numbers; no code changes. That is
  an acceptable outcome of the plan, not a failure of it.

## Decision 1 (folder-wide model ships or not)

(Written at the end of Step 2 from the pre-registered criterion in
`gate.md`.)

## Progress

- (2026-10-14) Plan written
