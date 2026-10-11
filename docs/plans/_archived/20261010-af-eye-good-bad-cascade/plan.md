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

# AF eye: Good / Bad / Unknown, the no-AF face, and the face-free sharpness fallback

## Purpose

The AF eye judgment (`crates/app/ui/src/focus.ts`, the `f` mark's color, the
strip icon and the filter menu's `AF eye` section) has four states today:
`good` (the photo tier), `candidate_only` (labelled `OK`: the AF eyes are in
focus but a good cut is missed), `not_candidate` (`Bad`) and `unknown`. The
user looked at the `OK` frames and found them mostly not good, so the
judgment becomes a three-way `Good` / `Bad` / `Unknown` (Step 1).

The judgment then reaches two groups of frames it never reaches today:

- **Frames with no trusted AF point** (manual focus, or a body that records
  none: every Leica M and Sigma BF file). Pass 2 already runs YuNet on the
  whole preview for them (`scan::whole_image_faces`) but only to place the
  sharpness window; the cue is `Cue::unknown()`. Step 2 runs the same face
  mesh, `eye_focus`, EAR, pose, `eye_offset` and `edge_gap` on the face it
  finds, so the same rule yields Good / Bad. Frames **with** an AF point
  and no face near it do not go here: the camera aimed at something else,
  and a face elsewhere would be a wrong subject.
- **Frames where no face is analyzed** (an AF point with no face near it;
  no AF point and no face found). Step 3 measures on the user's own
  folders whether the sharpness score (around the AF point, else the
  existing fallback) separates picks from rejects, by an absolute threshold
  and by a within-burst comparison; Step 4 ships a Good / Bad from it only
  if it does, else records that those frames stay Unknown.

`Unknown` keeps meaning: not yet analyzed, a JPEG-only (view-only) folder,
or a failure.

### What is known

- `Focus` (`crates/app/src/index.rs`, `indexed_file`) is built only when
  the row has `focus_w` / `focus_h`, i.e. the camera recorded an AF point.
  A Leica / Sigma frame has `focus: null` on the frontend, so `stripState`
  is `unknown` and `passes()` sees no candidate whatever pass 2 stores.
  A manual-focus Sony frame has a `Focus` with `manual_focus: true`;
  `stripState` and `passes()` already read its `candidate` / tier, only
  `focusMark` (the crosshair) returns `null` for it.
- `scan::extract_analysis_unless` (`crates/core/src/scan.rs`): with a
  trusted AF point, `candidate::focus_cue_unless` decodes the preview,
  detects faces in a `CATCH_CROP` square around the point, takes
  `nearest_face`, runs `scored_face` (the mesh via `eyes::mesh_of` when
  `meshes_face`, then `eye_focus`) and `cue_of`; without one it returns
  `Cue::unknown()` before any decode, and the scan runs
  `whole_image_faces` (`faces::detect_around(preview, orientation, None)`:
  YuNet at `WHOLE_INPUT` on a DCT-scaled decode, faces in stored
  coordinates) for `sharpness::score_preview` only.
- The on-demand `eyes_of` (`commands.rs` `read_eyes`) and the CLI
  `features` eyes path judge, without an AF point, the face
  `eyes::judged_face(faces, None)` picks: the largest face at or above
  `sharpness::FACE_CONFIDENCE` (0.8). `sharpness::chosen_face` (the score
  window) takes the highest-scoring confident face.
- `FACES_VERSION` = 8 (`index.rs`), `SCHEMA_VERSION` = 19. Every column
  Step 2 fills exists (`eye_focus`, `eyes_ear`, `pose_yaw` / `pose_pitch` /
  `pose_roll`, `eye_offset`, `edge_gap`). The rule for the bump is in
  `docs/agents/tauri-app.md`, "Bump `EXTRACTOR_VERSION`, not
  `SCHEMA_VERSION`, when extraction output changes".
- The frontend cannot tell "pass 2 ran and found no face" from "pass 2 has
  not run": both have `eye_focus: null`. Pass 2 writes `faces_extractor`,
  not serialized today.
- Ground truth for Step 3 (the `20261008-burst-keep-score` plan's
  `data.md`): the `.dop` flag is authoritative, the XMP flag counts without
  a `.dop`. The data set holds **58 rejects** in 27 ARW folders (50 in
  `2026-09-19`, 7 in `2026-09-13-b`, 1 in `2026-09-27-a`) and 1 in the two DNG sidecar folders;
  picks are plenty (about 5500). The dumps under
  `D:\Photos\tests\2026-10-08-burst-keep-score\dump\` carry per file the
  flags, `sharpness`, cue `state`, `af` / `noaf`, `cue_side`, `judged_side`
  and the capture time; the sharpness column is still current (the score
  has not changed since `FACES_VERSION` 5). The earlier plan's `fit.md`,
  "Without a faced frame": face-free bursts are kept 18.5% of the time,
  18.9% at the sharpest frame's absolute sharpness >= 200, 24.3% at >= 800
  (29% of them); face-free singles 7.7% picked, 8.1% at >= 200, no lift
  above 400. Those numbers are against non-picks (unlabeled), not rejects.
- `docs/plans/20261010-mesh-roll/` is archived (#759, #760); its
  follow-ups are `todo.md` items. Step 2 does not change
  `crates/core/src/eyes.rs`, `pose.rs` or `crates/cli/src/main.rs`.
- `docs/plans/20261008-burst-keep-score/` is complete and awaits its
  wrap-up; the usage / README paragraphs Step 1 edits are the ones its
  Steps 6 and 7 wrote (merged).
- `CLAUDE.md` is not edited by this plan; the edits it needs are listed
  under "CLAUDE.md follow-ups" below and repeated in `learnings.md` when a
  step makes them necessary.

## Steps

- [x] Step 1: Merge `OK` into `Bad`: the AF eye judgment becomes Good / Bad / Unknown
  - Done when:
    - `crates/app/ui/src/focus.ts`: `MarkState` is `"good" | "not_candidate"
      | "unknown"`; `candidate_only` is gone from `MarkState`,
      `FOCUS_MARK_COLORS` and `FOCUS_MARK_ICONS`; a focus candidate that is
      not good maps to `not_candidate` (gray `#999`, the bare `scan` icon,
      gray `f` mark). One exported pure helper (e.g. `afEyeState(focus):
      MarkState`, name free) is the single place the strip (`stripState`),
      the crosshair (`focusMark`) and the filter read the state from.
    - `crates/app/ui/src/filter.ts`: `passes()` lets the `not_candidate`
      item pass both a `candidate` (not good) and a `not_candidate` frame;
      `good` passes the tier only; `unknown` the rest. The `AfEye` type
      loses the `candidate` value if nothing else needs it. `filter.test.ts`:
      the `OK does not pass a good frame` test goes; "the four items
      partition the frames" becomes three; a new case pins that a focus
      candidate that is not good passes `Bad`.
    - `crates/app/ui/index.html`: the `data-candidate="candidate"` (`OK`)
      item is removed; `Good` / `Bad` / `Unknown` stay with their keys.
      `main.ts`: the icon-coloring loop over the `AF eye` items and the
      `drawFocusMark` comment follow; `strip.ts` needs no change beyond the
      type. `icons.ts`: `SCAN_EYE_SVG` is removed if nothing else uses it.
    - `focus.test.ts`: `FOCUS_MARK_COLORS` / `FOCUS_MARK_ICONS` / `stripState`
      / `focusMark` tests pin the three states; a candidate that is not
      good is `not_candidate`.
    - Docs, each pair in sync in the same PR: `docs/humans/usage.md` /
      `usage.ja.md` (the **Focus mark** paragraph: bright green means good
      only; gray means a face was found at the AF point but the frame is
      not good, the in-focus probability below the cut included; the strip
      icon sentence loses `scan-eye`; the **Filter menu** bullet: `Good` /
      `Bad` / `Unknown`, "each file falls under exactly one", the "`Good` and
      `OK` together" sentence goes), `README.md` / `README.ja.md` (the
      **Focus mark** bullet, the `AF eye` item list, the **Offline face
      detection** bullet's Lucide icon sentence: `scan-face` and `scan`
      only). `rg "scan-eye|\bOK\b"` over `README*.md`, `docs/humans`,
      `crates/app/ui` finds nothing about the AF eye.
    - `todo.md`, the item "App: real-device check of the AF eye Good / OK /
      Bad / Unknown icons and mark colors": retitled to Good / Bad /
      Unknown, its first checkbox reworded to the three states (bright green
      `scan-face` on good, gray `scan` on bad, none on unknown; the `f`
      mark bright green / gray / white); its second checkbox (the
      `CLAUDE.md` `src/focus.ts` sentence) stays and is referenced from
      "CLAUDE.md follow-ups" below.
    - `mise run ci` passes.
  - Implementation approach (as far as it is known):
    - Frontend and docs only. No backend change: `index.rs` keeps
      serializing `candidate` as `candidate` / `not_candidate` / `unknown`,
      and `photoTier` keeps requiring `candidate === "candidate"` (the
      0.772 floor stays inside the good rule). Keeping `candidate_only`
      internally is not needed once the helper exists.
    - Do not touch the good cuts, `crates/core`, or `crates/cli`.

