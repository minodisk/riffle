# Learnings: bad-photo-labels

## Step 1: sample.py, the sample folder and labeling.md

- **Import.** `sample.py` puts `_archived/20261016-reject-filter-eval` on
  `sys.path` and imports `rejectfilter as rf`, which sets up the five
  archived folders and the `GENRES` override (see that plan's learnings,
  Step 2); `gate` and `folder_gate` are taken from `rf`. Run from the
  scratchpad as `python -I -B`; no `__pycache__/` appeared in the archived
  folders, this plan folder or the data folders.
- **Run.** `python -I -B sample.py D:\Photos\tests\2026-10-12-rank-app
  D:\Photos\tests\2026-10-15-folder-wide-eval
  D:\Photos\tests\2026-10-16-reject-filter-eval
  D:\Photos\tests\2026-10-17-bad-labels`, stdout saved as
  `sample.out.md` beside the manifest (stderr, the copy progress, as
  `sample.err`).
- **Final counts (option A, all allocations met).** 27 folders, 32,459
  frames (none dropped), 4,419 picks, 58 explicit rejects, 1,943 singles.
  Strata (population / drawn / inclusion probability): `closed` 2,385 / 120 /
  0.0503; `focus_low` 8,545 / 120 / 0.0140; `away` 5,076 / 120 / 0.0236;
  `sharp10` 1,422 / 120 / 0.0844; `g0_bottom` 1,441 / 150 / 0.1041;
  `w0_bottom` 945 / 70 / 0.0741; `rest` 12,645 / 500 / 0.0395 overall, per
  folder 8 (the floor, 12 folders) to 61 (`2026-09-27-a`); every frame has a
  positive inclusion probability. The flags before the priority order fire
  on 2,385 / 9,072 / 8,346 / 3,234 / 4,856 / 4,854 frames.
- **Coverage (dump share / sample share).** Singles 6.0% / 6.8%; picks
  13.6% / 13.9% (167 sampled); explicit rejects 0.2% / 0.5% (6 sampled, 5
  of them from `2026-09-19`); `photoTier` good 17.1% / 21.3%; genres
  `sports` 85.1% / 79.8%, `snap` 7.8% / 10.9%, `Unknown` 5.9% / 7.1%,
  `portrait` 1.0% / 2.0% (24), `landscape` 0.1% / 0.2% (3), `birds /
  animals` 0.0% / 0.0% (0 of 10). Every folder appears (8 to 163 frames).
  No class is under-covered against its dump share, so the allocation was
  not adjusted; `birds / animals` (10 frames in the dump) is simply too rare
  to read.
