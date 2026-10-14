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

# Per-genre offsets for the personal ranking

## Purpose

The personal best-shot ranking (`riffle_core::rank`, the `Recommended`
order) learns one set of 44 weights for every photo. What makes the best
frame of a basketball burst is not what makes the best frame of a snap or a
portrait burst, so one shared model averages tastes that differ by genre.
This plan gives the model a shared weight vector plus one correction
(offset) per top-level genre of `crates/core/genres/siglip2.json`. Every
offset starts at zero, is pulled toward zero by an L2 penalty that is fixed
while the genre's pairs grow, so a genre with few judgments ranks exactly or
nearly as the shared model does and a genre with many judgments moves away
from it (partial pooling). The image-embeddings plan's Decision 2
(`../_archived/20261012-image-embeddings/gate.md`) tried per-genre offsets
on the technical columns with a 10x penalty and found no gain (weighted AUC
0.707 vs 0.708) on data that is 82% sports, so this plan starts with an
offline evaluation that measures the effect and checks the offsets do no
harm before anything ships.

Decisions taken in this plan (confirmed by the user before Step 1):

- **The genre of a burst** is the top-level label (one with no `parent` in
  the genre data file) that `genre::genres_of` turns on for the most of the
  burst's frames with an embedding, a frame counting toward every top-level
  label it turns on (a sub-genre already brings its parent); ties go to the
  label earlier in the data file; `UNKNOWN` when no frame turns a label on.
  The rule lives in `genre.rs` so the `Genre` filter and the ranking read
  the same labels.
- **An offset covers all `COLUMNS`** (technical, position and PCA
  columns) unless Step 1 shows only the technical-only shape passes, so the
  weights a burst of genre `g` is ranked by are `shared + offset[g]` and the
  code of `rank` is one vector add. `UNKNOWN` has no offset.
- **Shrinkage**: the fit's objective is the current one plus
  `OFFSET_SHRINK * tau / 2 * ||offset[g]||^2` per genre, `tau = 1 / (2 C)`
  the pull toward the prior. The penalty is fixed, so its weight relative
  to the genre's data term falls as that genre's pairs accumulate: this is
  the "takes effect more with more judgments" shape, and a genre with no
  pair keeps an offset of exactly zero. `OFFSET_SHRINK` is chosen by
  Step 1's grid.
