# Learnings: discard-labels

## Step 1: sample.py, the sample folder and labeling.md

- **Import.** `sample.py` puts `_archived/20261017-bad-photo-labels` on
  `sys.path` and imports `badfilter as bf`, which puts
  `_archived/20261016-reject-filter-eval` and its own folder on `sys.path`
  and imports `rejectfilter` (the five archived harness folders and the
  `GENRES` override) and then the archived `sample`, the order the archived
  learnings record. `rf`, `gate`, `folder_gate` and the archived `sample`
  (as `bs`) are taken from `bf`. Run from the scratchpad as `python -I -B`;
  no `__pycache__/` appeared in the archived folders or this plan folder.
- **Module name clash.** This plan's script is also named `sample.py`, and
  `badfilter` does `import sample` for the archived one. Run as a script it
  is `__main__`, so there is no clash; a later step that imports this
  `sample.py` as a module must load it under another name
  (`importlib.util.spec_from_file_location("discard_sample", ...)`, as the
  scratchpad dry run did), or `badfilter`'s `import sample` gets this one.
- **Run.** `python -I -B sample.py D:\Photos\tests\2026-10-12-rank-app
  D:\Photos\tests\2026-10-15-folder-wide-eval
  D:\Photos\tests\2026-10-16-reject-filter-eval
  D:\Photos\tests\2026-10-17-bad-labels
  D:\Photos\tests\2026-10-18-discard-labels`, stdout saved as
  `sample.out.md` beside the manifest, stderr (copy and hash progress) as
  `sample.err`. Run in the background and completed (exit 0): 600 copied
  in 319 s, the SHA-256 pass 114 s, 441 s in total. A rerun with the same
  arguments skipped all 600 copies (0 copied), passed every check again
  and wrote a byte-identical `manifest.tsv`; two dry runs (stopped before
  copying, into the scratchpad) also wrote identical `manifest.tsv` /
  `strata.tsv`.
- **Data.** 27 folders, 32,459 frames (none dropped), 4,419 picks, 58
  explicit rejects, 1,943 singles; 1,200 strict-sample frames excluded,
  population 31,259; `G0` on every frame.
- **Bands (dump / removed by the exclusion / population / drawn / `π`).**
  `b05` 1,609 / 71 / 1,538 / 150 / 0.0975; `b10` 1,625 / 77 / 1,548 / 150 /
  0.0969; `b20` 3,248 / 146 / 3,102 / 150 / 0.0484; `control` 25,977 / 906 /
  25,071 / 150 / 0.00598 overall, per folder 0.0042 (`2026-07-30`) to 0.43
  (`2026-07-23`, 3 of 7). Every population frame has a positive `π`, every
  strict-sample frame `π` = 0, and none of them is in the sample.
- **Pick share (label-free, population).** `b05` 29 picks of 1,538 (1.89%
  of the band, 0.68% of all picks); `b10` 57 / 1,548 (3.68%, 1.34%); `b20`
  128 / 3,102 (4.13%, 3.01%); `control` 4,038 / 25,071 (16.11%, 94.97%).
  As cuts on the full dump: `G0` q5 removes 4.96% of the frames with pick
  loss 0.68%, q10 9.96% / 2.10%, q20 19.97% / 5.18% (population: 4.92% /
  0.68%, 9.87% / 2.02%, 19.80% / 5.03%). So q20 sits right at the `PL` <= 5%
  bar on the label-free side.
- **Coverage (population share / sample share).** Singles 6.0% / 5.7%;
  picks 13.6% / 6.5% (39 sampled, most from the controls, as the bands
  hold few picks); explicit rejects 0.2% / 0.8% (5); `photoTier` good
  16.9% / 9.3%; genres `sports` 85.3% / 83.2%, `snap` 7.7% / 8.2%,
  `Unknown` 5.9% / 6.5%, `portrait` 1.0% / 2.2% (13), `landscape` 0.1% /
  0 and `birds / animals` 0.0% / 0 (20 and 10 frames in the population, too
  rare to draw). Every folder appears, 3 to 96 frames.
- **Disk.** 27.2 GB of RAW copies (26 GiB on disk); 455 GB free on D:
  before the copy.
