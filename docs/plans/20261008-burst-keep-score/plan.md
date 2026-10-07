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

# Keep-candidate score within a burst

## Purpose

Riffle groups frames shot within `BURST_GAP_MS` = 1000 ms of each other into
a burst (`crates/app/ui/src/burst.ts`) and marks the sharpest frame of a
burst (`crates/app/ui/src/sharpness.ts` `relativeSharpness`, Compare's green
bar). Sharpness alone misses the other technical failures a culler drops a
frame for: the AF eye out of focus, closed eyes, the face turned away. Three
of those are measured today, but only the first is stored: `eye_focus` and
`sharpness` come from the scan's second pass (`run_faces_scan` in
`crates/app/src/index.rs`); the eyes-open probability and the head pose are
computed on demand for the shown file (`eyes_of` / `read_eyes` in
`crates/app/src/commands.rs`).

**What a pick means here (the user's framing, 2026-10-08).** A pick is a
frame that already passes the mechanical checks (in focus, eyes open, not
turned away); among the frames that pass, the human chose by composition,
expression and moment, which the machine does not judge. So every pick is a
reliable positive for "technically OK", while a non-pick is unlabeled: it
failed a check, or it passed and was simply not chosen. The score's job is
therefore to separate the technical failures of a burst from its
technically-OK frames, protecting every pick, not to predict which OK frame
the human picked. A score that fails a pick is wrong; a score that passes a
non-pick may be right.

This plan decides, by measuring against the user's picks under
`D:\photos\2026`, whether a score combining sharpness, AF eye in focus, eyes
open and head pose can flag the failures of a burst while keeping nearly all
picks, and ships it only if it does. Sharpness varies with lens and ISO, so
it is compared within the burst; a frontal face is not always wanted, so the
pose is used either as a penalty on extremes or as a difference from the
burst's other frames.

### What is known

- **Ground truth and the data set (decided by the user, 2026-10-08).**
  Picks are in sidecars: XMP `xmpDM:good="True"` (`xmp::read_flag`) and
  `.dop` `ShouldProcess = 0` (`dop::read_flag`), `sidecar_path` in each
  module; `crates/app/src/trash.rs` `collect_folder` is the precedent for
  reading flags off disk for a folder never opened in Riffle. Rejects are
  rare (50 in `2026-09-19`, 4 in `2026-09-13-b`, 1 in `2026-09-27-a`). The
  **`.dop` flag is authoritative; the XMP flag counts only for a frame with
  no `.dop`**. The DxO export subfolder `Output/` holds one JPEG per picked
  frame, named by the RAW stem, and its count equals the `.dop` pick count
  in every sidecar folder; **for the Leica folders of 2026-01 to 2026-05
  that have no sidecars, a frame whose stem appears in `Output/` is a pick**,
  and those folders are reported in rows separate from the sidecar-labeled
  ones. The data set is **every culled ARW folder except `2026-09-13-a` and
  `2026-09-27-c`** (both unfinished), the two Leica sidecar folders
  (`2026-08-29-l`, `2026-09-05`), and the `Output/`-labeled Leica folders;
  folders where every frame is picked (`2026-06-06`, `06-09`, `06-21`,
  `06-27`, `06-02`, and the Leica folders that exported everything) carry no
  negatives and are excluded. The inventory taken on 2026-10-08 and the
  final list are in `data.md`.
- **Features and where they come from.** `scan::extract_analysis(path)`
  returns `Analysis { cue: Cue { state, eye_focus, face, .. }, sharpness }`,
  exactly what the scan stores. The eyes path (`read_eyes`): read the
  preview, with a trusted AF point decode full size and
  `faces::detect_around_rgb`, else `faces::decode_whole` and
  `detect_whole_upright`; `eyes::judged_face(faces, point)` picks the
  AF-nearest face, else the largest at or above 0.8; `faces::upright_rgb`,
  `faces::face_to_upright`, then `eyes::judge_mesh(upright, uw, uh, &face)
  -> Option<Judged { eyes: Eyes { probability (closed), state }, points,
  pose: Option<Pose { yaw, pitch, roll }> }>`; `None` below
  `EYES_MIN_FACE` = 60 px. The CLI's `eyes_cmd` / `eyes_line` already print
  side, EAR, probability, state, yaw / pitch / roll per file; `candidates`
  prints state, p, lap, edge, face box and the XMP flag per file.
  `reader::read_metadata` gives `shot.capture_time` (`YYYY:MM:DD HH:MM:SS`)
  and `shot.subsec` (Leica DNGs have none).
- **Cost.** Face mesh ~35 ms per face single-threaded, the whole `eyes_of`
  call 80-92 ms per ARW and ~117 ms per DNG (`docs/humans/performance.md`,
  "Closed-eyes judgment on demand"); pass 2 (`riffle-cli candidates <dir>
  24`) ~10 s on the 2134-ARW folder. The data set holds about 32,000 ARW
  frames; a 24-thread dump of all four features is a few minutes.
- **Bursts in the app.** `groupBursts(paths, lookup, gapMs)` in `burst.ts`
  orders by capture time + subsec and splits at a gap over `gapMs`
  (inclusive, a missing subsec counts as 0 ms); the strip draws a band and a
  count badge; `relativeSharpness` marks the burst's maximum `sharpness` as
  `best`. A shipped score must be computed on the same grouping or its mark
  will contradict the band.
- **Concurrent plan.** `docs/plans/20261008-mesh-eye-focus/` (Steps 1-4
  open) measures the AF eye cue over the mesh eye regions and, if adopted,
  runs the face mesh and the pose solve in pass 2 for every faced file; its
  Step 4 lists "store the EAR and pose in the index" as a follow-up. Steps
  1-3 here are CLI and documents only and do not collide; Step 4 here is
  that follow-up and is shaped by that plan's Step 2 Decision (Trade-offs,
  "Ordering against mesh-eye-focus").
