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