- **Checks, all passed.** Every copy's size and SHA-256 equal its source's
  (0 differ); no `.xmp` / `.dop` in `sample\`; 1,200 files = 1,200 manifest
  rows, none extra; the 27 library folders' recursive listings (path, size,
  mtime) were 58,227 files before and after with 0 changed. The output-path
  assertion refuses anything under `D:\Photos\2026`. A dry run (stopped
  before copying) twice and the real run wrote byte-identical
  `manifest.tsv`s.
- **The copy stalled once and was rerun.** The first run was killed by
  Claude Code's low-memory reaper of idle background shells after 482
  copies (the last progress line "copied 450 of 1200"; the 482nd file was
  partial). The rerun with the same arguments skipped 481 copies (same size
  and mtime, which `shutil.copy2` sets only after the data is written, so
  the partial one was recopied) and copied 719; the SHA-256 check over all
  1,200 then passed. The rerun's copy stage reads 18,556 s, inflated by the
  machine sleeping meanwhile (the first 950 copies took about 9 minutes);
  the SHA-256 pass of sources and copies took 1,906 s. The library listing
  comparison covers the rerun only; the killed run's listing was never
  compared, but it only read the library (`os.stat`, `shutil.copy2` from it)
  and the rerun's "before" listing equals its "after".
- **Choices the text left open:**
  - One generator `numpy.random.default_rng(0)`: its first `random(n)` is
    the archived seed-0 tie-break (so `rf.rules`' `sharpness bottom 10%`
    matches Decision 2's), then the stratum draws (`choice` without
    replacement, strata in priority order, `rest` folder by folder), then
    the shuffle (one uniform per sampled frame, argsort).
  - The `g0_bottom` / `w0_bottom` cuts remove `floor(15 * v / 100)` of the
    folder's `v` frames with a score, ties by the seed-0 draw;
    `2026-07-23`'s 18 frames have no `W0` and get no `w0_bottom` cut.
  - `rest` allocation: proportional to the folder's `rest` size, folders
    whose quota falls under 8 fixed at 8 and the rest re-spread, then the
    largest remainder (ties by folder order).
  - `genre`: among the top-level labels on for the frame in
    `folder_gate.extend`'s thresholded `genres` (a sub-genre bringing its
    parent, as the app does), the one with the highest top-level softmax
    from `gate.build`; `Unknown` when none is on.
  - A copy is skipped when its size and mtime equal the source's; the
    SHA-256 check then covers every copy, skipped or not.
- **Deviation: a second `x` does not clear a reject.** The plan's Step 1
  text said "`x` reject, press again to clear"; in the app a reject is
  sticky (`main.ts`: "Sticky, not a toggle"), cleared by `unflag` (`u`) or
  undo. `labeling.md` says so and the plan's text was corrected. It also
  notes Auto-advance (on, `x` moves to the next frame) and that `shift+x`
  rejects the other frames of the burst, which still forms among the copies
  since they keep their capture times.

## Step 2: the pre-registration in badfilter.md

- **Blindness.** Written from the plan, the archived `reject.md` /
  Decision 2 and Step 1's label-free outputs (`strata.tsv`,
  `sample.out.md`); the sample folder's sidecars were not opened (Step 1
  found none, and labeling has not started). The stratum sizes and `π` are
  quoted from `strata.tsv`; Decision 2's numbers from the archived
  `reject.md` "Results".
- **Choices the plan's text left open, fixed in the pre-registration:**
  - `G0`'s `quantile@95` share: `rejectfilter` computes it only for the 25
    qualifying folders; the 2 non-qualifying folders (`2026-07-21`,
    `2026-07-23`) get the share computed the same way from all 25.
  - The cue bins (EAR <= 0.137 / (0.137, 0.25] / > 0.25 / missing;
    `eye_focus` < 0.5 / [0.5, 0.9) / >= 0.9 / missing; `|yaw|` 35,
    `|pitch|` 45; sharpness per-folder deciles on the full dump; `tier`) and
    each cue's firing bin (the one its rule removes; `eye_focus`'s two lower
    bins pooled).
  - `B0` / `L0`'s within-sample cut is weighted by `1/π` (the longest prefix
    of the lowest-scored frames whose `1/π` sum is <= `q`% of the folder's,
    the crossing frame kept), and the best existing score is also
    read on that within-sample cut for the paired dedicated-model
    comparison, so the difference compares like with like.
  - The bootstrap resamples `rest` within each (`rest`, folder) cell,
    keeping each cell's drawn count (`plan.md` only says "within each
    stratum"), so each draw's total `1/π` stays at the design's.
  - `q` = 5 was added to the plan's cut grid (5 / 10 / 20 / 30 / 40): with
    `q` = 10 as the smallest cut, `NL` <= 5% is infeasible at a low or
    moderate base rate.
  - `L0`: columns standardized on the training frames, class balance by
    rescaling each class's weights to equal totals within the fold.
  - The bootstrap holds the cuts and scores fixed (no refit of `B0` / `L0`
    per draw); "interval above X" means the 2.5% quantile above X.
  - The weighted AUC / partial AUC are pooled over the sampled frames, with
    missing values at the folder's lowest finite score minus 1, as the
    archived `pAUC95`.
  - A copy whose sidecar holds a Pick or a rating counts by its reject state
    alone.
- **Links.** Only `../_archived/20261016-reject-filter-eval/` targets are
  linked; this plan's own files (`labeling.md`, `strata.tsv`) are named in
  backticks.

## Step 3: the user's labels and labels.py

- **The user's remarks.** The user said "done" with no further notes: no
  frame reported as failing to open, no doubt about the definition reported,
  no time taken reported. `skip.txt` was therefore not written; no copy left
  the sample.
- **Run.** `python -I -B labels.py D:\Photos\tests\2026-10-17-bad-labels`
  from the scratchpad, stdout saved as `labels.out.md` beside
  `labels.tsv`; seconds. No `__pycache__/` appeared.
- **Counts.** 1,200 manifest rows, 1,200 labeled, 0 copies missing, no
  other file in `sample\` beyond the 1,200 copies and their sidecars.
  **80 bad / 1,120 not bad** (6.7% unweighted). Per stratum (bad / drawn):
  `closed` 11 / 120, `focus_low` 9 / 120, `away` 3 / 120, `sharp10` 26 / 120,
  `g0_bottom` 19 / 150, `w0_bottom` 2 / 70, `rest` 10 / 500. The weighted
  base rate is Step 4's.
- **Sidecars.** 81 copies carry both a `.dop` and a `<stem>.xmp` (the
  user's `sidecarFormat` is `both`), 1,119 none; no copy has only one.
  80 hold a Reject in both (`ShouldProcess = 1`, `xmpDM:good="False"`); one,
  `0467_2026-09-27-a__DSC8339.ARW`, holds `ShouldProcess = 2` and no
  `xmpDM:good` (rejected, then unflagged), so it is `not_bad`. The `.dop`
  and the XMP agree on every copy. No Pick, no rating (`Rating = 0` /
  `xmp:Rating="0"` everywhere) and no color label in any sidecar.
- **Index cross-check (read only, outside `labels.py`).** A scratchpad query
  of `index.sqlite` opened `mode=ro` (the `ratings` and `files` rows of the
  sample dir, which the index keys with the `\\?\` prefix; `views` not read)
  found 1,200 `files` rows and 81 `ratings` rows: 80 `flag` 2 (reject), 1
  `flag` 0 (`0467`), none dirty, no rating or label, agreeing with the
  sidecar labels on all 1,200 copies.
- **Dump-label consistency.** 167 sampled dump picks, 0 labeled bad; 6
  sampled explicit dump rejects, 4 labeled bad.
- **Deviation: `labels.tsv` keeps only the label-free manifest columns.**
  The plan says "plus the manifest's columns"; to keep Step 4 blind,
  `labels.py` reads and carries only `source`, `folder`, `file`, `draw`,
  `stratum`, `inclusion_prob`, `pick`, `reject`, `single`, `burst_size`,
  `capture_ms` (the design and the dump labels), not the cue values, the
  rule flags, `genre` or the stored scores. Step 4 joins those from
  `manifest.tsv` by `name`.
- **Choices the text left open:** `source_sidecar` is the deciding sidecar
  (`dop`, `xmp`, or `-` for none); an XMP is looked for as `<name>.xmp` and
  `<stem>.xmp`, the first found used; a legacy `xmp:Rating="-1"` would count
  as a Reject; a skipped copy (from `skip.txt`) leaves `labels.tsv` and is
  listed with its reason.
- **`aiEnabled` is on now.** `settings.json` reads `aiEnabled: true` after
  labeling; `labeling.md` asked for it off. Whether it was off during the
  labeling is not known (not asked). With no picks in the sample folder the
  ranking forms no pairs from it either way.

## Step 4: badfilter.py, the measurement

- **Run.** `python -I -B badfilter.py D:\Photos\tests\2026-10-12-rank-app
  D:\Photos\tests\2026-10-15-folder-wide-eval
  D:\Photos\tests\2026-10-16-reject-filter-eval
  D:\Photos\tests\2026-10-17-bad-labels` from the scratchpad; 94 s wall
  (55 s inside the script: load 4 s, cuts / metrics / both bootstraps 51 s;
  the rest is the imports). Stdout saved by the script as
  `badfilter.out.md` in the data folder. No `__pycache__/` appeared.
- **Import.** `rejectfilter` by path as in Step 1, plus this plan folder on
  `sys.path` to import `sample` for `ALLOCATION`, `BOTTOM_SHARE` and
  `score_bottom` (so the strata are recomputed by Step 1's own code); `-I`
  drops the script's directory from `sys.path`, hence the explicit insert.
- **Reference checks, all as expected.** The 33 `strata.tsv` rows
  (population, drawn, `π`) recomputed from the dump: 0 differ; the sampled
  frames' recomputed stratum / `π` against `labels.tsv`: 0 / 0 differ;
  `manifest.tsv`'s `G0` / `W0` / `sharpness` against the dump's: 0 / 0 / 0
  differ; the sum of `1/π` over the 1,200 sampled frames is 32,459.0;
  random (seed 0) reads weighted `BR` 4.7 / 13.1 / 24.0 / 32.0 / 45.6% at
  q = 5 / 10 / 20 / 30 / 40 and AUC 0.542 / partial AUC 0.512 (intervals
  include 0.5); `G0`'s `quantile@95` cut reproduces `reject.md`'s mean
  realized pick recall / non-pick removal over the 25 qualifying folders,
  94.7 / 16.8 (and `rf.label_free`'s own 94.7 / 16.8), removing 15.2% of
  the dump's frames (4,943). `reject.md` reports that cut's recall and
  removal pair, not a frame share; the pair is what was matched, the share
  is recorded.
- **Headline.** Weighted base rate 5.52% [4.15, 7.04]; no score, baseline
  or rule passes the worth-a-filter bar at any cut (the highest `P` is
  27.3%); the best existing filter at `NL` <= 5% is `R0` q5 (`BR` 19.3%,
  `NL` 3.7%, `P` 23.1%); only the sharpness decile 1 catches bad photos by
  the cue rule (3.20x). See `badfilter.md` "Results".
- **Deviations.** None from the pre-registered metrics, cuts, intervals or
  bars.
- **Choices the text left open:**
  - The cue values, rule flags and stored scores are taken from the dump
    arrays (indexed by (`folder`, `file`) of `labels.tsv`), not from
    `manifest.tsv`; the manifest is joined by `name` only as the check above
    (they agree), since the cuts need the full dump's vectors anyway.
  - "`default_rng(0)` afresh per quantity" is implemented as one count
    matrix (1,000 draws x 1,200 frames) drawn once from `default_rng(0)`
    and reused for every quantity, which is what drawing afresh with the
    same seed gives; the folder-cluster bootstrap likewise (folder counts
    from `default_rng(0)`, 1,000 x 27).
  - Primary cut: the score's cut with the weighted `NL` nearest 5%, ties to
    the smaller `q`. A rule and `G0` `quantile@95` have one cut, which is
    their primary.
  - Paired differences: a graded score and a graded comparison at the
    score's primary cut (both at the same `q`); a rule or `G0`
    `quantile@95` against the comparison at the comparison's primary cut.
    Frames outside either score's coverage (`W0`'s 8 sampled frames of
    `2026-07-23`) are left out of both.
  - A folder is outside a graded score's coverage when any of its frames
    has a NaN (only the `W0` family on `2026-07-23`); `-inf` (missing
    sharpness / `eye_focus`, "missing last") is ranked, not excluded.
  - The AUC / partial AUC are also given for the binary rules and `G0`
    `quantile@95`, on the keep / remove split (as the archived frame AUC
    is), with their bootstrap intervals.
  - Sharpness deciles: per folder over its frames with a sharpness, ties by
    the seed-0 draw, decile `floor(10 * rank / n) + 1` for deciles 2-10, while decile 1 is
    exactly the `sharpness bottom 10%` rule's set (`floor(10 * n_folder /
    100)` frames, so the firing bin and the rule's `P` agree); no sampled
    frame lacks sharpness.
  - `|yaw|` > 35 and `|pitch|` > 45 are read as separate firing bins (the
    `face away` rule fires on either).
  - A "fragile" mark on the folder-cluster table would flag a pass or a
    catching cue whose cluster 2.5% quantile falls below its bar; none
    occurred (sharpness decile 1's cluster lower bound is 10.5%).
- **A stray tool call.** A `python -` heredoc (meant to patch
  `badfilter.md`) hung on the Git Bash `python` stub and was moved to the
  background; it never ran, and the edit was made by hand instead.

## Deferred issues (todo candidates)

- **Pending manual check: the sample opens in Riffle.** What: open
  `D:\Photos\tests\2026-10-17-bad-labels\sample` once in Riffle (Windows,
  the user's machine) and confirm the strip shows 1,200 frames, all
  unflagged, sorted by file name from `0001_...`. Steps: start Riffle, open
  the folder, sort by File name, clear filters, let the scan finish.
  Expected: 1,200 frames, no pick / reject / rating shown. Not done by the
  implementation agent (the user or the main agent does it, per the plan);
  Step 1's checkbox was ticked on the automated criteria. Basis: Step 1's
  "Checks" in `docs/plans/20261017-bad-photo-labels/plan.md`.
  - Step 3 update: the user did not separately confirm this check before
    labeling (only "done" was reported). What the data shows instead: the
    index holds 1,200 `files` rows for the sample folder (all copies opened
    and scanned), and after labeling the sidecars and the index hold no
    Pick, no rating and no color label on any copy, only 80 Rejects and one
    unflagged `.dop` (see Step 3). Step 1 found no sidecar in `sample\`
    before labeling, so every copy started unflagged. The strip's visual
    "1,200 unflagged frames" reading itself remains unconfirmed.
