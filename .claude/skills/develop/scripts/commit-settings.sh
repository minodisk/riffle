#!/usr/bin/env bash
set -euo pipefail

# Usage: commit-settings.sh
# Stage and commit .claude/settings.json if it has a diff. With no diff it does
# nothing and exits successfully (the usual case, where settings.local.json was
# empty).
#
# It does not push. archive-plan.sh, called right after, pushes everything
# together, keeping this chore branch to one push (an intermediate push advances
# the head, which makes the review GitHub requests at PR creation stale against
# an older head — the same reasoning as the develop skill's policy of batching
# pushes before PR creation). Phase S (single-PR mode) does not call
# archive-plan.sh, so there it is the S.6 pr-runner that pushes this commit.
#
# `.claude/settings.json` is write-protected under the sandbox, so on failure
# re-run with `dangerouslyDisableSandbox: true`.

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=_lib.sh
source "$script_dir/_lib.sh"

# To keep the "commit only the given paths" premise, verify the index is empty
# first. Pre-existing staged changes would be mixed into the same commit.
if ! git diff --cached --quiet; then
	echo "Error: staged changes already present; call commit-settings.sh with a clean index" >&2
	exit 1
fi

if git diff --quiet -- .claude/settings.json; then
	echo "No settings changes to promote."
	exit 0
fi

# merge.js --write writes the file directly, so there is no transcription route,
# but validate anyway in case a script bug writes broken JSON. `.claude/` is
# outside what `mise run fmt` / CI check, and if it lands on main broken, the
# permission settings stop working in every worktree — hence the check here.
if ! node -e "JSON.parse(require('fs').readFileSync('.claude/settings.json','utf8'))"; then
	echo "Error: .claude/settings.json is not valid JSON. Aborted the commit" >&2
	exit 1
fi

wait_for_index_lock

git add .claude/settings.json
git commit -m "chore(claude): promote worktree-local settings"
