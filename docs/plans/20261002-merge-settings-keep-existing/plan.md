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

# Keep existing settings.json entries during settings promotion

## Purpose

The develop skill's wrap-up runs `settings-promoter`, which calls
`node .claude/skills/merge-settings/scripts/merge.js --self --write` and then
`bash .claude/skills/develop/scripts/commit-settings.sh`. In commit 703d5bc3
that run had no `settings.local.json` and zero permissions to add, yet it
removed `Bash(gh pr view *)` from the git-tracked `.claude/settings.json`.

The cause is in `merge.js`: `normalizeArray` applies `EXCLUDED_PATTERNS` and
`sort()` to the whole merged array (destination entries included), the main
loop re-merges `./.claude/settings.json` into itself even with `--self`, and
`--write` writes unconditionally. So a no-op run filters and reorders the
destination, producing a diff that `commit-settings.sh` commits.

After this work, the exclusion patterns only screen entries coming in from a
`settings.local.json`; entries already in `settings.json` are never removed,
and a run with nothing to add leaves `.claude/settings.json` byte-for-byte
unchanged so no promotion commit is made.

## Steps

- [x] Step 1: Apply `EXCLUDED_PATTERNS` only to incoming entries and skip the write when nothing was added
  - Done when:
    - `merge.js --self --write` on a repo whose `.claude/settings.json` contains an entry matching an exclusion pattern (e.g. `Bash(gh pr view *)`) and whose `settings.local.json` is absent or `{}` leaves `.claude/settings.json` byte-for-byte unchanged (including order), prints `Added permissions.allow: 0`, and `commit-settings.sh` then reports `No settings changes to promote.`
    - An excluded entry arriving from `settings.local.json` is still dropped; a non-excluded one is added.
    - An excluded entry already present in `settings.json` survives a run that does add something.
    - A test covering those cases runs under `mise run lint` and passes; `shellcheck` / `lychee` in `mise run lint` still pass.
    - `.claude/skills/merge-settings/SKILL.md` describes the new behavior (exclusions screen incoming entries only; a no-op run does not touch the file).
  - Implementation approach:
    - Files: `.claude/skills/merge-settings/scripts/merge.js` (the fix), a new bash test beside it, `.claude/skills/merge-settings/scripts/merge.test.sh` (follows the existing `*.test.sh` convention), `.mise.toml` `[tasks.lint]` (add the test invocation next to the two existing `.test.sh` lines), `.claude/skills/merge-settings/SKILL.md` (the "Entries excluded from the merge automatically" paragraph), `learnings.md` in this folder (the PreToolUse finding below).
    - In `merge.js`, move the `isExcluded` filter out of `normalizeArray` so it runs on `override` only inside `deepMerge`'s array branch (`[...baseArr, ...override.filter(not excluded)]`), and drop the filter from the `override === undefined && Array.isArray(base)` branch. Dedupe stays as is.
    - Make the write conditional: compute the `added` counts first (the existing `permissionEntries` diff), and when every count is 0 and no `settings.local.json` was merged, print the counts and a "nothing to write" line to stderr and return without `fs.writeFileSync`. This is what guarantees byte-for-byte identity: the destination is unsorted today and `normalizeArray` still sorts on a real write, so skipping the write is simpler and safer than making the normalization order-preserving. Keep the stdout (non-`--write`) mode printing as before.
    - Do not change `EXCLUDED_PATTERNS` or the comment block above it (constraint). Do not change `commit-settings.sh`; its `git diff --quiet` already handles the no-diff case once the file is untouched.
    - Keep `settings-promoter.md`'s contract: it reads only the stderr lines `Merged settings.json:` / `Merged settings.local.json:` / `Added permissions.*:`, so keep those lines (the new "nothing to write" line is additive).
    - Test shape: run `merge.js` as a subprocess (it has no exports and executes on load) against a temporary directory containing a `.claude/settings.json` fixture and an optional `.claude/settings.local.json`, with `cwd` set to that directory; `--self` never calls `git`, so no git repo is needed. Compare bytes with `cmp`. Cases: (a) no local file, excluded entry present, unsorted order -> file bytes identical; (b) local `{}` -> identical; (c) local adds one allowed and one excluded entry -> allowed added, excluded not added, pre-existing excluded entry kept.
    - Measure during implementation: run the real sequence (`node merge.js --self --write` then `bash commit-settings.sh`) in this worktree with no `settings.local.json` and confirm `git status` stays clean.
    - Record in `learnings.md`: no PreToolUse hook exists in the repo `.claude/settings.json` (no `hooks` key, no `.claude/hooks/`) nor in the user-level settings (its hooks are `SessionStart`, `UserPromptSubmit`, `SessionEnd`, `PostToolUse` only; permissions use `defaultMode: "auto"`). The `merge.js` comment "auto-allowed by a PreToolUse hook" therefore does not hold here; `gh pr view` is only let through by the auto permission mode, not by a hook. Left as an open question for the user (see Trade-offs).

## Trade-offs and risks

- Test harness: decided on a bash `merge.test.sh`, following the existing `*.test.sh` convention under `.claude` (shellchecked by lint).
- Scope of "incoming": with `--self` the only incoming source is `settings.local.json`. In all-worktrees mode (the `merge-settings` skill), other worktrees' `settings.json` are also merged in. Every `override` array is treated as incoming (so entries from another branch's `settings.json` are still screened), which preserves the existing cross-worktree behavior; the destination's own entries are the only ones exempted.
- Skipping the write when nothing was added means an existing unsorted / duplicated `settings.json` is no longer normalized by a no-op run. That is the requested behavior (no churn); normalization now only happens when a real addition is written.
- Open question for the user (not resolved here): `EXCLUDED_PATTERNS` line 55 still rejects `Bash(gh ... view *)` on the premise of a PreToolUse hook that does not exist on this machine. Changing the pattern or its comment is a separate change.
- Risk: `settings-promoter` parses stderr. Adding a line is safe, but renaming or dropping the `Added permissions.*:` lines would break its report. Keep them.

## Progress

- (none yet)
