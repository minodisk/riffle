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

# Similar-scene groups for non-sports shooting: a gated evaluation, then the ranking over scene groups

## Purpose

The personal ranking (`crates/core/src/rank.rs`, `rank` / `bursts`) and the
`Recommended` order (`crates/app/ui/src/sort.ts`, `burst.ts`) compare frames
only inside a burst: frames within 1000 ms of each other (`BURST_GAP_MS`). In
non-sports shooting (landscape, travel, portrait, snaps) most frames are
singles — in the user's snap folders 30–80% of the bursts have one frame
(`2026-07-30`: 397 singles of 936 frames; `2026-07-11`: 291 of 468) — so
those frames get place 1 of 1, score 0, and no recommendation at all.

Two earlier attempts bear on this. The sports similarity check
([similarity.md](../20261012-image-embeddings/similarity.md), 150
hand-labeled pairs from 8 folders) found no SigLIP 2 cosine threshold that
separates "same play" from "different play" at 0.95 precision: the embedding
carries the place, the sport and the atmosphere, not which play. A low
cosine did reliably mean different (no "same" below 0.87). The folder-wide
pick model ([folder-wide-pick-model](../20261014-folder-wide-pick-model/plan.md))
failed its gate: comparing frames across bursts with one per-frame score did
not beat burst size (burst AUC 0.710 vs 0.751). The idea left is to widen
the *group* and keep comparing only within it: in non-sports shooting,
"same scene" is closer to "same place and composition", which is what the
global embedding does carry.

This plan asks, with the same method as the sports check (hand-labeled
pairs, stratified by cosine and now also by capture-time gap, a
pre-registered precision bar with a confidence bound), whether SigLIP 2
cosine plus capture-time proximity groups same-scene frames in the user's
non-sports folders well enough to ship. Only if it does, the ranking and
the `Recommended` order run over scene groups instead of 1 s bursts; the
strip's burst band, badge and navigation, and the ranking's training, do
not change.

Out of scope (another session owns them; do not touch):

- The in-burst ranking's internals: `pairs`, `Labeled`, `fit` in `rank.rs`,
  the views-based training scope, the refit triggers and the 0.5 s viewed
  threshold.
- Per-genre weighting (`docs/plans/20261014-genre-ranking-offsets/`).