- **The bar is "no harm"** (the user's choice): the offsets ship when they
  do not lower the pooled AUC beyond a small margin, even if no genre shows
  a significant gain, because a genre with few judgments falls back to the
  shared model by construction.
- **The prior ships no offsets** (all zero), so `crates/core/ranking/siglip2.json`'s
  weights do not change and `rank_prior.py` is not re-run; only its
  `rankVersion` moves with `RANK_VERSION`.
- **`RANK_VERSION` becomes 2** because `ranking.json` gains the offsets. A
  stored fit at version 1 falls back to the prior through the existing
  `Stored::parse` rule, and the startup refit the per-file-viewed-ranking
  plan adds rebuilds it on the next launch.
- **The meta pane** shows the genre a burst was ranked as, as a suffix on
  the `Recommended` row (Step 3).

## Steps

- [x] Step 1: Offline evaluation of the hierarchical offsets on the app dump, and the decision
  - Done when:
    - First, confirm that every implementation PR of
      `../20261014-per-file-viewed-ranking/plan.md` (PR #801's plan; the
      removal of `Culling Finished`, `Labeled.viewed`, the startup / folder
      switch refit, the dwell) is merged into `main`. If not, stop this
      plan here and say so; do not start.
    - `genre_offsets.py` in this plan folder (next to no other code;
      imports `gate.py` from `../_archived/20261012-image-embeddings/` by
      path, as `rank_prior.py` does) runs the leave-one-folder-out
      evaluation of `gate.py` on the app dump
      `D:\Photos\tests\2026-10-12-rank-app` for: the shared model (as a
      check that it still reads 0.710), and the hierarchical variant at
      `OFFSET_SHRINK` in {3, 10, 30, 100} with offsets on all columns, and
      at the chosen `OFFSET_SHRINK` with offsets on the technical columns
      only (the Decision 2 shape, for the comparison). The burst genre is
      the rule above, computed from the shipped
      `crates/core/genres/siglip2.json` (cosine thresholds, parent
      bringing), not from the provisional `genre_text.tsv` softmax
      `gate.py` used. The objective is the one `rank.rs` will use (shared
      weights with L2 toward zero at `1 / (2 C)`, offsets at
      `OFFSET_SHRINK` times that; in scikit-learn this is the column
      scaling trick `gate.py`'s `design` already uses, `1 / sqrt(OFFSET_SHRINK)`
      on the `[burst is genre g] * x` columns).
    - It prints, and `offsets.md` in this plan folder records: scorable
      bursts per genre under the shipped rule; pooled held-out weighted
      pairwise AUC and Top-1 of every variant; per-genre held-out weighted
      AUC for every genre with at least 20 scorable bursts; the paired
      per-burst difference (hierarchical minus shared) with a 1000-draw
      bootstrap 95% interval, pooled and per such genre; and a fallback
      check: with the second-largest genre's training bursts cut to 20
      (three seeded draws), its held-out AUC under the hierarchical variant
      against the shared model's, which must agree within 0.005.
    - The decision is recorded in `offsets.md` against a bar written
      before the run: **keep on** when, at the chosen `OFFSET_SHRINK`, the
      pooled weighted AUC is not below the shared model's by more than
      0.002 and the fallback check agrees within 0.005 (a significant gain
      on a genre is reported but not required); **drop** otherwise, with
      the numbers, in which case Steps 2 and 3 are not implemented and the
      plan is archived as the record (as Decision 2 was).
    - `OFFSET_SHRINK` (the value with the highest pooled AUC among those
      that pass) and the column scope (all columns, or technical only if
      that is the only shape that passes) are named in `offsets.md` for
      Step 2.
    - `mise run ci` passes (lint of the Markdown and links).
  - Implementation approach:
    - Reuse `gate.load`, `gate.build`, `gate.scorable`, `gate.pairs`,
      `gate.burst_metrics` and the `C` choice; add only the genre columns
      and the burst genre rule. Keep `gate.py` itself untouched.
    - The app dump's `features/` and `embed.tsv` already hold the same
      inputs the app stores (`ranking.md` in the archived plan); if the
      folder is gone, `ranking.md` lists how to rebuild it
      (`rank_dump_of_real_folders`, `rank_dump.py`).
    - Expect the genre counts to differ from Decision 2's (1478 sports /
      287 snap under the provisional softmax); report them as they come.