- **Related todo items** (`todo.md`, face/eye section): "Suggest the
  sharpest-eye frame within a burst group" (unchecked), "Flag looking-away
  frames from the head pose" (needs a threshold), and the user's pending
  review of the closed-eyes and head-pose labels.
- Samples and scratch outputs go to `D:\Photos\tests\2026-10-08-burst-keep-score\`
  (raw dumps, scripts' working copies, crops), not the repository. A copy of
  the analysis scripts is kept in this plan folder, as
  `20261008-mesh-eye-focus` keeps its fitting script, so the numbers can be
  reproduced from the saved dumps.

## Steps

- [ ] Step 1: Add `riffle-cli features` and dump every feature, the flags and the capture time per file for the data set
  - Done when:
    - `riffle-cli features <dir>... [threads]` (`crates/cli/src/main.rs`,
      registered in the `main` match and the usage string) prints one
      tab-separated record per RAW file of the given folders (not
      recursive, `scan::is_raw_file`, sorted), header line first, the format
      stated in a doc comment: folder, file name, `capture_time`, `subsec`,
      the XMP flag, the `.dop` flag (`Pick` / `Reject` / `None` / `-` when
      no sidecar, `err` when unreadable), `sharpness`, cue `state`,
      `eye_focus`, the cue face's box side in preview px, AF point present
      (`af` / `noaf`), the judged face's side (`-` when none), EAR, the
      eyes-open probability (`1 - Eyes.probability`, `-` when no judgment),
      `yaw`, `pitch`, `roll` (one decimal, `-` when none), and the per-file
      ms of the analysis and of the eyes path. Missing values are `-`,
      never blank, so a script can parse by column.
    - The analysis columns come from `scan::extract_analysis` (so they are
      what the scan stores); the eyes columns follow `read_eyes` step for
      step (the AF path decodes once for detection and crop; the no-AF path
      runs `decode_whole` + `detect_whole_upright` then a full decode;
      `judged_face`; `judge_mesh` on the upright face) so the dump equals
      what `eyes_of` would show. Both run on one rayon pool like
      `candidates`; errors print as a record with `err` in the failing
      columns and do not stop the run.
    - The flag columns use `xmp::read_flag` and `dop::read_flag` on
      `xmp::sidecar_path` / `dop::sidecar_path`; no new parsing. The script
      applies the label rule (Step 2), not the CLI.
    - A unit test pins one record line from synthetic inputs (the
      `eyes_line` test's shape) including the `-` cases; the real-folder
      run is manual.
    - The dump runs over every folder of the data set named in "What is
      known" and is saved as one `.tsv` per folder under
      `D:\Photos\tests\2026-10-08-burst-keep-score\dump\`; a listing of
      each folder's `Output/` stems is saved next to it.
    - `data.md` in this plan folder: the label rules as decided
      facts (`.dop` authoritative, XMP where no `.dop`, `Output/` stem for
      the sidecar-less Leica folders, reported separately), the data set
      and the excluded folders with the reason (unfinished, all picked, no
      label), and the inventory table: folder, format, frames, frames with
      a trusted AF point, frames with a cue face, frames with a judged face
      (>= 60 px), frames with a pose, picks by `.dop`, by XMP, by `Output/`,
      rejects, XMP / `.dop` disagreements. `learnings.md` records the
      dump's wall time and per-file ms distribution.
    - `mise run ci` passes.
  - Implementation approach:
    - Files: `crates/cli/src/main.rs` (new `features` function next to
      `candidates` and `eyes_cmd`), this plan's `data.md`, `learnings.md`.
    - No change to `riffle-core` or the app. Do not alter `candidates` or
      `eyes` output (other measurements depend on them).

- [ ] Step 2: Define the burst, the pick-protecting metrics and the baselines, and measure each feature and hand thresholds
  - Done when:
    - `metrics.py` in this plan folder (Python, standard library plus
      `numpy` if needed; reads only the Step 1 dumps and `Output/`
      listings): groups each folder's records into bursts with the rule of
      `burst.ts` (capture order, inclusive gap, missing subsec = 0 ms) at
      1000 ms (the app's), 2000 ms and 5000 ms; applies the label rule
      (pick = `.dop` Pick, else XMP Pick where no `.dop`; `Output/` stem
      for the sidecar-less Leica folders; everything else non-pick =
      unlabeled); and over the **scorable bursts** (two or more frames, at
      least one pick and at least one non-pick) computes, for every feature,
      threshold rule and score:
      - **Pick false-fail rate** (primary): the share of picks the rule
        marks as failing, pooled and per folder; should be near 0.
      - **Non-pick flag rate at a pick-keeping threshold** (primary): the
        threshold chosen on the pooled picks so that 99% (and 95%) of them
        pass, then the share of non-picks flagged, pooled and per folder,
        and the mean share of each burst's frames that remain (how much
        the burst narrows). Thresholds are **relative to the burst** for
        sharpness (score over the burst maximum; the absolute form is also
        reported) and **absolute** for `eye_focus`, the eyes-open
        probability and the pose axes (|yaw|, |pitch|, |roll|, and each as
        the absolute difference from the burst median as a second form).
      - **Pick position** (primary): the share of picks in the top half /
        top third of their burst by the score, and the distribution of the
        worst pick rank per burst (normalized (rank - 1) / (size - 1), 0
        best; no pick should sit near 1).
      - **Within-burst pairwise AUC** and **absolute AUC** (supporting).
      - **Top-1 hit rate** and **mean normalized pick rank** (secondary;
        the choice among OK frames is the human's, so these are noisy).
      - The counts behind every number (bursts, frames, picks, bursts
        dropped and why, frames without a face / judgment / pose), and how
        frames lacking a feature are treated (not flagged by that feature,
        as a second row ranked last).
    - Baselines on the same numbers: random, first frame, the app's current
      cue `sharpness` alone (relative to the burst maximum).
    - Hand threshold rules, at least: (a) relative sharpness below 0.7 /
      0.8 of the burst max fails; (b) `eye_focus` below
      `candidate_probability()` (the app's `Not a candidate`) fails; (c)
      eyes-open probability below 0.5 fails; (d) |yaw| over 45 deg or
      |pitch| over 30 deg fails (a second cut each); (e) any of a-d fails
      ("all checks"); (f) a-c only (no pose). A frame without a face is
      judged by (a) only, as the requirement says; the report says how many
      bursts are face-free, mixed and all-faced.
    - `results.md`: the burst statistics per gap (bursts,
      size distribution, bursts with one pick / several / none, per folder
      and pooled), every table above at the 1000 ms gap with the other gaps
      in a shorter table, in separate blocks for the sidecar-labeled ARW
      folders, the sidecar-labeled Leica folders and the `Output/`-labeled
      Leica folders, and a short reading of which checks fail picks, which
      narrow bursts, and what each adds over sharpness alone. A hand check
      of 30 flagged non-picks drawn at random (crops via `riffle-cli
      eyecrops` or the app, looked at and tallied as failed / OK / unsure)
      gives a rough precision the labels cannot; its tally is in
      `results.md`, the crops in the tests folder. Scripts' working copies
      and raw outputs go to `D:\Photos\tests\2026-10-08-burst-keep-score\`.
    - `mise run ci` (lint, lychee on the new files) passes.
  - Implementation approach:
    - Assumes Step 1 is merged and its dumps exist. Pure scripting and
      documents; no code in the workspace changes.
    - Report per-folder numbers next to pooled ones so `2026-09-27-a`
      (4830 frames) does not hide the rest.

- [ ] Step 3: Fit the thresholds and combinations from the picks with held-out folders and decide
  - Done when:
    - `fit.py` in this plan folder (the Step 2 feature matrix), each
      variant evaluated with **leave-one-folder-out** (thresholds or
      coefficients fitted on the other folders, metrics on the held-out
      one, then pooled) on the Step 2 metrics, the training fit alongside
      for reference:
      - (i) **Per-feature thresholds from the picks' distribution**
        (one-class on picks): each feature's threshold is the 1st / 2nd /
        5th percentile of the picks of the training folders (relative
        sharpness, `eye_focus`, eyes-open, |yaw|, |pitch|, |roll|); a frame
        fails if any feature is below (or beyond) it; reported with the
        resulting pick false-fail and non-pick flag rates held out.
      - (ii) **Positive-unlabeled logistic**: picks as positives, non-picks
        as unlabeled, a plain logistic on the Step 2 features (ln relative
        sharpness, the `eye_focus` logit, eyes-open, clipped |yaw| /
        |pitch| / |roll|, pose differences from the burst median,
        missingness indicators), its score thresholded at the 99% / 95%
        pick-keeping point; the unknown class prior is reported as a
        sensitivity (the ranking is unaffected by it, the calibration is).
      - (iii) The **pairwise within-burst** fit (logistic on feature
        differences of pick / non-pick pairs, no intercept) and (iv) the
        pointwise logistic, kept as alternatives for the position metrics.
      - A drop-one-feature run for the chosen variant, so each feature's
        held-out worth is on record.
    - `fit.md`: the tables, held-out numbers next to the best
      hand rule and the sharpness-alone baseline of Step 2, the per-folder
      spread, and the chosen variant's values at full precision in
      `frozen.json` (features, transforms, clip bounds,
      thresholds or coefficients, the face-free fallback, the gap).
    - A **Decision** section in this plan.md states, with the numbers:
      whether a combined check beats sharpness alone, i.e. flags clearly
      more non-picks at the same (near-zero, held-out) pick false-fail rate
      by more than the per-folder spread, and whether its hand-checked
      precision supports it; which variant and features; the gap; how
      face-free and mixed bursts are scored; and the proposed presentation
      (Trade-offs "Presentation"). Or: nothing beats sharpness, Steps 4-6
      are struck (marked so in Progress), the CLI dump stays as a
      diagnostic and Step 6's place is a docs-only PR recording the
      findings. **The user approves the Decision before Step 4 starts**
      (the conversation is in Japanese).
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 2 is merged. Keep it simple: percentile thresholds need
      no fitting library; the logistic variants are plain maximum
      likelihood (the AF eye fit's method), regularized only if a fit does
      not converge, and then said so. Scripts are copies in the plan
      folder, not workspace crates.
    - Head-pose labels are unreviewed (todo); if the pose carries the
      Decision, say so, since a label review could move it.

- [ ] Step 4 (gated on the Decision): Compute and store the eyes-open probability and the head pose in the scan
  - Done when:
    - The faces pass stores, next to `eye_focus` / `sharpness`, the judged
      face's eyes-open probability and `yaw` / `pitch` / `roll` (`NULL`
      when there is no judged face, the face is under `EYES_MIN_FACE`, or
      the pose solve failed) for every file it already analyzes;
      `SCHEMA_VERSION` 18 with the columns added in the migration and the
      history comment, `FACES_VERSION` bumped with its history line
      (`crates/app/src/index.rs`; `docs/agents/tauri-app.md` "The second
      pass has its own version" names the new values); `IndexedFile`,
      `FaceReady` and the `faces-progress` payload carry them;
      `crates/app/ui/src/main.ts` `entries` receive them.
    - The stored judgment uses exactly the on-demand path's face choice,
      crop and floor (`judged_face`, `judge_mesh`), so the meta pane's
      `Eyes open` and `Head pose` rows and the stored values agree; the
      rows keep coming from `eyes_of` (no behavior change there), and an
      `#[ignore]`d real-file test (env var path, as
      `times_eyes_of_on_real_files`) checks the stored probability equals
      `read_eyes`'s.
    - The second pass keeps its priority and `ScanFocus` order; the extra
      cost per faced file is measured and recorded in `learnings.md` for
      Step 6.
    - The closed-eyes constants, the crop, `pose.rs`, `eyes_of` and the
      focus candidate cue's logic and constants are untouched.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 3's Decision was approved. **Shape depends on the
      mesh-eye-focus plan**: if its Step 3 has merged, the mesh already
      runs in pass 2 (`focus_cue_unless`) and this step adds the
      EAR-derived probability and the pose to `Cue` / `Analysis` from the
      points already in hand (no second model run). If it was not adopted
      or has not merged, this step runs `judge_mesh` on the cue face in
      `extract_analysis_unless` after the cue, with a cancel point before
      it, and on the no-AF path on the largest confident whole-image face
      from a full decode (~14 ms per DNG more). Decide which at step start
      and record it.
    - Files: `crates/core/src/scan.rs`, `crates/core/src/candidate.rs` or
      `crates/core/src/eyes.rs`, `crates/app/src/index.rs`,
      `crates/app/src/commands.rs`, `crates/app/ui/src/main.ts`,
      `docs/agents/tauri-app.md`.

