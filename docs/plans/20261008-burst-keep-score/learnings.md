# Learnings: keep-candidate score within a burst

## Step 1

- `riffle-cli features <dir>... [threads]` prints the header and the records
  on stdout and its totals (`N files ..., E with an error: Ts total`) on
  stderr, so `> folder.tsv` is the table alone. One rayon pool runs both the
  analysis and the eyes path per file; the per-file ms are taken inside the
  worker, so under 24 threads they include contention.
- The dump (release build copied to
  `D:\Photos\tests\2026-10-08-burst-keep-score\riffle-cli.exe`, 24 threads,
  one folder per run, [dump.sh](dump.sh)): 36 folders, 33,519 files, no
  error, **503 s wall** in total (the CLI's own totals, which leave out the
  model build, sum to 484 s; `2026-09-27-a`'s 4830 ARWs took 67 s, about 14 ms of wall per ARW).
  The DNG folders cost more: `2026-03-29` (197 DNGs) 9.5 s, `2026-05-26`
  (163 DeepPRIME DNGs) 10.2 s.
- Per-file ms on a worker (24 threads, so inflated by contention):
  ARW (32,459) analysis median 141.3 / p95 159.8 ms, eyes path median
  203.5 / p95 229.9 ms; DNG (1060) analysis median 309.1 / p95 750.4 ms,
  eyes path median 412.9 / p95 1150.0 ms. All files: analysis mean 149.1,
  max 1396.6 ms; eyes path mean 191.4, max 2632.8 ms. Not comparable with
  the single-threaded 80-92 ms of `eyes_of` in `docs/humans/performance.md`.
- **Virtual copies hide picks from `dop::read_flag`.** The first inventory
  gave `2026-06-05` zero picks and `2026-09-13-b` four, against 199 and 232
  `ShouldProcess = 0` lines and `Output/` files. In those folders the user
  picked a PhotoLab virtual copy: the `.dop`'s second item has
  `ShouldProcess = 0`, the master item `2`, and `Output/` holds
  `<stem>_1.jpg`. `read_flag` reads the first item only (as the app does),
  so the `dop` column says `None`. The label rule now also takes a frame
  whose `<stem>_<n>` is in `Output/`; a throwaway script checked that, in
  every sidecar folder of the data set, the frames with a pick on any item
  equal `Output/`'s frames exactly. The CLI was left as the plan says (no
  new parsing); the rule lives in [inventory.py](inventory.py) and
  [data.md](data.md).
