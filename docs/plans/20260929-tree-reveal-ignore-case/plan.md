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

# Reveal a differently-cased open path in the folder tree

## Purpose

On macOS and Windows (case-insensitive filesystems) a folder can be opened
with a path whose case differs from the spelling `list_subfolders` returns
(a typed or dropped path, a path restored from the settings store). The
tree's `reveal` walks the chain `ancestorsWithin` builds from the caller's
own segments and stops at the first node key it cannot find, because
`tree.ts` compares case-sensitively apart from the drive letter. Once fixed,
such a folder is expanded and selected all the way down on case-insensitive
platforms, while Linux keeps exact comparison.

## Steps

- [x] Step 1: Match the reveal chain case-insensitively on macOS and Windows
  - Done when:
    - `ancestorsWithin` accepts a case-insensitivity flag: with it, a root
      holds the path when they match ignoring case (not only the drive
      letter), and the chain is spelled from the root's spelling as today.
    - `reveal` in `folders.ts` re-spells the rest of the chain from the
      listing at each level: after `setChildren`, the next chain element is
      looked up among `folder.children` (case-insensitively on macOS /
      Windows, exactly on Linux) and, when the listing spells it
      differently, the remaining chain elements are rebased onto the child's
      spelling, so `tree.nodes.has(dir)` succeeds at every level and
      `current` / `cursor` / the selection end up on the node key the tree
      uses.
    - Unit tests in `crates/app/ui/src/tree.test.ts` cover: a Windows path
      whose folder names differ in case from the root (flag on) resolves;
      a macOS-style POSIX path with the flag on resolves; the same POSIX path
      with the flag off (Linux) stays unmatched / keeps the caller's
      spelling; and the new re-spelling helper picks the listed child
      ignoring case only when asked.
    - The `### App: the folder tree does not reveal a differently-cased open
      path` section is removed from `todo.md`.
    - `mise run ci` passes.
  - Implementation approach:
    - Keep `tree.ts` pure: no `navigator` there. The platform decision lives
      in `folders.ts` next to the existing `const isMac =
      /Mac/.test(navigator.platform);` (line 155), e.g.
      `const ignoreCase = isMac || /Win/.test(navigator.platform);`, and is
      passed into the pure functions as a boolean. Linux (neither) stays
      exact.
    - `ancestorsWithin(roots, path, ignoreCase = false)`: when the flag is
      set, compare `normalize(root)` and `normalize(path)` lower-cased for the
      "holds" test; the chain construction (root spelling + caller segments)
      is unchanged. Keep the default `false` so the existing tests and the
      `rootOf` test that calls it without the flag keep passing.
    - Add one small exported pure helper in `tree.ts` (name it in the style
      of the file, e.g. `respell(chain, at, children, ignoreCase)`) that
      returns the chain with `chain[at]` and everything after it rebased onto
      the matching listed child's `path` via the existing `rebase`, or the
      chain unchanged when no child matches. Matching compares
      `normalize(child.path)` against `normalize(chain[at])`, lower-cased
      when `ignoreCase`. `rebase` already joins in the new prefix's spelling,
      so the deeper elements pick up the listing's case level by level.
    - In `reveal`, call the helper right after `tree = setChildren(...)`
      for the next index in the loop (convert the `for...of` into an index
      loop, or reassign `chain` in place). Do not touch the rest of `reveal`
      (the failure / `stillCurrent` branches, the roots handling), since a
      parallel run may be editing that function for reveal failure marks and
      roots refresh; keep the diff to the loop body and the platform flag.
    - Fix the comment at `folders.ts` line 497 so it no longer claims
      `ancestorsWithin` normalizes case; the chain's final spelling now comes
      from the listings.
    - Verify during implementation, in `tree.test.ts`: `"C:\\Users\\me"`
      root with `"c:\\users\\ME\\pictures"` (flag on) yields a chain whose
      elements, once re-spelled through children `[{name:"Pictures",
      path:"C:\\Users\\me\\Pictures"}]`, equal the listed paths; the same
      with the flag off on `/home/me` vs `/home/ME/x` returns `null` /
      leaves the chain untouched.
    - `relation` and `rebase` keep their current (case-sensitive) comparison;
      the re-spelling only ever passes `rebase` an `oldDir` that is the
      caller's own spelling of `chain[at]`, which `rebase` matches exactly.

## Trade-offs and risks

- Where the flag is decided: this plan decides it per platform in
  `folders.ts` (`isMac || Windows`), consistent with the existing `isMac`
  detection. An alternative is deciding per path in `tree.ts` (Windows paths
  are self-identifying via `isWindowsPath`, macOS is not), but that would
  make a POSIX path on Linux and on macOS indistinguishable inside `tree.ts`,
  so the caller must pass something either way. Per platform is the smaller
  change.
- Scope of case-insensitivity: only `ancestorsWithin`'s root match and the
  new re-spelling helper ignore case. `relation`, `rebase` and
  `renameFolder` (used by the rename flow and the watcher) stay exact; a
  later item could widen them if a differently-cased path shows up there,
  but doing it now would grow this PR beyond the todo item.
- Volumes mounted case-sensitively on macOS (rare, opt-in APFS) would be
  matched case-insensitively by the platform rule; a false match would only
  pick a listed child with the same name in another case, which such a
  volume can hold. Accepted as out of scope for this fix.
- Conflict risk: another run may be editing `reveal` (failure marks, roots
  refresh). Keeping the change to the loop body and a single new constant
  minimizes the merge conflict; rebase before opening the PR.

## Progress

- (none yet)
