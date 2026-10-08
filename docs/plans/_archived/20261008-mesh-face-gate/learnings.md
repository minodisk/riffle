# Learnings: mesh-face-gate

## Step 1

- The gate is `candidate::meshes_face(face)` (box long side
  `>= eyes::EYES_MIN_FACE`), used by `scored_face` and by both mesh calls in
  `crates/cli/src/main.rs`: the `candidates` report pass and the single-file
  debug subcommand that prints the nearest face's scored region (the latter
  is not in the plan, but without it the debug line would disagree with the
  cue it prints above it).
- No non-ignored `riffle-core` test runs the mesh model (the real-image test
  in `eyes.rs` is `#[ignore]`), so the new test proves the gate with a 59 px
  face inside the synthetic image (`Scored::Window`, `mesh: None`) and checks
  the boundary on the predicate alone (`EYES_MIN_FACE.next_down()` false,
  `EYES_MIN_FACE` true).
- `cargo` is not on the Git Bash `PATH` here; prefix
  `export PATH="$HOME/.cargo/bin:$PATH"`.
- Labeled sets (new build, `riffle-cli candidates <dirs> 24`), diffed in
  order against `D:\Photos\tests\2026-10-08-mesh-eye-focus\step3-*.txt` on
  name, state, p, lap, edge, face and the `eye` column: one frame changed,
  training `_DSC2748.ARW` (58x49 face, Reject): `p 0.209 eye L only` ->
  `p 0.328 eye - window`, still `NotCandidate`. **0 state changes.** Held-out:
  no change at all. Every other gated line only lost its mesh columns
  (`yaw -` ... `R - - - - -`), as expected.
  - Training: AUC 0.882 / precision 93.9% / coverage 91.4% before and after
    (330 candidates, 310 in focus).
  - Held-out: AUC 0.800 / 88.6% / 95.9% before and after (370 candidates,
    328 in focus).
  - Faces under 60 px (gated): 104 of 406 training, 102 of 400 held-out.
  - Per-file lines: `D:\Photos\tests\2026-10-08-mesh-face-gate\training.txt`
    and `heldout.txt`.
- `D:\photos\2026\2026-09-19` (2134 ARW), first timed run of each build
  diffed: 0 state changes, 1697 / 255 / 182 (candidate / not / unknown) for
  both; one probability moved, the same `_DSC2748.ARW` (the training frame
  comes from this folder). 261 of the 1952 faced files (13.4%) have a face
  box under 60 px and fall to the gate. All four timed runs of each build
  gave identical per-file lines apart from the mesh ms column and the total.
  Lines saved as `2026-09-19-old.txt` / `2026-09-19-new.txt` next to the
  labeled ones.
- Timing: base `176a96b6` built from a scratch `git worktree` with
  `CARGO_TARGET_DIR` on this worktree's `target`, exe copied aside, then this
  change built and copied aside. Warm-up: old 31.53 s (an outlier, not
  counted), new 20.31 s. Alternated runs, `... total`: before 20.28 / 20.94 /
  20.68 / 20.51 s (mean 20.6), after 19.98 / 21.13 / 19.84 / 19.88 s (mean
  20.2), -0.4 s (-2%), within the spread. The base's mean is ~1 s under the
  archived 21.5 s for the same code, so run-to-run and session-to-session
  drift is of the size of the expected gain.
- `FACES_VERSION` stays `6` (no state changed anywhere); its doc comment
  says why and names the one moved probability per set.
- `docs/humans/usage.md` / `usage.ja.md` and `CLAUDE.md` were left alone:
  they say the window is used when neither eye region counts ("under 24 px,
  as on small faces"), which still holds for a gated face.
