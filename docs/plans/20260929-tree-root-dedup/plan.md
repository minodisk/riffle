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

# Folder tree: a folder reachable from two roots is drawn once

## Purpose

The folder tree (`crates/app/ui/src/tree.ts`, drawn by
`crates/app/ui/src/folders.ts`) keys every `TreeNode` by path alone. A folder
reachable from two roots therefore shares one node: on Windows the home root
`C:\Users\me` is also listed as a child of `C:\` > `Users` (the backend's
`roots()` keeps `C:\` on purpose, since it is an independently browsable
volume); on Linux / macOS any root `rootOf` adds during `reveal` (`/`) lists
the way down to home too. Expanding either row expands both, and the subtree
and the `.current` highlight are drawn twice.

This plan takes the todo's second option (approved by the user): a folder
that is itself a root is not drawn as a child of another root. It follows the
precedent of `crates/app/src/folders.rs` `roots()`, which already drops a
volume that duplicates home because "home already stands for it". Once done,
every folder appears in the tree exactly once, so expansion, the `.current`
highlight, the keyboard cursor, the multi-selection, the tree watches and
`reveal`'s scroll all have one row to land on. See "Trade-offs and risks" for
the option not taken (keying rows by root-plus-path).

## Steps

- [x] Step 1: Hide a root's own path from the children of any other row
  - Done when:
    - `rows(tree)` never yields a row whose path is one of `tree.roots`
      at depth > 0, compared the way `ancestorsWithin` compares
      (`normalize`: separators, trailing slash, drive-letter case), so
      `C:\Users\me` under `C:\` > `Users` and `/home/me` under `/` > `home`
      are skipped while the root row itself is still drawn.
    - The hiding is order-independent: it holds both when the root is
      added before the parent is listed (launch: `folder_roots`, then the
      user expands `C:\`) and after (`reveal` adds `/` through `rootOf`
      after `/home` was already listed, or a later roots refresh adds a
      root).
    - Expanding the hidden path's root row does not expand anything under
      the other root, and the `.current` highlight, cursor and selection of
      that folder are on its root row only.
    - A parent whose only listed children are hidden roots (e.g. `C:\Users`
      holding only `me`) shows no expander and cannot be expanded: a small
      `drawnChildren(tree, node)` helper in `tree.ts` is used by `rows`,
      `canExpand` and the expander branch of `render` in `folders.ts`.
    - Unit tests in `crates/app/ui/src/tree.test.ts` cover: the Windows
      case (`C:\Users\me` + `C:\`), the Linux case (`/home/me` + `/`), a
      drive-letter / separator spelling difference between the root and the
      listed child (`c:/Users/me` vs `C:\Users\me`), the order-independence
      above, the no-expander parent, and that `watchedFolders` lists the
      folder once when its root row is expanded.
    - `mise run ci` passes (fmt, lint, type-check, vitest).
  - Implementation approach:
    - Do the filtering through `drawnChildren` in `tree.ts`, not in
      `setChildren` or the backend: `rows()` is the single source every
      drawn-row consumer derives from (`watchedFolders`, `pruneSelection`,
      `clickSelect`, `step`, `treeKey`, `typeAhead`, `render`,
      `menuTargets`), and filtering at draw time is order-independent for
      free, since it reads `tree.roots` then. Keep `TreeNode.children` as the
      backend listed it.
    - Compare with the existing `normalize` (module-private in `tree.ts`;
      reuse it, do not add a second normalizer).
    - Do not touch the node map's keying, `expand` / `collapse`, `current`,
      the selection or `reveal`; with one row per folder the existing
      path-keyed state is correct as is. `set_tree_watches`
      (`crates/app/src/treewatch.rs`) needs no change: `watchedFolders`
      derives from `rows()`.
    - Keep the `folders.ts` change to the expander branch only, since that
      file is under parallel change (case-insensitive reveal, reveal failure
      marks, roots refresh).
    - `docs/usage.md` (around line 12) describes the tree; add one clause
      only if its wording would otherwise contradict the new behaviour (a
      root is not repeated under another root). No `README` change.
    - Rename needs no special handling: root rows cannot be renamed
      (`depth > 0` guard in `render`), and `renameFolder` re-keys roots and
      children alike, so a hidden child stays hidden after its parent is
      renamed.

## Trade-offs and risks

- Option not taken: key rows by root-plus-path (the todo's first option).
  It would keep `C:\` > `Users` > `me` browsable as a second, independent
  row, but every path-keyed piece of state would need a row key instead:
  `TreeNode.expanded`, `cursor`, `selection` and `clickSelect` ranges,
  `current`, `editing` / `ended`, `menuTargets`, `watchedFolders`,
  `tree-changed` and `reveal`. That is a diff across most of `tree.ts` and
  `folders.ts` for a duplicate row whose only value is a second way to click
  home.
- UX change: `C:\` > `Users` (and `/` > `home`) no longer lists the home
  folder; home is reachable only through its own root row.
- Interaction with the parallel "roots refresh" work: a root added later is
  hidden from an already-listed parent at the next render because the
  filter runs at draw time.
- Interaction with the parallel "case-insensitive reveal" work: the root
  filter reuses `normalize`, so it inherits whatever comparison lands there.
- Only spelling differences `normalize` folds are caught; a symlink alias is
  out of scope, as it is for `ancestorsWithin` today.

## Progress

- (none yet)
