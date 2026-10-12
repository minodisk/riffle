# Step 7: the ranking on the app's stored data

Step 2's evaluation (`gate.py`, unchanged, criterion as pre-registered in
[gate.md](gate.md)) re-run on what the app itself stores for the same 27
sidecar-labeled ARW folders, the prior the app ships fit on that data, and
the check that `crates/core/src/rank.rs` reproduces it.

## How the app dump was made

The user's own index (`%LOCALAPPDATA%\com.minodisk.riffle\index.sqlite`)
held no embedding and only 4 of the 27 folders (AI analysis was never on
there), so the dump went through the app's code path into a fresh index:

1. The ignored test `rank_dump_of_real_folders` (`crates/app/src/commands.rs`)
   lists each folder as `scan_folder` does, reconciles the index and the
   sidecars (`SidecarFormat::Both`), and runs the app's three passes
   (`run_scan`, `run_faces_pass`, `run_embed_pass` with the SigLIP 2 file of
   Decision 1) into `D:\Photos\tests\2026-10-12-rank-app\index.sqlite`. It
   then writes what `Index::rank_rows` reads, the function the ranking
   commands call: `app/<folder>.tsv` (flag, capture time, the pass-2 values)
   and `embed.tsv`. 32,459 files, 0 errors in any pass, 3889 s on 22 threads
   (release build, Windows 11, i7-13700).
2. `rank_dump.py` turns that into a gate directory (`features/<folder>.tsv`
   in `riffle-cli features`' columns): the analysis columns from the app, the
   sidecar flags and the `Output/` listings from Step 2's dump, so the labels
   are gate.md's.
3. `python -I gate.py D:\Photos\tests\2026-10-12-rank-app 12`, then
   `python -I rank_prior.py D:\Photos\tests\2026-10-12-rank-app crates/core/ranking/siglip2.json D:\Photos\tests\2026-10-12-rank-app\prior_scores.tsv`.

```sh
RIFFLE_RANK_DIRS='D:/photos/2026/2026-06-05;...;D:/photos/2026/2026-10-03' \
RIFFLE_RANK_OUT=D:/Photos/tests/2026-10-12-rank-app \
RIFFLE_EMBED_MODEL=D:/Photos/tests/2026-10-12-embed-models/siglip2-b32-256/vision_model.onnx \
cargo test --release -p riffle-app rank_dump_of_real_folders -- --ignored --nocapture
python -I rank_dump.py <embed-gate> <rank-app>
python -I gate.py <rank-app> 12
python -I rank_prior.py <rank-app> crates/core/ranking/siglip2.json <rank-app>/prior_scores.tsv
python -I rank_compare.py <embed-gate> <rank-app>
RIFFLE_RANK_OUT=<rank-app> cargo test --release -p riffle-app the_scores_match_rank_prior_py -- --ignored --nocapture
```

## The app's inputs against Step 2's

The plan expected the app's eye and pose values to differ from Step 2's,
since `riffle-cli features` took them from the eyes path's judged face and
the app stores pass 2's cue face. Since `FACES_VERSION` 9 the two are the
same face: `rank_compare.py` finds every value equal at the precision the
`features` TSV prints, and the embeddings identical.

| Column | Equal | Differ | In one dump only | In neither |
| --- | ---: | ---: | ---: | ---: |
| sharpness | 32459 | 0 | 0 | 0 |
| eye_focus | 28437 | 0 | 0 | 4022 |
| ear | 21563 | 0 | 0 | 10896 |
| eyes_open | 21560 | 3 | 0 | 10896 |
| yaw / pitch / roll | 21440 | 0 | 0 | 11019 |
| eye_offset / edge_gap | 21563 | 0 | 0 | 10896 |
| state | 32459 | 0 | 0 | 0 |

The 3 `eyes_open` differences are at the fifth decimal (an open probability
computed from the stored EAR against one rounded to 4 decimals). Embeddings:
32,459 files in both, cosine min 1.000000000. What is left between the two
runs is the TSV rounding (4 decimals, 1 for the pose) that Step 2's inputs
went through and the app's do not.

The app's own flag (`ratings.flag`, from the `.dop` and the XMP, the newest
of the two) agrees with the flag gate.md's labels read on every frame: 3992
picks, 58 rejects, 28,409 neither. gate.md's 4419 picks add the 427 frames
picked only through a virtual copy in `Output/`, which the app does not see.

## Results: both numbers

