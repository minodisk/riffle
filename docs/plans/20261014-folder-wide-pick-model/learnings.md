# Learnings

## Step 1: pre-registration

- The plan text said the genre columns come "from `genres.tsv` thresholds of
  `crates/core/genres/siglip2.json`". The two do not fit together:
  `genres.tsv` holds SigLIP sigmoid scores against the in-burst gate's
  provisional prompt vectors (`genre_text.tsv`), while the shipped
  `siglip2.json` thresholds apply to the cosine of the embedding with that
  file's own label vectors (`genre.rs`: `dot >= threshold`, a sub-genre
  turning its parent on). `gate.md` pre-registers the shipped rule computed
  from `embed.tsv` and `siglip2.json` (9 labels, `Unknown` = all zeros),
  which is what "the shipped genre labels" means.
- Check 1's candidate set: the approval decision says "the best of all
  non-embedding baselines and variants", while the plan's criterion text
  listed a subset. `gate.md` takes every baseline and variant that uses no
  embedding (adding `photoTier` good and random to the plan's list); the
  in-burst prior's score is excluded because that prior uses the embedding.
- "Context only" was written as (`ln burst size`, position); `gate.md` reads
  position as `gate.py`'s `POSITION` pair (position and first frame), so the
  variant is all three context columns.
- Edge rules the plan left open and `gate.md` now fixes: a folder with no
  pick gives its whole weight to its non-picks (`2026-07-23` has 0 picks);
  the inner CV maximizes the mean per-folder frame AUC with ties to the
  smaller `C`; burst AUC skips a folder whose bursts are all of one label;
  singles AUC needs 5 single picks and 5 single non-picks per folder; lift
  ties at the cut break by seed 0.
- Before writing, only the dump's file list, column headers and the range
  of the sharpness score (all 32,459 positive, so `ln sharpness` needs no
  floor) were looked at; no label-dependent number was computed.
- Weight groups: `gate.md` follows `rank.rs`'s `REASONS` grouping with two
  extensions. `Burst position` is renamed `Burst context` (plan.md Step 3
  uses that name) and also takes `ln burst size`; `Sharpness` also takes the
  new absolute `ln sharpness` column, which `rank.rs` does not have.

## Step 2: folder_gate.py and Decision 1

- **Result: the gate fails on check 2**, so Decision 1 drops Steps 3–5.
  Frame AUC 0.666 vs 0.624 (without embedding, p = 0.0002, met); burst AUC
  0.710 vs burst size 0.751 (p = 0.9998, not met); mean lift 2.08 (met).
  The genre variant was not adopted (p = 0.245).
- **Python environment.** The system Python 3.13 has neither numpy nor
  scipy; the in-burst gate's venv at `%TEMP%\embedvenv` has numpy 2.5.4,
  scikit-learn 1.9.1, joblib 1.6.0 and scipy 1.18.1, so the run used
  `scipy.stats.wilcoxon` as pre-registered. The script keeps the exact
  signed-rank fallback (mid-ranks for ties) for an environment without
  scipy; on tie-free samples it gave the same p-values as scipy's exact
  path, with ties scipy's `method="auto"` switches to the normal
  approximation, so the two can differ slightly there.
- **Wall time** 120 s on 20 jobs (six variants x 27 folds x 21 fits, the
  learning curve's 243 folds of the full model, and the all-folder fit).
- **Deviation: output file name.** The plan says to save the output as
  `gate.out.md` next to the dump, but `D:\Photos\tests\2026-10-12-rank-app\`
  already holds the in-burst gate's `gate.out.md`; overwriting it would lose
  that record, so this gate's output is `folder_gate.out.md` (stderr in
  `folder_gate.err`, empty) there. The pre-registered text does not name the
  file, so this does not touch it.
- **Implementation choices the pre-registered text leaves open** (none
  changes the criterion):
  - As in `gate.py`, the inner CV for `C` reuses the outer fold's design
    (imputation, PCA, standardization fit on all training folders), not a
    design refit per inner split.
  - The model's score is the logit (`x @ w + b`); every metric is
    rank-based, so it reads the same as the probability.
  - The random baseline and the lift's tie-break use the same seed-0
    uniform draw.
  - Capture order is the frame's index within its folder in
    `gate.order_key` order (what `gate.build` already sorts by), so a frame
    without a capture time sorts last rather than being dropped.
  - Pick vs reject counts a reject that is also a pick (a virtual copy) as a
    pick only; there are none in the data.
  - The genre columns are computed in float64 from the L2-normalized
    vectors (`genre.rs` uses float32), which can flip a label only at a
    threshold's exact edge.
- **The in-burst prior's score is not near chance folder-wide** (frame AUC
  0.671, slightly above the full model's 0.666; burst AUC 0.697). The plan
  expected chance because the score is centered per burst; but most of a
  folder's pick vs not-chosen comparisons are inside bursts, where that
  prior was trained. It is not a check-1 candidate (it uses the embedding)
  and does not change the decision.
- **The burst AUC rewards size by construction.** Even the random score
  reads 0.651 under "the unit's score is its highest frame score", because a
  long burst takes the maximum of more draws. Check 2 against burst size was
  the guard for that, and it is the one that failed.
- **CRLF from Windows stdout.** The redirected output has CRLF line ends;
  pasting it into `gate.md` needed `sed 's/\r$//'`. A `printf` with
  `D:\Photos\tests\...` in single quotes turned `\t` and `\2026` into a tab
  and an octal byte, leaving invalid UTF-8; the line was rewritten with a
  small Python script.
