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

# Keep-candidate score within a burst, then the "good photo" mark

## Purpose

Riffle groups frames shot within `BURST_GAP_MS` = 1000 ms of each other into
a burst (`crates/app/ui/src/burst.ts`) and marks the sharpest frame of a
burst (`crates/app/ui/src/sharpness.ts` `relativeSharpness`, Compare's green
bar). Sharpness alone misses the other technical failures a culler drops a
frame for: the AF eye out of focus, closed eyes, the face turned away. This
plan set out to measure, against the user's picks under `D:\photos\2026`,
whether a score combining sharpness, AF eye in focus, eyes open and head
pose can rank or narrow a burst, and to ship it only if it does.

**What a pick means here (the user's framing, 2026-10-08).** A pick is a
frame that already passes the mechanical checks (in focus, eyes open, not
turned away); among the frames that pass, the human chose by composition,
expression and moment, which the machine does not judge. So every pick is a
reliable positive for "technically OK", while a non-pick is unlabeled: it
failed a check, or it passed and was simply not chosen.

**Reframed by the user on 2026-10-08, during Step 3.** The mark does not
have to catch every pick; what is wanted is to narrow a burst to the frames
that clearly meet the minimum conditions, which should be almost entirely
picks (precision close to 1, recall secondary). Step 3 measured strict keep
rules by held-out precision and coverage ([keep.py](keep.py), [fit.md](fit.md)
"Keep mark").

**The user's second insight, 2026-10-08.** Within a burst the user also
throws away good frames depending on the timing, so which frame of a burst
got picked matters little; the measure moved to the burst (does a strict
rule mark the scenes that hold a pick; [scene.py](scene.py), [fit.md](fit.md)
"Burst level"). The answer, recorded in "Decision 1" below: the technical
features do not predict picks at the frame or the burst level (non-picks are
mostly technically fine too), burst length does, and the strip already shows
it. No burst mark ships.

**The user's third turn, 2026-10-09 (this plan's Steps 4-6).** Forget
bursts: "I just want to find photos that are not misses". And: the face
icon the strip draws on every focus candidate (the `f` focus mark's green,
`span.candidate` in `strip.ts`, the filter's `AF eye: Sharp`) is on so many
frames that it means nothing; it should become something much more
selective. The mark is to mean **"good photo"**: shown only when **all three
hold at once** on the AF face: **the eyes are open** (the more open the
better), **the face is toward the camera**, and **the AF eye is in focus**
(`eye_focus` high, sharp eyes). Sharpness is not part of it (Step 3 found
sharpness cuts do not hold up held out once `eye_focus` is present). On the
pose the user's preference is: **exclude only extreme turns (profiles still
pass) if the mark narrows enough; if it barely narrows, tighten to "at
least both eyes visible"** (roughly the yaw at which the far eye is still
seen). The target is high precision of "not a miss" among the marked
frames; marking few frames is acceptable.

The user prefers to **implement the mark once, with provisional thresholds,
and look at it in the real app before any labeling** (2026-10-09). So Step 4
builds it in the frontend from the values pass 2 already stores, with the
cuts in one place; the user then checks it on their folders; Step 5 tunes
the cuts from that feedback (a small labeled sample only if the user wants
numbers) and records Decision B; Step 6 writes the docs. The focus
candidate cue's own constants, the closed-eyes judgment and the head pose
computation stay as they are; the new rule only combines their stored
outputs.

### What is known

- **Ground truth and the data set of Steps 1-3 (decided by the user,
  2026-10-08).** Picks are in sidecars: XMP `xmpDM:good="True"`
  (`xmp::read_flag`) and `.dop` `ShouldProcess = 0` (`dop::read_flag`);
  the **`.dop` flag is authoritative; the XMP flag counts only for a frame
  with no `.dop`**; for the sidecar-less Leica folders a frame whose stem
  appears in the DxO `Output/` export is a pick. The data set is every
  culled ARW folder except `2026-09-13-a` and `2026-09-27-c` (unfinished),
  the two Leica sidecar folders and the `Output/`-labeled Leica folders;
  all-picked folders are excluded. Inventory and refinements (virtual
  copies, DeepPRIME DNGs) in [data.md](data.md).
