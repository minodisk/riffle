#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset

# A watchdog that tells the main session whether the local review
# (`local-review-runner`) finished or ran out of time. Start it with the Bash
# tool's `run_in_background: true` and rely on the harness waking main when it
# exits.
#
# Background: when a grandchild agent (`local-review-addresser` /
# `local-review-reviewer`) dies on the harness side, its failure notification is
# never delivered to the parent subagent (`local-review-runner`), and both the
# runner and main wait forever (measured: main idle for 3 hours). This script's
# job is to bring main back through a path that does not depend on
# subagent-to-subagent notification delivery at all; it takes no part in the
# review itself.
#
# The completion signal: **the review history file exists in the tree of an
# origin branch head that has advanced past the starting point.** "Is there a
# commit in the review history directory" cannot be used:
# `local-review-addresser` **commits but does not push** the fixes and the
# review history every round (step 6 of
# `.claude/agents/local-review-addresser.md`), so commit detection would
# misread an intermediate round as completion. Only the terminal
# `commit-review-history.sh` pushes, so the push alone identifies the end.
#
# Requiring "a head that advanced past the starting point" is because the review
# history filename has minute granularity (`review-YYYYMMDD-HHmm.md`). Review
# the same branch again within the same minute and this run's target file can
# already exist in the origin head's tree at start time (the previous run's push
# having landed). Judging that as completion with a plain existence check would
# return `result=completed` at around waited=0 and end the watchdog even though
# this review has made no progress at all — leaving no way to detect the runner
# getting stuck in that round. Recording the head SHA at start time and only
# treating a different head as completion prevents this.
#
# The starting head SHA (start_sha) is handled as three distinct states:
# "unconfirmed", "a fetched SHA", and "confirmed that the remote has no such
# branch" (the empty string). Simply using "empty string on a fetch failure"
# makes a transient network failure indistinguishable from a definite empty, and
# the moment a later call happens to succeed, an existing review history of the
# same name is wrongly reported as `result=completed`. So while it is
# unconfirmed the main loop does not run the completion check at all and retries
# the fetch every `interval_seconds` until it is confirmed (the deadline keeps
# the loop bounded). A confirmed baseline is written to a state file determined
# uniquely by `--run-id` and reused across re-arms (restarts with the same
# arguments). Without this, each re-arm would take a fresh baseline and absorb a
# terminal push that happened while the watchdog was down (between the previous
# exit and this start) as "this run's starting point", making that push
# undetectable as completion.
#
# `--run-id` is a caller-issued token uniquely identifying "this series of
# re-arms", and a `--branch` + `--review-file` pair cannot substitute for it.
# Because the review history filename has minute granularity, independent review
# runs can share the same branch + review_file (for example, a previous run
# `TaskStop`ped after the terminal push but before the next poll, leaving its
# state file undeleted, while the same branch is reviewed again within the same
# minute). Keying the state file on branch + review_file alone would make this
# second run mistake the first one's remnant for "its own re-arm" and, with a
# remote head already past the old start_sha and a same-named file present,
# satisfy the condition on the very first pass and return `result=completed`
# immediately. The run-id is managed by the caller (generated at first start and
# passed through unchanged on re-arms), so the two runs stay reliably distinct.
# If the runner returns before the watchdog, the caller must explicitly delete
# that run-id's state file with `--cleanup` (below).
#
# The caller generates the run-id with `date +%s` (a UNIX timestamp at second
# precision). Second precision rather than a UUID is enough because two
# independent reviews of the same branch starting in the same second does not
# happen in practice, and `date +%s` can be generated with `Bash(date *)`, which
# is already on the `.claude/settings.json` allowlist — no new command such as
# `uuidgen` needs a permission entry.
#
# `git ls-remote` is a network call and can fail to respond, hanging on a
# credential prompt or on the connection. On top of disabling prompts with
# `GIT_TERMINAL_PROMPT=0`, and because macOS has no `timeout(1)`, the
# "background it and cut it off by polling" approach below
# (`ls_remote_heads_with_deadline`) keeps every single call within the polling
# interval (and within the time remaining until the deadline). A `git ls-remote`
# hang therefore cannot block the return to the deadline check.
#
# The main loop recomputes the time remaining until the deadline every single
# time: for each network call (the fetch while start_sha is unconfirmed, and the
# `review_file_pushed` fetch) and right before the `sleep` at the end of the
# loop. Computing it once at the top of the cycle and reusing it would have the
# two consecutive ls-remotes of an unconfirmed-start_sha cycle use the same
# stale `net_timeout_seconds`; if the first waited until just before the
# deadline and succeeded and the second then hung, it would burn the whole
# remaining budget a second time and overshoot the deadline. And because
# `ls_remote_heads_with_deadline` inserts a fixed 1-second `sleep` as a grace
# period after sending TERM, that 1 second is also subtracted from the limit
# passed in, so a single call never exceeds the caller's stated limit (see that
# function's comment). Together these keep the whole process, network calls
# included, from overshooting the deadline.
#
# A transient `git ls-remote` failure (a network drop, say) is treated as "no
# signal" and the loop continues. The deadline always bounds the loop, so this
# is fail-safe: even under permanent failure it functions as a pure timer (the
# deadline fires and the caller falls through to checking state). It is
# therefore not split into its own exit code (the caller's response is the same
# as for a deadline).
#
# It performs only read-only git operations, so it can run inside the sandbox.

