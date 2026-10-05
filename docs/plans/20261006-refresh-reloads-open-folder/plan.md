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

# Refresh on the open folder also reloads it

## Purpose

The folder tree's right-click `Refresh` re-lists a folder's subfolders and RAW
count and retries the tree's folder watches. It exists for folders whose watch
could not be set (a network share, a refused permission). The same folders do
not follow the disk in the strip either, and today the user has to know to
also press `File > Reload Folder` (`CmdOrCtrl+R`). When the right-clicked
folder is the one open in the strip, `Refresh` should also run the same rescan
as `Reload Folder` (`resync` in `crates/app/ui/src/main.ts`: re-list through
`list_arw`, refilter, start a scan that reconciles sidecars), so one menu item
brings both the tree and the strip in line with the disk. A folder that is not
open keeps the current behavior. No new menu item. Agreed with the user on
2026-10-06, including a dedicated `"refresh"` rescan trigger so `Riffle.log`
tells the menu item from `CmdOrCtrl+R`.

## Steps

- [x] Step 1: Make `Refresh` on the open folder also run `resync("refresh")`
  - Done when:
    - Right-clicking the open folder in the tree and choosing `Refresh`
      re-lists the folder in the tree (as today) and also triggers
      `resync("refresh")`; a `Refresh` on any other folder, including an
      ancestor or a subfolder of the open one, only re-lists the tree as
      before.
    - `RescanTrigger` (`crates/app/ui/src/refresh.ts`) has a new `"refresh"`
      variant, so `Riffle.log` reads `trigger=refresh` for the menu item and
      `trigger=reload` for `File > Reload Folder`; the `rescanLine` tests cover
      the new variant.
    - The "is the open folder" decision uses the same path comparison the
      trash-from-tree path uses (`opensTarget` / `relation` with
      `folders.ignoreCase`), so a trailing separator, `\` vs `/`, drive-letter
      case and, on macOS / Windows, path case do not defeat it.
    - A vitest case covers the same-folder decision for the cases above where
      the existing `opensTarget` / `relation` tests do not already (check
      `crates/app/ui/src/trash.test.ts` `describe("opensTarget")` and
      `crates/app/ui/src/tree.test.ts` around line 674 first; add only what is
      missing).
    - `docs/humans/usage.md` (the `Refresh` sentence, around line 55) and
      `docs/humans/usage.ja.md` (the matching `次の \`Refresh\` は、…` sentence
      in the folder paragraph) say that `Refresh` on the open folder also
      reloads it the way `File > Reload Folder` does, in the same PR.
    - `mise run ci` passes.
    - Pending manual real-device check, recorded in `learnings.md` as pending:
      on Windows, open a folder on a share (or any volume) whose watch does not
      fire, add a RAW to it and edit a sidecar outside Riffle, then right-click
      the open folder and choose `Refresh`; the new file appears in the strip
      and the edited judgment is picked up, the tree's RAW count updates, and
      `Riffle.log` shows `trigger=refresh`. Also check that `Refresh` during a
      running scan defers and runs when the scan ends, and that `Refresh` on a
      folder that is not open leaves the strip untouched.
  - Implementation approach (as far as it is known):
    - Change the `case "refreshFolder":` branch of the folder context-menu
      dispatch in `crates/app/ui/src/main.ts` (around line 2679). Keep
      `folders.refresh(path)` as is, then, when
      `openDir !== null && opensTarget(openDir, [path], false, folders.ignoreCase)`
      (or `relation(openDir, path, folders.ignoreCase) === "same"`;
      `opensTarget` is already imported from `./trash.js`), call
      `resync("refresh")`. Do not chain it on the tree listing's promise: the
      two are independent, and a failed tree listing must not block the strip
      rescan or vice versa.
    - Add `"refresh"` to `RescanTrigger` in `crates/app/ui/src/refresh.ts` and
      to whatever enumerates the triggers (e.g. `rescanLine` and its tests).
    - Do not touch `crates/app/ui/src/folders.ts`: it has no access to
      `openDir` or `resync`, and `refresh(path)` is also what the tree's own
      background re-lists use.
    - `resync` already returns when no folder is open and defers while a scan,
      a listing or a deferred operation is in flight; add no extra guard.
    - Multi-selection: `folderMenuGroups` (`crates/app/ui/src/context.ts`)
      offers only the trash items for more than one selected folder, so
      `Refresh` always acts on the single clicked `path`; keep that.
    - Verbatim `\\?\` paths: the frontend's `openDir` and the tree's paths are
      both the caller's own strings, never canonicalized (see
      `crates/app/src/watch.rs`), so `relation`'s normalization is enough.
    - Commit as `feat(app): reload the open folder from the tree's Refresh`.

## Trade-offs and risks

- Trigger name: a dedicated `"refresh"` trigger (chosen by the user) behaves
  identically to `"reload"` but keeps the two sources apart in `Riffle.log`.
- New pure helper vs existing comparison: reuse `opensTarget` / `relation`
  (already tested) rather than a single-use wrapper; add a test only for a gap.
- `Refresh` on a parent of the open folder intentionally does not rescan the
  strip (a parent's `Refresh` is about the tree). If the user expects it to,
  that is a follow-up.
- Risk: `Refresh` chosen during a scan now also queues a rescan that runs at
  `faces-done`; that is the same behavior `Reload Folder` already has, and the
  deferral collapses repeated triggers into one.

## Progress

- (none yet)