- [x] Step 2: Judge a frame with no trusted AF point on the largest face YuNet finds on the whole preview
  - Done when:
    - `crates/core/src/candidate.rs` exposes the cue of a given face (a
      refactor of `scored_face` + `cue_of` into something like
      `face_cue_unless(rgb, gray, width, height, orientation, &face,
      detection, cancel) -> Option<Cue>`, name free), and
      `scan::extract_analysis_unless`'s no-AF branch runs it on the face
      `eyes::judged_face(&faces, None)` picks from `whole_image_faces`
      (the largest face at or above `sharpness::FACE_CONFIDENCE`), after
      one full-size `decode_rgb` + `luma`; no face, a decode or mesh
      failure, or a panic keeps `Cue::unknown()` as today. The with-AF
      branch is unchanged, including "AF point, no face near it" staying
      `Unknown`. `Cue::face` / `detection` are filled so the `features`
      CLI's `cue_side` column shows the face. Unit tests in `scan.rs` /
      `candidate.rs` cover: a no-AF fixture with a face gets a state and
      `eye_focus`; without a face it stays `Unknown`; cancellation between
      the new stages returns `None`.
    - `crates/app/src/index.rs`: `FACES_VERSION` 9 with a doc sentence
      ("`9` fills the cue, the EAR, the pose, the eye offset and the edge
      gap for frames with no trusted AF point from the largest confident
      face of the whole preview"), so existing rows get re-analyzed. No
      `SCHEMA_VERSION` change. `Focus` carries the cue without a point:
      `point: Option<FocusPoint { sensor_w, sensor_h, x, y, frame }>` with
      `manual_focus`, `eye_focus`, `candidate` and the flattened
      `StoredEyes` beside it, built for every RAW row (a row with no
      recorded point gets `point: None`). The migration / serialization
      tests follow (`the_af_frame_and_manual_focus_round_trip`, the JSON
      shape tests).
    - Frontend: `main.ts`'s `Focus` and `focus.ts`'s `MarkFocus` mirror the
      new shape; `focusMark` returns `null` when `point` is `null` or
      `manual_focus` (no crosshair, as today); `stripState`, the Step 1
      helper, `photoTier` and `passes()` read the cue regardless of the
      point, so a Leica / manual-focus frame gets the strip icon, the
      filter state and the meta pane's `AF eye in focus` / stored `Eyes` /
      `Head pose` rows. `applyFaceReady` no longer skips a file whose
      `focus` has no point (today it skips `focus === null`). Tests pin a
      no-point focus: no mark, a strip state and a tier from its values.
      `meta.ts` needs no logic change (it keys on `eyes_ear`); verify its
      test for the stored no-AF case.
    - `eyes_of` / `faces_of`: unchanged in behavior; the plan records that
      the on-demand face (`judged_face`) is the same face pass 2 judged, so
      the mesh overlay and the stored rows agree. If a check on
      `D:\photos\2026\2026-02-01` (Leica DNG) and a manual-focus ARW shows
      the stored EAR / pose differ from the on-demand ones for the same
      file beyond rounding, record why in `learnings.md`.
    - Cost: `riffle-cli features` (or `scan`) over `D:\photos\2026\2026-02-01`
      and `2026-09-05` before and after, one thread, per-file ms in
      `learnings.md`; the `todo.md` item "Core: the no-AF-point path
      decodes the preview a second time in score_preview" is updated to
      say the path now decodes three times (scaled for YuNet, full RGB for
      the mesh and the eye regions, grayscale for the score) and that
      scoring from the RGB decode is the follow-up.
    - Docs, pairs in sync: `docs/humans/usage.md` / `usage.ja.md` (**Focus
      mark**: a frame with no AF point or in manual focus is judged on the
      largest face Riffle finds anywhere on the preview, so it can be good
      or bad although it shows no crosshair; "white when Riffle does not
      know" becomes: no face found (near the AF point when there is one,
      anywhere when there is none), under about 60 px, or not computed
      yet; the "never good" sentence drops the no-AF / manual-focus cases;
      the **Meta pane** and **Filter menu** `Eyes` text no longer lists
      "without an AF point" as an `Unknown` example), `README.md` /
      `README.ja.md` (the **Focus mark** bullet likewise),
      `docs/agents/tauri-app.md` (the `FACES_VERSION` paragraph: the cue
      covers the no-AF face too; `9` noted). `todo.md`: the item "Give the
      no-AF path a good-photo mark once it has a focus cue" is checked
      with this plan's folder in backticks; the "real-device check" items
      that expect the no-AF `Eyes` / `Head pose` rows from the on-demand
      path are reworded to the stored values.
    - `mise run ci` passes.
  - Implementation approach (as far as it is known):
    - Assumes Step 1 is merged (the three-state helper).
    - Files: `crates/core/src/candidate.rs`, `crates/core/src/scan.rs`,
      `crates/app/src/index.rs`, `crates/app/ui/src/{focus.ts,
      focus.test.ts, main.ts, meta.ts (verify), eyes.ts (types)}`,
      `docs/humans/usage*.md`, `README*.md`, `docs/agents/tauri-app.md`,
      `todo.md`. **Not** `crates/core/src/eyes.rs`, `pose.rs`, `faces.rs`
      or `crates/cli/src/main.rs` (another session may be in them; the
      new path uses `mesh_of`, `judged_face`, `detect_around` as they are).
    - Face choice: `eyes::judged_face` (largest confident face), for
      agreement with `eyes_of` and the CLI. Do not change which face
      `score_preview` uses (`chosen_face`) in this step.
    - The `f` mark draws nothing new on a no-point frame; the strip icon,
      the filter and the meta pane carry the judgment.
    - The good cuts in `focus.ts` are not re-tuned here; record that they
      were fitted on AF faces and add a fully specified `todo.md` item to
      check them on starred no-AF frames (the Leica folders' `.dop` picks,
      the same cuts, the share marked).