function usage() {
	cat <<EOF
Usage: $0 --branch='<branch>' --review-file='<path>' --run-id='<id>' [--deadline-seconds='<N>'] [--interval-seconds='<N>']
       $0 --run-id='<id>' --cleanup

Wait until the local review finishes (the review history file is pushed to an
origin head of that branch which has advanced past the starting point), or
until the deadline is reached.

Options:
  --branch='<branch>'           Branch to watch (required unless --cleanup)
  --review-file='<path>'        Repo-relative path of the review history file (required unless --cleanup)
  --run-id='<id>'               Caller-issued token uniquely identifying this
                                series of re-arms (required). Generate it at
                                first start and pass the same value on re-arms
  --deadline-seconds='<N>'      Maximum seconds to wait (default: 1800)
  --interval-seconds='<N>'      Polling interval in seconds (default: 30)
  --cleanup                     Do not wait; delete the state file for --run-id
                                and exit (used by the caller when the runner
                                returned before the watchdog)
  -h, --help                    Show this help

Exit status:
  0    Detected the completion signal, or --cleanup finished
       LOCAL_REVIEW_WATCH result=completed waited=<s> review_file=<path>
  1    Deadline exceeded (the caller should go check the runner's state)
       LOCAL_REVIEW_WATCH result=deadline waited=<s> deadline=<s>
  2    Bad arguments
EOF
}

# Reliably cut off git ls-remote within the deadline. macOS has no timeout(1),
# so background it and cut it off by polling. Disable credential prompts with
# GIT_TERMINAL_PROMPT=0 so it cannot hang on one. Returns stdout as-is.
#
# On timeout it inserts a fixed 1-second `sleep` to wait for the process to exit
# after sending TERM (KILLing immediately can leave the output file
# incomplete). That 1 second would otherwise be added outside `seconds` and
# could exceed the caller's stated limit, so the polling wait limit is narrowed
# to `seconds - 1`, keeping the total including the grace period within
# `seconds`.
#
# Returns: 0 = fetched (result on stdout) / 1 = failure or timeout
function ls_remote_heads_with_deadline() {
	local branch="$1"
	local seconds="$2"
	local output_file
	output_file="$(mktemp "${TMPDIR:-/tmp}/watch-local-review-ls-remote.XXXXXX")"

	GIT_TERMINAL_PROMPT=0 git ls-remote --heads origin "${branch}" >"${output_file}" 2>/dev/null &
	local pid=$!
	local waited=0
	local rc=0
	# Polling limit with the fixed 1-second post-TERM grace period subtracted.
	local poll_limit=$((seconds - 1))
	if [[ "${poll_limit}" -lt 0 ]]; then
		poll_limit=0
	fi

	while kill -0 "${pid}" 2>/dev/null; do
		if [[ "${waited}" -ge "${poll_limit}" ]]; then
			kill -TERM "${pid}" 2>/dev/null || true
			sleep 1
			kill -KILL "${pid}" 2>/dev/null || true
			wait "${pid}" 2>/dev/null || true
			rm -f "${output_file}"
			return 1
		fi
		sleep 1
		waited=$((waited + 1))
	done

	wait "${pid}" || rc=$?
	cat "${output_file}"
	rm -f "${output_file}"
	return "${rc}"
}

# Print the head SHA of <branch> on origin. Non-zero if it cannot be fetched.
function remote_head_sha() {
	local branch="$1"
	local timeout_seconds="$2"
	local output

	if ! output="$(ls_remote_heads_with_deadline "${branch}" "${timeout_seconds}")"; then
		return 1
	fi

	# A valid line is only ever "<sha>\trefs/heads/<branch>". Ignore any line
	# that does not match, such as a stray credential helper warning.
	awk -v ref="refs/heads/${branch}" '$2 == ref { print $1; exit }' <<<"${output}"
}

# Path of the state file that carries the baseline SHA (start_sha) across
# re-arms. The run-id is a token the caller issues for "this series of re-arms",
# so keying on it alone cannot collide with another run (including an
# independent review run that happens to share branch + review_file). See the
# header comment, which also explains why branch + review_file cannot be the
# key.
function state_file_path() {
	local run_id="$1"
	local sanitized="${run_id//\//_}"
	echo "${TMPDIR:-/tmp}/watch-local-review-state/${sanitized}.start_sha"
}

# Return 0 if the review history file is in the tree of an origin head that has
# advanced past the starting point (start_sha). A head equal to start_sha is
# treated as "this review has not pushed yet" (the filename collision guard; see
# the header comment).
function review_file_pushed() {
	local branch="$1"
	local review_file="$2"
	local timeout_seconds="$3"
	local start_sha="$4"
	local sha

	sha="$(remote_head_sha "${branch}" "${timeout_seconds}")" || return 1
	if [[ -z "${sha}" ]]; then
		return 1
	fi

	if [[ -n "${start_sha}" && "${sha}" == "${start_sha}" ]]; then
		return 1
	fi

	# The push comes from this worktree, so the head SHA is normally local; if
	# it is absent (pushed from another machine, say) we cannot judge, so treat
	# it as no signal.
	git cat-file -e "${sha}^{commit}" 2>/dev/null || return 1

	local found
	found="$(git ls-tree -r --name-only "${sha}" -- "${review_file}" 2>/dev/null)" || return 1
	[[ -n "${found}" ]]
}

function main() {
	local branch=""
	local review_file=""
	local run_id=""
	local deadline_seconds=1800
	local interval_seconds=30
	local cleanup=false

	while [[ $# -gt 0 ]]; do
		case $1 in
		-h | --help)
			usage
			exit 0
			;;
		--branch=*)
			branch="${1#*=}"
			shift
			;;
		--review-file=*)
			review_file="${1#*=}"
			shift
			;;
		--run-id=*)
			run_id="${1#*=}"
			shift
			;;
		--deadline-seconds=*)
			deadline_seconds="${1#*=}"
			shift
			;;
		--interval-seconds=*)
			interval_seconds="${1#*=}"
			shift
			;;
		--cleanup)
			cleanup=true
			shift
			;;
		*)
			echo "Error: unknown option: $1" >&2
			usage >&2
			exit 2
			;;
		esac
	done

	if [[ -z "${run_id}" ]]; then
		echo "Error: --run-id is required" >&2
		usage >&2
		exit 2
	fi

	if [[ "${cleanup}" == true ]]; then
		rm -f "$(state_file_path "${run_id}")"
		exit 0
	fi

	if [[ -z "${branch}" ]]; then
		echo "Error: --branch is required" >&2
		usage >&2
		exit 2
	fi

	if [[ -z "${review_file}" ]]; then
		echo "Error: --review-file is required" >&2
		usage >&2
		exit 2
	fi

	if ! [[ "${deadline_seconds}" =~ ^[1-9][0-9]*$ ]]; then
		echo "Error: --deadline-seconds must be a positive integer: ${deadline_seconds}" >&2
		exit 2
	fi

	if ! [[ "${interval_seconds}" =~ ^[1-9][0-9]*$ ]]; then
		echo "Error: --interval-seconds must be a positive integer: ${interval_seconds}" >&2
		exit 2
	fi

	# The state file carrying the starting head (start_sha) across re-arms (see
	# the header comment). If it is already confirmed, use it and skip the live
	# fetch.
	local state_file
	state_file="$(state_file_path "${run_id}")"
	mkdir -p "$(dirname "${state_file}")"

	local start_sha=""
	local start_sha_confirmed=false
	if [[ -f "${state_file}" ]]; then
		start_sha="$(cat "${state_file}")"
		start_sha_confirmed=true
	fi

	local start_epoch
	start_epoch="$(date +%s)"

	local waited
	local remaining
	local net_timeout_seconds
	local sleep_seconds
	while true; do
		# Recompute the time remaining until the deadline in three places every
		# pass: at the top of the cycle, after the unconfirmed-start_sha fetch
		# (just before calling review_file_pushed), and just before the sleep.
		# Computing it once at the top and reusing it for the two following
		# network calls leaves a stale remaining that is larger than reality by
		# however long the first call took, and if the second call hangs it
		# burns the whole remaining budget again and overshoots the deadline.
		waited=$(($(date +%s) - start_epoch))
		remaining=$((deadline_seconds - waited))
		if [[ "${remaining}" -le 0 ]]; then
			echo "LOCAL_REVIEW_WATCH result=deadline waited=${waited} deadline=${deadline_seconds}"
			exit 1
		fi

		# While unconfirmed we cannot distinguish "transient network failure"
		# from "confirmed that the remote has no such branch", so skip the
		# completion check entirely and retry the fetch (see the header
		# comment). Once confirmed, write it to the state file so later re-arms
		# can reuse it.
		if [[ "${start_sha_confirmed}" != true ]]; then
			net_timeout_seconds="${interval_seconds}"
			if [[ "${net_timeout_seconds}" -gt "${remaining}" ]]; then
				net_timeout_seconds="${remaining}"
			fi
			if start_sha="$(remote_head_sha "${branch}" "${net_timeout_seconds}")"; then
				start_sha_confirmed=true
				printf '%s' "${start_sha}" >"${state_file}"
			fi
		fi

		# Recompute the remaining time to account for the fetch just above
		# (which only happens while start_sha is unconfirmed). If remaining is
		# exhausted here, fall straight through to the deadline check without
		# calling review_file_pushed.
		waited=$(($(date +%s) - start_epoch))
		remaining=$((deadline_seconds - waited))
		if [[ "${remaining}" -le 0 ]]; then
			echo "LOCAL_REVIEW_WATCH result=deadline waited=${waited} deadline=${deadline_seconds}"
			exit 1
		fi

		if [[ "${start_sha_confirmed}" == true ]]; then
			net_timeout_seconds="${interval_seconds}"
			if [[ "${net_timeout_seconds}" -gt "${remaining}" ]]; then
				net_timeout_seconds="${remaining}"
			fi
			if review_file_pushed "${branch}" "${review_file}" "${net_timeout_seconds}" "${start_sha}"; then
				waited=$(($(date +%s) - start_epoch))
				echo "LOCAL_REVIEW_WATCH result=completed waited=${waited} review_file=${review_file}"
				rm -f "${state_file}"
				exit 0
			fi
		fi

		# Recompute the remaining time before the sleep too, to account for the
		# time the two calls above consumed.
		waited=$(($(date +%s) - start_epoch))
		remaining=$((deadline_seconds - waited))
		if [[ "${remaining}" -le 0 ]]; then
			echo "LOCAL_REVIEW_WATCH result=deadline waited=${waited} deadline=${deadline_seconds}"
			exit 1
		fi

		sleep_seconds="${interval_seconds}"
		if [[ "${sleep_seconds}" -gt "${remaining}" ]]; then
			sleep_seconds="${remaining}"
		fi
		sleep "${sleep_seconds}"
	done
}

main "$@"
