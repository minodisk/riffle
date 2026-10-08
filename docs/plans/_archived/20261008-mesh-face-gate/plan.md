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

# Skip the face mesh in the scan for faces under 60 px

## Purpose

The scan's second pass runs the face mesh (`eyes::mesh_of`) on the face
nearest the AF point of every trusted-AF file and scores the eyes over the
mesh eye regions, falling back to the eye window when no region reaches
`EYE_REGION_MIN` (24 px). That mesh made pass 2 about two thirds slower
(12.9 -> 21.5 s on the 2134-ARW folder at 24 threads,
[`../20261008-mesh-eye-focus/learnings.md`](../20261008-mesh-eye-focus/learnings.md),
Step 4). On the labeled sets no face whose box is under 58 px ever gets an
eye region that counts: of the 330 frames an eye region decided, one (a
58.2 px training face) lies under 60 px, while about 26% of the labeled faces
(13% of the 2134-ARW folder's) are under 60 px. Those faces pay for a mesh
whose result is thrown away.

After this work, a face whose YuNet box long side is under `EYES_MIN_FACE`
(60 px, the floor the on-demand closed-eyes judgment already uses) goes
straight to the window fallback in the scan, without running the mesh. The
cue is otherwise unchanged: no coefficient, threshold, margin or
`EYE_REGION_MIN` moves, and the on-demand `eyes_of` / `read_eyes` path,
`pose.rs`, the closed-eye constants and the face crop are not touched.

## Steps

- [x] Step 1: Gate the scan's mesh on `EYES_MIN_FACE`, prove the cue is
      unchanged on the labeled sets, measure pass 2, and bring the docs in line
  - Done when:
    - In `crates/core/src/candidate.rs` a face whose box long side
      (`face.width.max(face.height)`) is below `eyes::EYES_MIN_FACE` never
      reaches `eyes::mesh_of` from `focus_cue_unless` / `scored_face`; its
      `EyeFocus` has `scored == Scored::Window` and `mesh == None`, exactly
      as a face with no mesh does today. A face at 60 px or more is handled
      as before. The `candidate.rs` module doc, the `scored_face` doc and the
      `focus_cue_unless` doc say so.
    - `riffle-cli candidates` applies the same gate in its report pass, so
      its per-file lines (the mesh columns and the `eye` column) and its
      pooled AUC / precision / coverage describe the cue the scan computes.
    - A unit test in `candidate.rs` covers the gate (a face under 60 px
      inside the image gets `Scored::Window` and `mesh: None`; the 60 px
      boundary is asserted against `EYES_MIN_FACE`, not a literal).
    - Comparison on the labeled sets (training: `2026-06-05-focus-sample`,
      `2026-07-31-focus-sample`, `2026-09-13-a-focus-sample`,
      `2026-09-19-focus-sample`, `2026-09-19-focus-sample-2`; held-out:
      `2026-06-14-focus-sample`, `2026-07-18-focus-sample`,
      `2026-08-01-focus-sample`, `2026-08-22-focus-sample`, all under
      `D:\Photos\tests\`): `riffle-cli candidates <5 training dirs> 24` and
      `<4 held-out dirs> 24` with the new build, diffed line by line against
      the archived baselines `step3-training.txt` / `step3-heldout.txt` in
      `D:\Photos\tests\2026-10-08-mesh-eye-focus\`. At most about one frame
      changes its `p` / scored region (the 58.2 px training face), and the
      number of frames whose **state** changed is recorded (0 or 1). The
      new AUC / precision / coverage of both sets are recorded next to the
      baseline (training 0.882 / 93.9% / 91.4%, held-out 0.800 / 88.6% /
      95.9%) in `learnings.md`, and the new per-file lines are saved in
      `D:\Photos\tests\2026-10-08-mesh-face-gate\` (not the scratchpad).
    - On `D:\photos\2026\2026-09-19` (2134 ARW), the per-file lines of the
      new build are diffed against the old build's (same command, same
      threads): the number of files whose state changed and the candidate /
      not / unknown counts (baseline 1697 / 255 / 182) are recorded, together
      with how many faced files fell to the gate (the lines with a face box
      under 60 px).
    - Pass-2 timing measured the archived way: release builds of the base
      commit and of this change (the base built from a scratch `git worktree`
      with `CARGO_TARGET_DIR` on this worktree's `target`, the exe copied
      aside), `riffle-cli candidates D:\photos\2026\2026-09-19 24`, one
      warm-up each, then four alternated runs, reading the `... total` line
      only. All eight numbers and the means go into `performance.md`.
    - `FACES_VERSION` (`crates/app/src/index.rs`) is bumped to `7` if any
      state changed on the labeled sets or on the 2134-ARW folder, with the
      reason appended to its doc comment; if no state changed, it stays `6`
      and the doc comment gets a "stayed `6` when ..." sentence saying why
      (the pattern `EXTRACTOR_VERSION`'s comment uses), naming how many files'
      stored probability moves (see Trade-offs).
    - `docs/humans/performance.md` "Focus candidate pass" and
      `docs/humans/performance.ja.md` gain the before / after paragraph (date,
      machine, commits, the eight runs, the means, the share of faced files the
      gate skipped, the state-change count); `docs/agents/tract-onnx-inference.md`'s
      bullet on the second pass's mesh says the mesh runs only on faces at or
      above `EYES_MIN_FACE` and points at the new numbers.
    - `docs/humans/usage.md` (around the `AF eye` description at line ~180)
      and `usage.ja.md`, and `CLAUDE.md`'s layout sentence on `candidate.rs`,
      are updated only if their wording would now be wrong (they describe the
      region / window rule, not the face size; check and leave them alone if
      still true).
    - The `todo.md` item "Skip the face mesh in the scan for faces whose box
      is under 60 px" (face/eye section, around line 636) is removed.
    - `mise run ci` passes.
  - Implementation approach (as far as it is known):
    - Put the gate in `candidate.rs`, not in `eyes::mesh_of` (option A in
      Trade-offs, chosen by the user): add one small `pub fn` next to
      `scored_face` (for example a predicate on the face's box long side
      against `eyes::EYES_MIN_FACE`, or a `cue_mesh(rgb, width, height,
      orientation, face) -> Option<Mesh>` wrapper that returns `None` under
      the floor and otherwise calls `mesh_of`) and use it from both
      `scored_face` and the CLI report (`crates/cli/src/main.rs`, the
      `measured` pass of `candidates`, which currently calls `eyes::mesh_of`
      directly). Compare with `<`, as `judge_mesh` does
      (`face.width.max(face.height) < EYES_MIN_FACE`), so 60.0 itself is
      meshed.
    - Do not change `mesh_of`'s signature or its "whatever the face's size"
      contract, `judge_mesh`, `eyes_of`, `read_eyes`, `pose.rs`,
      `FACE_CROP`, `EYES_MIN_FACE` or any closed-eye constant.
    - Keep the cancel point after the mesh in `scored_face` where it is; a
      gated face simply reaches it with `mesh == None`. The existing test
      `a_set_flag_abandons_the_cue_after_the_mesh` uses a 60 px face outside
      the image to avoid running the model; the new test can use a 59 px
      face inside the synthetic `two_steps` image, which proves the model did
      not run only because `mesh_of` on a face inside a valid image returns
      `Some`. Before asserting the `Some` side at 60 px, check whether any
      non-ignored `core` test already runs the mesh model (the real-image
      test in `eyes.rs` is `#[ignore]`); if none does, keep the boundary
      test's model run out (assert the predicate / wrapper at 59.9 and 60.0
      instead) so `cargo test` does not pay the ~180 ms plan build.
    - The CLI's "mesh ms" column and the `mesh -` rendering already handle a
      `None` mesh; no new column is needed. A gated face prints `mesh -` and
      `eye - window`.
    - The comparison needs no `fit.py` run: the CLI's summary lines are the
      same AUC / precision / coverage `fit.md` reports (Step 3 of the archived
      plan reproduced `fit.md` with them), and the per-file diff against
      `step3-*.txt` counts the changed frames. Note that two held-out file
      names occur in two folders each, so diff in order, not by name; also
      note the archived lines carry the 0.5-margin regions, the same as now.
    - Expect the saving on the timing folder to be modest: about 13% of its
      faces are under 60 px, so roughly 0.13 x 8.5 s = ~1 s of the 21.5 s,
      close to the run-to-run spread (12.10-13.95 s before). Report the
      numbers as measured; if the difference is within the spread, say so in
      `performance.md` instead of claiming a gain, and record the share of
      faced files skipped as the hard number.
    - Record the measurements and anything that deviates in `learnings.md`
      in this folder as they happen.

## Trade-offs and risks

- **Where the gate lives.** Option A (chosen, the approach above): in
  `candidate.rs`, through a small shared helper the scan and the CLI report
  both call. `mesh_of` keeps its contract, the on-demand paths are untouched,
  and the rule sits next to the cue it belongs to. Option B: inside
  `eyes::mesh_of`, one place that covers every caller, but it changes a
  public function's documented behavior ("whatever the face's size").
- **`FACES_VERSION`.** The gate changes the stored `eye_focus` probability of
  the one labeled face under 60 px whose eye region counted (and of any such
  face in the wild, expected on the order of 0.1-0.3% of faced files), even
  when its state does not move. A bump makes every user's app re-run pass 2
  on every folder (~21 s per 2000 files at 24 threads) to refresh a handful
  of probabilities. The plan bumps on any **state** change and otherwise
  stays with a documented reason.
- **The gain may be small.** The archived 26% figure is the labeled sets'
  share of faces under 60 px; the timing folder's is about 13%, so the
  expected pass-2 saving there is ~1 s, within the measured spread. The docs
  must not overstate it.
- **The one moved frame.** The 58.2 px training face goes from a counting
  eye region to the window model; whether its state flips is unknown until
  the diff. The acceptance criterion allows that one change, and no refit is
  in scope.

## Progress

- (2026-10-08) Step 1 complete. Deviation (see learnings.md): the gate also covers the CLI's single-file `faces` debug subcommand, not only the `candidates` report; FACES_VERSION stays 6 (0 state changes).
