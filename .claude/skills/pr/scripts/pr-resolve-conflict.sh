#!/usr/bin/env bash
# Resolve conflicts with main by rebasing.
# Exits 2 and prints the situation when it cannot resolve them automatically.
set -euo pipefail

# Safety checks: on a protected branch such as main, mid-rebase, or with no
# upstream, this is dangerous, so fail explicitly
current_branch=$(git rev-parse --abbrev-ref HEAD)
if [[ "$current_branch" == "main" || "$current_branch" == "master" || "$current_branch" == "HEAD" ]]; then
	echo "pr-resolve-conflict.sh: refusing to run on '$current_branch'" >&2
	exit 1
fi
git_dir=$(git rev-parse --git-dir)
if [[ -d "$git_dir/rebase-merge" || -d "$git_dir/rebase-apply" ]]; then
	echo "pr-resolve-conflict.sh: rebase already in progress" >&2
	exit 1
fi
if ! git rev-parse --abbrev-ref --symbolic-full-name '@{u}' >/dev/null 2>&1; then
	echo "pr-resolve-conflict.sh: no upstream set for '$current_branch'" >&2
	exit 1
fi
if ! git diff --quiet || ! git diff --cached --quiet; then
	echo "pr-resolve-conflict.sh: working tree or index has uncommitted changes; commit or stash before rebasing" >&2
	exit 1
fi

git fetch origin

# Run the rebase. Continue even on failure, to judge whether manual resolution is possible
if git rebase origin/main; then
	git push --force-with-lease
	echo "RESOLVED=auto"
	exit 0
fi

# Manual resolution is only needed when the rebase started and stopped partway
if [[ -d "$git_dir/rebase-merge" || -d "$git_dir/rebase-apply" ]]; then
	echo "RESOLVED=manual_required"
	echo "Unresolved conflicting files:"
	git diff --name-only --diff-filter=U
	exit 2
fi

echo "pr-resolve-conflict.sh: git rebase origin/main failed before conflict resolution started" >&2
exit 1
