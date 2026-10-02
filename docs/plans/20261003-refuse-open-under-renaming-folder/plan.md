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

# Refuse to open a folder under a rename that is in flight

## Purpose

When the tree's inline edit confirms a folder rename, `main.ts`'s
`renameFolder` invokes `rename_folder` and, when it resolves, re-keys the tree
(`folders.renamed`) and reopens the open folder under its rebased path. While
that invoke is in flight (the backend flushes the sidecar writer, takes the
`Scans` lock, releases the watcher, renames on disk and rewrites the index),
the tree still draws the folder and its subfolders under their old paths, and
a click on one of them opens it there: `openDirectory(oldSubfolder)` lists a
path that is about to vanish (or has just vanished), records it through
`remember_folder`, and starts a scan on it, which can make the backend's
`Scans` re-check refuse the rename itself with `a scan is running`. This was
raised in the archived
`../_archived/20260928-rename-from-tree-and-strip/learnings.md` ("Deferred
issues") and tracked in `todo.md` under
`### App: opening a subfolder while its parent folder's rename is in flight`.

Of the three ways out, this plan takes **(a): refuse the open while the
rename is in flight**, for these reasons:

- The existing success path already covers the one case where a reopen is
  wanted: `renameFolder` rebases `openDir` and reopens it under the new path.
  Refusing new opens in the window keeps that single reopen as the only
  post-rename navigation, so no second async hop or stored "open this later"
  intent is needed.
- Deferring (b) would hold a click for a window of unknown length (the
  writer flush can take up to `DRAIN_TIMEOUT`) and then act on it after the
  user may have moved on, and it is ambiguous on failure (open at the old
  path, which still exists, or drop the click?). Reopening after the fact (c)
  still lets the stale open run first: it starts a scan on a path that is
  about to move, which is exactly what can make the backend refuse the
  rename, and it records the old path as the remembered folder.
- The window is short in the normal case (tens of milliseconds), so a refused
  click costs the user at most one more click, and the state it needs is one
  set of in-flight paths that both the click and the keyboard open consult.

The in-flight window starts at the `rename_folder` invoke, not when the edit
is confirmed: a rename held by the `IdleGate` while a scan runs has not
touched the disk, and opening a folder calls `idle.discard()`, which drops
the held rename, so nothing is stale during the held period.

## Steps

- [x] Step 1: Refuse to open a folder that is, or is under, a folder whose `rename_folder` invoke is in flight; cover it with unit tests; delete the todo section
  - Done when:
    - `crates/app/ui/src/rename.ts` gains a small DOM-free piece of state for
      the folder renames whose invoke is in flight, with: a way to add a
      path when the invoke starts, a way to remove it when the invoke
      settles (success or failure), and a predicate that answers whether a
      given path is one of those folders or lies under one, folded by the
      platform `ignoreCase` flag passed as a trailing parameter (as
      `rebase` and the trash relation helpers do; see
      `docs/agents/tauri-app.md` "Path comparisons in the UI take the
      platform `ignoreCase` flag"). Matching is component-wise: `D:\photos2`
      is not under `D:\photos`.
    - `crates/app/ui/src/rename.test.ts` covers the predicate: the renamed
      folder itself, a subfolder (and a deeper descendant), a sibling whose
      name shares the prefix (`photos` vs `photos2`), an unrelated folder, a
      case-only difference in a folder name (not the drive letter) with and
      without `ignoreCase`, that settling the rename clears the block, that
      settling one of two in-flight renames leaves the other's block, and
      that settling a path that was never added is a no-op.
    - `crates/app/ui/src/folders.ts` consults that state before every folder
      open it issues: the row `click` handler (the plain, unmodified click
      that calls `open(node.path)`) and the keyboard `open` command in
      `keydown`. A blocked open does nothing else to the tree (no selection
      change, no re-render) and is reported through the existing
      `reportError` status callback with one short note (e.g. that the
      folder is being renamed). `folders.ts` exports the two entry points
      `main.ts` needs to mark an invoke started and settled (for example
      `renameStarted(path)` and `renameSettled(path)`; whether the settle is
      folded into the existing `renamed()` for the success side is the
      implementer's call, but the failure side must clear it too).
    - `crates/app/ui/src/main.ts` `renameFolder` marks the path in flight
      right before `invoke("rename_folder", …)` inside the `whenIdle`
      callback (not at `whenIdle` itself) and clears it in both the success
      and the failure branch. On success the clear happens before (or as
      part of) `folders.renamed(path, newPath, name)`, so the tree's re-key
      and the following `openDirectory(reopen, …)` never see the old path
      still blocked.
    - A Cmd/Ctrl- or Shift-click on a blocked row still only changes the
      selection, as today (it never opens); the right-click menu and the
      expander are left as they are (see Trade-offs).
    - `todo.md`: the section
      `### App: opening a subfolder while its parent folder's rename is in flight`
      (its heading, background paragraph and `#### TODO` bullet) is deleted.
      No other todo section is touched. If the implementer finds a
      reproducible way to widen the window by hand (for example a large
      pending sidecar flush), a hands-on check procedure may be left in
      `todo.md` in its place; otherwise nothing replaces it.
    - `mise run ci` passes (`pnpm exec vp {check,fmt,test}` for the
      frontend, the Rust lints and tests unchanged).
  - Implementation approach:
    - Keep the pure state in `rename.ts` next to `SlowClick` and test it in
      `rename.test.ts`; `folders.ts` cannot be imported by a node-env test.
      `rename.ts` may import `rebase` from `tree.ts` (DOM-free) and use
      `rebase(path, dir, dir, ignoreCase) !== null` as the "equal to or
      under" check, or `tree.ts` may grow a small exported `within`-style
      helper that `rebase` and the new predicate share. Do not duplicate the
      path folding (`normalize` / `fold` / `segments`) in `rename.ts`.
    - In `folders.ts`, pass `ignoreCase` (the module's exported constant)
      to the predicate, as `renamed()` does for `renameFolder` / `rebase`.
    - Match the existing module shape: `folders.ts` keeps module-level state
      and exports plain functions (`startRename`, `renamed`,
      `cancelSlowClick`); `main.ts` calls them from `renameFolder`. The
      `renameFile` path in `main.ts` is not touched (a file rename has no
      subtree to open).
    - Follow `../_archived/20260928-rename-from-tree-and-strip/learnings.md`
      Step 2 for the pointer-sequence caveats: do not add a `render()` on a
      refused click; the row stays as drawn.
    - Record in `learnings.md` whether a hands-on reproduction was possible.

## Trade-offs and risks

- **Refuse (a) vs defer (b) vs reopen after (c).** (a) was chosen for the
  reasons in Purpose. (b) would give the smoothest result when the rename
  succeeds, at the cost of a stored pending open, a rebase of it on
  success, and a decision on failure; it can be added later on top of the
  same in-flight state if a refused click turns out to annoy in practice.
  (c) alone leaves the stale open's side effects (scan start, remembered
  folder) in place and is already what happens for the folder that was open
  before the rename.
- **A status note on the refused click.** The user chose one short note
  through the existing `reportError` callback over a silent refusal, so a
  click that does nothing has a visible cause. The window is usually shorter
  than the note is visible, and the next `show()` clears it anyway.
- **Scope of the block.** Only the two open paths are blocked. The
  expander (`list_subfolders` on a path about to move) and the right-click
  menu (`reveal_folder`, `Copy Path`, a nested `Rename…`) during the window
  are left alone: they do not open a folder, their failures are already
  reported or silently tolerated (`toggle`'s listing error marks the row
  failed), and widening the block would grow the change beyond the todo
  item.
- **A rename that never settles.** The in-flight set is cleared only by the
  invoke's resolution. Tauri invokes always settle (resolve or reject), so
  the set cannot leak; but if a future change moves the invoke behind
  another gate, the clear must move with it.
- **Hands-on check.** The window is hard to hit by hand; the unit tests on
  the predicate and the two wired call sites are the verification. The
  todo section is deleted either way; a procedure is left only if a
  reliable reproduction is found during implementation.

## Progress

- (none yet)
