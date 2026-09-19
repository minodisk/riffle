#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset

# https://github.com/not-an-aardvark/git-delete-squashed
#
# Sandbox: this script assumes it is always called with the Bash tool's
# `dangerouslyDisableSandbox: true`. `cleanup_branch_section` (below) deletes
# the deleted branch's `[branch "<name>"]` section from the shared `.git/config`
# (the main repository's `.git/config`, shared even from a linked worktree), and
# under the sandbox that write always fails with
# `could not lock config file ...: Operation not permitted` (measured;
# deterministic, not probabilistic).

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=_lib.sh
source "$script_dir/_lib.sh"

# Clean up the `[branch "<name>"]` section left behind after deleting a branch.
# `git branch -d`/-D returns exit 0 even when the config write fails, so under
# lock contention an orphaned section lingers in .git/config.
# The `error: could not lock config file` warning `git branch -d` itself prints
# is git's own behavior and cannot and will not be suppressed (no stderr
# filtering: it would also hide real sandbox-induced errors).
#
# A failed cleanup does not stop the remaining branches. The branch deletion
# itself succeeded and an orphaned section is self-repairable, so one cleanup
# failure is no reason to stop the whole deletion pass. The leftovers can be
# collected with the manual command the echo below prints.
cleanup_branch_section() {
	local branch="$1"
	if ! remove_branch_section "$branch"; then
		echo "Warning: could not delete the [branch \"${branch}\"] section" >&2
		echo "  The branch deletion itself succeeded. Collect the leftover with:" >&2
		echo "    git config --local --remove-section branch.${branch}" >&2
	fi
}

# A suppression directive only applies to the block right after it, so it goes
# immediately before this function.
# shellcheck disable=SC2046,SC1083,SC2162
delete_merged_branches() {
	local targetBranch targetRef
	local headRef
	if ! headRef=$(git symbolic-ref -q refs/remotes/origin/HEAD); then
		git remote set-head origin -a >/dev/null 2>&1 || true
		if ! headRef=$(git symbolic-ref -q refs/remotes/origin/HEAD); then
			echo "Error: refs/remotes/origin/HEAD is not set. Run: git remote set-head origin -a" >&2
			exit 1
		fi
	fi
	targetBranch=${headRef#refs/remotes/origin/}
	# The basis for the judgement is the latest remote tip (origin/<default>).
	targetRef="origin/${targetBranch}"

	# Detach HEAD at origin/<default>. This aligns the basis on which the later
	# `git branch -d` decides "is it merged into HEAD" with targetRef, avoiding
	# the case where a branch is merged yet -d bails out. Attaching to the local
	# <default> branch (`git checkout <default>`) would conflict when another
	# worktree holds it, but detaching to a commit is not subject to that.
	git checkout -q --detach "$targetRef"

	# Delete merged branches. The worktreepath field is non-empty when the
	# branch is checked out in any worktree (including the current one), so
	# skipping those also skips the target/current branch. for-each-ref lists
	# only refs/heads/, so (unlike `git branch`) the detached-HEAD pseudo-entry
	# never appears even when HEAD is detached at origin/main.
	git for-each-ref --merged="$targetRef" refs/heads/ "--format=%(refname:short)%09%(worktreepath)" |
		while IFS=$'\t' read -r branch worktreepath; do
			if [[ "$branch" == "$targetBranch" || -n "$worktreepath" ]]; then
				continue
			fi
			echo "  Deleting: $branch"
			git branch -d "$branch"
			cleanup_branch_section "$branch"
		done

	# Delete squashed branches
	git for-each-ref refs/heads/ "--format=%(refname:short)%09%(worktreepath)" |
		while IFS=$'\t' read -r branch worktreepath; do
			# Skip target branch and branches checked out in any worktree
			if [[ "$branch" == "$targetBranch" || -n "$worktreepath" ]]; then
				continue
			fi

			# Check if branch is squashed
			mergeBase=$(git merge-base "$targetRef" "$branch" 2>/dev/null || true)
			if [[ -n "$mergeBase" ]]; then
				# Create a temporary commit with the same tree as the branch
				tempCommit=$(git commit-tree "$(git rev-parse "$branch"^{tree})" -p "$mergeBase" -m "temp" 2>/dev/null || true)
				if [[ -n "$tempCommit" ]]; then
					cherryResult=$(git cherry "$targetRef" "$tempCommit" 2>/dev/null || true)
					if [[ "$cherryResult" == "-"* ]]; then
						git branch -D "$branch"
						cleanup_branch_section "$branch"
					fi
				fi
			fi
		done
}

delete_merged_branches
