<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# AF eye in-focus probability: a combined, validated cue score and an `AF eye` filter section

## Purpose

The focus candidate cue today is one number, the Laplacian variance of the
luma over the eye window (`lap`, "AF eye sharpness"), thresholded at 80. A
Marziliano-style mean edge width over the same window, combined with `lap`
by a logistic regression fitted on the 500 hand-labeled frames, separates
in-focus from off frames better and generalizes to new folders: on the
training set (5 folders, 406 faced labeled frames) AUC goes 0.816 -> 0.852
and precision at the threshold 92.5% -> 93.1% at the same 91.4% coverage; on
a held-out set (4 new folders, 400 frames, 342 in focus / 58 off) AUC goes
0.635 -> 0.754 (bootstrap difference +0.120, 95% CI +0.065..+0.179),
precision 88.3% -> 89.1%, coverage 93.0% -> 95.3%. Contrast- and
noise-normalized Laplacians were worse (AUC 0.70-0.79).

This work replaces the score with the combined one, shown to the user as an
in-focus probability (`AF eye in focus 87%`), re-runs the second scan pass
on existing rows, and turns the strip filter's single `Focus candidates`
item into an `AF eye` section with `Sharp` / `Soft` / `Unknown`. The
candidate / not-candidate / unknown state, the strip cell icon, the focus
mark color and the MCP companion keep working off the state. A second
(`Very sharp`) tier is out of scope.

Frozen model (`frozen.json` in this folder; copy the numbers, do not refit):

- `edge_w` = mean edge width over the eye window (px); `edge_w_rel = edge_w / window side`
- `logit = c + k1 * ln(lap + 1) + k2 * ln(edge_w_rel)`, `p = sigmoid(logit)`
- `c = -4.725633355883976`, `k1 = 0.6826458385175557`, `k2 = -1.1949836425055467`
- Candidate when `logit >= 1.2194` (p >= ~0.772); this keeps the
  old `lap >= 80` in-focus coverage on the training set. `frozen.json` has
  `1.2194865955352432`, the logit of the boundary frame `_DSC1686.ARW` from
  the reference CSV's rounded columns; at full precision that frame's logit
  is `1.21944427...`, so Step 1 lowered the threshold to `1.2194` (user
  decision; coefficients unchanged)

The validation code (`reference-metrics.rs` in this folder, a scratch crate
depending on `riffle-core`, not part of the workspace) holds the reference
`edge_width` and shows how `lap` / `edge_w_rel` derive from
`candidate::luma` + `candidate::eye_window`.