- [x] Step 3: Measure whether the sharpness score separates picks from rejects on face-free frames, and record the result
  - Done when:
    - A script in this plan folder (e.g. `facefree.py`, reading the
      `features` dumps) and `sharpness-fallback.md` here, with:
      - The face-free set, from the existing dumps under
        `D:\Photos\tests\2026-10-08-burst-keep-score\dump\` (and a re-dump
        with the Step 2 CLI of at least the two Leica sidecar folders and
        `2026-09-19`, saved under `D:\Photos\tests\2026-10-10-facefree\`,
        to confirm the grouping): frames with an AF point whose cue
        `state` is `Unknown` (no face near the point), plus frames with no
        AF point and no judged face (`noaf`, `judged_side` `-`). Counts per
        folder, picks, rejects, non-picks.
      - **Absolute threshold**: over the face-free frames, per folder and
        pooled, the pick precision and coverage at absolute sharpness >=
        t for t in {100, 200, 300, 400, 600, 800, 1000}, the AUC of pick vs
        reject (where rejects exist) and pick vs non-pick, held out by
        folder for the chosen t; the base rate beside each.
      - **Within-burst relative**: bursts by the rule of the
        `20261008-burst-keep-score` plan's `metrics.py` (1000 ms),
        face-free bursts of two or more frames only: the share of picks in
        the burst's sharpest frame / top half / at `sharpness / burst max`
        >= r for r in {0.5, 0.7, 0.8, 0.9}, pairwise AUC pick vs reject
        and pick vs non-pick, held out by folder. Singles are reported
        separately (no relative form applies).
      - The pre-registered criterion, written before the numbers:
        "separates well" means held-out AUC >= 0.70 (pick vs non-pick) and
        a cut whose precision is at least twice the base while it marks at
        least 20% of the face-free frames; the same on pick vs reject
        where a folder has 30 or more rejects.
      - A hand check of 20 face-free frames the best rule marks and 20 it
        fails (crops via `riffle-cli crop`, file names listed), so the
        user can look.
      - **Decision C** in this plan.md: ship an absolute rule, a relative
        rule, both, or none (those frames stay Unknown), with the numbers.
    - `learnings.md` records how many rejects the data actually held and
      which measure the Decision rests on.
    - `mise run ci` passes.
  - Implementation approach (as far as it is known):
    - Assumes Step 2 is merged (so "no AF point and a face found" frames
      leave the face-free set and the dump's `state` column tells it).
    - Reuse the `20261008-burst-keep-score` plan's `metrics.py` / `fit.py`
      for the burst rule and the held-out loop; no change to
      `crates/cli/src/main.rs` (the columns needed exist; derive anything
      else in the script).
    - Before starting, the caller asks the user (in Japanese) whether
      folders with real reject flags exist beyond `2026-09-19`; the
      unfinished `2026-09-13-a` / `2026-09-27-c` were excluded by the
      earlier plan and may hold rejects.
    - The user's answer (2026-10-10): the folder with real reject flags to
      use besides `2026-09-19` is `D:\Photos\samples\ARW\good-mark-2026-10-09`
      (20 rejects among its 180 rated frames). `2026-09-13-a` and
      `2026-09-27-c` hold no reject flag. Every good-mark frame has a face at
      its AF point, so the face-free set gains no reject from it (see
      [sharpness-fallback.md](./sharpness-fallback.md)).

- [x] Step 4: Ship the face-free Good / Bad from the sharpness score if Decision C says so, else record that those frames stay Unknown
  - Done when (ship):
    - `crates/app/src/index.rs`: `Focus` and `FaceReady` carry `analyzed:
      bool` (`faces_extractor == FACES_VERSION`), so the frontend tells
      "analyzed, no face" from "not yet"; no schema change; tests pin it.
    - `crates/app/ui/src/focus.ts`: the rule lives next to the good cuts
      as named constants (`FACEFREE_SHARPNESS` for an absolute cut, and /
      or the relative form reading `relativeSharpness` of `sharpness.ts`
      with the burst grouping of `burst.ts`), applied only when `analyzed`
      and `eye_focus === null` and `sharpness !== null`; the Step 1 helper
      returns `good` / `not_candidate` from it; `focus.test.ts` pins the
      boundaries, the `analyzed: false` case (unknown) and the
      `sharpness: null` case (unknown). `main.ts`: the relative form, if
      shipped, recomputes the states when `applySharpnessReady` reports a
      change, as the bars do today.
    - Docs, pairs in sync: `usage.md` / `usage.ja.md` and `README.md` /
      `README.ja.md` say what Good / Bad mean on a frame with no face
      (the score, the cut, the numbers from Decision C in one clause).
    - `mise run ci` passes.
  - Done when (no ship):
    - This plan.md's Decision C says so with the numbers; `usage.md` /
      `usage.ja.md` and `README.md` / `README.ja.md` say that a frame with
      no face stays `Unknown` and why (one sentence); `todo.md` gains a
      fully specified unchecked item (re-measure once labels exist or a
      cue other than the score is available) pointing at
      `sharpness-fallback.md`.
    - `mise run ci` passes.
  - Implementation approach (as far as it is known):
    - Assumes Step 3 is merged. The sharpness score stays computed and
      serialized in either outcome.
    - Files (ship): `crates/app/src/index.rs`, `crates/app/ui/src/{focus.ts,
      focus.test.ts, main.ts}`, docs. Files (no ship): this plan, docs,
      `todo.md`.

- [x] Step 5: Remove the sharpness displays: the strip's bar, the burst "best" mark and the meta pane's Sharpness row
  - Added 2026-10-10 at the user's request, after the burst-best-mark
    session's measurement (its copies of `results.md`, `measure.py` and plan,
    and fresh `features` dumps of the 36 burst-keep-score folders, are in
    `D:\Photos\tests\2026-10-10-burst-best-mark\`): the burst's sharpest
    frame lands on a good frame in 47.6% of the 1717 ARW bursts with one
    (random 45.8%, first frame 55.0%), pick AUC 0.577 (first frame 0.653);
    the score follows the texture under the AF point, not the subject's
    focus. Step 3 reads those dumps too, and Step 4 does not ship a
    within-burst relative form unless Step 3 overturns this.
  - Done when:
    - The strip draws no sharpness bar (`.sharpness` span, `paintSharpness`,
      `setSharpness`, the bar CSS), the compare view shows no `BEST` label or
      green outline, and the meta pane's Analysis group has no `Sharpness`
      row.
    - `comparisonCandidates` (`crates/app/ui/src/compare.ts`) pairs a lone
      file with its burst's good frame (`photoTier`), else the burst's first
      frame, not with the highest-sharpness frame; tests pin both cases.
    - `relativeSharpness` (`crates/app/ui/src/sharpness.ts`) and its tests
      are deleted unless Step 4 shipped a relative form that reads it; no
      dead code is left (`applySharpness` in `main.ts`, the `sharpness`
      timing field of `refresh.ts` if the measured call goes).
    - The score stays computed in pass 2, stored, serialized in `Focus` /
      `FaceReady` if Step 4 needs it, and returned by MCP `get_view` /
      `get_photo`.
    - Docs, pairs in sync: `README.md` / `README.ja.md`,
      `docs/humans/usage.md` / `usage.ja.md` and the other `docs/humans`
      pages that describe the bar, the `BEST` mark or the row
      (`cameras*.md`, `performance*.md`), and `docs/agents/tauri-app.md`'s
      examples, no longer describe them. `todo.md`: the item "App: the burst
      \"best\" sharpness mark ranks the user's picks near random" is removed.
    - `mise run ci` passes.
  - Implementation approach (as far as it is known):
    - Assumes Step 4 is merged (so it is known whether the score feeds the
      AF eye state).
    - Files: `crates/app/ui/src/{strip.ts, main.ts, compare.ts,
      compare.test.ts, meta.ts, meta.test.ts, refresh.ts, refresh.test.ts,
      sharpness.ts, sharpness.test.ts}`, `crates/app/ui/style.css`, docs,
      `todo.md`. No `crates/core` change.

## CLAUDE.md follow-ups (this plan does not edit `CLAUDE.md`)

- Layout, `src/focus.ts`: "puts the strip's face icon on a good frame" ->
  the per-state icon (`stripState` / `FOCUS_MARK_ICONS`), Good / Bad /
  Unknown (after Step 1; also the `todo.md` item's second checkbox).
- Layout, `crates/app` `run_faces_scan` ("the face mesh on the face nearest
  the AF point"), `eyes_of` ("the shown file's AF face") and
  `src/candidate.rs` ("the eyes of the face nearest the AF point"): add
  "else the largest confident face on the whole preview when there is no
  trusted AF point" (after Step 2).
- After Step 4, if it ships: `src/focus.ts` also holds the face-free
  sharpness rule.

## Trade-offs and risks

- **Step 1 naming (chosen: drop `candidate_only`, keep the data keys).**
  `MarkState` = `good | not_candidate | unknown`, the backend's `candidate`
  values and the `data-candidate` keys unchanged, the `OK` item removed.
  Renaming the keys to `good` / `bad` / `unknown` would be clearer but a
  wider diff.
- **Step 1 loses information.** The in-focus-but-not-good frames are no
  longer separable in the filter; the meta pane's `AF eye in focus`
  percentage still shows the probability.
- **Step 2 face choice (chosen: `eyes::judged_face`, the largest confident
  face).** It is what `eyes_of` and the CLI already judge, so the stored
  and the on-demand values agree and the mesh overlay sits on the judged
  face. `sharpness::chosen_face` (highest score) can differ on a frame. A
  group photo without an AF point is judged on one face only; the docs say
  so.
- **Step 2 `Focus` shape (chosen: an optional `point` inside `Focus`).**
  Keeps every `focus?.candidate` / `focus?.eyes` call site; `focusMark`
  returns `null` without a point. A separate `cue` field on the entry would
  touch every call site.
- **Step 2 cost.** One more full-size RGB decode plus the mesh per no-AF
  file. Measured and recorded; scoring from the same decode would change
  the score's pixels and is a follow-up.
- **Step 2 re-analysis.** `FACES_VERSION` 9 re-runs pass 2 on every row of
  every folder on its next open; thumbnails are kept. Parallel branches
  bumping the version take the next free number (tauri-app.md rule).
- **Step 2 cuts were fitted on AF faces.** A Leica face judged by the same
  cuts may mark more or fewer frames than on ARW; a check on the Leica
  `.dop` picks is a follow-up todo, not a blocker.
- **Step 3 ground truth is thin.** 58 rejects in the ARW data set and 1 in
  the DNG set; pick-vs-reject on face-free frames may have a handful of
  rejects. The step reports pick vs non-pick too (non-picks are unlabeled:
  failed, or fine and not chosen), pre-registers the criterion, and the
  caller asks the user for folders with more rejects. The earlier plan's
  face-free numbers (lift 1.0-1.3 over the base) suggest the likely
  outcome is "no ship"; the step still records it.
- **Step 3 criterion numbers** (AUC >= 0.70; precision >= 2x base at >= 20%
  marked) are fixed before the numbers are seen.
- **Step 4 and the on-hold removal of the sharpness bar and the Sharpness
  row.** If Step 4 ships, the score is an input of the AF eye state, so it
  must stay computed in pass 2 and serialized in `Focus` / `FaceReady`
  even when the bar and the row go; the relative form would keep
  `relativeSharpness` and `burst.ts` as rule inputs. If Step 4 does not
  ship, the removal is independent of this plan.
- **Step 4 needs `analyzed`.** Without it, a frame pass 2 has not reached
  would be judged on an absent score; the flag is a serialization change
  only.
- **Overlap with other sessions.** `crates/core/src/eyes.rs`, `pose.rs`
  and `crates/cli/src/main.rs` are not changed by any step.

## Decision C (the face-free sharpness fallback)

**None: frames where no face is analyzed stay `Unknown`.** Step 4 takes its
no-ship branch. Neither form meets the pre-registered criterion
([sharpness-fallback.md](./sharpness-fallback.md), [facefree.py](./facefree.py)),
on 4361 face-free frames (4059 with an AF point and no face near it, 302
without an AF point and no face found after Step 2) in 34 folders, 549 picks
(base 12.6%), 9 rejects:

- **Absolute:** held-out (folder-stratified) AUC pick vs non-pick 0.579
  (needed 0.70); the cut chosen on the other folders is `abs >= 200` for
  every folder, marking 51.4% at 14.7% precision, lift 1.17 (needed 2.0);
  no cut from 100 to 1000 reaches lift 1.25 in sample. Face-free singles:
  AUC 0.489, every cut below the base.
- **Within-burst relative:** within face-free bursts (244 bursts, 703
  frames, 53 picks) pairwise AUC 0.536, held-out `rel >= 0.9` lift 1.25;
  face-free frames of every burst against the whole burst (3970 frames)
  AUC 0.598, held-out lift 0.95.
- **Pick vs reject:** not testable to the criterion: no folder has 30
  face-free rejects (most 7, `2026-09-19`); on the 9 there are, AUC 0.663
  pooled, 0.622 within folder.

The score measures the texture under the AF point, not the subject's focus
(hand check: a pick at score 3.1 on a flat background, a non-pick at 1316 on
a sharp jersey), which agrees with the burst-best-mark result behind Step 5.

## Progress

- (2026-10-10) Plan written
- (2026-10-10) Step 1 complete
- (2026-10-10) Step 2 complete
- (2026-10-10) Step 5 added: remove the sharpness bar, the burst "best" mark and the meta pane's Sharpness row (user request after the burst-best-mark measurement)
- (2026-10-10) Step 3 complete
- (2026-10-10) Step 4 complete
- (2026-10-10) Step 5 complete
