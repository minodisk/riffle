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

# Shared XMP of same-stem RAWs: document as unsupported

## Purpose

Two RAW files of the same stem but different extensions in one folder (`a.ARW`
and `a.DNG`) share the stem-named `a.xmp`. `Rename…` (`file_plan` in
`crates/app/src/rename.rs`) and Move Rejected to Trash (`collect_folder` in
`crates/app/src/trash.rs`) treat `a.xmp` as the sidecar of whichever RAW they
act on and carry it away, silently dropping the other RAW's judgment. The user
decided not to change the code: the case is declared unsupported by design,
documented in the READMEs with the workaround, and its todo item is closed.

## Steps

- [x] Step 1: Document the shared-`.xmp` case as unsupported and drop its todo item
  - Done when:
    - `README.md` carries a short note in "Working with other software" (right
      after the format list, next to the paragraph beginning "A sidecar created
      by other software is edited in place", around line 178) stating that two
      RAWs of the same stem with different extensions in one folder share the
      stem-named `.xmp` and are not supported: renaming or trashing one of them
      moves the shared `.xmp` with it, taking the other's judgment along. It
      names the workaround: use the `.dop` sidecar format alone
      (`<raw name>.dop`, so one per RAW; **Both** still writes the shared
      `.xmp`) or keep such RAWs in separate folders.
    - `README.ja.md` carries the equivalent note in Japanese at the same spot
      (section "他のソフトとの連携", after the format list, around line 69).
    - `todo.md` no longer has the section
      `### App: a rename can carry away an `.xmp` shared by two RAWs of the same stem`
      (through its TODO list, ending before
      `### App: a pending rename waits silently with no visible pending state`).
    - `mise run ci` passes.
  - Implementation approach:
    - Docs only; no code changes. Files: `README.md`, `README.ja.md`,
      `todo.md`.
    - Keep the note to one short paragraph in each README, in the existing
      register (the `FOO.ARW` / `FOO.xmp` example style used just above it).
    - Commit as `docs: declare a shared .xmp of same-stem RAWs unsupported` (or
      similar Conventional Commits `docs` type).

## Trade-offs and risks

- Placement: the note could instead sit under the `Rename…` bullet or the Move
  Rejected to Trash bullet, but the cause is the sidecar naming, so the
  sidecar-format section is the one place that covers both operations.
- The `.dop` workaround only holds with `.dop` alone, since **Both** still
  writes the shared `.xmp`; the note says so.

## Progress

- (none yet)