| Ranking (held out, leave one folder out) | gate.md (Step 2 dump) | App dump | |
| --- | ---: | ---: | --- |
| Shared model, weighted pairwise AUC | 0.708 | **0.710** | bar >= 0.703: met |
| Shared model, per-folder SD of the weighted AUC | 0.060 | 0.054 | |
| Shared model, unweighted pairwise AUC | 0.763 | 0.763 | |
| Shared model, Top-1 | 54.4% | **54.9%** | |
| First frame, Top-1 (per-folder SD) | 39.4% (8.7%) | 39.4% (8.7%) | |
| Top-1 gain over the first frame | +15.0 | **+15.4** | bar > 8.7: met |
| Shared model, Top-3 | 87.7% | 87.7% | |
| Per-genre variant, weighted AUC | 0.707 | 0.707 | not kept (Decision 2) |
| Without embedding, weighted AUC | 0.652 | 0.652 | |
| Embedding only, weighted AUC | 0.668 | 0.668 | |
| First frame, weighted AUC | 0.650 | 0.650 | |
| Learning curve, weighted AUC at 5 / 10 / 20 / 26 folders | 0.687 / 0.699 / 0.704 / 0.708 | 0.688 / 0.700 / 0.705 / 0.710 | |

**The shared model clears the pre-registered bar on the app's data too**:
weighted pairwise AUC 0.710 >= 0.703, Top-1 gain 15.4 points > 8.7. The
margin on the AUC bar is still small (0.007, under one per-folder SD of
0.054), so the feature ships as a modest, ignorable suggestion, as Decision
2 said. The same 1801 scorable bursts and 41,942 pairs; `C` per fold
0.01 / 0.1 / 1 / 10 on 6 / 7 / 5 / 8 folds. The full tables are in
`D:\Photos\tests\2026-10-12-rank-app\gate.out.md`; per folder, the shared
model reads below the first frame in `2026-07-24`, `2026-07-31`,
`2026-08-16` and `2026-10-03`, as in gate.md.

## The shipped prior

`rank_prior.py` fit the shared model on all 27 folders of the app dump as
gate.py's last table does: `C` = 0.1 by the grouped inner cross-validation,
1801 scorable bursts.

| Column | Weight (standardized) |
| --- | ---: |
| ln rel sharpness | +0.231 |
| `eye_focus` logit | +0.170 |
| eyes open | +0.211 |
| \|yaw\| | -0.024 |
| \|pitch\| | +0.028 |
| \|roll\| | +0.026 |
| `photoTier` good | +0.125 |
| no `eye_focus` | -0.048 |
| no eyes judgment | +0.203 |
| no pose | -0.171 |
| position | -0.373 |
| first frame | -0.156 |
| PCA dims (L2 norm of the 32) | 1.225 |

`crates/core/ranking/siglip2.json` (318 KB) holds these weights, the
imputation means, the PCA mean and its 32 whitened components, the
standardization and `C`.

## rank.rs against rank_prior.py

`the_scores_match_rank_prior_py` (`crates/app/src/rank.rs`) reads every
folder of the dump index through `Index::rank_rows`, forms the bursts and
columns with `rank::bursts` / `rank::rows` and scores them with the prior's
weights: all 32,459 scores match `prior_scores.tsv` within 2.9e-6. The
Rust burst grouping, the technical columns (the `photoTier` rule included),
the imputation, the PCA projection and the standardization are therefore
the ones the prior was fit on.

The same test fits `rank::fit` on the 27 folders as if all were marked
finished, with the app's own flags: 1602 bursts give pairs (fewer than 1801:
the 427 `Output/`-only picks are not picks to the app) from 25 folders, and
the weights move 0.300 (L2) from the prior's, whose norm is 1.372. That is
the shrinkage at work: the prior is this user's own taste, so the fit lands
near it.

## Deviations from the plan's text

- **No per-genre clause.** Decision 2 dropped the per-genre offsets, so the
  reason display never names a genre.
- **The local fit keeps the prior's PCA basis and standardization** and
  refits only the 44 weights, with an L2 pull toward the prior's weights at
  the prior's own strength (`1 / (2 C)` per burst). Refitting the basis on
  the user's folders would change the coordinates the prior's weights live
  in, and the shrinkage toward them would lose its meaning; the basis was
  fit on 32,459 frames and does not need the user's few folders.
- **Finished folders outlive the 30-day eviction.** Their index rows are
  the fit's training data, so `evict`'s age rule skips a folder marked
  culling finished (the user's decision after Step 7); the 1 GB size cap and
  Clear Cache still drop it.
- **Explicit rejects count double** (`REJECT_WEIGHT` = 2) within their burst.
  The 58 rejects in 3 scorable bursts cannot measure the weight; 2 is a
  choice, not a fit.
