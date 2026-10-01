# Learnings

## Step 1

- The test failed in all three cases against the pre-fix `merge.js` (both
  no-op cases rewrote the file; the adding case removed the pre-existing
  `Bash(gh pr view *)`), and passes after the fix.
- `node` and `shellcheck` are not on the Git Bash PATH outside mise here; run
  the test directly with `mise x -- bash .claude/skills/merge-settings/scripts/merge.test.sh`
  (inside `mise run lint` they resolve normally).
- Measured the real sequence in this worktree with no `settings.local.json`:
  `node merge.js --self --write` printed `Added permissions.*: 0` and
  `Nothing to write: .claude/settings.json left unchanged`, `git status` showed
  no change to `.claude/settings.json` (which holds `Bash(gh pr view *)`), and
  `commit-settings.sh` printed `No settings changes to promote.`
- PreToolUse hook finding: no PreToolUse hook exists in the repo
  `.claude/settings.json` (no `hooks` key, no `.claude/hooks/`) nor in the
  user-level settings (its hooks are `SessionStart`, `UserPromptSubmit`,
  `SessionEnd`, `PostToolUse` only; permissions use `defaultMode: "auto"`).
  The `merge.js` comment "auto-allowed by a PreToolUse hook" therefore does not
  hold here; `gh pr view` is only let through by the auto permission mode, not
  by a hook. Left unchanged per the plan's constraint (see Deferred issues).

## Deferred issues (todo candidates)

- `EXCLUDED_PATTERNS` in `.claude/skills/merge-settings/scripts/merge.js`
  (line 55, `/^Bash\(gh [^)]+ (?:list|view) ?\*?\)$/`, and the gcloud pattern
  above it) and its comment rest on a PreToolUse hook that auto-allows
  read-only CLI calls, but no such hook exists in the repo or user-level
  settings. Decide whether to drop the patterns or fix the comment. Basis: the
  plan's open question and the Step 1 investigation recorded above.
- In all-worktrees mode (`merge-settings` skill, no `--self`), the write is
  now skipped when no permission string was added and no `settings.local.json`
  was merged, so a non-permission change arriving only from another worktree's
  `settings.json` (a `hooks` block, `env`, and so on) is no longer written.
  The condition follows the plan as written; a structural comparison of the
  merged result against the destination would cover that case. Basis: Step 1
  implementation of the conditional write in
  `.claude/skills/merge-settings/scripts/merge.js`.