Labeled sample folders (XMP `xmpDM:good`: Pick = in focus, Reject = off),
all under `D:\Photos\tests\`:

- Training: `2026-06-05-focus-sample`, `2026-07-31-focus-sample`,
  `2026-09-13-a-focus-sample`, `2026-09-19-focus-sample`,
  `2026-09-19-focus-sample-2`
- Held-out: `2026-06-14-focus-sample`, `2026-07-18-focus-sample`,
  `2026-08-01-focus-sample`, `2026-08-22-focus-sample`
- Reserved for a post-implementation check: `2026-08-29-focus-sample`,
  `2026-09-13-b-focus-sample` (100 ARW each; the user is still labeling them)

## Decisions (made with the user)

- **Undefined edge width** (no Sobel local maximum at or above
  `max(p90, 16)`, or a window too narrow for the interior loops): the frame
  is `NotCandidate` with probability 0 (`Soft`, `0%`). A window with no
  gradient above the noise floor has no sharp edge, and a face the detector
  found is never hidden under `Unknown`. Measure how often it triggers on the
  sample folders and record it in `learnings.md`.
- **What the index stores**: the probability only, one `eye_focus REAL`
  column. A coefficient change is a `FACES_VERSION` bump (pass 2 only).
- **Old column**: the v16 migration drops `eye_sharpness` and adds
  `eye_focus` (not a rename), so a stale Laplacian value can never be read as
  a probability; rows are `Unknown` until pass 2 refills them.
- **Candidate comparison**: compare the logit against `CANDIDATE_LOGIT` (`1.2194`)
  directly; derive `p` from the same logit for display and storage. The state
  read back from the index is derived from the stored probability against
  `sigmoid(CANDIDATE_LOGIT)`; make sure a stored `p` produced from a logit at
  or above the threshold reads back as `Candidate` (e.g. store and compare in
  a way that round-trips exactly, and test the boundary).

## Steps

- [x] Step 1: Compute the combined score and in-focus probability in `riffle-core`, and reproduce the validation numbers with `riffle-cli candidates`
  - Done when:
    - `crates/core/src/candidate.rs` computes, for the eye window of the
      nearest face, `lap` (unchanged `laplacian_variance`), the edge width
      (a faithful port of `edge_width` from `reference-metrics.rs`: Sobel
      `gx`/`gy` on the window interior, threshold = the window's 90th
      percentile of `max(|gx|, |gy|)`, at least 16; local maxima of `|gx|`
      along rows (`m >= t`, `m >= |gx(x-1)|`, `m > |gx(x+1)|`) and of `|gy|`
      along columns; walk both ways while the intensity stays strictly
      monotonic in the edge's direction, bounded by the window; width =
      distance between the extrema; rows and columns pooled into one mean),
      `edge_w_rel = edge_w / window.width`, the logit and the probability
      with the frozen coefficients, and the state from the frozen logit
      threshold. `Unknown` stays what it is today (no trusted AF point, no
      face near it, decode / detection failure).
    - The undefined-edge-width fallback (see Decisions) is implemented,
      unit-tested and documented in the doc comments.
    - `Cue` carries the probability (`eye_focus: Option<f64>`) and no longer
      has a bare `eye_sharpness` field with the old meaning.
    - The threshold constants' doc comments cite the new training / held-out
      numbers from Purpose instead of the old 500-frame numbers;
      `CANDIDATE_WINDOW_MIN`'s comment no longer cites "AUC 0.82" as the
      current figure without qualification.
    - `riffle-cli candidates <dir>... [threads]` prints per file the state,
      the probability (and lap / edge width), and, over the labeled files of
      all given folders pooled, AUC of `lap` alone and of the combined
      logit, and precision / coverage at the threshold; `riffle-cli faces`
      prints the new state and probability.
    - Running the CLI on the training folders and on the held-out folders
      reproduces the numbers in Purpose within rounding (AUC 0.816 / 0.852
      training, 0.635 / 0.754 held-out; precision 93.1% / coverage 91.4%
      training, 89.1% / 95.3% held-out), and the runs are recorded in
      `learnings.md`. If `2026-08-29-focus-sample` / `2026-09-13-b-focus-sample`
      carry labels by then, run them too and record the result; otherwise
      note it as a follow-up in `learnings.md`.
    - Unit tests: a synthetic step edge of known blur width gives the
      expected mean edge width; a flat window gives the fallback; the
      probability / state at the threshold (logit just below / at / above
      `1.2194`); existing `candidate.rs` tests updated.
    - `cargo test -p riffle-core -p riffle-cli` and `mise run ci` pass.
  - Implementation approach:
    - Port the arithmetic of `reference-metrics.rs` as is (integer Sobel on
      `u8` luma, `i32`, `mags.len() * 9 / 10` index, `.max(16)`, the `<` /
      `<=` asymmetry in the local-maximum test, the `w.x` / `w.x + w.width`
      walk bounds); the frozen coefficients only carry over if the numbers
      match.
    - Keep `luma`, `eye_window`, `nearest_face`, `CANDIDATE_WINDOW_MIN` and
      `focus_cue` / `focus_cue_unless` (including the cancel points) as they
      are; only the scoring after the face is picked changes.
      `sharpness::laplacian_variance` is untouched (pass 1 uses it).
    - Suggested shape: replace `CANDIDATE_THRESHOLD: f64 = 80.0` with the
      frozen constants (e.g. `LOGIT_INTERCEPT`, `LOGIT_LAP`,
      `LOGIT_EDGE_WIDTH`, `CANDIDATE_LOGIT`), a
      `pub fn edge_width(gray, width, window) -> Option<f64>` (`None` when no
      edge qualifies), a `pub fn eye_focus(gray, width, height, face) -> EyeFocus`
      (or similar) holding `lap`, `edge_width`, `edge_width_rel`, `logit`,
      `probability`, and a state function that compares logits. `Cue` gains
      `eye_focus: Option<f64>` (the probability); name nothing
      `eye_sharpness` any more.
    - `crates/cli/src/main.rs`: `candidates` currently takes one dir; extend
      it to several dirs (the trailing numeric argument stays the thread
      count) so the sets can be pooled, and add the AUC (the pairwise rank
      formula from the reference `auc`, ties 0.5) of `lap` and of the logit.
      The AUC skips frames whose edge width is undefined, as the reference
      did; precision / coverage include every labeled faced frame. Keep the
      `faces` subcommand output line in step with the new fields.
    - Doc comments in `candidate.rs` and the module doc describe the combined
      score; put the validation numbers next to the constants.
    - If a number is off by more than rounding, the port differs in
      `edge_width` (percentile index, the `<` / `<=` pair, the walk bounds);
      do not adjust the coefficients.

- [x] Step 2: Persist the probability in the index, re-run the second pass on existing rows, and show `AF eye in focus NN%` in the meta pane
  - Done when:
    - `crates/app/src/index.rs` stores the probability in `eye_focus REAL`;
      `SCHEMA_VERSION` is bumped with an in-place migration that drops
      `eye_sharpness`, adds `eye_focus`, and keeps `files`, `ratings` and
      `folders`; `FACES_VERSION` is bumped to 3 (Step 1 already bumped it to
      2 for the probability switch) so every row's second pass re-runs on
      the next open. `EXTRACTOR_VERSION` is not bumped (pass 1 output is
      unchanged; pass-1 latency must not grow).
    - `Focus` and `FaceReady` (the `faces-progress` payload) serialize the
      probability as `eye_focus` and `candidate` as today; `faces_todo` /
      `write_faces` / `run_faces_scan` / `indexed_file` use the new column;
      the schema-migration tests and the `eye_sharpness_round_trips` test are
      updated and a v15 -> v16 migration test added (a stale
      `eye_sharpness` value does not survive as `eye_focus`).
    - `crates/app/ui/src/main.ts` `Focus` and `crates/app/ui/src/meta.ts`
      `FocusCue` carry `eye_focus`; `metaGroups` emits
      `["AF eye in focus", "87%"]` (`Math.round(p * 100)` + `%`) after
      `Sharpness` and no row when the value is null; `meta.test.ts` updated.
      Check the `get_photo` description in `crates/app/src/mcp.rs` and any
      test fixture in `companion.test.ts` / `mcp.test.ts` that spells
      `eye_sharpness`.
    - `docs/agents/tauri-app.md` "FACES_VERSION" bullet still describes the
      mechanism correctly.
    - `cargo test -p riffle-app`, `pnpm exec vp test` and `mise run ci` pass;
      opening an already-indexed folder re-runs only the faces pass and the
      marks / meta row fill in.
  - Implementation approach:
    - Assumes Step 1 is merged.
    - Follow the v14 -> v15 pattern in `Index::open` (the
      `ALTER TABLE files ADD COLUMN ...` / `DROP COLUMN` block) and the
      `SCHEMA_VERSION` doc comment's per-version history.
    - `indexed_file` reads by column index; keep the `INDEXED_FILE` column
      list and the indices in step.
    - `FaceReady` is what `applyFaceReady` in `crates/app/ui/src/faces.ts`
      applies to `entries`; update its type and test (`faces.test.ts`).
    - Frontend `Focus.candidate` stays `"candidate" | "not_candidate" | "unknown"`,
      derived in Rust from the stored value, exactly as today.

- [x] Step 3: Replace the `Focus candidates` filter item with an `AF eye` section: `Sharp` (icon) / `Soft` / `Unknown`, OR-ed
  - Done when:
    - `crates/app/ui/index.html`: after the orientation `<hr />`, a
      `<div class="heading">AF eye</div>` and three `menuitemcheckbox`
      buttons `data-candidate="candidate"` (`Sharp`),
      `data-candidate="not_candidate"` (`Soft`), `data-candidate="unknown"`
      (`Unknown`); the `Focus candidates` button is gone.
    - `Sharp` shows the green Lucide `scan-face` icon before its text; `Soft`
      and `Unknown` show no icon but their text aligns with `Sharp`'s.
    - The SVG and its `/*!` Lucide notice live once: `SCAN_FACE_SVG` is
      exported from `crates/app/ui/src/strip.ts` and injected by `main.ts` at
      startup (or the `<svg>` is inlined in `index.html` with a short comment
      pointing at the notice in `strip.ts` / `LICENSE-lucide`; pick one, do
      not copy the full notice).
    - Checking several of the three ORs them; `filter.test.ts` gains cases
      for `not_candidate`, `unknown` and a two-state selection (`unknown`
      also passes a file whose state is not computed yet, i.e. `undefined`).
    - `Reset` clears the section; the toggle's `active` state counts it; the
      pass-2 refilter (`faces-progress` handler) keeps working for `Soft` /
      `Unknown` as files move out of `unknown`.
    - `crates/app/ui/style.css`: the icon sizing / alignment rule under the
      existing `#filter-menu` block; no change to `.cell span.candidate`.
    - `mise run ci` passes; checked by hand in the app.
  - Implementation approach:
    - The `filterItems` selector, the `aria-checked` sync and the click
      handler in `main.ts` already key on `data-candidate` and
      `shownCandidates`; verify nothing assumes a single candidate item.
    - Alignment: mirror the `.dot` pattern, e.g. every `[data-candidate]`
      button starts with `<i class="face"></i>`, `Sharp`'s holding the SVG,
      colored from the same source as the strip icon
      (`FOCUS_MARK_COLORS.candidate` in `focus.ts`) rather than a second
      literal.
    - The heading reuses `#filter-menu .heading` (the EXIF groups' menu
      builds the same `hr` + `div.heading` sequence); `#filter-exif` stays
      after this section.

- [x] Step 4: Update the user-facing docs
  - Done when:
    - `README.md` and `README.ja.md` (in sync, same PR): the Focus mark
      bullet says the meta pane shows `AF eye in focus` as a percentage and
      the filter menu's `AF eye` section (`Sharp` / `Soft` / `Unknown`)
      narrows by the state; the cue is described as combining eye sharpness
      and edge width. The Lucide sentence mentions the icon also appears on
      the filter's `Sharp` item.
    - `docs/usage.md`: the Focus mark bullet describes the combined score
      and the ~77% probability threshold instead of "Laplacian variance ...
      at least 80"; the meta pane `Analysis` paragraph names
      `AF eye in focus` (a percentage); the Filter menu bullet lists the
      `AF eye` section with its three items and OR semantics.
    - `CLAUDE.md` Layout: `src/candidate.rs` description, the column name in
      the `index.rs` sentence, the meta pane sentence and the `filter.ts`
      parenthetical updated.
    - `docs/performance.md` "Focus candidate pass": one sentence noting the
      cue now also computes the edge width (with the measured pass-2 cost if
      it changed noticeably in Step 1's runs; otherwise say it did not).
    - `mise run ci` (lychee / prettier) passes.
  - Implementation approach:
    - Assumes Steps 1-3 are merged. Grep for `eye sharpness`,
      `AF eye sharpness`, `Focus candidates`, `eye_sharpness` across
      `README.md`, `README.ja.md`, `docs/`, `CLAUDE.md` (not
      `docs/plans/_archived/**`, which is history).

## Trade-offs and risks

- **Reproducing the numbers.** The reference pooled only frames with a face
  and a label and skipped non-finite edge widths from the AUC; the CLI must
  do the same. Precision / coverage at the threshold include every labeled
  faced frame.
- **Held-out AUC is modest (0.754).** The cue is better than before, not
  reliable on its own; the UI keeps calling it a candidate / probability.
- **Step granularity.** Step 2 changes the index schema and the JSON
  contract while Step 3 is HTML/CSS/filter only; keeping them apart keeps
  the schema review small. Docs land once in Step 4 after the wording is
  final.
- **Reserved folders.** If still unlabeled at Step 1, `learnings.md` carries
  the follow-up ("run `riffle-cli candidates` on `2026-08-29-focus-sample`
  and `2026-09-13-b-focus-sample` once labeled and record AUC / precision /
  coverage").

## Progress

- (2026-09-26) Step 1 complete
- (2026-09-26) Step 2 complete
- (2026-09-26) Step 3 complete
- (2026-09-26) Step 4 complete