- **Checks, all passed.** Every copy's size and SHA-256 equal its source's
  (0 differ); no `.xmp` / `.dop` in `sample\`; 600 files = 600 manifest
  rows, none extra; the 27 library folders' recursive listings (path,
  size, mtime) were 58,227 files before and after with 0 changed (both
  runs). The output-path check refuses anything under `D:\Photos\2026`.
- **Choices the text left open:**
  - One generator `numpy.random.default_rng(0)`: its first `random(n)` is
    the seed-0 tie-break for `badfilter.graded_cut` and `rf.rules`, then
    the band draws (`choice` without replacement, `b05`, `b10`, `b20`, then
    `control` folder by folder), then the shuffle (one uniform per sampled
    frame, argsort).
  - The bands come from `badfilter.graded_cut` on `G0` over the full dump
    (q5 / q10 / q20, nested since they share one order); the exclusion is
    applied afterwards, so the bands are exactly the cuts Step 4 reads.
  - `control` floor 3 (the archived `rest` floor 8 does not fit 150 frames
    over 27 folders): folders whose quota falls under 3 fixed at 3 (or the
    whole cell), the rest proportional, largest remainder. 13 folders sit
    at the floor.
  - `g0_rank_share` is the frame's 0-based per-folder rank by `G0` (the
    `rf.bottom` order) over the folder's frame count. With `floor(q * n /
    100)` the band edges sit slightly below `q`% in small folders, so a
    `b10` frame can read a share under 0.05 (`2026-07-21`, 32 frames: q5
    removes 1, q10 3). Sampled shares: `b05` 0–0.049, `b10` 0.031–0.099,
    `b20` 0.056–0.199, `control` 0.201–0.991.
  - `strict_label` is `-` on every row; the rule flag columns are
    `rf.rules`' `closed`, `focus_low`, `away`, `sharp10`, as before.
  - A copy is skipped when its size and mtime equal the source's; the
    SHA-256 check then covers every copy, skipped or not.
- **labeling.md.** Keys checked in the code: `rate1` = `1` and `clear` =
  `0` (`crates/app/src/shortcuts.rs`); `rate1` is in `advance.ts`'s
  advancing set, `clear` is not; a rating is set, not toggled, so pressing
  `1` again keeps it.

## Step 2: the pre-registration in discard.md

- **No label existed.** Before writing, `sample\` was listed by file name
  only (600 files, 0 `.xmp` / `.dop`); no sidecar was opened.
- **Two bars are decided label-free.** `G0` q20's pick loss is 5.18% on
  the full dump and 5.03% on the population, above the 5% bar on either
  definition, so q20 fails. And `G0` q5 removes 4.96% of the dump's frames
  (4.92% of the population's), under the ">= 5% of frames" condition,
  because `floor(q * n / 100)` rounds each folder down, so as first
  written q5 failed it too.
- **User decision (2026-10-18, before any labeling).** On that finding the
  user lowered the removed-share condition from >= 5% to **>= 4.5%** of the
  population's frames, to allow for the rounding. Now q5 (4.92%, `PL`
  0.68%) and q10 (9.87%, `PL` 2.10%) pass both label-free conditions, q20
  fails `PL`, and `quantile@95` (15.2%) clears the removed share with its
  pooled `PL` left to Step 4. Applied to `discard.md` (Bars, the label-free
  verdicts, the rejected "delete" option's text) and `plan.md` (Choices,
  Step 2). No bar moves after this.
- **Choices the text left open:**
  - `PL` for the bar is read on the **full dump** (32,459 frames, 4,419
    picks), the population value beside it: the cut is defined on the full
    dump, a shipped cut acts on every frame of a folder, the strict-sample
    exclusion is a property of the labeling sample, and the full-dump value
    is the higher (conservative) one at every `G0` cut.
  - The removed-share condition keeps the plan's wording, the
    **population's** frames, the full dump's beside it; both give the same
    verdict at every `G0` cut.
  - `quantile@95` has no `q`, so its "beats random" condition compares it
    with random at q20, the nearest grid cut at or above its 15.2% removed
    share.
  - The hypothesis `RR@5` <= 5% is called confirmed on the weighted point
    estimate, its interval beside it; it is not a bar.

## Step 3: the labels in labels.tsv

- **User remarks (2026-10-18).** The user said "done" after viewing the
  sample, with no remarks: no frame reported as failing to open, no doubt
  about the definition, no `skip.txt`. Asked afterwards (2026-10-19), the
  user reported: time taken about 30 minutes (the estimate was 15–25, so
  about 3 s per frame); `aiEnabled` stayed off throughout.
- **Run.** `labels.py D:\Photos\tests\2026-10-18-discard-labels`, stdout
  saved as `labels.out.md` beside `labels.tsv`. 600 manifest rows, 600
  labeled, 0 skipped, 0 copies missing, 0 other files in `sample\`.
- **Counts.** `keep` 211, `discard` 389. Per band (label counts only, no
  score read): `b05` 26 keep / 124 discard, `b10` 45 / 105, `b20` 48 /
  102, `control` 92 / 58 (150 each).
- **Sidecars.** 212 copies carry a `.dop` + `.xmp` pair (both Riffle's,
  the setting writes both), 388 none; the `.dop` decides on all 212; 0
  `.dop` / XMP disagreements on rating 1. One copy holds rating 0 in both
  sidecars (`0381_2026-07-21__DSC4605.ARW`: marked, then cleared with
  `0`), listed as unexpected and labeled `discard` by the rating-1 rule.
  No Pick, Reject or color label in any sidecar (a `.dop`
  `ShouldProcess = 2` is the unflagged value).
- **Dump-label consistency.** Sampled dump picks 39: 27 keep, 12
  discard. Sampled explicit rejects 5: 0 keep, 5 discard.
- **Choice the text left open.** `labels.py` reports the rating of every
  sidecar not equal to 1, including 0, as unexpected (the brief said "a
  rating other than 1"); a cleared rating is therefore visible in the
  list rather than silently absent.

## Step 4: discardfilter.py, the measurement

- **Import.** `discardfilter.py` imports only the archived `badfilter` (as
  `bf`, which brings `rejectfilter` and the archived `sample`), as Step 1's
  `sample.py` does; it does not import this plan's `sample.py`, so the
  module-name clash never arises. The bands are recomputed directly with
  `bf.graded_cut` on `G0` (q5 / q10 / q20, seed-0 `tb`). Run from the
  scratchpad as `python -I -B`; no `__pycache__/` appeared.
- **Run.** Arguments as in the plan, stdout saved as
  `discardfilter.out.md` in `D:\Photos\tests\2026-10-18-discard-labels\`.
  18 s in total (load 5 s, metrics and bootstraps 12 s).
- **Reference checks, all passed.** `strata.tsv`: 0 of 30 rows differ
  (population, drawn, `π`) when recomputed from the dump; 0 sampled frames
  whose recomputed band or `π` differ from `labels.tsv`; 0 sampled frames
  in the strict sample; sum of `1/π` 31,259.0 = the population; random
  `DR` at q5 / q10 / q20 2.6 / 10.6 / 21.0 (q5 low but its interval [0.8,
  5.4] covers 5) and AUC 0.528; `G0` `quantile@95` mean pick recall /
  non-pick removal 94.7 / 16.8 (and `rf.label_free` 94.7 / 16.8); strict
  base rate recomputed from the strict sample 5.52% [4.15, 7.04].
- **Headline.** Discardable base rate 48.45% [42.03, 55.11]. No `G0` cut
  is usable: q5 `RR` 17.3% [11.3, 24.0] (fails the 15% point bar, passes
  the quantile) and its `DP` lead over random +30.7 has a 2.5% quantile of
  -3.8; q10 / q20 / `quantile@95` `RR` 23.7 / 27.9 / 26.0. Hypothesis
  `RR@5` <= 5% not confirmed. The second branch's conditions of the
  follow-up rule hold (`RR@5` 17.3 <= 30, `DO@5` 46.7 >= 20).
- **Choices the text left open:**
  - The stratified bootstrap reuses `bf.strat_counts` by setting
    `bf.STRATA` to `b05`, `b10`, `b20`, `rest` for the call (restored
    after) and naming `control` `rest` in the sample's stratum column,
    since `bf.cells_of` reads the module global and the literal `rest` for
    the per-folder cells. The strict sample's interval uses
    `bf.strat_counts` with the archived strata unchanged.
  - The metrics reuse `bf.ratio`, `bf.quant`, `bf.aucs` (with
    `s["bad"]` = `discard`), `bf.cluster_counts`, `bf.graded_cut`,
    `bf.g0_q95_cut` and `bf.finite_by_folder`; `DP` / `DR` / `DO` are
    computed in `metrics` rather than through `bf.rates`, whose third
    column has no `DO`.
  - `PL` and the removed share of a score are read on the folders the
    score covers (`W0` leaves `2026-07-23` out), as `badfilter` did; for
    every `G0` cut that is the whole dump.
  - "The weighted share of sampled picks labeled `discard`" is reported
    once overall (24.4%, 12 of 39) and per cut as a count (sampled picks
    removed / of them `discard`), not as a per-cut weighted share (2–16
    sampled picks per cut).
  - The "beats random" condition uses the stratified paired difference on
    the frames both cuts cover; the folder-cluster quantile is printed as
    sensitivity ("fragile" would apply only to a pass). For
    `quantile@95`, random at q20, as Step 2 fixed.
  - The AUC is reported without the partial AUC (`bf.aucs` computes both;
    the pre-registration names only the AUC).
- **No deviation** from the pre-registration; no bar moved.

## Deferred issues (todo candidates)

- **Narrowed manual check: the strip showed no judgment before labeling.**
  Outcome from Step 3: the user viewed all 600 frames and reported none
  failing to open. Before labeling, `sample\` had 0 sidecars (Step 2), and
  `labels.py` found no Pick, Reject or color label afterwards. What was
  never confirmed: that the strip showed no judgment (rating, pick, reject,
  label) before labeling began (the user reported `aiEnabled` off
  throughout). Basis: Step 1's checks in
  `docs/plans/20261018-discard-labels/plan.md`; review Round 1, item 2, of
  Step 3.
