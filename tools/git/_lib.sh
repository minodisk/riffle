#!/usr/bin/env bash
# _lib.sh — helpers shared by the scripts under tools/git.
# Source it; do not run it directly.

# remove_branch_section <branch>
# Call this right after `git branch -d` / `git branch -D` to delete the deleted
# branch's `[branch "<name>"]` section from .git/config.
#
# Why it is needed: config writes are guarded by a single-shot .git/config.lock
# (O_EXCL, with no retry, no wait, and no stale reclaim), and several worktrees
# sharing one .git/config contend for it. `git branch -d` returns exit 0 even
# when it fails to delete the section (it only prints
# `error: could not lock config file <path>` and
# `warning: update of config-file failed` to stderr), so an orphaned section
# lingers.
#
# That warning from `git branch -d` itself cannot and will not be suppressed.
# git takes the config lock even when deleting a branch with no section, so the
# warning cannot be prevented, and filtering stderr would also hide a real
# sandbox-induced error (`: Operation not permitted`).
#
# Exit codes measured on git 2.52.0:
#   0   = deleted
#   128 = `fatal: no such section: branch.<name>` (there was no section)
#   255 = `error: could not lock config file ...` (lock contention / sandbox refusal)
# The lock check runs before the section-existence check, so while locked even a
# missing section gives 255. Only 128 is therefore treated as "nothing to clean".
#
# Returns: 0 = the section is gone, or never existed /
#          1 = could not remove it even with retries (git's stderr already printed)
function remove_branch_section() {
	local branch="$1"
	# Measured: with a 0.2s lock held, it succeeded on the 4th try, 269ms total.
	# 50ms × 20 covers a little over a second.
	local retries=20
	local interval=0.05
	local i
	local err=""
	local status=0

	for ((i = 1; i <= retries; i++)); do
		# Keep each attempt's output and print it all on the final failure.
		# Printing every time gives 20 identical lines under lock contention.
		err=$(git config --local --remove-section "branch.${branch}" 2>&1) && return 0
		status=$?
		if [[ "${status}" -eq 128 ]]; then
			return 0
		fi
		sleep "${interval}"
	done

	echo "${err}" >&2
	return 1
}

# set_branch_upstream <branch> <upstream>
# Runs `git branch --set-upstream-to=<upstream> <branch>` with retries.
#
# `git push -u` returns exit 0 even when the config write fails, leaving a
# branch that pushed successfully but has no upstream. This recovers it.
# `--set-upstream-to` returns exit 1 on lock contention (asymmetric with
# `git push -u`'s exit 0), so retrying on its return value is enough.
#
# Returns: 0 = set / 1 = could not set it even with retries
#          (git's stderr already printed)
function set_branch_upstream() {
	local branch="$1"
	local upstream="$2"
	# Same basis as remove_branch_section (measured: 4th try, 269ms total).
	local retries=20
	local interval=0.05
	local i
	local err=""

	for ((i = 1; i <= retries; i++)); do
		# Keep each attempt's output and print it all on the final failure.
		err=$(git branch --set-upstream-to="${upstream}" "${branch}" 2>&1) && return 0
		sleep "${interval}"
	done

	echo "${err}" >&2
	return 1
}