- **What the index stores now (`origin/main` at #741,
  `20261008-mesh-eyes-index`).** Pass 2 runs the face mesh on the AF face
  for the cue (#733, `20261008-mesh-eye-focus`) and stores, next to
  `eye_focus` and `sharpness`, `files.eyes_ear` (the EAR of the more closed
  eye) and `files.pose_yaw` / `pose_pitch` / `pose_roll`
  (`crates/app/src/index.rs`, `SCHEMA_VERSION` 18, `FACES_VERSION` 7).
  `Focus` serializes `eye_focus`, the derived `candidate`, and a flattened
  `StoredEyes { eyes_ear, eyes (state), eyes_closed, pose }`; `FaceReady`
  (the `faces-progress` payload) carries the same. The mesh is skipped for
  faces under 60 px (#737), so those have no EAR or pose. The no-AF path
  (Leica, manual focus) has no cue: `eye_focus`, EAR and pose are all
  `None`. **Nothing new has to be computed or stored for the mark.**
- **What the frontend has.** `main.ts`'s `Focus` interface and `focus.ts`'s
  `MarkFocus` / `FaceReady` carry `eye_focus` and `candidate` only;
  `applyFaceReady` patches those two; the serialized `eyes_ear`,
  `eyes_closed` and `pose` fields arrive but are not typed or read. The
  strip icon: `strip.ts` `candidates: Set<number>`, `setCandidate(index,
  bool)`, `paintCandidate` toggles `cell.candidate` (Lucide `scan-face`,
  colored `FOCUS_MARK_COLORS.candidate`); `main.ts` `applyCandidates()`
  feeds it from `entries.get(path)?.focus?.candidate === "candidate"`. The
  crosshair color: `drawFocusMark` uses `FOCUS_MARK_COLORS[mark.candidate]`
  (green `#3f3` / orange `#f93` / white). The filter: `filter.ts`
  `candidates: Set<FocusCandidate>` from the menu's `[data-candidate]`
  items (`Sharp` / `Soft` / `Unknown`). The meta pane shows `AF eye in
  focus`, `Eyes open`, `Head pose` (`meta.ts`); `get_view` (MCP) exposes
  `eyes` and the pose (#739).
- **The CLI.** `riffle-cli features <dir>... [threads]` (Step 1) dumps per
  file the flags, `sharpness`, cue `state`, `eye_focus`, the cue face's
  side, `af` / `noaf`, the judged face's side, EAR, eyes-open probability,
  yaw / pitch / roll and timings; on `main` it goes through
  `scan::extract_analysis`, so a new dump carries the **mesh-based**
  `eye_focus` of #733, which the Step 1 dumps (taken before #733) do not.
  `riffle-cli eyecrops` writes face and eye crops with an index;
  `riffle-cli crop <file> <out.png> [size]` a crop around the AF point.
- **Constants that stay fixed.** `candidate_probability()` =
  sigmoid(`CANDIDATE_LOGIT`) = 0.772 decides `candidate`; `EYES_CLOSED_EAR`
  = 0.137 and the logistic slope decide the open probability from the EAR
  (it saturates near 1 above an EAR of roughly 0.2, so "more open" is only
  visible on the EAR itself); `MAX_ROLL` = 90 deg bounds the pose. The
  yaw sign is right on ~94% of turned faces, the yaw class agrees with the
  eye on 68% with the gap at the frontal / oblique boundary
  (`docs/plans/_archived/20261007-head-pose/pose-results.md`); |yaw|
  medians were 10 deg frontal / 41 oblique / 65 profile; the pose labels
  are unreviewed. Both eyes are normally still visible up to roughly
  45-60 deg of yaw; the exact figure is what Step 5 would measure if the
  tight form is needed.
- **Labeling precedent** (if Step 5 labels): `eyes-truth.md` /
  `pose-truth.md`, a stratified draw shuffled blind, crops on sheets, the
  agent labels, the user's review is final.
- Scratch outputs of Steps 1-3 are in
  `D:\Photos\tests\2026-10-08-burst-keep-score\`; those of Steps 4-6 go to
  `D:\Photos\tests\2026-10-09-good-mark\`.

## Steps

- [x] Step 1: Add `riffle-cli features` and dump every feature, the flags and the capture time per file for the data set
  - Done when:
    - `riffle-cli features <dir>... [threads]` (`crates/cli/src/main.rs`)
      prints one tab-separated record per RAW file: folder, file name,
      `capture_time`, `subsec`, the XMP and `.dop` flags, `sharpness`, cue
      `state`, `eye_focus`, the cue face's side, `af` / `noaf`, the judged
      face's side, EAR, the eyes-open probability, `yaw`, `pitch`, `roll`,
      and the per-file ms; `-` for a missing value, `err` for a failed
      stage; a unit test pins one line.
    - The dump of the data set is under
      `D:\Photos\tests\2026-10-08-burst-keep-score\dump\`; [data.md](data.md)
      holds the label rules, the data set and the inventory; `learnings.md`
      the timings. `mise run ci` passes.

- [x] Step 2: Define the burst, the pick-protecting metrics and the baselines, and measure each feature and hand thresholds
  - Done when:
    - [metrics.py](metrics.py) groups the records into bursts with the rule
      of `burst.ts` at 1000 / 2000 / 5000 ms and computes the pick
      false-fail rate, the non-pick flag rate at the 99% / 95%
      pick-keeping thresholds, the pick position, the pairwise and absolute
      AUC and the secondary ranking metrics, with baselines and hand rules;
      [results.md](results.md) with the tables, the reading and a hand
      check of 30 flagged non-picks. `mise run ci` passes.

- [x] Step 3: Fit the thresholds and combinations from the picks with held-out folders and decide
  - Done when:
    - [fit.py](fit.py), [keep.py](keep.py), [scene.py](scene.py) held out
      by folder; [fit.md](fit.md); [frozen.json](frozen.json) /
      [frozen-fail-check.json](frozen-fail-check.json); "Decision 1" below.
      `mise run ci` passes.

- [x] Step 4: Build the "good photo" mark in the frontend with provisional cuts read from a re-dump, and show it instead of the candidate icon
  - Done when:
    - **Re-dump first.** `riffle-cli features` from the current `main` over
      at least five of the sidecar-labeled ARW folders of [data.md](data.md)
      (include `2026-09-19`, `2026-09-27-a` and three smaller ones), saved
      under `D:\Photos\tests\2026-10-09-good-mark\dump\`. The EAR and pose
      columns are checked on 20 files against what the app's index holds
      for the same files (the same AF face, so they must match; `sqlite3`
      on the index or the `get_view` companion); if the dump's eyes path
      picks another face than the cue's mesh on some files, add the cue's
      `eyes_ear` / `pose` as columns to `features` and use those (a CLI
      change in this step is fine, recorded in `learnings.md`).
    - **Provisional cuts** in `provisional.md` (this plan folder): from the
      re-dump's distributions over the faced AF frames (percentiles of
      `eye_focus`, EAR, |yaw|, |pitch|; the pick / non-pick split alongside
      for information only), three cuts chosen by hand: `eye_focus` high
      (start near 0.9; well above the 0.772 candidate boundary), EAR high
      (start near the EAR where the open probability saturates, ~0.2, so
      "wide open" rather than "not closed"; the derived probability noted
      beside it), pose **loose** (only extreme turns excluded: start at
      |yaw| <= 60 and |pitch| <= 45; roll unused), and the share of faced
      AF frames each cut and the AND of the three would mark, per folder,
      next to the share today's green icon marks (`candidate`). The user's
      preference is written down: the loose pose cut stays if the AND is
      selective enough (a few percent of frames, say under ~10%); if it
      barely narrows, the tight "both eyes visible" cut (about |yaw| <=
      45; measured in Step 5 if needed) is the next thing to try.
    - **The rule, in one place.** `crates/app/ui/src/focus.ts`: `MarkFocus`
      and `FaceReady` gain the serialized fields the backend already sends
      (`eyes_ear: number | null`, `pose: { yaw; pitch; roll } | null`;
      `eyes_closed` typed too if read), `applyFaceReady` patches them, and
      a pure `goodPhoto(focus: MarkFocus | null | undefined): boolean`
      (name free) returns true only when `candidate === "candidate"` and
      `eye_focus`, `eyes_ear` and `pose` exist and pass the cuts, which are
      exported constants in `focus.ts` (`GOOD_EYE_FOCUS`, `GOOD_EYE_EAR`,
      `GOOD_MAX_YAW`, `GOOD_MAX_PITCH`), each with a comment saying it is
      provisional, the date and the re-dump percentile it sits at, so
      Step 5 tunes numbers, not code. `focus.test.ts` pins each boundary
      (at, just below, just above each cut), every `null` case, a
      `not_candidate` frame with good eyes and pose (false), and the sign
      independence of yaw / pitch.
    - **Display, option (b).** The crosshair is **green only for a frame
      that passes `goodPhoto`**. Orange keeps its meaning (`not_candidate`,
      the cue's `Soft`). A focus candidate that is not good draws in a
      fourth state, `candidate_only`, whose color (a dim green, or the
      existing white) is decided while implementing and recorded in
      `learnings.md`; the point is that bright green now means good.
      `focusMark` returns the state the color is read from (`"good" |
      "candidate_only" | "not_candidate" | "unknown"`), so `drawFocusMark` and
      the tests change in `focus.ts` only. The strip's face icon
      (`strip.ts` `setCandidate` / `paintCandidate`, `main.ts`
      `applyCandidates`, renamed to `good`) shows **only on frames that
      pass**, in bright green. The filter menu's `AF eye` section, the meta
      pane and `get_view` are unchanged. `main.ts`'s `Focus` interface
      mirrors `MarkFocus`.
    - **Before / after.** `learnings.md` records, from the re-dump and the
      cuts (not from the GUI), the share of faced AF frames the icon marks
      before (`candidate`) and after (`goodPhoto`) per re-dumped folder,
      and a dozen file names on `2026-09-19` that keep the icon so the user
      can look at the same ones.
    - **No `docs/humans` change in this step** (the cuts are provisional);
      the `README` / `usage` paragraphs stay as they are until Step 6, and
      `learnings.md` says so.
    - `mise run ci` passes (`pnpm exec vp check`, `fmt`, `test`).
  - Implementation approach:
    - Files: `crates/app/ui/src/focus.ts`, `focus.test.ts`, `main.ts`,
      `strip.ts` (rename only; `style.css` untouched unless a fourth color
      needs a token), `crates/cli/src/main.rs` (only if the dump needs the
      cue's columns), this plan's `provisional.md`, `learnings.md`.
    - No Rust change in the app or core beyond the optional CLI columns;
      no schema or `FACES_VERSION` change; the serialized fields are
      already there (`StoredEyes` flattened into `Focus` and `FaceReady`).
    - If option (b) proves awkward (the fourth color, or the rename
      touching too much), fall back to: the icon follows `goodPhoto`, the
      crosshair colors stay exactly as today; say so in `learnings.md` and
      in the Progress line.

- **Manual check by the user (the plan stops here until it is done).** The
  main agent asks the user (in Japanese) to open their recent folders with
  the Step 4 build and say: are the icons few enough to mean something; do
  the marked frames look like good photos (sharp open eyes, face toward the
  camera); which marked frames should not be, and which unmarked ones
  should; does the loose pose cut let through turned faces they would not
  call good (then the tight cut). The answers go to `learnings.md`.

- [x] Step 4b: Show the eyes as openness (0-100 from the EAR) instead of the open probability
  - Why (the user, 2026-10-09): the meta pane's `Eyes open` percentage is
    the logistic of the EAR around `EYES_CLOSED_EAR`; it saturates near
    100% above an EAR of ~0.2, so it says neither how open the eyes are
    nor how sure the judgment is, and reads as a probability it was never
    checked to be (its calibration was not measured and its labels are
    unreviewed). The user asked for an openness value instead.
  - Done when:
    - A pure `eyesOpenness(ear: number | null): number | null` (in
      `focus.ts` next to the good-photo cuts, or `meta.ts`; say which)
      maps the EAR linearly to 0-100: 0 at `EYES_CLOSED_EAR` (0.137, the
      open / closed boundary) and below, 100 at a fixed "wide open" EAR and
      above, clamped. The 100 anchor is a named constant read from the
      Step 4 re-dump (e.g. the EAR's 90th percentile over faced AF frames;
      the percentile and value recorded in `provisional.md`), not tied to
      the tunable `GOOD_EYE_EAR`. Tests pin both anchors, the clamps, a
      mid value and `null`.
    - The meta pane's Analysis row shows the openness as an integer
      percent-like value with the judgment, e.g. `Eyes open` →
      `Open · 82` / `Closed · 0` (exact label and format decided while
      implementing, recorded in `learnings.md`), from the stored
      `eyes_ear` when the stored values apply and from the on-demand
      `eyes_of` judgment's EAR otherwise (add the EAR to that payload only
      if it is not already there; verify). The row no longer shows the
      logistic percentage. `meta.test.ts` (or the module's test) covers
      both paths and the missing case.
    - The `Eyes` filter section, the closed-eyes judgment, `EYES_CLOSED_EAR`
      and the logistic stay unchanged; the MCP `get_view` keeps its
      fields (a follow-up todo if it should carry the openness).
    - No `docs/humans` change here; Step 6 updates the meta pane wording.
    - `mise run ci` passes.
  - Implementation approach:
    - Frontend only unless the on-demand payload lacks the EAR (then the
      one field in `commands.rs`'s eyes reply, noted). Files: `meta.ts`
      (+ test), `focus.ts` (+ test) if the helper lives there, this plan's
      `provisional.md`, `learnings.md`.

- [ ] Step 5: Tune the cuts from the user's feedback and record Decision B
  - Done when:
    - The constants in `focus.ts` are moved to the values the user's
      feedback asks for (each change and its reason in `learnings.md`),
      the tests' boundary values follow, and `provisional.md` gains the
      per-folder mark rate at the final cuts (from the re-dump). The pose
      cut follows the stated preference: **loose (extreme turns only) if
      the mark is selective enough and the user does not see turned faces
      marked; otherwise the "both eyes visible" cut**, placed at the yaw
      where the far eye is still seen on the user's frames (the user names
      the turned frames they saw; their |yaw| in the re-dump locates the
      cut).
    - **Optional, only if the user wants numbers:** a small labeled sample
      as the earlier truth sets were made: 100-150 faced AF frames drawn
      stratified over the `eye_focus` / EAR / |yaw| buckets around the
      cuts and at random, shuffled blind, crops and sheets under
      `D:\Photos\tests\2026-10-09-good-mark\`, labels in `miss-truth.md`:
      `ok` (sharp AF eye, open eyes, face toward the camera with both eyes
      visible), `miss` with the reason (`blur`, `focus`, `closed`),
      **`turned`** (the face turned so that one eye is hidden or nearly, or
      the back of the head; kept separate from `ok` so the pose cut can be
      measured against it), `unsure`. The agent labels first, the user's
      review is final. `mark.md` then reports per condition the precision
      and coverage as its cut moves, the pose grid from tight to loose
      (|yaw| <= 15 / 20 / 30 / 45 / 60 / 90 / none, |pitch| <= 15 / 20 / 30
      / 45 / none), the AND at the chosen cuts with a Wilson interval, and
      today's `candidate` icon as the baseline. Without labels, Decision B
      rests on the user's look and the mark rate alone, and says so.
    - **Decision B** below: the final cuts, the pose form chosen and why
      (selectivity and what the user saw), the per-folder mark rate, the
      labeled precision / coverage if measured, and the display kept
      (option (b) or the fallback). The user approves it.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 4 is merged and the user has looked. Numbers only in
      `focus.ts` unless the user asks for a different display; a display
      change here is allowed but recorded as such.
    - Files: `crates/app/ui/src/focus.ts`, `focus.test.ts`, this plan's
      `provisional.md`, `plan.md` (Decision B), `learnings.md`; optionally
      `miss-truth.md`, `mark.md`, `mark.py`.

- [ ] Step 6: Bring the user docs, `CLAUDE.md` and the todo in line
  - Done when:
    - `docs/humans/usage.md` **Focus mark** paragraph (and the strip icon
      sentence) say what the green mark and the icon now mean: all three of
      the AF eye in focus, the eyes open and the face toward the camera,
      the cuts in words (e.g. "in-focus probability about 90% or more, eyes
      clearly open, head turned less than about 60 degrees"), that it is
      selective by design (most frames get neither), what orange and the
      intermediate color mean now, that a frame without an AF point, with
      manual focus, with no face near the point or with a face under 60 px
      never gets it, and that a strongly turned face never gets it even
      when intended; the labeled numbers in one clause if Step 5 measured
      them. `usage.ja.md` in sync. `README.md` / `README.ja.md` Focus mark
      bullet in sync (the "strip marks each candidate with a green face
      icon" clause changes). The filter menu's `AF eye` section text stays
      (unchanged behavior).
    - `CLAUDE.md` Layout: `src/focus.ts` named with the good-photo rule
      next to the focus mark; `src/strip.ts` icon sentence if present.
    - `docs/agents/tauri-app.md` only if it names the candidate icon.
    - `todo.md`, face/eye section: a checked item for the mark with the
      plan folder in backticks and Decision B's cuts and mark rate;
      "Suggest the sharpest-eye frame within a burst group" stays unchecked
      with a pointer to [results.md](results.md) / [fit.md](fit.md) and
      Decision 1's reason (the user picks within a burst by timing; burst
      length, not a technical feature, predicts a kept scene; non-picks are
      mostly technically fine); "Flag looking-away frames from the head
      pose" is checked or rewritten to the pose cut that shipped; fully
      specified follow-ups, unchecked: a `Good` item in the filter's `AF
      eye` section; moving the rule to the backend (derived next to
      `candidate` in `index.rs`) so the MCP `get_view` can carry it; a
      labeled precision figure if Step 5 did not make one; re-tuning after
      the closed-eyes / head-pose label reviews; the no-AF path once a cue
      exists there.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 5 is merged. Docs only. Grep `candidate icon`,
      `face icon`, `scan-face`, `green for a focus candidate` across
      `README*.md`, `docs/humans`, `docs/agents`, `CLAUDE.md`, `todo.md`
      (not `docs/plans/_archived`).

## Trade-offs and risks

Steps 4-6 (the mark):

- **Where the rule lives (chosen at planning: the frontend, `focus.ts`).**
  One module, constants easy to tune, no Rust change, Vitest covers it. The
  cost: the MCP companion and a future filter item cannot see the state
  without recomputing it; moving the rule to a derived `good` next to
  `candidate` in `index.rs` is a follow-up if that is wanted.
- **Display (chosen: option (b), with a fallback).** Bright green and the
  icon only for frames that pass; orange keeps meaning `Soft` (the cue
  alone); a candidate that is not good needs a fourth color or white. Risk:
  a fourth color crowds the mark's palette; if it reads badly, the fallback
  keeps the crosshair exactly as today and moves only the icon. The filter
  is left alone in this plan so its `Sharp` / `Soft` / `Unknown` items keep
  matching the meta pane's `AF eye in focus`.
- **Provisional cuts are a guess from distributions**, not from labels.
  That is the point of the user's turn: see it, then tune. The risk is a
  cut that looks fine on `2026-09-19` and wrong on a folder with other
  lenses or light; the before / after rates on five folders and the user's
  look at their own recent folders are the guard.
- **Pose cut (user's preference, decided in Step 5).** Loose (extreme turns
  excluded, profiles pass) if the mark is selective enough; tight ("both
  eyes visible", about |yaw| <= 45) if it barely narrows. Risks: the yaw
  estimate is rough near profiles and at the frontal / oblique boundary;
  the pose labels are unreviewed; with the tight cut an intended profile
  never gets the mark, which the user accepts for this mark and the docs
  state.
- **Eyes condition on the EAR.** The derived open probability saturates
  near 1 above an EAR of ~0.2, so a "wide open" cut must be on the EAR;
  the EAR of a downcast eye reads low (counts as not good, as
  `eyes-truth.md` ruled for closed), and sunglasses or glare give odd
  meshes. The constant is on the EAR with the probability noted beside it.
- **Faces the rule never reaches.** No AF point (Leica, manual focus), no
  face near the point, a face under 60 px (no mesh), a mesh failure: never
  marked. A folder of such frames shows no icon at all, which the docs
  state so the absence is not read as "all misses".
- **Face choice.** The AF face only; a good frame whose subject is a second
  person is judged on the wrong face. Not solved here.
- **Labels, if made, are small and reviewed late.** 100-150 frames give a
  precision with about +/-6 points at 95%; the user's review is final and
  may be pending; the Decision says which it rests on.

Steps 1-3 (kept for the record):

- **Negatives are unlabeled.** A non-pick flagged as failed may be a good
  frame the user did not choose; no metric on the picks can tell, which is
  why the mark is now judged by the user's look (and labels, optionally).
- **Relative vs absolute sharpness**, **pick-percentile thresholds**, **PU
  class prior**, **burst gap**, **`Output/` as a label**: measured as
  recorded in [results.md](results.md) and [fit.md](fit.md); none changed
  the answer.
- **Ordering against `20261008-mesh-eye-focus` / `20261008-mesh-eyes-index`**:
  both have merged (#733, #741); the storage this plan once planned as
  Step 4 landed there, so Steps 4-6 here build on `main` as it is.

## Decision 1 (burst score): no mark ships

Concluded on 2026-10-09: no burst mark. The user approved a no-ship
Decision on 2026-10-08, then reopened it for the burst-level measurement
below; that measurement's no-ship conclusion was not approved explicitly,
and the user moved on to the good-photo mark on 2026-10-09. Numbers from
[fit.md](fit.md) "Burst level" (every burst of the 27 sidecar-labeled ARW
folders, gap 1000 ms: 5026 bursts of two or more frames, 36.1% of them
holding a pick; 1943 single frames, 12.5% picked), held out by folder:

- **Technical rules mark kept scenes at about 56%, not 80-90%.** The best
  held-out result is 55.8% of the marked bursts kept (lift 1.55) on 9.0% of
  the bursts (absolute sharpness >= 200, `eye_focus` >= 0.95, eyes open >=
  0.995 on some frame); per folder 40.0-91.7%. From 80% up no rule reaches
  the target even on its training folders.
- **The burst's length is the signal, and the strip already shows it.**
  Size alone gives 79.3% held out at 15 or more frames (9.6% of the bursts)
  and 80.1% at 20 or more; adding the technical rules moves it by about one
  point, inside the per-folder spread. Without the sharpness cuts, size >=
  20 with `eye_focus` >= 0.95 and eyes open >= 0.995 reaches 87.4% held out
  on 1.9% of the bursts, and both eye features are needed for that (either
  alone adds 0-3 points over size). The count badge on the burst band
  already shows the length.
- **Single frames: nothing works** (best in-sample 32.0% on 25 singles,
  against the 12.5% base). Face-free bursts and singles: sharpness lifts
  kept scenes from 18.5% to at most 24.3%.
- The earlier rounds agree: per frame the keep mark's precision stops at
  about 42% held out ([fit.md](fit.md) "Keep mark"); the failure check flags
  3.3% of the non-picks against sharpness alone's 2.8% at a 1% pick
  false-fail ([fit.md](fit.md) "Failure check"). Caveats that do not change
  the answer: `eye_focus` in those dumps is the eye-window cue from before
  #733; the pose labels are unreviewed; the DNG blocks are too small.

Outcome: no burst mark; `riffle-cli features`, [metrics.py](metrics.py),
[fit.py](fit.py), [keep.py](keep.py), [scene.py](scene.py) and the dumps
stay as diagnostics; the todo records the reason (Step 6). The original
Steps 4-5 (store the eyes and pose; a burst keep mark) are replaced by the
Steps 4-6 above, the storage having landed in #741.

## Decision B (the good-photo mark)

(Written in Step 5: the final cuts, the pose form chosen and why, the
per-folder mark rate, the labeled precision / coverage if measured, the
display kept. Awaiting the user's approval before Step 6.)

## Progress

- (2026-10-08) Step 1 complete
- (2026-10-08) Step 2 complete
- (2026-10-09) Step 3 complete; Decision 1 concluded (no burst mark)
- (2026-10-09) Steps 4-6 rewritten for the good-photo mark after the user's
  third turn (implement first with provisional cuts, the user looks, then
  tune, then docs) and approved by the user; the storage of the eyes and
  pose landed in #741
- (2026-10-09) Step 4 complete
