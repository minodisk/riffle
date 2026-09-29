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

# Close five small todo.md items

## Purpose

`todo.md` carries five small, self-contained items that need no design work:
a stale header comment in `crates/app/src/sequence.rs`, a missing conflict
rule in `.claude/agents/pr-conflict-resolver.md`, a focus regression when the
focused file is deleted from outside the app, case-sensitive path comparison
in three `tree.ts` helpers on case-insensitive filesystems, and an unrecorded
Bash-tool heredoc pitfall on Windows. Each step closes one item, removing its
`###` section from `todo.md` in the same PR, so the list shrinks to what still
needs real work.

Every step touches `todo.md`. Two pairs of the sections to remove are
adjacent (`Agents: Bash-tool heredocs…` at ~line 1151 sits right above
`App: deleting the focused file…`; `App: tree.ts's relation/rebase/…` sits
right above `Agents: pr-conflict-resolver…`), so run the steps one after
another, each branched from `main` after the previous PR merged, rather than
in parallel. Step 1 goes first so that any `todo.md` rebase conflict a later
step hits is resolved by the rule it adds.

## Steps

- [x] Step 1: Add the "keep deleted `todo.md` sections deleted" rule to `pr-conflict-resolver`
  - Done when: `.claude/agents/pr-conflict-resolver.md` "2. Manual resolution"
    tells the agent that in `todo.md` (and similar tracking docs) a `###`
    section one side deleted stays deleted rather than keeping both sides,
    and that after resolving it checks
    `grep '^### ' todo.md | sort | uniq -d` for duplicated headings and
    compares the branch's deleted headings against `origin/main`; the
    `### Agents: \`pr-conflict-resolver\` resurrects deleted \`todo.md\`
    sections` section (Background + TODO) is removed from `todo.md`;
    `mise run ci` passes.
  - Implementation approach:
    - Add the rule as one more bullet in the existing "Do not hand-merge
      generated artifacts." list (after **Generated metadata**, before
      **Only for ordinary source code**), or as a short paragraph right
      after that list; match the surrounding bold-lead-in bullet style and
      keep it to a few lines. Give the two check commands as fenced `bash`
      blocks like the rest of the file.
    - Cite the source the todo names
      (`docs/plans/_archived/20260929-todo-drop-resurrected-sections/learnings.md`)
      in the rule's "why" so the next reader knows what it prevents.
    - Files: `.claude/agents/pr-conflict-resolver.md`, `todo.md`.
    - Edits under `.claude/` need the sandbox override the agent files
      describe (`.claude/` is on the write deny list).