- [ ] Step 5 (gated on the Decision): Judge each burst's frames and present the result
  - Done when:
    - A new DOM- and Tauri-free module `crates/app/ui/src/keep.ts` with
      `keepJudgments(paths, lookup, bursts)` that applies the Decision's
      rule per file from the entry's `sharpness`, `eye_focus`, eyes-open
      probability and pose over the burst grouping of `burst.ts` (same
      `BURST_GAP_MS`), with the face-free fallback of the Decision, the
      constants copied from `frozen.json` and documented with the held-out
      numbers, returning per file whether it passes (and, if the Decision
      kept a ranking, its score and whether it is the burst's best);
      `keep.test.ts` covers a burst with a face on every frame, a face-free
      burst (sharpness only), a mixed burst, missing pose, a single frame
      (never flagged, no comparison), the threshold boundary and the tie
      rule.
    - The presentation the user chose in the Decision, one of: a
      "technically failed" mark or dimming on the strip cell within a burst
      (`strip.ts`, `style.css` with the shared tokens of
      `docs/agents/ui-styling.md`; dimming must stay distinct from the
      rejected dimming); a "keep candidates" mark on the frames that pass;
      a filter menu entry hiding failed frames (`filter.ts` and its test,
      `index.html`); or a single best mark driving `relativeSharpness`'s
      `best` and Compare's best frame (`sharpness.ts`, `compare.ts`, their
      tests). The Analysis group of the meta pane gets a row only if the
      Decision asked for it.
    - Recomputed where `bursts` and the sharpness cue are recomputed
      (`refreshEntries`, after `refilter`, `faces-progress` / `faces-done`),
      so the marks fill in as pass 2 lands.
    - `docs/humans/usage.md` (the burst and sharpness cue paragraphs) and
      `usage.ja.md` describe it: which checks, that they compare within a
      burst only, that frames without a face are checked on sharpness
      only, that a pass is not a recommendation (the choice among passing
      frames is the user's), the held-out pick false-fail and narrowing
      numbers in one clause; `README.md` / `README.ja.md` feature bullet if
      they name the sharpness cue (check; `README.md` does name the burst
      band and the cue).
    - Manual checks (GUI; list in `learnings.md` as pending if not run):
      on `D:\photos\2026\2026-09-19` no pick of a scorable burst is marked
      failed beyond those `results.md` lists; a face-free burst flags only
      soft frames; a Leica folder (`2026-09-05`) shows marks on the no-AF
      path; filtering out frames leaves the marks consistent with the
      remaining band (the `relativeSharpness` rule).
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 4 is merged. Keep the decision logic in `keep.ts` so
      Vitest covers it without mocks (the pattern of `burst.ts`,
      `sharpness.ts`). Do not change `BURST_GAP_MS` here unless the
      Decision chose another gap, and then change it once for both.
    - Files: `crates/app/ui/src/keep.ts`, `keep.test.ts`, `strip.ts`,
      `main.ts`, `filter.ts` / `sort.ts` / `sharpness.ts` / `compare.ts`
      per the chosen presentation, `style.css`, `docs/humans/usage.md`,
      `usage.ja.md`, `README.md` / `README.ja.md`.

- [ ] Step 6 (gated on the Decision): Measure the scan cost and bring the docs and the todo in line
  - Done when:
    - `docs/humans/performance.md`: a paragraph under "Focus candidate pass"
      with pass 2 before (the commit before Step 4) and after on
      `D:\photos\2026\2026-09-19`, 24 threads, alternated, four runs each,
      warm cache, Windows 11; the "Which pass carries which cost" table's
      closed-eyes row says the eyes-open probability and the pose are now
      stored by pass 2 while the meta pane's rows stay on demand; the app's
      `scan faces` line if the GUI is run (else say so); `performance.ja.md`
      in sync.
    - `CLAUDE.md` Layout: `run_faces_scan` names the new columns; the
      `src/eyes.rs` / `src/pose.rs` sentences no longer say the values are
      display-only; `keep.ts` is listed next to `sharpness.ts`.
    - `todo.md`: "Suggest the sharpest-eye frame within a burst group" is
      checked or rewritten to what shipped, with the plan folder in
      backticks and the held-out numbers; "Flag looking-away frames" notes
      the pose is now stored (threshold still unlabeled, or the Decision's
      cut if one shipped); fully specified follow-ups, unchecked: an `Eyes`
      / `Head pose` filter section now that the values are stored; the MCP
      companion field for the judgment; re-fitting after the user reviews
      the closed-eyes and head-pose labels; the hand check of flagged
      non-picks extended if the user wants a precision figure.
    - If the Decision struck Steps 4-5: this step is the docs-only PR that
      records the measured variants and the reason in `todo.md` (the
      "sharpest-eye frame" item stays unchecked with a pointer to
      `results.md` / `fit.md`) and leaves `performance.md` untouched.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Steps 4 and 5 (or the strike) are merged. A pre-change CLI can
      be built from a temporary `git worktree` under the scratchpad with
      `CARGO_TARGET_DIR` pointed at this worktree's `target`
      (mesh-eye-focus Step 4's note). Grep `sharpest`, `best frame`,
      `on demand`, `not stored` across `README*.md`, `docs/humans`,
      `docs/agents`, `CLAUDE.md`, `todo.md` (not `docs/plans/_archived`).

## Trade-offs and risks

- **Negatives are unlabeled.** A non-pick flagged as failed may be a good
  frame the user did not choose; no metric on these labels can tell. The
  plan therefore holds the pick false-fail rate near zero as the hard
  constraint, reports the non-pick flag rate as "how much the burst
  narrows" rather than as precision, and adds a small hand check of flagged
  non-picks for a rough precision. If the hand check shows many OK frames
  flagged, the Decision should prefer a presentation that does not hide
  frames (a mark over a filter).
- **Relative vs absolute sharpness.** Relative to the burst maximum, a
  burst whose every frame is soft still has an unflagged "sharpest" frame;
  absolute sharpness varies with lens, ISO and subject. Both forms are
  measured; a combination (relative, with an absolute floor) is allowed if
  the data supports it.
- **Pick-percentile thresholds assume no pick fails a check.** The labels
  say otherwise at the margin: children looking down read as closed eyes
  (`eyes-truth.md`), intentional profiles read as turned away, and the
  pick's AF eye can be `Not a candidate` when the camera tracked the wrong
  eye. So the 1st-5th percentile of the picks, not their minimum, is the
  honest floor, and the pose check may legitimately be dropped if it fails
  picks.
- **PU logistic class prior.** The share of technically-OK frames among
  the non-picks is unknown; the model's ranking does not depend on it, its
  calibration does, so it is reported as a sensitivity and the threshold
  is set on the picks (99% / 95% kept), not on a probability.
- **Burst gap (measured at 1, 2 and 5 s; shipped at the app's 1000 ms
  unless the Decision moves it).** A longer gap merges separate moments; a
  shorter one splits Leica bursts (1 s stamps). Changing `BURST_GAP_MS`
  changes the strip's bands for everyone, so a different gap for the score
  alone is not planned.
- **`Output/` as a label.** An export is a slightly different act from a
  pick (and the Leica folders that exported everything are excluded as
  having no negatives). Those folders are reported separately so the
  Decision can rest on the sidecar-labeled folders alone if the two
  disagree.
- **Ordering against `20261008-mesh-eye-focus` (recommended: run Steps 1-3
  now, start Step 4 after that plan's Step 2 Decision).** Steps 1-3 touch
  only the CLI and documents. Step 4 either rides on that plan's pass-2
  mesh (fields only) or brings the mesh into pass 2 itself (+~35 ms per
  faced file, the cost that plan is also weighing). Doing both
  independently would run the mesh twice per file. Shared files if both
  proceed: `candidate.rs`, `scan.rs`, `index.rs` (`FACES_VERSION`),
  `todo.md`, `performance.md`; a rebase, not a design conflict, if the
  order above is kept.
- **Scan cost and the on-display-only policy (user decision at the
  Decision, confirmed by Step 6's numbers).** Storing the eyes-open
  probability and the pose costs roughly 65 ms per ARW and 117 ms per DNG
  single-threaded if the mesh is not already in pass 2 (2-4 min for 2000
  frames on one core; pass 2 runs on all cores at lowest priority). The
  closed-eyes plan chose on-demand to avoid this; the Decision weighs the
  measured gain against it.
- **Presentation (user choice at the Decision).** (a) A "technically
  failed" mark or dimming within a burst: shows what the machine can judge
  and nothing more, but adds a state to a small cell already carrying the
  rejected dimming. (b) A "keep candidates" mark on the passing frames: the
  positive reading of the same set. (c) A filter hiding failed frames:
  narrows the strip fastest, but hides frames the user may have wanted
  (see "Negatives are unlabeled"). (d) A single best mark (driving the
  existing `best` of `relativeSharpness` / Compare): the smallest change,
  but it claims a choice among OK frames the framing says is the human's.
  The plan does not pre-decide.
- **Face choice.** The checks use the one judged face (AF-nearest, else
  the largest confident one); a burst whose pick is about a second person
  is judged on the wrong face. Reported as a limitation, not solved here.
- **Nothing may beat sharpness.** A valid outcome: the dump, the burst
  statistics and the tables remain, Steps 4-6 are struck, and the todo
  records why. The acceptance criteria are met by the recorded numbers and
  the Decision, not by shipping.

## Decision

(Written in Step 3.)

## Progress

- (none yet)
