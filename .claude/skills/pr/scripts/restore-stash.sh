#!/usr/bin/env bash
# Create a new branch and restore only "this run's stash", the one
# `stash-work.sh` recorded in the marker. Unrelated stashes (other sessions,
# past work) are left alone.
#
# It drops the stash and deletes the marker only on a successful apply. On a
# failure such as a conflict, both the stash and the marker survive and it exits
# non-zero, leaving things recoverable by hand.
#
# usage: restore-stash.sh <branch-name>
# exit code:
#   0 = ok (see RESTORED= on stdout for whether anything was restored)
#   1 = stash apply failed (stash and marker survive; needs manual resolution)
#   2 = bad arguments
#   3 = branch creation failed (the stash was not touched)
set -euo pipefail

if [[ $# -ne 1 ]] || [[ -z "${1:-}" ]]; then
	echo "Usage: $0 <branch-name>" >&2
	exit 2
fi

branch="$1"
if ! git checkout -b "$branch"; then
	echo "restore-stash.sh: failed to create branch ${branch}; the stash is still there" >&2
	exit 3
fi

git_dir=$(git rev-parse --git-dir)
marker="${git_dir}/PR_SKILL_STASH_TAG"

if [[ ! -f "$marker" ]]; then
	echo "RESTORED=false (no stash marker)"
	exit 0
fi

tag=$(cat "$marker")
# With no stash matching the tag, grep exits non-zero. Under `set -o pipefail`
# the assignment itself fails and errexit kills us before reaching the "not
# found" branch below, so swallow it with `|| true` (not finding one is not an
# error).
stash_ref=$(git stash list | grep -F "$tag" | head -n1 | cut -d: -f1 || true)

if [[ -z "$stash_ref" ]]; then
	# If no matching stash is found, just clean up the marker
	rm -f "$marker"
	echo "RESTORED=false (stash ${tag} not found)"
	exit 0
fi

if git stash apply "$stash_ref"; then
	git stash drop "$stash_ref"
	rm -f "$marker"
	echo "RESTORED=true"
	exit 0
fi

echo "stash apply failed; kept stash ${stash_ref} (${tag}). Resolve it by hand" >&2
exit 1
