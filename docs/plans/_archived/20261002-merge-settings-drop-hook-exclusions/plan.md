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

# Drop the hook-based read-only CLI exclusions from merge.js

## Purpose

`EXCLUDED_PATTERNS` in `.claude/skills/merge-settings/scripts/merge.js` drops
incoming `Bash(gh ... list|view *)` and `Bash(gcloud ... list|describe *)`
allow entries, and the comment above them says a PreToolUse hook auto-allows
those read-only calls. The `merge-settings-keep-existing` work found that no
such hook exists in the repo `.claude/settings.json` nor in the user-level
settings (recorded in
`../_archived/20261002-merge-settings-keep-existing/learnings.md`); those
calls pass only through the auto permission mode. The exclusion therefore
silently throws away allow entries that are as legitimate as any other. The
user decided to delete the two patterns and their comment so such entries are
promoted like everything else. This closes the todo.md heading
"### Agents: `merge.js` EXCLUDED_PATTERNS assume a PreToolUse hook that does not exist"
(the wrap-up's todo-curator deletes it).

## Steps

- [x] Step 1: Remove the gh / gcloud read-only exclusion patterns, align SKILL.md, and re-point the tests
  - Done when:
    - `merge.js` no longer contains `/^Bash\(gcloud [^)]+ (?:list|describe) ?\*?\)$/` and `/^Bash\(gh [^)]+ (?:list|view) ?\*?\)$/` nor the comment bullet "A read-only CLI call auto-allowed by a PreToolUse hook ...". Every other pattern and comment bullet is untouched, including `/^Bash\(gh run watch ?\*?\)$/` and its "duplicate already covered by `Bash(gh run *)`" bullet.
    - `.claude/skills/merge-settings/SKILL.md`'s "Entries excluded from the merge automatically" paragraph is consistent with the new list. It currently never mentions gh / gcloud or a hook, so verify during implementation whether any wording change is needed at all; if none is, leave it as is.
    - `.claude/skills/merge-settings/scripts/merge.test.sh` keeps exercising exclusion with patterns that still exist, and asserts that `Bash(gh pr view *)` arriving from `settings.local.json` is now promoted.
    - `mise run lint` (which runs `merge.test.sh`) and `mise run ci` pass.
  - Implementation approach:
    - `merge.js`: delete the two regex lines and the one comment bullet only. No other reordering or rewording.
    - `merge.test.sh`: the fixture currently holds `Bash(gh pr view *)` as "an entry an exclusion pattern matches", and the "local adds entries" case uses `Bash(gh issue list *)` as the excluded incoming entry. Replace both with patterns still in `EXCLUDED_PATTERNS`, keeping the two distinct, e.g. fixture `Bash(gh run watch *)` and incoming excluded `Write(notes.md)`. Update the fail messages / comments that name them.
    - Add the promotion assertion inside the existing "local adds entries" case: put `Bash(gh pr view *)` in its incoming allow list, assert with `grep -qxF` that it was added, and bump the expected summary count accordingly. Keep the fixture free of `Bash(gh pr view *)` so the count is unambiguous.
    - Run the test with `mise x -- bash .claude/skills/merge-settings/scripts/merge.test.sh` (`node` / `shellcheck` are not on the Git Bash PATH outside mise), then `mise run lint` and `mise run ci`.
    - Do not edit `todo.md`; the wrap-up removes the heading above.

## Trade-offs and risks

- `settings.json` will from now on accumulate individual `Bash(gh ... view *)` / `Bash(gcloud ... list *)` entries as sessions produce them. That is the user's decision. Do not reintroduce a middle-wildcard aggregate such as `Bash(gcloud * list *)`: it spans whitespace and would also allow `gcloud secrets delete list`.
- SKILL.md may need no change; confirm rather than add text for its own sake.

## Progress

- (2026-10-02) Step 1 complete
