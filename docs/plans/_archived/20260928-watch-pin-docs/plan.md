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

# Correct the Windows watcher-pinning statements

## Purpose

`docs/agents/tauri-app.md`'s folder-watcher entry still says a `notify` watch
on Windows keeps the open folder from being deleted or renamed. The
tree-live-watch Step 1 measurement (`notify` 8.2, Windows 11,
`RecursiveMode::NonRecursive`; see
`../_archived/20260928-tree-live-watch/learnings.md`) showed the opposite for
rename: the watched folder itself renames fine, and it is an *ancestor* of a
watched path that fails with os error 5. Delete was never measured. The entry
right after it already states the measured rule, so the guide contradicts
itself, and one code comment in `watch.rs` repeats the stale claim. This
corrects both so the guide and the code match the measurement; no behavior
changes.

## Steps

- [x] Step 1: Reword the stale watcher-pinning statements in the guide and `watch.rs`
  - Done when:
    - `docs/agents/tauri-app.md` lines 1201-1202 no longer claim the open
      folder cannot be deleted or renamed; the paragraph says a Windows watch
      pins the watched folder's ancestors against rename (renaming the
      watched folder itself succeeds), points to the next entry for the
      measurement, and says delete was not measured (the open TODO in
      `todo.md`, "measure whether a `notify` watch blocks deleting a folder
      on Windows"), claiming nothing further about delete
    - The `release_under` doc comment in `crates/app/src/watch.rs` (lines
      122-124) says the handle blocks renaming `dir` when the watched folder
      is under it, not that it keeps `dir` itself from being renamed
    - `rg -n "cannot be deleted or renamed|keeps .* from being renamed" --glob '!docs/plans/_archived/**'`
      finds nothing that claims the watched folder itself is pinned
    - `mise run ci` passes
  - Implementation approach:
    - Docs and comments only; no Rust or TypeScript logic changes.
    - Keep the surrounding sentences of the guide entry (the `set` drops the
      previous watcher, the SMB warn-and-ignore fallback) as they are; only
      the first sentence is wrong.
    - Match the wording already used in `docs/usage.md:20-22` ("the folder
      itself and those under it stay free") and in the next guide entry
      ("pins the watched folder's ancestors, not the folder itself").
    - Do not touch the statements that are already correct:
      `docs/usage.md`, `crates/app/src/rename.rs:68-70` and `:209`,
      `crates/app/src/treewatch.rs:320-321`, `todo.md`, and the in-flight
      `docs/plans/20260928-trash-rejected-from-tree/learnings.md`.
      `README.md` / `README.ja.md`, `main.rs` and `commands.rs` contain no
      such claim, so they stay unchanged.

## Trade-offs and risks

- The `FILE_SHARE_DELETE` explanation in
  `docs/plans/20260928-trash-rejected-from-tree/learnings.md` is a mechanism
  claim beyond the measurement. It lives in an in-flight plan folder, so this
  plan leaves it alone and does not fold it into the guide.
- `rename.rs:209` ("the watcher holds the directory, not its files") is
  neutral and left as is.

## Progress

- (2026-09-28) Step 1 complete