- [x] Step 2: Learn and rank with the offsets in `riffle-core`, store them in the app
  - Done when:
    - `crates/core/src/genre.rs`: `pub fn top_level() -> Vec<&'static str>`
      (the labels with no parent, in file order) and
      `pub fn of_burst<'a>(vecs: impl IntoIterator<Item = &'a [f32]>) -> &'static str`
      with the rule above; tests cover a majority, a tie going to the
      earlier label, a frame counting for two labels, and no vector or no
      label on giving `UNKNOWN`. A `#[cfg(test)] pub(crate) fn vector_of(label) -> &'static [f32]`
      (or equivalent) so `rank.rs`'s tests can build frames of a known genre.
    - `crates/core/src/rank.rs`: `RANK_VERSION` is 2. A
      `pub struct Weights { pub shared: Vec<f64>, pub offsets: BTreeMap<String, Vec<f64>> }`
      (a label absent from `offsets` means zero); `prior_weights()` returns
      a `Weights` with empty offsets (or the function is renamed, the
      implementer's choice, with every caller following); `rank(frames, &Weights)`
      ranks each burst with `shared + offsets[of_burst(...)]`, the reasons
      summing as today; `Fit.weights` is a `Weights` whose `offsets` hold
      every `top_level()` label. `fit` minimizes the current objective plus
      `OFFSET_SHRINK * tau / 2 * ||offset_g||^2` per genre, by the same
      Newton loop: accumulate one gradient and one `COLUMNS`-square
      Hessian block per genre from its pairs, assemble the
      `(1 + G) * COLUMNS` system (shared block = the sum plus `tau I`,
      off-diagonal blocks = the genre's block, diagonal genre blocks = the
      genre's block plus `OFFSET_SHRINK * tau * I`), and solve it with
      `solve` generalized to any `n` (its test follows). Pairs from an
      `UNKNOWN` burst enter the shared block only. `OFFSET_SHRINK` is a
      `pub const` with Step 1's value and a doc comment citing `offsets.md`.
      `Fit` also reports the bursts per genre (for the log line).
    - Tests in `rank.rs`: a genre with no pair keeps an exactly zero
      offset and ranks exactly as the shared weights do; a genre with two
      bursts whose picks contradict a genre with 300 bursts ranks its own
      burst in the same order as the shared model (the fallback), while the
      same genre with 300 contradicting bursts ranks it the other way (the
      offset at work); an `UNKNOWN` burst ranks as the shared model under
      any offsets; `the_prior_is_made_for_the_shipped_embedding_and_these_columns`
      still passes (the prior's `rankVersion` is edited to 2 by hand and
      `rank_prior.py`'s `RANK_VERSION` constant too, with a comment in the
      module doc that the prior carries no offsets so no re-run was needed);
      the existing fit tests follow the `Weights` shape.
    - `crates/app/src/rank.rs`: `Stored` gains `offsets: BTreeMap<String, Vec<f64>>`;
      `parse` also drops an offset whose label is not in `genre::top_level()`
      or whose length is not `COLUMNS` (dropping the one offset, not the
      file; the test `a_fit_at_other_versions_or_of_another_size_falls_back_to_the_prior`
      gains those cases); `refit` stores and uses them; `ranking_of` ranks
      with the stored `Weights`; the `ranking fit:` log line adds the
      bursts per genre (`sports 120, snap 9`); `the_scores_match_rank_prior_py`
      follows (`fit.weights.shared`, and prints each offset's L2 norm).
      `ModelInfo` and the frontend's `RankModel` are unchanged.
    - Docs in the same PR: `docs/humans/usage.md` **Recommended order**
      (the model learns a correction per genre on top of what it learns for
      every photo; a genre you have judged little in is ranked as the rest
      are until it has enough judgments) with `usage.ja.md` in sync;
      `README.md` / `README.ja.md` if the Recommended bullet describes
      what is learned; `CLAUDE.md` layout (`rank.rs` and `genre.rs`
      sentences); `docs/agents/tauri-app.md` ranking bullet (`ranking.json`
      holds the shared weights and the per-genre offsets; a version-1 file
      means the prior until the next refit). `todo.md`'s ranking section
      gains a manual check that `Riffle.log`'s fit line lists bursts per
      genre and that `ranking.json` holds `offsets`.
    - `mise run ci`, `cargo test -p riffle-core` and `cargo test -p riffle-app` pass.
  - Implementation approach:
    - Assumes Step 1 merged and decided "keep on", and the
      per-file-viewed-ranking plan on `main`: `pairs` keeps its `viewed` /
      `REJECT_WEIGHT` logic and only gains the burst's genre index per pair.
    - Per-genre blocks keep the per-pair cost at `COLUMNS^2` as today; only
      the one `(1 + G) * COLUMNS` solve per iteration grows (G = 5 today,
      so 264 square: negligible).
    - Each burst's genre needs its frames' vectors, which `Frame.vec`
      already carries; compute it once per burst in `rows` or next to it and
      hand it to both `rank` and `pairs`.
    - Rename `prior_weights` only if it keeps the code clearer; do not
      restructure the prior file or `Prior`.

- [x] Step 3: Show the genre a burst was ranked as in the meta pane
  - Done when:
    - `Ranked` (`rank.rs`) gains `genre: Option<&'static str>`, `None` for
      `UNKNOWN`; `meta.ts`'s `Ranked` mirrors it, and `rankingRows` appends
      ` · as sports` to the `Recommended` row's value when it is set
      (`2 of 5 in its burst · +0.31 · as sports`); the `Reason` and
      `Confidence` rows are unchanged. `meta.test.ts` covers both cases.
    - `docs/humans/usage.md` **Recommended order** names the suffix with
      `usage.ja.md` in sync; `todo.md`'s ranking manual checks gain the
      row's text.
    - `mise run ci` and `cargo test -p riffle-core` pass.
  - Implementation approach:
    - Assumes Step 2 merged. The suffix is the only visible sign that the
      genre took part; the reasons keep their seven names because the
      offset changes their sizes, not their meaning.

## Trade-offs and risks

- **A "no harm" bar.** The user chose to ship the offsets when they do not
  hurt, rather than requiring a measured gain. The mechanism is harmless
  for a genre with few pairs by construction, but a pass may mean the
  feature is in without a demonstrated effect; Step 1 still reports any
  per-genre gain so that is visible.
- **The data cannot show much.** The author's 27 folders are mostly sports
  under any rule; snap is the only other genre likely to have enough
  held-out bursts for a per-genre number. A pass says the offsets do not
  hurt for this user; it says nothing about another photographer's library.
- **Burst genre: majority of frame labels vs the mean embedding.** The
  majority rule reuses `genres_of` and so agrees with the `Genre` filter
  the user sees; the mean embedding would need its own threshold handling
  (the mean's norm shrinks). A burst of frames split between two labels
  goes to the earlier label on a tie, a deterministic but arbitrary rule.
- **Offsets on all columns vs technical only.** All columns lets the
  `Look` (embedding) weights differ per genre, which is where taste most
  plausibly differs; it also multiplies the parameters the offset can
  overfit, which the penalty is for. Step 1 measures both; Step 2 takes
  what passed.
- **A fixed penalty, not a count-dependent one.** A penalty that itself
  shrank with the pair count would double the effect the data term already
  has; a fixed `OFFSET_SHRINK * tau` is the standard partial-pooling form
  and is one constant to choose. The grid {3, 10, 30, 100} spans a 10x
  stronger penalty than Decision 2's on either side.
- **`RANK_VERSION` 2 vs a serde default.** A `#[serde(default)]` on
  `offsets` would keep a version-1 `ranking.json` valid; bumping the
  version follows the existing rule and the stored fit is rebuilt at the
  next startup anyway (per-file-viewed-ranking Step 2). The bump means
  editing the prior's `rankVersion` by hand once, documented in the module
  doc.
- **Sub-genres get no offset.** Basketball, group photo, pets and camping
  fold into their parents; a per-sub-genre offset would need a further
  level of pooling and the data has none to measure it with.
- **Genre file changes.** An offset is keyed by label; a label removed from
  the genre data file loses its offset at parse, a new one starts at zero.
  No `GENRE_VERSION` exists and none is added.
- **Meta pane.** Option taken: a short suffix on the `Recommended` row.
  Options not taken: no visible change at all (the `Genre` filter already
  shows the labels), and a separate `Genre` row.
- **Conflict with the in-flight plan.** `pairs`, `Labeled` and
  `crates/app/src/rank.rs`'s `refit` are rewritten by
  per-file-viewed-ranking; Step 1's first check exists so Step 2 starts
  from that end state and never rebases over it.

## Progress

- (2026-10-15) Step 1 complete: keep on, OFFSET_SHRINK = 30, all columns (offsets.md)
- (2026-10-15) Step 2 complete
