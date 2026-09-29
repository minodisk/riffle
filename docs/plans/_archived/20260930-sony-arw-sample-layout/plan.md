<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Sony ARW sample layout in the archived learnings

## Purpose

The archived learnings of sony-arw-coverage
(`../_archived/20260930-sony-arw-coverage/learnings.md`, Method bullet at
lines 7-11) say the 66 ARW samples were downloaded into
`D:\photos\samples\ARW\<MODEL>\`, one folder per model. The samples have since
been flattened into `D:\photos\samples\ARW\` with each file renamed
`<Model>_<original name>` from the EXIF Model string, because default camera
names such as `DSC00002.ARW` collide across bodies (the α6700 and ZV-E10
samples both had one). Whoever re-runs the verification from the learnings
would look for folders that no longer exist. After this work the learnings
describe the current layout while still stating accurately how the
verification was run.

## Steps

- [x] Step 1: Update the Method bullet of the archived learnings to the flat sample layout
  - Done when:
    - `docs/plans/_archived/20260930-sony-arw-coverage/learnings.md` lines
      7-11 (the first Method bullet) describe the current layout: the files
      sit directly in `D:\photos\samples\ARW\` (next to the CR3 and NEF
      samples), each renamed `<Model>_<original name>` using the EXIF Model
      string (e.g. `ILCE-6700_DSC00002.ARW`), the prefix skipped when the
      original name already starts with `<Model>_` (the α7 IV files, e.g.
      `ILCE-7M4_DSC06673_FullFrame-Raw-Uncompressed.ARW`); the α7C
      `DSC00107%5b1%5d.ARW` is `ILCE-7C_DSC00107[1].ARW`; the reason
      (default names collide across bodies) is stated.
    - The bullet says the flattening happened after the verification ran,
      when the files were downloaded into one folder per model
      (`D:\photos\samples\ARW\<MODEL>\`), so that the next bullet's
      "`bench` (per folder)" and "`scan` ran over every folder" and the
      "0 errors in every folder" line stay accurate as a description of how
      it was run then. Those bullets are not changed.
    - The A9 II URL-encoding note (`(`, `)` and `:`) is kept.
    - No other file changes. `plan.md` line 66 of the same archive folder is
      the raw.pixls.us URL and stays as is.
    - `mise run ci` passes (lychee included; the Windows paths are in
      backticks, not links, so no new link target is introduced).
  - Implementation approach:
    - Edit only the first Method bullet (lines 7-11); keep the rest of the
      Method list verbatim. Keep the line wrapping style of the file (about
      76 columns).
    - Commit as `docs(plans): describe the flat Sony ARW sample layout in the
      archived learnings` (Conventional Commits, English).

## Trade-offs and risks

- Where to note the flattening: in the same first bullet (chosen, keeps the
  edit surgical and puts "downloaded per model, flattened since" next to the
  layout) versus a new dated sub-note under Method. The single-bullet edit is
  the smaller diff.
- The archive is normally frozen; this edit changes a historical record. The
  wording must make clear the per-model folders were the state at
  verification time, not rewrite the method itself.

## Progress

- (2026-09-30) Step 1 complete