Constraints, binding on every step: English in the repository; everything
local; a recommendation never changes a judgment; everything behind
`aiEnabled`; the in-burst ranking keeps working unchanged on bursts; the
labeling material lives in `D:\Photos\tests\2026-10-15-scene-pairs\`
(outside the repository, never in the scratchpad); Python runs with
`python -I` from the venv `%TEMP%\embedvenv` (numpy, scipy, scikit-learn,
Pillow; the system Python has no numpy).

### What is known (investigated 2026-10-15)

- **Embeddings.** The live app index has almost none for the 17 folders the
  user supplied (2026-07-23: 18, 2026-07-21: 6, all others 0; `2026-05-22`
  is not indexed; `2026-06-14` only partly). The archived app dump
  `D:\Photos\tests\2026-10-12-rank-app\` (`embed.tsv`, 768-d L2-normalized
  SigLIP 2 at `--near 404`, `EMBED_VERSION` 1; `features/<folder>.tsv` with
  capture time, subsec, `xmp`, `dop` flags; `features/<folder>.output.txt`;
  `genres.tsv` with per-file genre scores) covers 14 of the 17 folders —
  every ARW folder. The three DNG folders `2026-09-05` (411 frames, 66
  picks), `2026-08-29-l` (211, 33 picks) and `2026-05-22` (37, labels only
  through `Output/`, 19 exports) are not in it.
- **Folder character** (argmax of the per-file top-level genre softmax in
  the dump's `genres.tsv`): `2026-06-14` is **sports** (91%). `2026-08-08`
  is snap (88%); 12 of its pairs (24 files) were labeled in the sports round
  under the sports definition. `2026-08-02` (16% sports frames),
  `2026-07-31` (11%), `2026-08-15` (8%) are mildly mixed. `2026-07-23` (18
  frames), `2026-07-21` (32) and `2026-07-26` (64) are too small to yield
  many pairs but may stay in the pool. The rest are 88–100% snap.

  | Folder | Frames | Picks | Bursts | Singles | Span | Genre mix |
  |---|---:|---:|---:|---:|---:|---|
  | 2026-08-16 | 59 | 6 | 19 | 9 | 1.8 h | snap 97% |
  | 2026-08-15 | 324 | 37 | 182 | 120 | 6.6 h | snap 78%, portrait 10%, sports 8% |
  | 2026-08-08 | 415 | 73 | 191 | 95 | 3.2 h | snap 88% |
  | 2026-08-02 | 150 | 28 | 69 | 32 | 2.8 h | snap 73%, sports 16%, portrait 9% |
  | 2026-07-31 | 349 | 52 | 159 | 88 | 3.4 h | snap 80%, sports 11%, landscape 6% |
  | 2026-07-30 | 936 | 105 | 563 | 397 | 10.7 h | snap 90%, sports 6% |
  | 2026-07-26 | 64 | 7 | 16 | 6 | 0.4 h | snap 53%, portrait 44% |
  | 2026-07-24 | 218 | 34 | 163 | 122 | 4.3 h | snap 88% |
  | 2026-07-23 | 18 | 4 | 17 | 16 | 1.1 h | snap 100% |
  | 2026-07-22 | 189 | 32 | 101 | 64 | 2.8 h | snap 91% |
  | 2026-07-21 | 32 | 7 | 27 | 23 | 12.0 h | snap 78%, portrait 22% |
  | 2026-07-11 | 468 | 57 | 359 | 291 | 10.5 h | snap 94% |
  | 2026-07-02 | 106 | 15 | 32 | 21 | 0.1 h | snap 100% |
  | 2026-06-14 | 1520 | 161 | 285 | 69 | 5.0 h | sports 91% |

- **Picks.** Sidecar `.dop` Pick per the dump (as the in-burst gate's rule:
  `.dop` Pick, else a virtual copy in `Output/`, else XMP Pick with no
  `.dop`). Pick rates run 10–17%.
- **Tooling to reuse** (import or run, do not copy):
  [pairs.py](../20261012-image-embeddings/pairs.py) (`load`,
  the `BINS` draw, `previews`, the `PAGE` labeling template with S / D / U
  keys and `labels.tsv` export),
  [similarity.py](../20261012-image-embeddings/similarity.py)
  (threshold table), [gate.py](../20261012-image-embeddings/gate.py)
  (`load`, `bursts_of`, label rule). `exiftool` is on PATH (`previews`
  needs it). `riffle-cli embed <model.onnx> siglip2 <dir>... [threads]
  --near 404 --out <tsv>` and `riffle-cli features <dir>... [threads]`
  produce the dump format; the vision model the app downloaded is at
  `C:\Users\daisu\AppData\Local\com.minodisk.riffle\models\siglip2-base-patch32-256\vision_model.onnx`.
  Alternatively the ignored test `rank_dump_of_real_folders`
  (`crates/app/src/commands.rs`, env `RIFFLE_RANK_DIRS` / `RIFFLE_RANK_OUT`
  / `RIFFLE_EMBED_MODEL`) writes the app-side dump, DNG included.
- **Rust / TS side.** `rank::bursts(frames) -> Vec<Vec<usize>>` is the only
  grouping `rank` and `pairs` use; `rows` takes the groups as a parameter.
  `Index::rank_rows` already delivers capture time, subsec and the vector
  per file. `ranking_of` (`crates/app/src/rank.rs`) returns
  `{ model, files: path -> Ranked { score, place, of, reasons } }` and no
  group id; the frontend derives the burst id itself with `groupBursts`
  (`main.ts`) and feeds it to `orderFiles` as `facts.burst`. `recommend.ts`
  marks place 1 of >= 2. `docs/humans/usage.md` "Recommended order" and
  `README.md` describe the per-burst semantics.

### Decisions taken at approval (2026-10-15)

The user approved the plan with about 200 labeled pairs and, as in the
previous plan, asked for the more accurate judgment where there is a choice:

- **Precision bar**: the one-sided 95% Clopper-Pearson lower bound on
  precision >= 0.95 decides (not the point estimate); the draw holds about
  200 pairs, about 100 of them at cosine >= 0.90.
- **Recall and coverage**: recall >= 0.30 over labeled same-scene pairs
  (anchors excluded) and >= 20% of today's singles grouped with something.
- **Mixed folders**: keep every frame in the pool (no genre filter) and
  report the sports share, because the app would not filter by genre at run
  time either. `2026-06-14` (91% sports) is excluded as a sports folder;
  `2026-08-08` stays, minus its 24 previously labeled files.
- **DNG folders**: dump `2026-09-05` and `2026-08-29-l` fresh and include
  them; `2026-05-22` (37 frames, no sidecars) is dropped.
- **What the group replaces if it ships**: only `rank`'s grouping and the
  `Recommended` order; the strip's band, badge and navigation and the
  training stay on 1 s bursts.

## Steps

- [x] Step 1: Pre-register the scene-group evaluation in `groups.md` (definition, data, grouping candidates, sampling, bars, confidence rule), no pair drawn
  - Done when:
    - `groups.md` in this plan folder has a "Pre-registration" section
      merged **before any script exists, any pair is drawn or any label is
      given** (the merge commit is the record, as `ce0ea14` was for the
      in-burst gate). Nothing in it changes afterwards; a deviation found
      later goes to `learnings.md` with its reason.
    - The section fixes, in this order:
      - **Definition of "same scene" (non-sports)**: interchangeable
        candidates of which you would keep one — the same place and
        composition (a landscape or street reframed slightly, re-exposed,
        re-shot a moment later), or the same subject in the same pose or
        activity (a portrait sitting, a child at the same spot doing the
        same thing). "Different": another composition of the same place
        (a wide and a detail), another pose or activity, another subject,
        another moment of a moving scene you would keep both of. "Unsure"
        allowed and dropped from the counts. Written in English in
        `groups.md` and in Japanese on the labeling page.
      - **Data**: the 13 non-sports ARW folders of the app dump
        (`2026-08-16`, `2026-08-15`, `2026-08-08` minus its 24 previously
        labeled files, `2026-08-02`, `2026-07-31`, `2026-07-30`,
        `2026-07-26`, `2026-07-24`, `2026-07-23`, `2026-07-22`,
        `2026-07-21`, `2026-07-11`, `2026-07-02`) plus the DNG folders
        `2026-09-05` and `2026-08-29-l` once dumped (Step 2);
        `2026-05-22` dropped; `2026-06-14` excluded as sports. Every frame
        of the pool folders stays (no genre filter); the sports share of the
        drawn pairs is reported. Bursts as `rank::bursts` (gap 1000 ms).
      - **Grouping rule candidates**: chained in capture order like
        `bursts`: a frame joins the open group when its cosine to the
        group's last frame >= t and its capture gap to that frame <= W,
        for t on a grid (0.80 … 0.98 step 0.01) and W in {unbounded, 60 s,
        300 s, 1800 s}; a 1 s burst always stays together (the burst is
        the atom). Single linkage to any member and the centroid rule are
        fallbacks evaluated only if the chained rule fails the recall bar,
        to limit the search.
      - **Pair sampling**: within-folder pairs of frames from *different*
        bursts, stratified over cosine bins (0.70–0.80, 0.80–0.85,
        0.85–0.90, 0.90–0.93, 0.93–0.96, 0.96–1.00) crossed with time-gap
        bins (<= 60 s, 60 s–300 s, 300 s–1800 s, > 1800 s), the counts
        weighted toward the top three cosine bins so that about 100 labeled
        pairs lie at cosine >= 0.90; about 15 within-burst anchors; about
        200 pairs in all (fix the exact per-cell counts here); no file
        reused; seeded; sides swapped at random; the page shows neither
        cosine, gap nor file names. Folders with too few candidates in a
        cell contribute what they have.
      - **Precision bar**: at the chosen (t, W), among labeled pairs
        predicted "same" (cosine >= t and gap <= W), the one-sided 95%
        Clopper-Pearson lower bound on precision >= 0.95 (so at least 59
        predicted-same pairs with zero "different", or about 93 with one;
        the point estimate is reported beside it). Record the alternative
        (point >= 0.95 only) as rejected and why.
      - **Minimum recall worth shipping**: recall over labeled same-scene
        pairs (anchors excluded) >= 0.30 at that (t, W), and, applied to
        the pool folders, the rule puts >= 20% of today's singles into a
        group of two or more (the "coverage" number). Both must hold.
      - **Selection rule**: the (t, W) with the highest recall among those
        meeting the precision bar; ties to the larger W then the lower t.
      - **Interaction check (reported, not deciding)**: with the shipped
        prior weights, Top-1 within scene groups of >= 2 frames that hold
        at least one pick and one non-pick vs the first-frame baseline, on
        the pool folders; because the `Burst position` columns were fit on
        1 s bursts, a poor number here is raised again before Step 5.
      - **What happens on fail**: Steps 4–6 are dropped, Decision 1 records
        the tables and the plan ends with the wrap-up.
    - `mise run lint` passes (lychee on the links).
  - Implementation approach:
    - Model the section on the archived
      [gate.md](../20261012-image-embeddings/gate.md) and
      [folder-wide gate.md](../20261014-folder-wide-pick-model/gate.md)
      pre-registrations; keep their vocabulary.
    - No Python, no Rust in this PR; a doc-only PR.

- [x] Step 2: `scene_pairs.py` draws the pairs and writes the labeling page; stop for the user to label
  - Status: **awaiting labels.** The page is at
    `D:\Photos\tests\2026-10-15-scene-pairs\label.html`; Step 3 waits for
    the user's `labels.tsv` saved beside `pairs.tsv` there.
  - Done when:
    - A fresh dump of `2026-09-05` and `2026-08-29-l` exists in
      `D:\Photos\tests\2026-10-15-scene-embed\` in the dump format
      (`features/<folder>.tsv`, `features/<folder>.output.txt`,
      `embed.tsv`), made with `riffle-cli features` and `riffle-cli embed
      … siglip2 … --near 404` against the app's downloaded
      `vision_model.onnx` (or with the ignored `rank_dump_of_real_folders`
      test); the command lines and timings go to `learnings.md`. Verify
      that the CLI reads DNG (it dispatches through `reader`); if it does
      not, use the app test path and record it. If neither works, skip the
      DNG folders and record why.
    - `scene_pairs.py` in this plan folder (`python -I scene_pairs.py
      <dump-dir> [<dump-dir>...] <out-dir>`) imports the archived
      `pairs.py` (`sys.path.insert(0, <archived folder>)`) for `load`,
      `previews` and `PAGE`, and implements the pre-registered pool,
      the cosine x gap stratification, the excluded-file list
      (2026-08-08's 24 files, read from the old `pairs.tsv`), and writes
      `pairs.tsv` (id, folder, a, b, cosine, gap_s, cosine_bin, gap_bin;
      `burst` for anchors), `img/`, `label.html` into
      `D:\Photos\tests\2026-10-15-scene-pairs\`. The page's definition
      text is the Japanese rendering of the Step 1 definition; the
      `localStorage` key differs from the sports page's
      (`riffle-scene-pairs-v1`) so old labels do not bleed in.
    - The per-cell counts actually drawn are printed and recorded in
      `groups.md` under "Draw" (not in the pre-registration section).
    - **Hand-off**: the PR merges with the page produced and the plan
      noting "awaiting labels". The implementer does not label. The user
      labels in `label.html` and saves `labels.tsv` beside `pairs.tsv`.
      Step 3 starts only when `labels.tsv` exists.
  - Implementation approach:
    - Depends on Step 1 merged.
    - The capture gap of a pair comes from the dump's capture time and
      subsec as `pairs.capture_ms` does.
    - Keep `scene_pairs.py` small: reuse, do not re-implement, the
      previews and page.

- [ ] ~~Step 3: `scene_groups.py`, the results and Decision 1 (ship or not)~~ (dropped, Decision 1)
  - Done when:
    - `scene_groups.py` in this plan folder (`python -I scene_groups.py
      <pairs-dir> <dump-dir> [<dump-dir>...]`) joins `pairs.tsv` and
      `labels.tsv`, prints the per-cell same / different counts, the
      precision (point and one-sided 95% Clopper-Pearson lower bound) and
      recall for every (t, W) on the pre-registered grid, the selected
      (t, W) by the selection rule or "none", the coverage number (share
      of singles grouped) at that rule over the pool folders, and the
      interaction check (Top-1 vs first frame within scene groups using
      the prior weights from `crates/core/ranking/siglip2.json`, applied
      through a small Python port of `rows` / `rank` or by reading
      `prior_scores.tsv` of the dump if its scores are usable as-is —
      decide and record).
    - `groups.md` gains "Results" with the tables and the reading; the
      top-20 cosines listing as `similarity.md` has.
    - **Decision 1** is written in this plan: pass -> Steps 4–6 proceed
      with the chosen (t, W) as constants; fail -> Steps 4–6 are struck
      through "dropped (Decision 1)" and the plan ends with the wrap-up.
      The numbers that decided are in the Decision text.
  - Implementation approach:
    - Depends on Step 2 merged and on `labels.tsv` existing.
    - Clopper-Pearson lower bound: `scipy.stats.beta.ppf(0.05, k, n - k + 1)`
      (0 when k = 0; for k = n the closed form `0.05 ** (1 / n)`).
    - Unsure labels dropped, as before.

- [ ] ~~Step 4 (only if Decision 1 passes): `crates/core/src/scene.rs`, the scene grouping, used by `rank` for placing and scoring~~ (dropped, Decision 1)
  - Done when:
    - `scene.rs` exposes `pub fn groups(frames: &[Frame]) -> Vec<Vec<usize>>`
      implementing the chosen rule: start from `rank::bursts`, then chain
      bursts in capture order into one group when the cosine between the
      last frame of the open group and the first frame of the next burst
      (both with a vector) >= `SCENE_COSINE` and the capture gap <=
      `SCENE_GAP_MS` (constants with the Decision 1 values and a doc
      comment citing `groups.md`); a burst without vectors or capture time
      stays its own group. Unit tests: bursts never split; chaining;
      the gap bound; missing vectors.
    - `rank::rank` groups with `scene::groups` instead of `bursts`;
      `pairs` / `fit` keep using `bursts` (training unchanged). `Ranked`
      gains `pub group: usize` (the group's index in capture order) so the
      frontend no longer has to derive it. `rank.rs` tests updated; the
      `rank_version` is not bumped (the columns do not change).
    - `CLAUDE.md`'s Layout paragraph mentions `scene.rs` in one clause.
    - `cargo test -p riffle-core` and `mise run ci` pass.
  - Implementation approach:
    - Depends on Step 3's Decision 1 = pass.
    - Keep out of `pairs`, `Labeled`, `fit`, `COLUMNS`, `REASONS`; the
      `Burst position` columns are computed per *group* by `rows` once
      `rank` passes groups — verify this is intended (it is what the
      interaction check in Step 3 measured) and say so in the doc comment.
    - Rebase after the `genre-ranking-offsets` or
      `ball-game-ranking-features` PR that touches `rank.rs` if one lands
      first; the diff here is one call site plus the `group` field.

- [ ] ~~Step 5 (only if Decision 1 passes): the app orders and marks by scene group~~ (dropped, Decision 1)
  - Done when:
    - `ranking_of` returns the `group` with each `Ranked` (serialized by
      the Step 4 field; `crates/app/src/rank.rs` needs no logic change,
      verify).
    - `sort.ts`: `SortFacts` gains `group?: number`; `recommended` orders by
      `group` (falling back to `burst` for a file without a ranking entry)
      then score, then capture, as today with the burst. `main.ts` fills
      `group` from `fileRanks`. `recommend.ts` unchanged (place 1 of >= 2
      now means within the group).
    - `meta.ts`: the `Recommended` row's wording says "in its scene group"
      (or "in its burst" when the group equals the burst — decide the
      simplest truthful wording and record it).
    - The strip's burst band, badge and `burstNext` / `burstPrevious` stay
      on `groupBursts` (1 s).
    - `pnpm exec vp test` covers the new sort path; `mise run ci` passes.
    - A hand check on `2026-07-30` or `2026-07-11` (opened with
      `aiEnabled` on, so the embed pass fills the folder): singles now show
      place x of n and the `Recommended` order interleaves them by group;
      recorded in `learnings.md` with the folder and the observed group
      count.
  - Implementation approach:
    - Depends on Step 4 merged.
    - Surgical: do not touch `burst.ts`.

- [ ] ~~Step 6 (only if Decision 1 passes): user docs~~ (dropped, Decision 1)
  - Done when:
    - `docs/humans/usage.md` "Recommended order" and `README.md`
      "Recommended order (AI analysis)" say the ranking compares frames of
      one scene group (frames of the same place and composition shot close
      in time, which contains the 1 s burst) and name the numbers from
      Decision 1 (precision bound, recall); `usage.ja.md` and
      `README.ja.md` updated in the same PR. `mise run lint` passes.
  - Implementation approach:
    - Depends on Step 5 merged.

## Trade-offs and risks

- **Confidence rule for the precision bar.** Decided: the one-sided 95%
  Clopper-Pearson lower bound >= 0.95. A point estimate over 20
  predicted-same pairs says little; the bound needs >= 59 predicted-same
  pairs with no "different" (or about 93 with one), hence the draw weighted
  toward high cosines and about 200 pairs.
- **Minimum recall and coverage numbers** (0.30 recall, 20% of singles
  grouped). Too low a bar ships a grouping that changes little; too high
  fails a useful one.
- **Mixed folders.** Decided: keep every frame, report the sports share,
  since the app would not filter by genre at run time.
- **The DNG folders.** Decided: dump `2026-09-05` and `2026-08-29-l` fresh
  (the only Leica / travel-style material); skip with the reason recorded if
  neither dump path reads DNG.
- **Grouping rule shape.** Chaining to the open group's last frame mirrors
  `bursts`, is O(n) and needs no all-pairs matrix at run time. Single
  linkage or a centroid rule can group more but risk "lumping a whole
  afternoon" (the sports failure mode). Pre-registered as fallbacks only, to
  avoid choosing the rule after seeing the labels.
- **What the group replaces if it ships.** Decided: only `rank`'s grouping
  and the `Recommended` order. Risk: the `Burst position` columns were fit
  on 1 s bursts; inside a scene group spanning minutes "position" and "first
  frame" mean something else. Step 3's interaction check measures this with
  the prior weights; if it reads below the first-frame baseline, the user
  decides before Step 5 whether to zero the position columns inside
  non-burst groups (a `rank.rs` change outside `pairs` / `fit`) or to accept
  it.
- **Reusing previously labeled files.** The 12 pairs of `2026-08-08` from
  the sports round were judged under "same play"; the new definition
  differs, so those 24 files are excluded rather than relabeled or reused.
- **Concurrent plans on `rank.rs`.** `genre-ranking-offsets` and
  `ball-game-ranking-features` change the columns and `RANK_VERSION`. Step 4
  adds a module and changes one call site plus a `Ranked` field; rebase
  after whichever lands first.
- **Fail path.** If Decision 1 fails, the artifacts are `groups.md`,
  `scene_pairs.py`, `scene_groups.py` and this plan with the numbers, and
  the finding that the embedding does (or does not) carry "same place and
  composition" for non-sports shooting; no code changes. That is an
  acceptable outcome of the plan.

## Decision 1 (scene groups ship or not)

Stopped before labeling (2026-10-15); not decided by the pre-registered
criterion. No pair was labeled and `scene_groups.py` was not written.

The user redefined the ranking's goal the same day: find good photos
anywhere in a folder, a single shot or a burst frame alike, without
treating bursts as special. Scene groups extend the within-burst choice
("which frame of this group") to singles, so they serve the old framing,
not the new one, and the user chose to stop the plan. Steps 3–6 are
dropped. The folder-wide measurement the new goal calls for starts from
the archived `20261014-folder-wide-pick-model` results (`folder_gate.py`:
frame AUC 0.666, singles AUC 0.611, top-tenth lift 2.08).

What remains: `groups.md` (the pre-registration), `scene_pairs.py`, and
outside the repository the 200 drawn pairs and the unused labeling page in
`D:\Photos\tests\2026-10-15-scene-pairs\` and the DNG dump in
`D:\Photos\tests\2026-10-15-scene-embed\`. Should scene groups be wanted
again (for example to fold near-duplicates), labeling those pairs and
running Step 3 as pre-registered is still valid.

## Progress

- (2026-10-15) Plan written
- (2026-10-15) Step 1 complete
- (2026-10-15) Step 2 complete
- (2026-10-15) Stopped before labeling; Steps 3–6 dropped (Decision 1)
