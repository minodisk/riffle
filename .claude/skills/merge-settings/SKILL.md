---
name: merge-settings
description: Deep-merge settings.json and settings.local.json from all worktrees.
allowed-tools: Bash, Write
---

Follow these steps in order.

## 1. Fetch latest main and create a branch

```bash
git fetch origin main
git checkout -B merge/settings-local origin/main
```

Use `merge/settings-local` as the branch name. The `-B` flag overwrites any
existing branch with the same name.

## 2. Merge settings from all worktrees into settings.json

```bash
node .claude/skills/merge-settings/scripts/merge.js --write
```

Run it with `dangerouslyDisableSandbox: true` (`.claude/` is on the sandbox's
Bash write deny list).

The script collects `settings.json` and `settings.local.json` from **all git
worktrees** (via `git worktree list`) and deep-merges them into the current
`settings.json`. `settings.json` in other worktrees may contain permissions
added on other branches that have not yet been merged into main; merging them
captures those pending changes.

`--self` stops the sweep and targets **only the current worktree**. That is the
path the develop skill's `settings-promoter` uses to ride only its own
`settings.local.json` along in the archive PR before the worktree is discarded,
so do not break that dependency when changing argument handling. Which mode it
ran in shows on stderr's `Mode:` line. The point here is the cross-worktree
sweep, so do not pass `--self`.

Without `--write`, it only prints the merged result to stdout and writes
nothing (for a dry check). **Do not copy that output into
`.claude/settings.json` yourself.** The path where an LLM transcribes hundreds
of lines of JSON was removed after it rewrote an existing entry in a case where
there was nothing to promote.

Entries excluded from the merge automatically (see `EXCLUDED_PATTERNS` in
`scripts/merge.js`):

- An allow-everything wildcard that defeats a per-domain restriction, such as
  `WebFetch(domain:*)`
- `Read(...)` / `Edit(...)` / `Bash(...)` referring to a host-dependent absolute
  path such as `/Users/...`, `/private/tmp/...`, `/tmp/...`, `/home/...`, or
  `/var/folders/...`
- Every `Write(...)`, absolute path or not (an `Edit(...)` allow/deny applies to
  the Write tool as well, so it is redundant)

When you get new feedback of the same kind, do not just delete the entry at
commit time: update `EXCLUDED_PATTERNS` so it cannot creep back in.

The script writes the merged JSON to `.claude/settings.json` in the current
worktree and prints a summary to stderr:

- `Merged settings.json: <path1>, <path2>, ...`
- `Merged settings.local.json: <path1>, <path2>, ...`
- `Added permissions.allow: <n>` / `deny:` / `ask:`

Do **not** touch `settings.json` in other worktrees — those are part of each
branch's state.

**Do not empty `settings.local.json` yet** (that happens after the commit in
the next step). If the `settings.json` that was written turns out to be broken,
you would have deleted the only original first. And what gets emptied here spans
every worktree, so worktrees under active development on other branches would be
caught too.

## 3. Commit changes

Check that `.claude/settings.json` is valid JSON (`.claude/` is outside what
`mise run fmt` / CI check, so if it lands on main broken, the permission
settings stop working in every worktree):

```bash
jq empty .claude/settings.json
```

On a failure, do not commit: restore it with
`git checkout .claude/settings.json` and **re-run step 2's
`merge.js --write`**. `settings.local.json` has not been emptied yet, so the
input is unchanged and re-running is safe.

Commit the merged result:

```bash
git add .claude/settings.json
git commit -m "chore: merge settings from all worktrees"
```

Once the commit succeeds, overwrite each path listed on the
`Merged settings.local.json:` line with `{}\n`, using the Write tool.

## 4. Create a PR

Create the PR with the `/pr --no-branch` skill.

- Title: `chore: merge settings from all worktrees`
- Body:
  `Deep-merge settings.json and settings.local.json from every worktree into settings.json.`

**Do not let automated review be skipped.** This PR folds in permissions from
every worktree, so it carries more than the archive PR the develop skill's
wrap-up creates. Dangerous new entries such as an over-broad wildcard or a
host-dependent absolute path cannot be caught by `EXCLUDED_PATTERNS`
(`EXCLUDED_PATTERNS` is itself an accumulation of mechanized review feedback,
sourced from review in the first place) — only review finds them.

Report the result of each step to the user.
