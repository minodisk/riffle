#!/usr/bin/env bash
# _lib.sh — helpers shared by the develop scripts.
# Source it; do not run it directly.

# wait_for_index_lock
# Call this immediately before any git command that writes the index
# (`git add` / `git commit` / `git mv` / `git checkout -b`, etc.). It waits for
# `.git/worktrees/<worktree>/index.lock` to clear, deleting locks that are
# clearly stale and carrying on.
#
# Why it is needed: an interrupted git process (a Bash tool timeout, an ESC
# interrupt, a subagent killed) leaves a 0-byte index.lock behind. git neither
# retries nor reclaims a stale lock, so every subsequent index write in that
# worktree fails with `Unable to create '...index.lock': File exists.` and
# exit 128. One has been measured surviving over an hour and 40 minutes, while a
# healthy lock lives 1–2 seconds — so anything older than a threshold can safely
# be called a remnant.
#
# The lock's mtime is the only liveness signal available (`pgrep` / `ps` cannot
# run under the sandbox, and `lsof` always returns nothing because git closes
# the fd).
#
# Note: mtime comes from `stat -f %m` (macOS only). The develop scripts assume
# they only run on a local macOS machine, so there is no GNU `stat -c %Y`
# fallback. On Linux `stat` fails, which reads as "the lock disappeared" and
# exits the wait immediately (not the safe side).
#
# Returns: 0 = no lock / it cleared / a stale one was deleted
#          1 = a live lock outlasted the wait limit (the caller dies on errexit)
function wait_for_index_lock() {
	# A healthy lock lives 1–2 seconds, so 15 seconds covers it comfortably. Cut
	# off on elapsed time rather than iteration count (each pass spawns stat and
	# date, so count × interval overshoots the real wait by about 30%; measured:
	# 0.2s × 75 took 20s).
	local max_wait_seconds=15
	local stale_after_seconds=60
	local interval=0.2
	local lock
	local now
	local mtime
	local age
	local deadline
	local size

	lock="$(git rev-parse --git-path index.lock)"
	deadline=$(($(date +%s) + max_wait_seconds))

	while true; do
		if [ ! -e "$lock" ]; then
			return 0
		fi

		# There is a race where the lock disappears between the `-e` test and
		# `stat`. Treat a stat failure as "the lock is gone = it cleared".
		if ! mtime="$(stat -f %m "$lock" 2>/dev/null)"; then
			return 0
		fi
		now="$(date +%s)"
		age=$((now - mtime))

		# A non-zero-byte lock means git has already begun writing the index, so
		# a live process cannot be ruled out; AND in a size of 0. Every stale
		# lock observed so far has been 0 bytes (the signature of a process
		# dying after creating the lock and before writing the index), so this
		# AND costs no reclaim power.
		size="$(stat -f %z "$lock" 2>/dev/null)" || size=""
		if [ "$age" -gt "$stale_after_seconds" ] && [ "$size" = "0" ]; then
			# `rm -f` does not fail if a concurrent script already removed it,
			# which absorbs the double-delete race.
			rm -f "$lock"
			echo "Notice: removed a stale index.lock (age: ${age}s, size: 0)" >&2
			echo "  path: ${lock}" >&2
			echo "  It is the remnant of an interrupted git process. Continuing." >&2
			return 0
		fi

		if [ "$now" -ge "$deadline" ]; then
			break
		fi
		sleep "$interval"
	done

	echo "Error: index.lock was not released within ${max_wait_seconds}s" >&2
	echo "  path: ${lock}" >&2
	echo "  age: ${age}s (within ${stale_after_seconds}s, so not judged stale and not deleted)" >&2
	echo "  Most likely another git process is writing the index." >&2
	echo "  Wait for that process to finish and run this again." >&2
	echo "  If the age keeps growing (the mtime is not being updated) it is a remnant," >&2
	echo "  and you may delete it by hand: rm -f \"${lock}\"" >&2
	return 1
}

# ensure_upstream <branch>
# Call this right after `git push -u origin <branch>` to verify the upstream was
# actually set. If it was not, run `git branch --set-upstream-to` with retries.
#
# Why it is needed: `git push -u` writes the upstream into .git/config *after*
# sending the ref to the remote, and that config write is guarded by a
# single-shot .git/config.lock (O_EXCL, with no retry, no wait, and no stale
# reclaim). When several worktrees share one .git/config they contend, and
# `git push -u` still returns exit 0 when the write fails (it only prints
# `error: unable to write upstream branch configuration` to stderr). The result
# is a successful push with the upstream missing, and downstream
# pr-resolve-conflict.sh hard-fails on the unset upstream.
#
# Returns: 0 = upstream is set / 1 = repair also failed (the caller dies on errexit)
function ensure_upstream() {
	local branch="$1"
	# Measured: with a 0.2s lock held, it succeeded on the 4th try, 269ms total.
	# 50ms × 20 covers a little over a second.
	local retries=20
	local interval=0.05
	local i
	local err=""

	if git rev-parse --abbrev-ref --symbolic-full-name "${branch}@{upstream}" >/dev/null 2>&1; then
		return 0
	fi

	for ((i = 1; i <= retries; i++)); do
		# Keep each attempt's output and print it all on the final failure.
		# Printing every time gives 20 identical lines under lock contention,
		# and swallowing it hides the real sandbox error
		# (`: Operation not permitted`).
		if err=$(git branch --set-upstream-to="origin/${branch}" "${branch}" 2>&1); then
			return 0
		fi
		sleep "${interval}"
	done

	echo "${err}" >&2
	echo "Error: the push succeeded but setting the upstream failed (branch: ${branch})" >&2
	echo "  The commit and push are done. There is no work to redo." >&2
	echo "  Do not re-run this script (it would fail pointlessly with nothing to commit)." >&2
	echo "  This one command is enough to recover:" >&2
	echo "    git branch --set-upstream-to=origin/${branch} ${branch}" >&2
	echo "  If the error above ends in \`: File exists\` it is .git/config.lock contention;" >&2
	echo "  if \`: Operation not permitted\`, the sandbox refused the write." >&2
	return 1
}
