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

# Folder-wide ranking evaluation: re-measure the ranking, its cues and its columns on absolute photo quality

## Purpose

The user's goal for the personal ranking is that Riffle surfaces good photos
wherever they are: a single shot or a frame of a burst alike; bursts are not
special. The shipped ranking (`crates/core/src/rank.rs`: a pairwise logistic
on pick vs not-chosen pairs **within one burst**, scores centered on the
burst's mean, singles unranked) and every evaluation so far
([image-embeddings gate.md](../_archived/20261012-image-embeddings/gate.md),
[genre-ranking-offsets offsets.md](../_archived/20261014-genre-ranking-offsets/offsets.md),
[ball-game-ranking-features](../_archived/20261014-ball-game-ranking-features/plan.md))
compare frames only inside a burst. Under that metric every scene-level cue
(the genre, the ball-game composition, how a frame looks against the rest of
the day) cancels out, and a single shot never enters. The "no gain" verdicts
on genre offsets and on Candidates 1 / 2 may therefore be artifacts of the
metric rather than of the cues.

This plan re-measures everything on a **folder-wide** criterion: how well a
score orders a folder's picks above everything else in the folder, singles
included. It compares the shipped within-burst training with folder-wide
training, adds the genre, Candidate 1 and Candidate 2 cues to each, and
reports the contribution of every technical column (head pose yaw / pitch /
roll, sharpness, AF eye focus, eyes EAR, good photo), of the embedding PCA
block and of the burst-position columns, folder-wide and within-burst side
by side. It ends at a recorded decision: whether folder-wide training wins,
which cues add value folder-wide, and whether the genre feature is kept or
dropped. App changes, if any, come in a follow-up plan.

### What is known (investigated 2026-10-15)

- **A folder-wide gate already ran once.**
  [folder-wide-pick-model](../_archived/20261014-folder-wide-pick-model/plan.md)
  (`folder_gate.py`, [gate.md](../_archived/20261014-folder-wide-pick-model/gate.md))
  fit a per-frame logistic (44 shipped columns + `ln burst size` + absolute
  `ln sharpness`), leave one folder out, on the same dump. Held out, over
  the 25 qualifying folders: per-folder frame AUC 0.666 (full), 0.624
  (without embedding), 0.629 (embedding only), 0.598 (context only); the
  in-burst prior's **in-sample** score applied folder-wide 0.671; the full +
  genre-columns variant 0.667 (p = 0.245 against full); lift at the top
  tenth 2.08. It failed only on "burst AUC above burst size" (0.710 vs
  0.751), a cross-burst criterion this plan deliberately does not use
  (the user's goal is per-photo quality, not which burst holds a pick).
  **These numbers are known before this plan's pre-registration**; the
  pre-registration cannot be blind to them and says so. What is new here:
  the shipped model is scored folder-wide **held out** (not the in-sample
  prior), the Candidate 1 / 2 columns and the per-genre offsets (the
  shipped `OFFSET_SHRINK` 30 form) enter, every column group is ablated on
  both metrics, a precision-at-k metric and bootstrap intervals are added,
  and a folder-wide pairwise formulation is tried beside the pointwise one.
- **Data.** `D:\Photos\tests\2026-10-12-rank-app\`: 27 folders, 32,459
  frames (all with a vector), 4,419 picks by the gate label rule (`.dop`
  Pick, else a virtual copy in `Output/`, else XMP Pick without a `.dop`;
  the dump's own app `flag` misses the 427 `Output/`-only picks), 58
  explicit rejects, 6,969 bursts at gap 1000 ms, 1,943 singles (242 picks),
  1,801 scorable bursts; folder pick rates 10–22%. **No `views` rows**: the
  folders are culled, so "viewed but unflagged" is not distinguishable
  offline; every non-pick is a negative, as both earlier gates took it.
  Genre under the shipped rule: sports 4,462 bursts, snap 1,371, portrait
  299, landscape 26, birds / animals 7, `Unknown` 804.
- **Cue files, full coverage.** `D:\Photos\tests\2026-10-14-ball-labels\extras_c1.tsv`
  (4 columns: `ball visible`, `contested carrier`, and the two contrasts)
  and `extras_c2.tsv` (6 columns: `ball present`, `ball indicator`,
  `ln persons`, `contested`, `ball size`, `ln persons x ball present`) each
  hold one row for every one of the 32,459 frames of every folder
  (checked: per-folder counts equal the dump's). `detections.tsv` (403 MB,
  one row per NanoDet box) is the raw source if a column needs recomputing;
  `D:\Photos\tests\2026-10-14-detectors\` holds the models (untrusted
  downloads, not needed here).
- **Harnesses to import, not copy** (`sys.path.insert(0, <folder>)`,
  `sys.dont_write_bytecode = True`, as `rank_prior.py` does):
  - [gate.py](../_archived/20261012-image-embeddings/gate.py): `load`,
    `build` (`tech`, `pos`, `emb`, `burst_of`, `folder_of`, `bursts`,
    `pick`, `reject`), `scorable`, `design(d, mask, "shared")`, `pairs`,
    `fit`, `choose_c`, `lofo` (held-out within-burst-trained scores for
    every frame of a folder, uncentered `x @ w`), `burst_metrics`,
    `summarize`, `table`, `f3`; `TECH` (10 names), `POSITION`, `CS`.
  - [folder_gate.py](../_archived/20261014-folder-wide-pick-model/folder_gate.py):
    `extend` (burst size, order, absolute sharpness, shipped genre 0 / 1
    columns from `crates/core/genres/siglip2.json`, the per-folder
    class-balanced sample weights), pointwise `design` / `fit` / `choose_c`
    / `lofo`, `folder_metrics` (frame AUC, burst AUC, singles AUC, lift,
    pick vs reject), `signed_rank`, `paired`, `mean_sd`.
  - [ballgate.py](../_archived/20261014-ball-game-ranking-features/ballgate.py):
    `load_extras(paths, d)` (joins an extras TSV to `d["rows"]`, fails
    above 0.1% missing).
  - [genre_offsets.py](../_archived/20261014-genre-ranking-offsets/genre_offsets.py):
    `burst_genres` (the shipped `of_burst` rule), `hier_design` (the
    `[burst is g] * x / sqrt(OFFSET_SHRINK)` columns).
  - Python: the venv `%TEMP%\embedvenv` (numpy 2.5.4, scikit-learn 1.9.1,
    scipy 1.18.1, joblib); run `python -I` from outside the data
    directories. Timings on 20 jobs: `folder_gate.py` 141 s (6 pointwise
    variants + curve), `gate.py` about 3 min, `genre_offsets.py` 49 min
    (the hierarchical design multiplies the columns; one such variant here,
    not five).
- **The shipped score.** `rank::rank` centers each frame's score on its
  burst's mean, so the number the app shows is by construction
  uninformative across bursts; the uncentered `x @ w` of the same weights
  is what a folder-wide reading of the shipped model means. `rank.rs` is at
  `RANK_VERSION` 2 with per-genre offsets (`OFFSET_SHRINK` 30, all columns,
  `Unknown` without an offset).
- **Out of scope, owned elsewhere (do not touch):** `rank.rs`'s `pairs`,
  `Labeled`, `fit` and the `views` training data; the dlg-genre-weights,
  dlg-folder-best and dlg-likely-pick-icon sessions; the
  [similar-scene-groups](../20261015-similar-scene-groups/plan.md) plan
  (being stopped by its session; it reads the same dump read-only).
  This plan adds files only under its own folder and writes data only to
  `D:\Photos\tests\2026-10-15-folder-wide-eval\`.

## Steps

- [ ] Step 1: Pre-register the folder-wide metric, the variants, the ablations and the bars in `folderwide.md`, no script and no fit
  - Done when:
    - `folderwide.md` in this plan folder has a "Pre-registration" section
      merged **before any script exists or any fit runs** (the merge commit
      is the record, as for the earlier gates). Nothing in it changes
      afterwards; a deviation found later goes to `learnings.md` with its
      reason. The section states up front which numbers were already known
      from the archived folder-wide gate (listed above) and that the bars
      were set by the user before the run.
    - The section fixes, in this order:
      - **Data and labels**: the 27 folders of the app dump through
        `gate.load` / `gate.build`; picks by the gate label rule; **every
        non-pick is a negative** (no `views` offline; explicit rejects stay
        among the negatives and are also reported separately); bursts as
        `rank::bursts` (gap 1000 ms); a single is a burst of one.
      - **Unit and folds**: every frame is a sample; leave one folder out
        (27 folds); every number is held out. `C` per fold from
        `gate.CS` by the inner grouped-by-folder CV, maximizing the inner
        folders' mean primary metric for a folder-wide-trained variant and
        `gate.choose_c`'s weighted pairwise AUC for a within-burst-trained
        one. **Qualifying folders**: at least 50 frames and 5 picks (25 of
        27); means, intervals and tests are over them; every folder is
        trained on and listed.
      - **Primary metric (folder-wide)**, each per held-out folder, then
        the unweighted mean over the qualifying folders (one folder, one
        vote, as the earlier gates), with the frame-weighted pooled value
        reported beside it:
        1. **Frame AUC**: picks vs every other frame of the folder.
        2. **R-precision**: the pick share among the folder's top `k`
           frames by score, `k` = the folder's pick count (precision@k =
           recall@k there), ties at the cut broken by a seed-0 draw;
           reported as a percentage, chance = the folder's pick rate.
      - **Secondary metric (within-burst)**, from the **same held-out
        scores**: `gate.burst_metrics` weighted pairwise AUC and pooled
        Top-1 over the 1,801 scorable bursts (the archived gate's numbers;
        the shipped model must reproduce 0.710 / 54.9%).
      - **Reading aids** (reported, not deciding): singles AUC (folders
        with >= 5 single picks and 5 single non-picks); lift at the top
        tenth; burst AUC by max-per-burst against the burst-size baseline
        (continuity with the archived gate; size-confounded, so not a
        criterion); pick vs explicit reject AUC; the burst-centered shipped
        score (what the app displays today) scored folder-wide, expected
        near chance.
      - **Variants** (names fixed here; each one held-out score vector):
        - `W0` **shipped as shipped**: `gate.lofo(d, sc, "shared")`, the
          within-burst pairwise fit on the 44 columns, its uncentered score
          applied to every frame of the held-out folder (singles get a
          score too; they only did not train). **The reference.**
        - `W0+off`: `W0` with the shipped per-genre offsets
          (`hier_design`, `OFFSET_SHRINK` 30, all columns, burst genre by
          `burst_genres`); the model `rank.rs` ships today.
        - `F0` **folder-wide pointwise**: `folder_gate`'s weighted
          per-frame logistic on exactly the 44 shipped columns (no new
          columns), so `W0` vs `F0` is the training criterion alone.
        - `F0+ctx`: `F0` plus `ln burst size` and absolute `ln sharpness`
          (the archived full model; reproduces 0.666 as a check).
        - `P0` **folder-wide pairwise**: pick vs non-pick pairs over the
          whole folder (not only within bursts), each folder's pairs
          summing to weight 1, capped at **20,000 seeded pairs per folder**
          (seed = folder index), no intercept, 44 columns; the
          formulation closest to `rank.rs`'s objective if the app ever
          switched.
        - **Cue additions**, each on `F0+ctx` and, as the comparison the
          earlier plans made, on `W0`: `+genre cols` (the 9 shipped 0 / 1
          labels, `Unknown` all zero); `+genre off` (per-genre offsets at
          `OFFSET_SHRINK` 30; for the pointwise model the genre is the
          frame's burst's genre, so singles use their own frame's labels);
          `+c1` (4 columns of `extras_c1.tsv`); `+c2` (6 columns of
          `extras_c2.tsv`); `+c1+c2`. Extras standardized on the training
          folders as `ballgate.design` does.
        - **Baselines**: random (seed 0), capture order both ways, burst
          size (larger first), absolute sharpness, `eye_focus` (missing
          last), `photoTier` good.
      - **Ablations** (Step 3), on `F0+ctx` and on `W0`, each a full
        held-out rerun without the named columns, reported as the change in
        the primary and secondary metrics against the unablated model with
        its bootstrap interval:
        - drop each reason group of `rank.rs`'s `REASONS` (`Look` = the
          32 PCA columns; `Burst position`; `Sharpness`; `AF eye focus`;
          `Eyes open`; `Head pose`; `Good photo`), each missing indicator
          leaving with its group; drop `ln burst size` alone (`F0+ctx`
          only);
        - drop `|yaw|`, `|pitch|`, `|roll|` one at a time;
        - **each group alone** (a model of only that group's columns), so
          the raw folder-wide information of head pose, sharpness, eye
          focus, eyes open, good photo, position and the embedding is
          read without the others.
      - **Intervals and tests**: a 1,000-draw bootstrap over the
        qualifying folders (seed 0) of the paired per-folder difference
        (variant minus comparison) for frame AUC and R-precision, 2.5% /
        97.5% quantiles; the paired one-sided Wilcoxon signed-rank test
        across folders (`folder_gate.signed_rank`); for the within-burst
        secondary, `genre_offsets`'s per-burst bootstrap.
      - **Bars** (option C, chosen by the user on 2026-10-15 before any
        score; options A (effect size only) and B (significance only) are
        listed as rejected). A comparison **passes** when all hold:
        - mean frame AUC gain >= **+0.010**;
        - mean R-precision gain >= **+2.0 points**;
        - the variant at or above the comparison on frame AUC in
          **>= 16 of the 25 qualifying folders**;
        - the paired one-sided Wilcoxon **p < 0.05** on frame AUC **and**
          on R-precision.
        Applied as:
        - *Folder-wide training wins* when the best of `F0`, `F0+ctx`,
          `P0` passes against `W0` (the burst-size share of `F0+ctx` is
          noted from its ablation).
        - *A cue adds value folder-wide* when the cue variant passes
          against the base it was added to; the within-burst secondary is
          reported, not required.
        - *Genre keep / drop*: **keep** when `+genre cols` or `+genre off`
          passes on either base, **drop** (for the ranking) otherwise; what
          "drop" covers in the app is a separate user decision recorded in
          Decision 1 (see trade-offs).
        - *A column contributes* when its ablation lowers the primary
          metric with a bootstrap interval entirely below zero; reported
          per column, not acted on in this plan.
      - **What happens after**: no step of this plan changes the app. The
        Decision names the follow-up plan(s) that would.
    - `mise run lint` passes (lychee on the links).
  - Implementation approach:
    - Model the section on the archived
      [folder-wide gate.md](../_archived/20261014-folder-wide-pick-model/gate.md)
      and keep its vocabulary (qualifying folders, held out, per-folder SD).
    - Before writing, look only at file lists and headers (the extras
      coverage was already counted label-free); compute no label-dependent
      number.
    - Doc-only PR; no Python, no Rust.

- [ ] Step 2: `folderwide.py`: the variants, the baselines, the reference checks and the results tables
  - Done when:
    - `folderwide.py` in this plan folder (`python -I folderwide.py
      <app-dump-dir> <ball-labels-dir> <out-dir> [jobs]`) imports the four
      archived harnesses by path, implements every pre-registered variant,
      baseline, metric, interval and test, and prints Markdown tables; its
      stdout is saved as `folderwide.out.md` under
      `D:\Photos\tests\2026-10-15-folder-wide-eval\` together with one
      `scores.tsv` (`folder`, `file`, one column per variant's held-out
      score) so Step 3 and any later reading can recompute metrics without
      refitting.
    - **Reference checks** printed and recorded in `learnings.md`: `W0`
      reads 0.710 weighted AUC / 54.9% Top-1 within bursts; `F0+ctx` reads
      the archived 0.666 frame AUC (25 folders); `W0+off` reads 0.709 as
      `offsets.md`. A mismatch is investigated before anything else is
      read.
    - `folderwide.md` gains a "Results" section: data counts; the held-out
      table of every variant and baseline with frame AUC and R-precision
      (mean, per-folder SD, pooled), within-burst weighted AUC and Top-1,
      singles AUC, lift, burst AUC; the paired-difference table (each
      variant minus its comparison: `F0` / `F0+ctx` / `P0` minus `W0`,
      each cue variant minus its base) with the bootstrap interval, the
      Wilcoxon p and the count of folders above; the per-folder table
      (frames, picks, pick rate, qualifying, `W0`, best folder-wide, best
      cue variant); `C` per fold; the pick vs reject table; wall time and
      jobs. A "Reading" paragraph that states the facts without yet
      deciding (the Decision is Step 3's, after the ablations).
    - `learnings.md` records deviations from the pre-registered text and
      why, and the implementation choices the text left open.
  - Implementation approach:
    - Assumes Step 1 merged. Do not modify any archived script; extend by
      import. `folder_gate.extend` already builds the context, genre and
      weight arrays; `ballgate.load_extras` the cue blocks;
      `genre_offsets.burst_genres` / `hier_design` the offsets (for the
      pointwise model, pass the frame's burst genre as `genre_of_row`).
    - `W0` must be the exact archived path (`gate.lofo(d, sc, "shared",
      tiebreak, jobs)` returns the held-out uncentered score for every
      frame of the held-out folder, scorable bursts or not); the
      burst-centered reading aid subtracts the burst mean afterwards.
    - `P0`: build pairs per folder from `pick` / `~pick` indices with a
      seeded `rng.choice` cap, the flip trick of `gate.pairs`, weights
      summing to 1 per folder; `LogisticRegression(fit_intercept=False)`.
      Measure the memory of the design (pairs x columns x 8 bytes) and
      note it; lower the cap only if it does not fit and record it.
    - The inner CV for a pointwise variant maximizes the inner folders'
      mean frame AUC (as `folder_gate.choose_c`); for `P0`, the same.
    - Bootstrap over folders: draw 25 folder indices with replacement,
      mean of the paired per-folder differences; `np.random.default_rng(0)`.
    - Run the long part in the background and poll (one blocking call
      tripped the stream watchdog before). Expect a few minutes for the
      pointwise variants, more for `W0+off` / `+genre off` (one
      hierarchical design each; `genre_offsets.py` took 49 min for five
      plus a fallback check).
    - Strip CRLF before pasting into `folderwide.md` (`sed 's/\r$//'`).

- [ ] Step 3: `ablate.py`: the column and group ablations with intervals, and Decision 1 (training criterion, cues, genre keep / drop)
  - Done when:
    - `ablate.py` in this plan folder (`python -I ablate.py <app-dump-dir>
      <ball-labels-dir> <out-dir> [jobs]`, importing `folderwide.py`'s
      model builders) runs every pre-registered ablation on `F0+ctx` and
      on `W0`, and prints: the drop-one table (per group, per head-pose
      column, `ln burst size`) with the change in frame AUC, R-precision,
      within-burst weighted AUC and Top-1 against the unablated model,
      each with its bootstrap interval; the each-alone table (one model per
      group: head pose alone, sharpness alone, AF eye focus alone, eyes
      open alone, good photo alone, position alone, embedding alone, c1
      alone, c2 alone, genre labels alone) on both metrics; the fitted
      weights of `F0+ctx` and `W0` on all 27 folders grouped as
      `rank.rs`'s `REASONS`. Output saved as `ablate.out.md` next to
      `folderwide.out.md`; the tables pasted into `folderwide.md` under
      "Ablations".
    - **Decision 1** is written in this plan with the numbers that
      decided it, against the Step 1 bars:
      1. folder-wide training wins or not (which formulation, by how much,
         interval, p, folders above);
      2. per cue (genre columns, genre offsets, c1, c2, c1 + c2): adds
         value folder-wide or not, on each base, with the within-burst
         number beside it, and whether the earlier "no gain" verdicts
         stand or were artifacts of the metric;
      3. per technical column group and per head-pose axis, and for the
         PCA block and burst position: contributes / does not, folder-wide
         and within-burst, as a table;
      4. **genre: keep or drop** for the ranking, and the recommended
         scope of a drop (ranking offsets only vs the whole feature:
         `genre.rs`, `crates/core/genres/siglip2.json`, the `genres_of`
         command, `filter.ts`'s Genre section, `rank.rs`'s offsets and the
         meta pane suffix) left as the user's choice with the consequences
         written out;
      5. the follow-up: which app change, if any, a next plan should make
         (e.g. a folder-wide `fit` in `rank.rs` once its owner session is
         done; dropping genre; adopting a cue), or none.
    - `learnings.md` records the run times and any deviation.
    - `mise run lint` passes.
  - Implementation approach:
    - Assumes Step 2 merged; reuse its `scores.tsv` for the unablated
      reference so the comparison is the same held-out score.
    - Each ablation is a column mask over the Step 2 design; keep the
      same `C` grid and inner CV so an ablation is not penalized by a
      fixed `C`. About 12 drop-one + 10 each-alone models x 2 bases x 27
      folds: pointwise fits are cheap; the `W0` reruns cost about one
      `gate.py` each (3 min on 20 jobs), so budget an hour and run in the
      background.
    - "Head pose alone" includes the `no pose` indicator; "eyes open
      alone" the `no eyes judgment` indicator; "AF eye focus alone" the
      `no eye_focus` indicator, as `REASONS`' `MISSING` mapping does.
    - The Decision is written only after both tables exist; do not
      pre-write a direction in Step 2's "Reading".

## Trade-offs and risks

- **This is partly a re-run.** The archived folder-wide gate already showed
  the pointwise folder-wide model at frame AUC 0.666 against the in-sample
  prior's 0.671, and genre columns at +0.001. The likely outcome is that
  folder-wide training does not beat the shipped model on frame AUC either,
  and that the frame-level signal that exists is mostly within bursts
  (most of a folder's pick vs non-pick comparisons are inside bursts). What
  this plan adds that could change the picture: a held-out reference
  instead of the in-sample prior, R-precision (top of the folder, where
  the user looks), the cue columns, and the per-column ablations the user
  asked for. If the result repeats, the artifact is the ablation table and
  the keep / drop decision, which do not exist today.
- **Bar size: decided (2026-10-15), option C.** The user chose effect size
  and significance together (+0.010 frame AUC, +2.0 points R-precision,
  >= 16 of 25 folders, and Wilcoxon p < 0.05 on both). Rejected: A, effect
  size only (a consistent small gain below 0.010 is "no gain"); B,
  significance only (a gain of +0.003 can pass, which may not be worth an
  app change). For the column-contribution reading (ablations) the
  bootstrap interval entirely below zero applies regardless, since nothing
  is acted on.
- **Negatives.** Offline, every non-pick is a negative (no `views`). The
  app today trains on picks vs rejects vs viewed-unflagged; the dump's
  folders are fully culled so "unflagged" there means "seen and not
  chosen", the closest offline analogue. Explicit rejects (58) are too few
  to carry weight and are reported separately. Alternative not taken:
  weighting rejects by `REJECT_WEIGHT` as the app does; with 58 rejects it
  cannot move a number and would complicate the comparison with the
  archived gates.
- **Aggregation across folders.** Primary: one folder, one vote (mean over
  the 25 qualifying folders), as every earlier gate; a 4,830-frame folder
  and a 59-frame folder count the same. The frame-weighted pooled value is
  reported beside it.
- **`ln burst size` as a column.** A long burst almost always holds a pick,
  so a folder-wide model learns "long bursts first" (the archived gate's
  burst-size baseline reads 0.751 burst AUC). The user says bursts are not
  special, so `F0` (the 44 shipped columns, no burst size) is the clean
  training comparison and `F0+ctx` the stronger but burst-aware one; both
  are reported, and the drop-`ln burst size` ablation shows how much of
  `F0+ctx` is that one column. "Best folder-wide" is the best of `F0`,
  `F0+ctx`, `P0`, with the burst-size share noted.
- **Pointwise vs pairwise folder-wide.** Pointwise with per-folder
  class-balanced weights exists and is fast; folder-wide pairwise is the
  natural extension of `rank.rs`'s objective but needs a pair cap (20,000
  per folder; the largest folder would otherwise give 3M pairs). Both are
  variants; the plan does not choose up front.
- **Genre offsets in a pointwise model.** The shipped offsets are per
  burst; a single's "burst" is itself, so its genre is its own labels.
  That is the same rule `of_burst` applies to a one-frame burst; no new
  rule is introduced.
- **Genre keep / drop scope.** "Drop entirely" removes a user-facing
  filter section and the meta pane suffix as well as the ranking offsets
  (`genre.rs`, `genres/siglip2.json`, `genres_of`, `filter.ts` Genre,
  `rank.rs` `Weights.offsets` / `OFFSET_SHRINK`, `meta.ts` suffix, docs);
  the `rank.rs` part touches `fit`, which another session owns, so a drop
  there waits for that session. "Drop for the ranking only" keeps the
  filter. The evaluation can only speak to the ranking; the filter's
  worth is a product judgment. Decision 1 records the measurement and
  lays out both scopes; the user picks.
- **Multiplicity.** About 10 cue comparisons and 40 ablation cells are
  read; at p < 0.05 two or three false positives are expected. The bars
  are per comparison (an intersection where several must hold), as the
  earlier gates; the Decision notes the count of comparisons when a lone
  marginal pass appears.
- **Pre-registration is not blind to the archived numbers.** Stated in
  `folderwide.md`. The bars were set by the user, not tuned to those
  numbers; the new quantities (R-precision, held-out `W0`, c1 / c2
  folder-wide, ablations) have no prior number.
- **The data is one photographer, mostly basketball.** 1,439 of 1,801
  scorable bursts are sports; portrait and snap have 57 and 214. A genre
  effect may not be measurable here even if it exists; a "drop" decision
  is for this library and is recorded as such.
- **Run time.** The hierarchical (offset) designs are the slow part (49
  min for five in `genre_offsets.py`); this plan has two such variants in
  Step 2 and none in Step 3's ablations unless the Decision needs one. Run
  in the background; record wall time.
- **Fail path is a valid outcome.** If nothing beats `W0` and no cue
  clears its bar, the artifacts are `folderwide.md`, the two scripts, the
  ablation table and a recorded genre keep / drop; no code changes.

## Decision 1 (training criterion, cues, columns, genre keep / drop)

(Written at the end of Step 3 from the pre-registered bars in
`folderwide.md`.)

## Progress

- (2026-10-15) Plan written; bar option C chosen by the user