- **DeepPRIME DNGs in the sidecar-less folders.** Most of the 2026-01 to
  2026-05 DNG folders hold `<stem>-DxO_DeepPRIME 3.dng` files, which
  `scan::is_raw_file` lists like any DNG: `2026-03-29` has 155 camera DNGs
  and 42 DeepPRIME ones, and `2026-01-08`, `2026-02-14`, `2026-05-26` have
  only DeepPRIME DNGs. `Output/` names them by the DeepPRIME stem. Counting
  files made several all-exported folders look partly exported (and
  `2026-04-10`'s two files are one exported frame). The inventory groups by
  the base stem (the `-DxO_...` suffix removed) and prefers the camera
  file's row. A DeepPRIME DNG has a full-size JPEG preview, the capture
  time, and no AF point.
- Two of the "Leica" `Output/` folders are Sigma BF (`BF_*.DNG`):
  `2026-05-22` (camera DNGs, every frame `af` but no cue face and no judged
  face, a night folder) and `2026-05-26` (DeepPRIME DNGs only). The
  stripping of a virtual copy's `_<n>` only applies when the stem is not a
  frame itself, since `BF_00634` already ends in `_<digits>`.
- Every ARW frame of the data set has a trusted AF point; no sidecar DNG
  frame has one, so the ARW eyes columns come from the AF path and the
  Leica ones from the whole-image path.
- In Git Bash, a `python - <<'EOF'` here-document mangled `'\\'` in one
  attempt; `chr(92)` or a script file avoided it.

## Step 2

- No `numpy` in the local Python 3.13, so [metrics.py](metrics.py) is
  standard library only (the AUC by rank sums, percentiles by index). It
  runs in about 1.5 s over all 36 dumps.
- **The relative sharpness hand cuts are unusable.** `rel < 0.7` fails
  60% of the ARW picks: the median pick is at 0.585 of its burst's
  maximum. The AF-region sharpness changes with what is under the AF point
  (a face, lettering, a ball) from frame to frame, so the hand check found
  sharp frames at `rel` 0.11-0.67. Only a 99% pick-keeping point
  (`rel >= 0.027`) protects the picks. See [results.md](results.md).
- **The first frame of a burst is a better ranking than any feature**
  (pairwise AUC 0.653 against sharpness 0.577): 39.5% of first frames are
  picks against 19.2% from the sixth on. Worth keeping in mind for Step 3
  (the capture position is not a technical check, so it is not a feature
  of the plan, but it is a baseline to beat).
- The pose separates nothing (pick false-fail equals the non-pick flag
  rate at every cut, AUC around 0.5), and the hand check found every
  pose-only flag a fine frame (profiles, faces looking up).
- `python -I` does not stop bytecode writes: a scratch script that put
  this folder on `sys.path` to import `metrics` left a `__pycache__/`
  here, removed before the commit. Python's text-mode `open(..., "w")` on
  Windows also wrote CRLF into `plan.md` and `metrics.py`; `sed -i
  's/\r$//'` put them back to LF. A Git Bash heredoc containing a quoted
  `'EOF'` and backticks failed to parse once; writing the text with the
  Write tool avoided it.
- The hand check (30 flagged non-picks of rule (e)) was the agent's own
  look at the crops, as the caller asked, not the user's: failed 9, OK 16,
  unsure 5. The crops are in
  `D:\Photos\tests\2026-10-08-burst-keep-score\handcheck\` for the user to
  re-check.

## Step 3

- [fit.py](fit.py) imports [metrics.py](metrics.py) from its own folder
  (`sys.path` plus `sys.dont_write_bytecode = True`, so no `__pycache__/`
  lands here) and is standard library only: Newton's method on
  column-major lists with `sum(map(operator.mul, ...))` for the Hessian
  keeps one logistic fit on 16,522 frames at about a second. The whole run
  (every leave-one-folder-out variant, the pairwise fit on 41,944 pairs,
  the drop-one runs) takes about 6 minutes; every fit converged in 5
  iterations without regularization.
- **Compare at a matched pick false-fail, not at a fixed percentile.** A
  union of per-feature 1st percentiles fails up to the sum of them (the
  three-feature union at p 1% fails 2.7% of the picks), which makes any
  union look like it flags more than sharpness alone. The matched form
  (one common p, bisected so that the union fails at most the target on
  the training picks) is the fair comparison, and on it the combinations
  gain 0.5-0.8 pt at 1% and nothing at 5%.
- **The dumps predate the mesh `eye_focus`.** The Step 1 CLI was built
  (09:21) before `20261008-mesh-eye-focus` Step 3 (#733, merged 10:08)
  moved the cue to the mesh eye regions, so every `eye_focus` here is the
  eye-window cue. It does not change the Decision (dropping `eye_focus`
  raises the chosen set's flag rate), but a shipped `eye_focus` cut would
  need a re-dump.
- (Superseded by the approval entry below.) The Decision was first written
  as a proposal awaiting the user's approval, as the caller asked.
- (Superseded by the 2026-10-09 entry below.) **User intervention
  (2026-10-08): the user approved the no-ship Decision.** Steps 4-5 were
  struck on their headings, and Step 6 became docs-only.
- **User intervention (2026-10-09): forget bursts and build the "good
  photo" mark.** After the burst-level result, the user redirected the work
  to a frame-level `goodPhoto` mark, and Steps 4-6 were rewritten and
  approved (nothing is struck now). The burst-level no-ship conclusion of
  the Decision stands.
- **User intervention (2026-10-08): the goal was reframed before the first
  Decision was approved.** The first proposal (a failure check that keeps
  nearly every pick; nothing beat sharpness alone) went to the user, who
  answered that catching every pick is not needed, since the human makes
  the final choice: the mark should narrow a burst to the frames that
  clearly meet the minimum conditions (sharp, eyes in focus, eyes open,
  face toward the camera), and then, more strictly, those frames should be
  almost all picks (precision near 1, marked frames a subset of the picks).
  Step 3 was extended on the same branch with [keep.py](keep.py) and a
  "Keep mark" part of [fit.md](fit.md), the Purpose records the reframing,
  and the Decision was rewritten against it. The failure-check variant's
  file was renamed to [frozen-fail-check.json](frozen-fail-check.json);
  [frozen.json](frozen.json) now holds the keep rule of the Decision's
  alternative.
- **The keep mark's precision has a ceiling of about 42%.** Held out, the
  best is the burst's sharpest frame when it is faced (41.6% picks, 11.1% of the
  faced frames), and no grid rule reaches 60% even on its training
  folders. The strictest face cuts do not help because the picks' and
  non-picks' `eye_focus` and eyes-open distributions are the same at the
  top. The hand check of 12 marked non-picks found 8 OK, 4 unsure and none
  failed: the features find technically fine frames, and the user chooses
  among those by what the features do not see.
- keep.py counts every rule of the 1600-rule grid in one pass: each faced
  frame becomes a tuple of the highest level it passes per condition, the
  counts go into a 5-D table per folder, and a suffix sum over each
  dimension gives every rule's marked / picked counts, so the
  leave-one-folder-out selection is sums over folders (about 15 s in all).
- A Git Bash heredoc into `python -` turned `"\\n"` and `\t` inside the
  Python source into real control characters twice in this step (once in
  this file's text, once in keep.py); text with backslashes went through
  the Write tool or a file instead.
- **User intervention (2026-10-08): the approved Decision was reopened.**
  The user added that within a burst they also throw away good frames
  depending on the timing, so which frame of a burst was picked matters
  little, and chose to re-measure at the burst level. Step 3 was extended
  with [scene.py](scene.py) (every burst, including those without a pick,
  and the single frames) and a "Burst level" part of [fit.md](fit.md); the
  Decision went back to "Proposed" and was rewritten. A local review that
  had been stopped mid-round left partial fixes; the sound ones (the
  "sharpest frame, when that frame is faced" wording, the approval entry
  above) were kept, and its untracked `review-history` folder was deleted.
- **Burst length predicts a kept scene; the technical features do not.**
  Held out, the best technical rule marks bursts that hold a pick 55.8% of
  the time (base 36.1%), while a size cut alone gives 79.3% at 15 or more
  frames and 80.1% at 20 or more, and size plus a rule 80.7%. Single
  frames stay under 50% even in training. The user shoots longer at the
  scenes they keep, which the count badge already shows.
- scene.py counts a burst as marked by every rule any of its faced frames
  passes: each frame's level tuple maps to the list of rules it passes
  (precomputed for the 1600 tuples), and the burst takes their union, so
  the whole run, including a size cut times the grid held out, is about
  3 s. The held-out selection subtracts the held-out folder from the
  pooled totals instead of re-summing the other folders.
- **Without sharpness cuts the burst-level numbers hold or improve.** The
  user asked (2026-10-08) to drop the relative and absolute sharpness cuts,
  since `eye_focus` already measures sharpness at the eyes. Over the 64
  remaining rules the held-out results match the full grid up to an 80%
  target and beat it beyond (size >= 20 with `eye_focus` >= 0.95 and eyes
  open >= 0.995: 87.4% of 95 bursts, 1.9%, against 81.7% with sharpness):
  the larger grid overfit at the strict end. The Decision's conclusion was
  unchanged; its one sentence saying no held-out variant reached 85% was
  corrected.
- **Eyes open helps only with `eye_focus` on long bursts.** The user
  asked (2026-10-09) whether eyes open adds anything. Split by eye
  feature, the held-out burst-level results are within a point of size
  alone up to an 80% target; only `eye_focus` and eyes open together
  reach 87.4% (1.9% of the bursts), and dropping either from that rule
  falls to 83-84%, near size >= 20 alone (82.3%). The Decision did not
  change.

## Step 4

- **The frontend already carried the stored eyes and pose.** The plan's
  "What is known" said `Focus` / `MarkFocus` / `FaceReady` carry
  `eye_focus` and `candidate` only. On `main` at 69d49ccd (after #743)
  `MarkFocus` and `FaceReady` extend `StoredEyes` (`eyes.ts`: `eyes_ear`,
  `eyes`, `eyes_closed`, `pose`), `main.ts`'s `Focus` has the same fields,
  and `applyFaceReady` already patches them (with a test). So Step 4 added
  no field; `goodPhoto` reads what is there.
- **The dump matches the index; no CLI change.** The app's index
  (`%LOCALAPPDATA%\com.minodisk.riffle\index.sqlite`, read from a copy with
  Python's `sqlite3`, no `sqlite3` binary here) held stored eyes for one
  folder only, `2026-09-27-c`. Its `features` dump agreed with the index on
  all 1815 files (`eye_focus`, `eyes_ear`, yaw / pitch / roll), so the
  dump's eyes path picks the cue's face and the six re-dumped folders are
  what the app would store. The check was on every file of that folder
  rather than 20.
- **The re-dump.** Six folders (`2026-09-19`, `2026-09-27-a`, `2026-10-03`,
  `2026-07-11`, `2026-08-08`, `2026-07-24`), 9868 ARWs, 24 threads,
  about 4.3 min of CLI time, no error, under
  `D:\Photos\tests\2026-10-09-good-mark\dump\`;
  [provisional.py](provisional.py) reads them.
- **The plan's starting cuts barely narrow.** The mesh `eye_focus` is
  saturated (p50 0.940, p75 0.997), so `candidate` is 87.4% of the faced AF
  frames, and `eye_focus` >= 0.9 / EAR >= 0.2 / |yaw| <= 60 / |pitch| <= 45
  still marks 41.8%. The provisional cuts are `eye_focus` >= 0.998 (p79),
  EAR >= 0.30 (p54, open probability 0.991), |yaw| <= 60, |pitch| <= 45:
  9.0% of the faced AF frames, so the loose pose form stays. Note the EAR
  of 0.2 the plan called saturated reads only 0.861 open (`EYES_LOGIT_SLOPE`
  29); the probability is near 1 from about 0.25 (0.964). Table and
  reasoning in [provisional.md](provisional.md).
- **Before / after (share of the faced AF frames marked by the strip icon,
  from the re-dump and the cuts).** Before is `candidate`, after is
  `goodPhoto`:

  | Folder | Faced AF | Before | After |
  | --- | ---: | ---: | ---: |
  | `2026-07-11` | 441 | 98.4% | 7.9% (35) |
  | `2026-07-24` | 173 | 98.3% | 16.2% (28) |
  | `2026-08-08` | 367 | 95.9% | 7.9% (29) |
  | `2026-09-19` | 1952 | 86.9% | 13.4% (261) |
  | `2026-09-27-a` | 4367 | 84.3% | 6.6% (288) |
  | `2026-10-03` | 1615 | 90.0% | 9.9% (160) |
  | Total | 8915 | 87.4% | 9.0% (801) |

- **A dozen `2026-09-19` files that keep the icon** (a seeded draw of the
  261, `provisional.py`): `_DSC1853.ARW` (pick), `_DSC1966.ARW`,
  `_DSC2022.ARW`, `_DSC2572.ARW`, `_DSC2778.ARW`, `_DSC2780.ARW` (pick),
  `_DSC2836.ARW`, `_DSC2880.ARW`, `_DSC2944.ARW` (pick), `_DSC3361.ARW`
  (pick), `_DSC3495.ARW`, `_DSC3598.ARW`. `_DSC1966` (pitch 39) and
  `_DSC2022` (yaw -38, pitch 30) are the most turned of them.
- **Display: option (b).** `focusMark` returns `state: MarkState` (`good` /
  `candidate_only` / `not_candidate` / `unknown`) in place of `candidate`;
  `FOCUS_MARK_COLORS` is keyed by it. `candidate_only` is a dim green,
  `#8b8`: a focus candidate still reads as "eyes sharp" and stays apart from
  white (unknown), while only `good` is the bright `#3f3`. Since 87% of the
  faced frames are candidates, most crosshairs are now dim green.
- **The rename went to the CSS class too.** `strip.ts` `candidates` /
  `setCandidate` / `paintCandidate` / `cell.candidate` became `goods` /
  `setGood` / `paintGood` / `cell.good`, and `main.ts` `applyCandidates`
  became `applyGood`; the cell's `span.candidate` became `span.good`, which
  changed its three `style.css` selectors and their comment (no new
  token). The filter menu's `Sharp` face icon reads
  `FOCUS_MARK_COLORS.good`, so it looks as before.
- **No `docs/humans` or `README` change in this step**: the cuts are
  provisional; the Focus mark paragraphs wait for Step 6.

## Step 4b

- **The on-demand payload lacked the EAR.** `EyesJudgment` (`commands.rs`)
  carried only the state, the closed probability, the pose and the mesh, and
  `riffle_core::eyes::Judged` had no EAR either. Added `ear: f64` to `Judged`
  (filled from the `more_closed_ear` `judge_mesh` already computes) and the
  `ear` field to `EyesJudgment` and the frontend `Eyes`. The MCP `get_view`
  summary is a `Pick` of `state` / `probability` / `pose`, so it is unchanged.
- **`eyesOpenness` lives in `focus.ts`** next to the good-photo cuts, with
  `EYES_CLOSED_EAR` = 0.137 (mirroring core) and `EYES_WIDE_OPEN_EAR` = 0.41,
  the 90th percentile (0.4076) of the EAR over the 7261 faced AF frames with
  an EAR in the Step 4 re-dump, rounded. It returns the unrounded value; the
  meta pane rounds.
- **The row format.** The label is now `Eyes` (not `Eyes open`, which read
  wrong next to `Closed`), the value `Open · 41` / `Closed · 0`: the stored
  `eyes` state (or the judgment's `state`) then the integer openness. With
  the 0.41 anchor: EAR 0.09 (closed) -> `Closed · 0`, 0.25 (half open) ->
  `Open · 41`, 0.30 (`GOOD_EYE_EAR`) -> `Open · 60`, 0.45 (wide open) ->
  `Open · 100`. An EAR just above 0.137 shows `Open · 0`; that is the
  boundary, not a contradiction.
- **The stored path keys on `eyes_ear`, not `eyes_closed`.** `FocusCue` now
  carries `eyes_ear` and `eyes` (both already in `main.ts`'s `Focus`) instead
  of `eyes_closed`, which the meta pane no longer reads.
- **Line endings.** Editing `eyes.rs` / `commands.rs` through Python's
  `write_text` on Windows turned them CRLF; `sed -i 's/\r$//'` restored LF.
  Write bytes (`write_bytes`) instead.

## Step 4c

- **The second tier is `Fair`** (`PhotoTier = "good" | "fair"`), the filter
  item `Fair` next to `Good`. Cuts in `focus.ts`: `FAIR_EYE_FOCUS` = 0.99
  (p67 of `eye_focus`), `FAIR_EYE_EAR` = 0.25 (p35 of the EAR, open
  probability 0.964, openness 41), `FAIR_MAX_YAW` / `FAIR_MAX_PITCH` = 60 / 45
  (the good tier's loose pose cut; the data gave no reason to change it).
  Good + fair mark 20.3% of the faced AF frames pooled (good 9.0%, fair
  11.3%); per folder 16.3-31.3% ([provisional.md](provisional.md), "The fair
  tier"). The candidates tried: 0.995 / 0.25 gave 18.1%, 0.98 / 0.25 21.8%,
  0.99 / 0.24 21.3%, 0.99 / 0.22 23.3%; 0.99 / 0.25 with |yaw| <= 45 19.4%.
- **`goodPhoto` is replaced by `photoTier`**, which returns `"good"` when the
  good cuts clear, else `"fair"` when the fair ones do, else `null` (a
  non-candidate is always `null`). The good-boundary tests stay, through a
  local `photoTier(...) === "good"`; a frame just below a good cut is now
  `fair`, which a new test pins.
- **Colors.** Fair is azure `#5af`, a new entry of `FOCUS_MARK_COLORS` (the
  focus mark's colors are an app-semantic exception of
  `docs/agents/ui-styling.md`, so no `style.css` token); it is used by the
  strip icon, the crosshair's new `fair` state and the filter's `Fair` icon.
  It keeps away from the green of good and the dim green `#8b8`, the orange
  `#f93` of `not_candidate`, the cyan `#3ff` of the detected faces
  (`FACE_MARK_COLOR`) and the yellow stars. The filter's `Sharp` icon moved
  from the good green to the dim green `candidate_only` (`#8b8`), which is
  what a focus candidate in neither tier draws in, so the bright green face
  now only ever means good. This resolves the Step 4 deferred item about the
  `Sharp` icon, removed from the list below.
- **The filter items share the `AF eye` group.** `data-candidate="good"` /
  `"fair"` go in the same `candidates` set (now `Set<AfEye>`, `AfEye =
  FocusCandidate | PhotoTier`) and are OR-ed with `Sharp` / `Soft` /
  `Unknown`, as the menu ORs the items of one section; `passes` takes the
  tier as a new last argument. So `Good` + `Sharp` is just `Sharp` (a good
  frame is a candidate). The click handler needed no change.
- **Strip renames.** `cell.good` / `setGood` / `paintGood` / `span.good` are
  now `tier` / `setTier` / `paintTier` / `span.tier`; the icon's color is set
  per paint from the tier. `main.ts`'s `applyGood` is `applyTiers`.
- **`provisional.py` takes the fair cuts** as four optional extra arguments
  and then prints the tier table. Writing it through a bash heredoc with
  `\n` inside a Python literal put a raw newline in the file; edit such
  lines with the Edit tool.
- No `docs/humans` change here (Step 6).

## Step 5

- **The user's feedback (2026-10-09)** is in the plan's Step 5 block: stars
  on a 60-frame sample and three 1-star good frames, each a different
  failure. The changes, each with its reason:
  - `GOOD_MAX_YAW` / `FAIR_MAX_YAW` 60 -> **30**: `_DSC2827` (yaw -49) was
    good on an EAR inflated by the turned, foreshortened eye. 30 is the
    widest cut before a turned 1-2 star sample frame comes in; 20-30 give the
    same sample means (grid in [provisional.md](provisional.md)).
  - New **`MAX_EYE_OFFSET` = 0.10**: `_DSC2638` (a baby lying down) had its
    mesh fitted upright off the face yet read EAR 0.36, yaw 3, AF eye 100%.
    Its mesh eyes sit 0.125 face sides from YuNet's eye landmarks; the
    sample's well-fitted tier faces are all under 0.09.
  - New **`MIN_EDGE_GAP` = 0.02**: `_DSC3345`, the AF face cut by the left
    edge. Its YuNet box is 1 px inside the edge (gap 0.008), so "extends
    outside the image" (gap < 0) alone would not have caught it; YuNet's
    box of a cut face covers only the visible part. (It also fails the eye
    offset, 0.352, since the mesh misfits a cut face.)
- **The data was not stored, so pass 2 now stores two measures**, not
  booleans, to keep both thresholds in `focus.ts`: `Cue::eye_offset` /
  `Cue::edge_gap` (from `EyeMeasures`, computed in `mesh_eye_measures`,
  which now takes the face) -> `files.eye_offset` / `files.edge_gap`
  (`SCHEMA_VERSION` 19, the columns at the end of `CREATE TABLE`, the v10 to
  v17 fixtures dropping them too, a new v18 migration test) and
  `FACES_VERSION` 8 so pass 2 re-runs once. `StoredEyes` (Rust and TS)
  carries them, so `Focus` and `FaceReady` serialize them and
  `applyFaceReady` patches them. `eye_region` now shares its grown box with
  `edge_gap` through `grown_bounds` (same windows; its tests pass
  unchanged). `riffle-cli features` gained the `eye_offset` and `edge_gap`
  columns before the timings.
- **The eye offset**: per eye the mean of the 16 eyelid contour points
  against YuNet's landmark, the pairing (straight or crossed) with the
  smaller worst distance, over the box's longer side. Pairing-free so the
  stored landmarks' left / right order on a quarter-turned preview does not
  matter. Over the re-dump it is broad (median 0.062, p75 0.111) and
  excludes 28.6% of the meshed frames, but only 7.6% of the frames the cuts
  alone would tier.
- **A first edge rule was too strict.** Testing the 1.25x face crop the mesh
  is fitted on (`FACE_CROP`) flagged two whole, large faces near the top
  edge on the sample (`DSC4884`, 4 stars; `DSC0148`, 4 stars), only hair
  outside. Measuring the box and the grown eye regions with a small margin
  instead keeps both (gaps 0.093 / 0.065).
- **`riffle-cli faces <file> <out.png>`** draws the whole preview with the
  boxes, which is what showed the cut face of `DSC3345` (stored orientation
  8: the face at stored y 1 is at the upright left edge); `crop` only shows
  the AF area.
- **The six-folder re-dump took 50 minutes this time** (`2026-10-03` alone
  2823 s against 105 s in Step 4, the other folders as before), probably the
  disk; nothing failed. Every column it shares with the Step 4 re-dump gives
  the same Step 4c tiers.
- **Re-scored sample**: mean stars good 3.90 / fair 3.69 / neither 2.43
  (were 3.20 / 3.25 / 2.40), Spearman 0.54 (was 0.29); one 1-2 star frame
  stays tiered (`2026-09-19__DSC1805`, fair). Re-dump: good 6.0%, fair 7.7%,
  13.7% together (was 20.3%), per folder 10.0-22.3%.
- **Step 5's checkbox stays unticked**: everything in its Done-when is met
  except the user's approval of Decision B, which is written as proposed.
- No `docs/humans` change (Step 6). `docs/agents/tauri-app.md` names the two
  new columns next to the v18 ones in its schema and `FACES_VERSION` notes.

## Step 5b

- **The user's star scale.** 1 = no subject at all, 2 = likely to be
  rejected, 3 and up = a pass. So the stars measure "is this a miss" at the
  bottom and taste above 3; the mark can only speak to the former.
- **Batch 2 (120 frames, drawn without regard to the tiers, seed 20261010,
  twelve folders; `D:\Photos\tests\2026-10-09-good-mark\samples-scored-batch2.tsv`,
  `scripts\score_batch2.py`).** At 4+ stars: good 22%, fair 55%, none 22%,
  so good / fair says nothing about quality. Passing (3+): good 9 / 9, fair
  9 / 11 (both misses 2 stars), none 64 / 100. Batch 1 at the Step 5 cuts:
  good 10 / 10, fair 12 / 13. Good and fair together 40 / 43, no 1-star
  frame.
- **The decision.** Fold the two into one `good` tier meaning "not a miss",
  at the Step 5 fair cuts (the good cuts were stricter on the eyes alone,
  the pose cut and exclusions shared, so the union is exactly the fair cuts),
  and re-tune on all 180 rated frames after the mesh roll correction.
- **`photoTier` kept its name** and returns `"good" | null` (`PhotoTier` is
  `"good"`), so `filter.ts`, `strip.ts` and `main.ts` keep their tier
  plumbing as is and only lose the `fair` entries; a boolean `goodPhoto`
  would have rewritten that plumbing for no behavior change, and the type
  leaves room for the re-tune to add a tier back.

## Deferred issues (todo candidates)

- **The app's `.dop` reader ignores a picked virtual copy.** Found in
  Step 1 (the inventory): `dop::read_flag` (`crates/core/src/dop.rs`
  `locate`, first `Items` entry only) reads `None` for 199 frames of
  `2026-06-05` and 228 of `2026-09-13-b` whose virtual copy (a later
  item) is picked in PhotoLab. Riffle therefore shows those frames
  unflagged, and `trash.rs` `collect_folder` would not treat them as
  picked. Whether Riffle should read any item's flag (and which item it
  writes) is a product decision; this plan only works around it in the
  label rule ([data.md](data.md)).
- **Nearly-all-exported DNG folders left out.** `2026-02-21` (258 of 259
  frames exported), `2026-03-21` (454 of 460), `2026-04-26` (111 of 112) and
  `2026-05-02` (17 of 18) carry one to six negatives each but were excluded
  as "`Output/` not clearly smaller than the RAW count" (the data-set
  decision). If Step 2 wants more `Output/`-labeled negatives, they can be
  dumped with `dump.sh`'s loop and added to the `dng-output` block. Basis:
  the Step 1 inventory ([data.md](data.md), Excluded folders).
- **The app's burst "best" mark compares different textures.** Found in
  Step 2 ([results.md](results.md), Reading): the sharpness cue that
  `crates/app/ui/src/sharpness.ts` `relativeSharpness` marks as the
  burst's best (Compare's green bar) ranks the user's picks only slightly
  above random (pairwise AUC 0.577 against 0.495, the first frame 0.653),
  and the hand check found sharp frames at 0.11-0.67 of the burst maximum,
  because the AF-region score follows the content under the AF point. If
  Step 3's Decision does not replace it, the mark's wording or the
  measurement (e.g. on the face only) may deserve its own todo item.
- **The head pose misreads a face behind a ball.** Hand check 16 of
  [results.md](results.md) (`D:\photos\2026\2026-07-05\_DSC2445.ARW`): a
  profile looking up behind the ball reads pitch -68 deg. Basis: Step 2
  hand check; related `crates/core/src/pose.rs`. Fits the pending review
  of the head-pose labels in `todo.md`.
- **Re-measure the face checks with the mesh `eye_focus`.** Found in
  Step 3 ([fit.md](fit.md), Reading, last bullet): the Step 1 dumps under
  `D:\Photos\tests\2026-10-08-burst-keep-score\dump\` were taken with a
  CLI built before `20261008-mesh-eye-focus` Step 3 (#733), so the burst
  keep-check fit used the eye-window `eye_focus`. If the burst check is
  revisited (Step 4's re-dump covers the frame-level mark), re-run
  [dump.sh](dump.sh) with a current `riffle-cli` and re-run
  [keep.py](keep.py) / [fit.py](fit.py) before trusting the frozen
  `eye_focus` cuts in [frozen.json](frozen.json) and
  [frozen-fail-check.json](frozen-fail-check.json). Related: `crates/core/src/candidate.rs`,
  `crates/cli/src/main.rs` (`features`).
- **Pending manual check (the user's, Windows app build of Step 4).** Open
  recent folders (at least `D:\photos\2026\2026-09-19`) after pass 2 has
  filled them; check that the strip's face icon is on few frames (about
  13% of the faced AF frames on `2026-09-19`), that the marked frames look
  like good photos (sharp open eyes, face toward the camera; start with the
  dozen files listed under Step 4), which marked frames should not be and
  which unmarked ones should, whether turned faces pass the loose pose cut
  (then the tight |yaw| <= 45), and that the crosshair is bright green on a
  marked frame, dim green on another candidate, orange and white as
  before. Step 4's checkbox was ticked on the automated criteria; this is
  the plan's separate manual-check item before Step 5.
- **The docs still describe `Eyes open: NN%`.** Step 4b changed the meta
  pane row to `Eyes` with `Open · NN` / `Closed · 0` (openness from the EAR);
  `docs/humans/usage.md` (lines ~207-278), `usage.ja.md`,
  `performance.md` (~578), `performance.ja.md` and `README.ja.md` still say
  `Eyes open` as a probability. The plan leaves that to Step 6.
- **The MCP `get_view` eyes summary carries no openness.** Step 4b kept its
  `state` / `probability` / `pose` fields as the plan said; whether it should
  carry the EAR or the openness is a follow-up. Related:
  `crates/app/ui/src/companion.ts` (`EyesSummary`), `crates/app/src/mcp.rs`.
- **Pending manual check (the user's, Windows app build of Step 4c).** On
  `D:\photos\2026\2026-09-19` after pass 2 has filled it. Expected: the strip's
  the face icon in bright green on good frames and in azure on fair ones
  (about 13% + 8% of the faced AF frames there), the crosshair is azure on a
  fair frame; the filter's `AF eye` section lists `Good` (green face) and
  `Fair` (azure face) above `Sharp` (now a dim green face), and checking
  `Good` or `Fair` alone narrows the strip to that tier's icons only. Step
  4c's checkbox was ticked on the automated criteria.
- **Pending manual check (the user's, Windows app build of Step 5).** Open
  `D:\photos\2026\2026-07-11` and `D:\photos\2026\2026-09-19` and let pass 2
  re-run (`FACES_VERSION` 8 re-runs it on every folder once; thumbnails stay).
  Expected: `_DSC2638`, `_DSC2827` (2026-07-11) and `_DSC3345` (2026-09-19)
  show no face icon and a dim green crosshair; faces turned beyond about 30
  degrees no longer carry the icon; the icons are on about 14% of the faced
  AF frames on both folders. Step 5's checkbox is not ticked (Decision B
  awaits the user's approval), and this check belongs with that approval.
- **The eye offset is high on many meshed frames.** 28.6% of the faced AF
  frames with a mesh sit above 0.10 (p75 0.111, p90 0.189), most of them
  outside the tiers anyway. Whether that is the mesh misfitting turned or
  small faces, or YuNet's landmarks drifting on them, was not looked at; if
  the offset is later used for anything else (e.g. trusting the EAR or the
  pose shown in the meta pane), sample those frames first. Basis: Step 5's
  re-dump (`D:\Photos\tests\2026-10-09-good-mark\step5\dump\`); related
  `crates/core/src/candidate.rs` (`mesh_eye_offset`).
- **One 2-star frame stays fair with nothing the rule sees.**
  `2026-09-19__DSC1805` (yaw 18.5, pitch -21.8, eye offset 0.037, edge gap
  3.68). If the user says why it scored 2, that may name a fourth exclusion.
  Basis: Step 5's re-scored sample ([provisional.md](provisional.md)).
- **Pending manual check (the user's, Windows app build of Step 5b).** Open
  `D:\photos\2026\2026-09-19` after pass 2 has filled it. Expected: the strip's
  face icon is bright green on every frame that had a green or azure icon at
  Step 5 (about 14% of the faced AF frames) and no icon is azure; the
  crosshair is never azure; the filter's `AF eye` section lists `Good` and
  `Sharp` with no `Fair`, and `Good` alone narrows the strip to the
  icon-bearing frames. Step 5b's checkbox was ticked on the automated
  criteria.
