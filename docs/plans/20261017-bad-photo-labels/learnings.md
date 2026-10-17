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
  - `B0` / `L0`'s within-sample cut is weighted by `1/π` (the removed `1/π`
    sum reaching `q`% of the folder's), and the best existing score is also
    read on that within-sample cut for the paired dedicated-model
    comparison, so the difference compares like with like.
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