- [ ] Step 2: Update `crates/app/src/sequence.rs`'s header comment to the folder-tree flow
  - Done when: the `//!` header of `crates/app/src/sequence.rs` no longer
    says "a sibling of the picked folder" and instead describes that the
    folder is the one right-clicked in the folder tree (the output is
    written to a sibling of that folder, so the open RAW folder's
    non-recursive watcher still sees no `folder-changed`); `cargo fmt`
    unchanged; the `### App: \`crates/app/src/sequence.rs\`'s comment still
    describes a picker-based flow` section is removed from `todo.md`;
    `mise run ci` passes.
  - Implementation approach:
    - Comment-only change to the first eight `//!` lines; do not touch code.
      Keep the second paragraph's point (JPEGs are not indexed; a run is not
      refused during a scan; sibling output so the watcher stays quiet),
      only replace "picked folder" with the tree-right-click wording. The
      first line already says "The folder tree's `Sequence JPEG
      Timestamps…`", so the fix is the one phrase plus whatever reads
      naturally around it.
    - Files: `crates/app/src/sequence.rs`, `todo.md`.

- [ ] Step 3: Fall back to the vanished anchor's neighbour when the focused file is deleted from outside the app
  - Done when: after `resync` re-lists the folder and the current file is
    gone from the listing, the current index lands on the deleted file's
    next passing survivor in the previous list, else its previous one, else
    (nothing survives) index 0 as today; `crates/app/ui/src/filter.test.ts`
    has Vitest cases for next-neighbour, previous-neighbour (deleted last
    file), a neighbour that does not pass the filter (skipped to the next
    one that does), and a run of consecutive deletions; existing
    `anchorAfterFilter` cases still pass; the `### App: deleting the focused
    file from outside the app moves focus to the first file, not its
    neighbour` section is removed from `todo.md`; `mise run ci` passes.
  - Implementation approach:
    - Root cause (verified at planning): `resync` keeps `ratings` / `flags`
      / `labels`, so `passes(anchor)` in `main.ts` is usually still true for
      a deleted file; `anchorAfterFilter` then returns the anchor itself and
      `refilter` does `fileIndex.get(target) ?? 0`. When the anchor is
      filtered out instead, `allFiles.indexOf(anchor)` is `-1` and the
      first passing file wins. So the new logic must treat "anchor not in
      the new list" as vanished regardless of `pass(anchor)`.
    - `refilter` computes `order = ordered()` after `resync` has already
      replaced `allFiles`, so the previous list is not available there
      today. `resync` must capture it before `allFiles = found` (the
      current `files`, i.e. the filtered, ordered list the user was looking
      at, is the natural choice: `target = files[index]` is taken from it
      already) and pass it through `refilter` to the picker.
    - Keep the picker pure and tested in `filter.ts`: either extend
      `anchorAfterFilter` with an optional `previous?: readonly string[]`
      parameter (when `anchor` is absent from `allFiles` and `previous` is
      given, walk `previous` forward from the anchor's index for the first
      path that is in `allFiles` and passes, then backward), or add a small
      sibling helper that `refilter` calls first and feed its result into
      `anchorAfterFilter` as the anchor. Membership in `allFiles` should be
      a `Set` lookup, not `includes`, for a 2000-file folder.
    - `refilter(anchor, force, keepScroll)`'s other callers (judgments,
      filter changes, open) pass no previous list and keep today's
      behaviour; only `resync` supplies it.
    - `mustReshow(force, files[index], anchor)` already reshows when the
      current file changed, so the neighbour gets drawn without further
      work; verify that in the flow rather than assume.
    - Files: `crates/app/ui/src/filter.ts`, `crates/app/ui/src/filter.test.ts`,
      `crates/app/ui/src/main.ts`, `todo.md`.
    - Manual check if a Windows/macOS GUI is available: open a folder, focus a
      middle file, delete it in Explorer/Finder, confirm the strip stays on
      the next file; record the result (or that it was not run) in
      `learnings.md`.

- [ ] Step 4: Make `relation` / `rebase` / `renameFolder` ignore case on macOS and Windows
  - Done when: `relation`, `rebase` and `renameFolder` in
    `crates/app/ui/src/tree.ts` take an `ignoreCase = false` parameter
    (the same shape as `ancestorsWithin` / `respell`) and compare through
    the existing `fold`, so a path differing only in case matches on
    macOS / Windows while Linux stays case-sensitive; `folders.ts`'s
    `renamed` passes its module flag; `crates/app/ui/src/tree.test.ts` has
    cases for each of the three (`relation("D:\\photos\\2026", "d:/Photos", true)`
    is `"under"` and `null` without the flag; `rebase` of a differently
    cased path under `oldDir` re-joins the remaining segments in `newDir`'s
    spelling; `renameFolder` re-keys a node whose key differs in case from
    `oldPath`); the `### App: tree.ts's \`relation\`/\`rebase\`/\`renameFolder\`
    stay case-sensitive on case-insensitive filesystems` section is removed
    from `todo.md`; `mise run ci` passes.
  - Implementation approach:
    - `tree.ts` already has `fold(path, ignoreCase)` (`normalize` +
      optional `toLowerCase`) used by `ancestorsWithin` and `respell`;
      swap the `normalize` comparisons in `relation`, `rebase` and
      `renameFolder` (`renamed`, `stale`, `moved`, `list`) for `fold` with
      the flag. `rebase` must keep joining the *remaining segments of the
      caller's `path`* (not lower-cased text) onto `newDir`, as the
      "Case-insensitive path matching must be corrected level by level"
      item in `docs/agents/tauri-app.md` and `respell` rely on.
    - `respell` calls `rebase(p, dir, child.path)` after matching with
      `fold`; pass its `ignoreCase` through so the chain rebase agrees with
      the match.
    - The platform flag lives in `folders.ts:158`
      (`const ignoreCase = isMac || /Win/.test(navigator.platform)`), used
      by `reveal`. `renamed` in the same module passes it directly. The
      other callers, `main.ts:2376` (`rebase(openDir, path, newPath)` after
      a rename) and `trash.ts` (`restoredInto`, `opensTarget` via
      `relation`), stay case-sensitive (user decision, option (a) in
      "Trade-offs and risks"); note the untouched callers in
      `learnings.md`.
    - Files: `crates/app/ui/src/tree.ts`, `crates/app/ui/src/tree.test.ts`,
      `crates/app/ui/src/folders.ts`, `todo.md`.

- [ ] Step 5: Record the Windows Bash-tool heredoc backslash pitfall for agents
  - Done when: the behaviour is re-confirmed by writing a string holding
    doubled backslashes (e.g. `D:\\Photos \\?\C:\x "a\\b" \\\\server`) to
    the scratchpad once through a Bash-tool heredoc (quoted delimiter) and
    once through the Write tool, and diffing them; the result is recorded in
    `learnings.md`; if it reproduces, a rule is added to the agent-tooling
    doc telling agents on Windows to write text holding backslashes
    (Windows paths, verbatim `\\?\` prefixes, regex escapes) with the Write
    / Edit tools, or a script file written by them, never through a Bash
    heredoc; if it does not reproduce, no rule is added and `learnings.md`
    says so; either way the `### Agents: Bash-tool heredocs on Windows
    mangle doubled backslashes` section is removed from `todo.md`;
    `mise run ci` passes.
  - Implementation approach:
    - Planning-time probe on this machine (Windows 11, Bash tool): a
      `cat > f <<'EOF'` heredoc wrote every `\\` as a single `\` (single
      backslashes untouched), with a quoted and an unquoted delimiter
      alike, so expect it to reproduce; the Write-tool half still needs
      running since the planner had no Write tool.
    - Placement: `docs/agents/tauri-app.md` already carries "### On
      Windows, editing files with a Python heredoc can corrupt line endings
      and escapes (Hit)" (line ~1878, end of the "Frontend" section, just
      before "## CI"). Add the new rule as a sibling `### … (Hit)` item right
      after it, in the same shape (one-paragraph symptom, `- Fix:` and
      `- Source:` bullets pointing at
      `docs/plans/_archived/20260928-trash-rejected-from-tree/learnings.md`
      and this plan's `learnings.md`).
    - Optionally cross-reference from `.claude/skills/delegate/SKILL.md`'s
      quoted-heredoc advice (it recommends a quoted heredoc for verbatim
      text) only if the probe shows that path is affected too; otherwise
      leave it alone.
    - Files: `docs/agents/tauri-app.md`, `todo.md`, this plan's
      `learnings.md`.

## Trade-offs and risks

- **Step order and `todo.md` conflicts.** All five steps delete a `###`
  section from `todo.md`, and two pairs are adjacent, so parallel branches
  would conflict on rebase. The plan orders them sequentially and puts the
  conflict-resolver rule first.
- **Step 3: which "previous list".** Using `files` (the filtered, ordered
  list on screen) as the previous list matches what the user saw and is
  already the source of `target`. Extending `anchorAfterFilter`'s signature
  or adding a separate helper are both acceptable.
- **Step 3: sort-order changes between listings.** If the sort key changed
  between the previous list and the re-listing, "neighbour in the previous
  list" may not be the on-screen neighbour in the new one. The old behaviour
  (first file) is strictly worse, so accept it.
- **Step 4: scope of case-insensitivity.** Chosen: widen only the three
  `tree.ts` functions and thread the flag from `folders.ts`'s `renamed`,
  leaving `main.ts`'s post-rename `rebase(openDir, …)` and `trash.ts`'s
  `relation` calls case-sensitive. The open folder may fail to reopen after a
  rename when `openDir` differs in case from the tree key (rare: both come
  from the same tree / `list_arw` spellings).
- **Step 4: `renameFolder`'s `stale` check** uses
  `rebase(path, newPath, newPath) !== null` to detect a node already at the
  new path; with `ignoreCase` a node whose key differs only in case from
  `newPath` becomes "stale" and is dropped, which is right for a case-only
  rename on Windows but must be covered by a test so the old node's state
  survives and only a genuinely different pre-existing node is dropped.
- **Step 5: where the rule lives.** `docs/agents/tauri-app.md` is where the
  sibling Python-heredoc item already sits. If the Write-tool output is also
  mangled, the rule must say to use a script file or `printf '%s'` instead.

## Progress

- (2026-09-30) Step 1 complete
