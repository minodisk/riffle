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

# Discard labels: a blind human-rescue review of `G0`'s discard candidates, measured against pre-registered bars

## Purpose

What the user wants from the AI feature is that it **discards** photos so the
human picks from what remains. The archived
[bad-photo-labels](../_archived/20261017-bad-photo-labels/plan.md) plan
measured every score against a **strict** label ("you would never keep this
frame on its own; when unsure, do not reject") and found a 5.5% base rate
and no usable filter (its Decision: best `R0` / `G0` at q5, `P` 23–27%
against a 50% bar). Afterwards the 49 frames of that 1,200-frame sample that
`G0`'s per-folder bottom-5% cut (q5, full dump) removes but the user had left
unflagged were given a Red label through the MCP companion and reviewed; the
user said all but one (`0935_2026-07-31__DSC6292.ARW`) were fine to discard.
Under the question **"OK to discard?"** `G0` q5 thus read about 70 of 71 in
that sample. That review was **not blind**: the user knew the frames were the
model's discard candidates, so the number is a hypothesis, not a measurement.

This plan measures that hypothesis blind, under a **human-rescue** labeling
scheme the user approved: the model proposes discards, the human marks only
the frames that should **not** be discarded, and, so that misses are
measurable and the candidate status does not anchor the judgment, blind
random non-candidates are shuffled in with the candidates and nothing marks
which is which. It ends at a Decision against bars set before any label is
read: is a discard cut usable, at which depth, at what cost in rescued and
lost frames, how does the earlier "non-pick ≈ OK to discard" framing hold up,
and which follow-up, if any, deserves its own plan (an in-app discard
candidates view with a rescue loop, a retrain on rescue labels, or nothing).
**No app code changes in this plan**; if a follow-up needs one, the Decision
names a separate later plan.

### Choices (decided by the user on 2026-10-18)

- **Discard semantics: "hide / sort last"** (rejected: "delete"). Bars:
  rescue rate `RR` <= **15%** with its stratified 97.5% quantile <= **25%**,
  pick loss `PL` <= **5%**, removed share >= **4.5%** of the population's
  frames (the rejected "delete" option was `RR` <= 5% with the quantile <=
  15% and `PL` <= 1%). The removed-share floor was lowered from 5% to 4.5%
  by the user on 2026-10-18, before any label, to allow for the per-folder
  `floor(q * n / 100)` rounding that puts `G0` q5 at 4.92% of the
  population's frames.
- **Cuts (bands): `G0` per-folder 5 / 10 / 20** (bands 0–5, 5–10, 10–20,
  controls above 20) (rejected: 10 / 20 / 30).
- **Sample size: 600** frames, 150 per band for the three candidate bands and
  150 controls, about 15–25 minutes of labeling and about 27 GB (rejected:
  1,000).
- **Already-seen frames: excluded.** The population is the dump minus the
  1,200 frames of the strict sample (31,259 frames); the strict labels
  compare only by band rate (rejected: include them).
- **Marking key: rating 1** (`1`; auto-advances when Auto-advance is on; `0`
  clears) (rejected: the Green label `ctrl+alt+4`).

### What is known (investigated 2026-10-18)

- **Data and scores.** The dump of 27 folders and 32,459 frames
  (`D:\Photos\tests\2026-10-12-rank-app\`, `features/<folder>.tsv`,
  `embed.tsv`, `genre_text.tsv`), the held-out scores `W0`, `W0+off`, `F0`,
  `F0+ctx`, `P0` (`D:\Photos\tests\2026-10-15-folder-wide-eval\scores.tsv`,
  `W0` missing on `2026-07-23`) and `R0`, `G0`, `rule count`
  (`D:\Photos\tests\2026-10-16-reject-filter-eval\scores_reject.tsv`, `G0`
  on every frame). Library path of a dump frame:
  `D:\photos\2026\<folder>\<file>`, one ARW about 45 MB; D: has 424 GB free.
  The library is read only.
- **The strict labels.** `D:\Photos\tests\2026-10-17-bad-labels\labels.tsv`
  (1,200 frames, 80 strict-bad, `manifest.tsv` with each frame's stratum,
  `π`, cue values and stored scores). Its sample folder now also carries the
  49 Red labels of the unblinded review (130 sidecar pairs in `sample\`).
  Known from that plan's results, listed here so the pre-registration can
  quote them as prior knowledge: strict base rate 5.52% [4.15, 7.04]; `G0`
  q5 on the strict labels `BR` 18.9% / `NL` 2.9% / `P` 27.3%; the
  `g0_bottom` stratum (bottom 15% by `G0`) strict-bad rate 12.7%; `G0`
  `quantile@95` (label-free) keeps 94.7% of the picks, removes 16.8% of the
  non-picks and 15.2% of the dump's frames
  ([reject.md](../_archived/20261016-reject-filter-eval/reject.md),
  Decision 2); the strict-bad rate among the dump's non-picks is 6.41%, so
  the non-pick criterion overstated **strict** bad about fifteenfold.
- **What enters the personal ranking's refit (read in code).**
  `Index::rank_rows` (`crates/app/src/index.rs`) reads per file only
  `ratings.flag` and whether a `views` row exists; `rank::fit`
  (`crates/core/src/rank.rs`) pairs each **Pick** against the **Rejects**
  and the **viewed unflagged** frames of the **same burst**;
  `Index::rank_dirs` lists every folder with a view or a reject, so the
  sample folder enters the refit once opened, and `refit_in_background`
  runs at startup and on each folder switch while `aiEnabled` is on. Copies
  keep their capture times, so bursts form among them. **Therefore a Pick in
  the sample folder would form pairs and move `ranking.json`; a color label
  or a star rating never enters the fit** (`RankRow` has no label and no
  rating). Keys (`crates/app/src/shortcuts.rs`): `rate1` = `1`
  (auto-advances when Auto-advance is on; `0` clears). A rating is written
  to the copy's sidecar as `xmp:Rating` / `.dop` `Rating`; the archived
  `labels.py` already parses it.
- **Harnesses, imported by path, none modified**: the archived
  `rejectfilter.py` (`bottom`, `rules`, `load_scores`, `load_ear`; it sets
  up `gate`, `folder_gate` and the `GENRES` override), the archived
  `sample.py` of bad-photo-labels (`under`, `allocate`, `listing`, `sha256`,
  `cell`, `score_bottom`) and its `badfilter.py` (`graded_cut`,
  `g0_q95_cut`, `rates`, `aucs`, the stratified and folder-cluster
  bootstrap count matrices). Python: the venv `%TEMP%\embedvenv` (numpy,
  scikit-learn), run as `python -I -B` from the scratchpad with the plan
  folder put on `sys.path` explicitly (`-I` drops the script's directory);
  check `git status` for stray `__pycache__/`. Use the venv's `python.exe`
  by full path; the Git Bash `python` / `python3` stubs hang on heredocs.
- **Where the data goes**: `D:\Photos\tests\2026-10-18-discard-labels\`,
  the copies in its `sample\` subfolder (the folder the user opens), the
  manifests, labels and outputs beside it. Names `<nnnn>_<folder>_<file>`,
  shuffled, as before.
- **Out of scope, owned elsewhere (do not touch):** `rank.rs`'s `pairs`,
  `Labeled`, `fit`; the `views` table and its data; the dlg-genre-weights,
  dlg-folder-best and dlg-likely-pick-icon sessions. No app code changes;
  the library `D:\Photos\2026` is read only. This plan adds files only
  under its own folder and writes data only to
  `D:\Photos\tests\2026-10-18-discard-labels\`.

## Steps

- [x] Step 1: `sample.py`, the blind review sample of `G0` candidates and controls with its manifest, and the labeling guide `labeling.md`
  - Done when:
    - `sample.py` in this plan folder (`python -I -B sample.py
      <app-dump-dir> <folder-wide-eval-dir> <reject-filter-eval-dir>
      <strict-labels-dir> <out-dir> [n]`) loads the dump through the
      archived harnesses, joins the stored scores, computes **`G0`'s
      per-folder bottom-`q` sets on the full dump** for `q` = 5 / 10 / 20
      (ties by the seed-0 draw, `floor(q * n / 100)` frames, exactly as
      `badfilter.graded_cut` does, so the sample's bands are the cuts Step 4
      reads), assigns every frame of the population to exactly one
      **band**: `b05` (bottom 5%), `b10` (5–10%), `b20` (10–20%), and
      `control` for every frame outside the bottom 20%; draws the sample
      with `numpy.random.default_rng(0)`; writes `manifest.tsv` and
      `strata.tsv`; and copies the RAW files (no sidecar, `shutil.copy2`)
      into `<out-dir>\sample\`. It refuses to write under `D:\Photos\2026`,
      is reproducible, and skips existing identical copies.
    - **Population**: the dump minus the 1,200 frames of
      `<strict-labels-dir>\manifest.tsv`. The script prints how many frames
      of each band the exclusion removes.
    - **Allocation**: 600 frames, 150 per candidate band as simple random
      samples within the band; 150 `control` drawn per folder proportional
      to the folder's control size with a floor (the archived `rest`
      scheme, `allocate`), so every folder appears. The inclusion
      probability `π` is `n_drawn / N` of the frame's band, or of its
      (`control`, folder) cell, positive for every frame of the population.
    - **Order and names**: shuffled (one uniform per sampled frame, seed 0)
      and named `<nnnn>_<folder>_<file>`; nothing in the name, the order or
      the folder says which band a frame is in.
    - `manifest.tsv` columns as the archived one (`name`, `source`,
      `folder`, `file`, `draw`, `band`, `inclusion_prob`, `pick`, `reject`,
      `single`, `burst_size`, `capture_ms`, `genre`, the raw cue values, the
      rule flags, the stored scores) plus `g0_rank_share` (the frame's
      per-folder rank by `G0` as a share, for the band check) and
      `strict_label` (`-` everywhere, since the strict sample is excluded;
      kept for a later plan that includes it). `strata.tsv`: per band (per
      folder for `control`) the population, the drawn count and `π`.
    - Printed and recorded in `learnings.md`: the band sizes and `π`; the
      coverage tables (picks, explicit rejects, singles, `photoTier` good,
      genres, per folder); the disk use; the checks (every copy's size and
      SHA-256 equal its source's; no `.xmp` / `.dop` in `sample\`; copies =
      manifest rows; the library folders' listings unchanged). The
      label-free **pick share of each band** (dump picks inside each cut) is
      printed too, since it needs no label.
    - `labeling.md` in this plan folder (English; the main agent relays it
      in Japanese) states:
      - the purpose: the question is **"OK to discard?"**, not "bad" and not
        "would you pick it";
      - the definition, **mark a frame when it should not be discarded**:
        you might pick it, you would want to see it before deciding, or you
        would mind if Riffle had hidden it from what you pick from; leave it
        unmarked when you would not miss it (a blurred or closed-eyes frame,
        a dead frame of a burst you would never choose, a duplicate you
        would not look at twice); **when unsure, mark it (keep)**;
      - that the folder mixes model candidates with random frames and
        nothing shows which is which, so each frame is judged on its own;
      - the setup: open `D:\Photos\tests\2026-10-18-discard-labels\sample`,
        `aiEnabled` off so the recommended icon and the ranking rows are
        absent, file-name sort, no filter, scan finished;
      - the marking key: **rating 1** (`1`), which auto-advances with
        Auto-advance on, and `0` clears it;
      - the keys **not** to use: `p` Pick (it would train the personal
        ranking); `x` / `shift+x` (the strict label of the last round, not
        this question); other star ratings;
      - the rule to look at the picture, not the `f` mark, the strip icons
        or the meta pane;
      - the pace and the time estimate (600 frames, about 15–25 minutes);
      - the stopping rule: every frame viewed once; an unmarked frame counts
        as OK to discard, so skip none;
      - what to say when done, including roughly how long it took.
    - `mise run lint` passes.
  - Implementation approach:
    - Import the archived `sample` and `badfilter` of bad-photo-labels and
      `rejectfilter` by path (the archived folders on `sys.path`, in the
      import order the archived learnings record); reuse their functions,
      write nothing into the archive.
    - Print tables and checks before copying; copy last. Run the copy in
      the foreground or in chunks: the archived learnings record a
      background copy killed after 482 files.
    - Script and docs only; no Rust, no app change.

- [x] Step 2: Pre-register the labels, cuts, metrics, comparisons and bars in `discard.md`, before any label exists
  - Done when:
    - `discard.md` in this plan folder has a "Pre-registration" section
      merged **before the user has marked anything and before any label is
      read** (the merge commit is the record). It states what is already
      known (the strict-label results above, Decision 2's numbers, the
      unblinded 70/71 reading, the band sizes and pick shares of Step 1,
      which are label-free), that no discard label existed when it was
      written, and that the bars were chosen by the user before labeling.
    - It fixes, in this order:
      - **Labels and population.** `keep` = the copy's sidecar carries
        rating 1; `discard` = reviewed and unmarked (the stopping rule makes
        "unmarked" a label; "when unsure, keep" makes the rescue rate the
        conservative side). A Pick, a Reject, a color label or another
        rating found in a sidecar is listed as unexpected and counts by the
        rating-1 mark alone. The population is the dump minus the strict
        sample; every estimate is **Horvitz–Thompson weighted** (`1/π`) with
        the unweighted value beside it. The dump's `pick` / `reject` are
        consistency checks and the pick-loss metric's input, never a target.
      - **Cuts.** `G0`'s per-folder bottom `q` on the full dump for `q` = 5
        / 10 / 20 (the bands' boundaries), plus `G0`'s label-free
        `quantile@95`; the same `q` grid on the comparison scores `W0`,
        `R0`, absolute sharpness, `eye_focus` (missing last) and random
        (seed 0); a frame outside the deepest band (a `control`) can still
        be removed by a comparison score's cut, which is why the controls
        are labeled too.
      - **Primary metrics**, per cut, weighted, with the unweighted value
        and the sampled counts beside them:
        1. **Discard precision `DP@q`**: the share of removed frames
           labeled `discard`.
        2. **Rescue rate `RR@q`** = 1 − `DP@q`: the share of removed frames
           the user would keep.
        3. **Pick loss `PL@q`**: the share of the dump's picks inside the
           cut (label-free, on the full dump), beside the weighted share of
           sampled picks labeled `discard` (a consistency reading: a pick
           should rarely be OK to discard).
        4. **Discard recall `DR@q`** and **miss rate `MR@q`** = 1 − `DR@q`:
           the share of `discard` frames the cut removes / leaves.
        5. **Discardable share outside the cut `DO@q`**: the `discard` rate
           among the kept frames (what the human still wades through).
        Plus the weighted AUC of `discard` vs `keep` per score, and the
        **discardable base rate** of the population as the first number.
      - **Comparisons**: paired differences (same draws) of `DP` and `DR`
        between `G0` and each comparison score at the same `q`, and `G0`
        against random.
      - **The non-pick framing re-read**: the weighted `discard` rate among
        the dump's non-picks and among its picks; the share of removed
        non-picks (Decision 2's `RN`) that are `discard`; and, per band,
        the `discard` rate beside the band's pick share, so the answer to
        "is non-pick ≈ OK to discard?" is a number with an interval.
      - **The secondary label set**: the strict labels of bad-photo-labels
        (`labels.tsv`), read as the strict-bad rate per `G0` band from that
        sample's own `π`. The ordering expected if the labels are coherent
        (a band's strict-bad rate below its `discard` rate) is stated as a
        check, not a bar.
      - **Intervals**: the 1,000-draw stratified bootstrap within each band
        and each (`control`, folder) cell, seed 0, one count matrix reused
        across quantities so paired differences share draws; a
        folder-cluster bootstrap as the sensitivity reading.
      - **Bars** ("hide / sort last" semantics, written out with the
        rejected "delete" option): a cut of `G0` is a **usable discard
        cut** when its weighted `RR` <= 15% with the stratified 97.5%
        quantile <= 25%, `PL` <= 5%, it removes >= 4.5% of the population's
        frames (lowered from 5% before any label, see "Choices"), and its `DP` exceeds random's at the same `q` with the
        paired 2.5% quantile above zero. The **best cut** is the deepest
        passing one. The **follow-up rule**: a usable cut exists → the
        Decision recommends a later plan for an in-app discard candidates
        view with a rescue loop at that depth; no usable cut but `RR` at
        the shallowest cut is within 2x the `RR` bar and `DO` at that cut
        is >= 20% (the model misses much that is discardable: something to
        learn) → the Decision recommends a later plan that trains on the
        rescue labels with the controls as negatives (never on candidates
        alone); otherwise → nothing. The non-blind 70/71 reading is
        pre-registered as the hypothesis `RR@5` <= 5%, to be stated as
        confirmed or not.
      - **Multiplicity** (about 6 scores, 4 cuts and a few readings) and
        the caveat that the population is one photographer's 27 folders.
    - `mise run lint` passes.
  - Implementation approach:
    - Assumes Step 1 merged (band sizes and pick shares quoted). Model the
      section on the archived
      [badfilter.md](../_archived/20261017-bad-photo-labels/badfilter.md)
      pre-registration and keep its vocabulary. Doc-only PR.

- [x] Step 3: The user labels the sample folder (manual, the plan pauses here), then `labels.py` exports `labels.tsv`
  - Done when:
    - **The user has said "done"** after viewing every frame of
      `D:\Photos\tests\2026-10-18-discard-labels\sample` in Riffle per
      `labeling.md`. Nothing of this step starts before that; the main
      agent waits and reads no sidecar meanwhile.
    - `labels.py` in this plan folder (`python -I -B labels.py <out-dir>`)
      reads the sidecars beside the copies (`xmp:Rating` and `.dop`
      `Rating`; the `.dop` decides when both exist), joins them to
      `manifest.tsv` and writes `labels.tsv` (`name`, `label` = `keep` /
      `discard`, `source_sidecar`, and only the design and dump-label
      columns: `source`, `folder`, `file`, `draw`, `band`,
      `inclusion_prob`, `pick`, `reject`, `single`, `burst_size`,
      `capture_ms`, `strict_label`), and prints: the counts per label and
      per band, the sidecar formats found, any sidecar holding a Pick, a
      Reject, a color label or a rating other than 1 (listed, not dropped),
      and the dump-label consistency counts (sampled picks labeled
      `discard`, sampled explicit rejects labeled `keep`). **No score is
      read against the labels in this step.** An optional `skip.txt` takes
      out copies the user could not judge, with the reason.
    - `learnings.md` records the user's remarks (time taken, frames that
      would not open, doubts about the definition, whether `aiEnabled` was
      off) and the counts.
    - `mise run lint` passes.
  - Implementation approach:
    - Assumes Steps 1 and 2 merged. Standard library only; copy the regex
      approach of the archived `labels.py`; do not import `riffle-core`.
    - Ask the user explicitly for the time taken (the last round did not
      record it) so the next workload estimate is grounded.

- [x] Step 4: `discardfilter.py`: every pre-registered cut, metric, comparison and reading against the labels, reweighted, with intervals
  - Done when:
    - `discardfilter.py` in this plan folder (`python -I -B
      discardfilter.py <app-dump-dir> <folder-wide-eval-dir>
      <reject-filter-eval-dir> <strict-labels-dir> <out-dir>`) imports the
      harnesses by path, loads the dump, the stored scores, `labels.tsv`
      and the strict `labels.tsv` / `manifest.tsv`, computes everything in
      the Pre-registration, prints Markdown tables and saves stdout as
      `discardfilter.out.md` in `<out-dir>`.
    - **Reference checks** recorded in `learnings.md`: the bands and `π`
      recomputed from the dump match `strata.tsv`; the sum of `1/π` equals
      the population size; every sampled frame's band agrees with the
      recomputed `G0` cut; random reads `DR@q` near `q` and AUC near 0.5;
      `G0` `quantile@95` reproduces 94.7 / 16.8; the strict base rate
      recomputed from the strict sample reads 5.52%.
    - `discard.md` gains a "Results" section: the discardable base rate
      with its interval; the per-band table (`discard` rate, pick share,
      strict-bad rate from the secondary set); the main table (`G0` and
      each comparison score at each cut: `DP`, `RR`, `PL`, `DR`, `MR`,
      `DO`, removed share, with intervals); the `quantile@95` row; the
      paired differences; the non-pick framing table; the secondary-label
      table; the folder-cluster sensitivity table; the bars applied per
      cut; the hypothesis `RR@5` <= 5% stated confirmed or not; wall time.
      A "Reading" paragraph stating the facts without deciding.
    - `learnings.md` records deviations and the choices the text left open.
  - Implementation approach:
    - Assumes Step 3 merged. Reuse `badfilter.graded_cut`, `g0_q95_cut`,
      `rates`, `aucs` and the bootstrap matrices by import; metric-only
      work, seconds to run.

- [x] Step 5: The Decision
  - Done when:
    - A "Decision" section in this plan, written against the Step 2 bars
      with the numbers that decided it: (1) the discardable base rate and
      the non-pick framing verdict; (2) whether a usable discard cut exists,
      the best cut, what it rescues (`RR`), loses (`PL`) and misses (`MR`),
      and whether the unblinded 70/71 held up blind; (3) how `G0` compares
      with the other scores and random at that depth; (4) how the strict
      labels relate to the discard labels; (5) the follow-up by the
      pre-registered rule: a later plan for an in-app discard candidates
      view with a human-rescue loop (with the depth, the semantics and the
      app surface it would need to specify), a later plan to train on the
      rescue labels with the controls as negatives, or nothing; stated as a
      recommendation, with the conditions under which it would change. No
      app change in this plan.
    - `learnings.md` records the labeling workload actually spent against
      the estimate, for the next round's sizing; `mise run lint` passes.
  - Implementation approach:
    - Assumes Step 4 merged. Doc-only PR; write it only after the Results
      tables exist; do not move a bar.

## Trade-offs and risks

- **The five choices** are recorded under Purpose, "Choices", with the
  rejected options.
- **The 70/71 reading is the hypothesis, not evidence.** It was made on
  frames the user knew were candidates, after a strict pass over the same
  frames; the plan states it as prior knowledge and tests `RR@5` <= 5%
  blind. If the blind `RR` is far higher, the gap is itself a finding about
  automation bias in the rescue scheme.
- **"Unmarked = OK to discard" rests on the stopping rule.** A skipped frame
  is a false `discard`, which flatters precision; "when unsure, keep" and
  the explicit "done" are the mitigations, the `views` table is not read.
- **Training only on candidates reinforces the model's bias.** The controls
  give labels off the candidate set, so a later retrain can use negatives
  the model did not choose; the retrain itself stays out of this plan.
- **Population.** One photographer, 27 folders, mostly basketball; the
  Decision speaks of this dump, minus the 1,200 frames already labeled.
- **Fail path is a valid outcome.** If no cut passes, the artifacts are the
  blind rescue labels, the sampler, the bars and the per-band readings,
  reusable by any later plan.

## Decision (usable discard cut, the non-pick framing, follow-up)

Written 2026-10-19 at Step 5, after the Step 4 Results existed, against the
bars pre-registered in Step 2 ("hide / sort last": weighted `RR` <= 15%
with its stratified 97.5% quantile <= 25%, `PL` <= 5% on the full dump,
removed share >= 4.5% of the population's frames, `DP` above random's at the
same `q` with the paired 2.5% quantile above zero). Every number is weighted
to the population of 31,259 frames (`1/π`) with the stratified bootstrap 95%
interval, and comes from [discard.md](discard.md) "Results". No bar was
changed after the labels were read.

**1. Base rate: 48.45% of the population's frames are OK to discard**
[42.03, 55.11] (folder-cluster [40.74, 53.87]; 389 of 600 sampled frames,
64.83% unweighted, the sample leaning toward the bands). That is almost nine
times the 5.52% strict-bad rate: about half of the library is frames the
user would not miss, so the low-base-rate obstacle of the strict round does
not apply here. **The non-pick framing does not hold**: 51.7% [44.9, 59.0]
of the dump's non-picks are `discard`, so about half of the non-picks are
frames the user would keep (or want to see), and 24.4% [8.6, 42.8] of the
sampled picks (12 of 39) are `discard`. "Non-pick" overstates "OK to
discard" about twofold (where it overstated strict bad about fifteenfold),
and it misses in the other direction too. Of the non-picks `G0`'s
`quantile@95` removes (the reject-filter evaluation's Decision 2 `RN`), 74.8% [69.9, 79.4] are `discard`.

**2. No usable discard cut exists; there is no best cut.** Per `G0` cut
(the bars applied in discard.md):

| Cut | `RR` % | `PL` % | `MR` % | Removed % | `DP` - random, points | Fails |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| q5 | 17.3 [11.3, 24.0] | 0.68 | 91.6 [90.2, 92.8] | 4.92 | +30.7 [-3.8, +63.9] | `RR` point; random |
| q10 | 23.7 [18.7, 28.7] | 2.10 | 84.5 [81.9, 86.4] | 9.87 | +14.8 [-8.7, +38.6] | `RR` point and quantile; random |
| q20 | 27.9 [23.3, 32.5] | 5.18 | 70.5 [66.1, 74.1] | 19.80 | +16.1 [+0.3, +31.7] | `RR` point and quantile; `PL` |
| `quantile@95` | 26.0 [21.3, 31.0] | 3.64 | 77.3 [73.7, 80.5] | 15.05 | +18.0 [+2.0, +34.7] (vs random q20) | `RR` point and quantile |

The nearest is q5: it would hide 4.92% of the frames, lose 0.68% of the
picks, and the user would rescue 17.3% of what it hides, above the 15% bar (its upper
quantile 24.0% passes, but the folder-cluster one, 25.7%, does not), and it
leaves 91.6% of the discardable frames in place (`DO@5` 46.7%, near the base
rate). Its `DP` lead over random is large on the point (+30.7) but not
separated from zero, random q5 removing only 27 sampled frames. Every deeper
cut rescues 24-28% (q10 23.7%, `quantile@95` 26.0%, q20 27.9%). **The unblinded 70/71 did not hold up
blind**: the hypothesis `RR@5` <= 5% is not confirmed, the blind `RR@5`
being 17.3% [11.3, 24.0] (26 of the 150 sampled `b05` frames marked keep)
against about 1.4% (1 of 71) when the user knew the frames were the model's
candidates, about twelve times higher, with the interval far above 5%. The
two readings are of the same band (`G0`'s per-folder bottom 5% on the full
dump), the unblinded one on the strict sample's 71 frames of it. This is a
finding about **automation bias in the rescue scheme**: shown a frame as a
discard candidate, the user accepted the discard far more often than when
judging it blind, so a rescue loop that labels its frames as candidates
would understate what it costs. The unblinded frames had also had a strict
pass first, and 22 of the 71 were strict-bad and not re-asked (1 of 49
rescued among the rest), so prior exposure is a second explanation the two
readings cannot separate.

**3. `G0` is not distinguishable from the other scores; `eye_focus` has the
highest AUC.** At q5 every paired `DP` and `DR` difference between `G0` and
`W0`, `R0`, absolute sharpness and `eye_focus` has an interval spanning zero
(`DP` +5.8, +0.5, +4.7, -0.5 points; `DR` +1.3, -0.8, -1.3, +0.3), and so
does every one at q10 and q20. Against random, `G0` q5 removes more of the
discardable frames (`DR` +5.8 points [+2.8, +8.2]) but its `DP` lead is not
separated from zero. Threshold-free, the weighted AUC of `discard` vs `keep`
is highest for `eye_focus` (missing last) 0.777 [0.716, 0.841], then `R0`
0.740, `G0` 0.710, `W0` 0.671, absolute sharpness 0.540 and random 0.528.
`eye_focus` reaches `RR` 14.3% at q10, under the point bar, but at `PL`
9.14%, and the comparison scores are not candidates for the usable-cut bar.
Multiplicity (about 6 scores, 4 cuts) applies: none of these differences
reads as a pass.

**4. The strict labels are coherent with the discard labels.** In every
`G0` band the strict-bad rate (from the disjoint strict sample) is below the
`discard` rate: `b05` 27.3% vs 82.7%, `b10` 12.5% vs 70.0%, `b20` 10.1% vs
68.0%, `control` 3.7% vs 42.6%; overall 5.52% vs 48.45%. Both fall from the
bottom band to the controls in the same order, so the two questions rank
the bands alike, and strict bad is the narrow subset of "OK to discard" it
was meant to be. No inversion.

**5. Recommendation: a later plan that trains on the rescue labels with the
controls as negatives**, by the pre-registered follow-up rule's second
branch: there is no usable cut, but `RR@5` 17.3% is within twice the `RR`
bar (<= 30%) and `DO@5` 46.7% is >= 20%, so the model misses much that is
discardable and there is something to learn. Not an in-app discard
candidates view: the first branch needs a usable cut. This is a
recommendation only; no plan is opened here and no app change follows from
this plan. That later plan would need to specify:

- **The training labels and their weights**: these 600 blind labels
  (`labels.tsv`, `keep` / `discard`) with their `π`, the 150 controls
  included so the model sees frames `G0` did not choose (never the 450
  candidates alone), and whether the 1,200 strict labels enter as a second
  set (strict bad is coherent with `discard`, 4); and whether a second blind
  round is drawn first to give the controls more weight (150 controls, 58
  `discard`, now stand for 25,071 frames).
- **The features and the model**: which of the stored columns it combines
  (the embedding PCA, `eye_focus`, which reads the highest AUC here, the
  pass-2 cues, `R0`'s inputs), its form, and that it stays offline, apart
  from `rank.rs`'s `fit` and `ranking.json`.
- **The evaluation**: held out by folder (27 folders, 3 to 96 sampled frames
  each) or on a fresh blind sample drawn as Step 1 drew this one, with
  candidates and random frames mixed and unmarked, given the automation-bias
  finding of item 2 above.
- **The bars, pre-registered before training**: the same "hide / sort last"
  bars as here (`RR` <= 15% with the 97.5% quantile <= 25%, `PL` <= 5%,
  removed share >= 4.5%, `DP` above random's), plus a paired gain over `G0`
  at the same `q`, and what follows a pass (then the in-app view plan of the
  first branch, with its depth, semantics and app surface).
- **The labeling budget**, sized at about 3 s per frame (learnings Step 5).

The recommendation would change:

- to **an in-app discard candidates view with a rescue loop** at the best
  cut's depth if a fresh blind round (or the trained model of the later plan)
  put a cut over every usable-cut bar (weighted `RR` at or under 15% with the
  97.5% quantile at or under 25%, `PL` at or under 5%, removed share at or
  above 4.5%, `DP` above random's with the paired 2.5% quantile above zero); that plan would then have to specify the
  depth, the "hide / sort last" semantics and the app surface (a filter or a
  sort that puts the candidates last, the rescue key, and how the rescue
  marks are kept), and, by the automation-bias finding, how its rescue rate
  is measured while the user can see which frames are candidates;
- to **nothing** if the user moves to "delete" semantics (`RR` <= 5%, `PL`
  <= 1%), which every cut here misses by more than three times on `RR`, or if
  a re-read put `RR@5` above 30% or `DO@5` under 20%;
- in its reach, not its branch, if the goal moves away from removing frames
  before the human picks: the 24.4% of picks labeled `discard` says the
  user's own picks are not a clean "keep" class, which a later plan training
  on picks rather than on these labels would have to face.

## Progress

- (2026-10-18) Plan written; the five choices decided by the user (see
  Purpose, "Choices")
- (2026-10-18) Step 1 complete except the manual check that the 600-frame
  sample opens unjudged in Riffle (pending, see `learnings.md` "Deferred
  issues (todo candidates)"; do it before the user starts labeling)
- (2026-10-18) Step 2 complete: `discard.md` pre-registration written before any
  label; removed-share bar lowered from 5% to 4.5% by the user before labeling
  (see "Choices" and `learnings.md` Step 2)
- (2026-10-19) Step 3 complete: 600 labeled, 211 keep / 389 discard, 0
  skipped. The user later reported about 30 minutes taken and `aiEnabled`
  off throughout (`learnings.md` Step 3)
- (2026-10-19) Step 4 complete
- (2026-10-19) Step 5 complete
