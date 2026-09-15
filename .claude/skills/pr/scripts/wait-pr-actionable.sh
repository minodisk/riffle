#!/usr/bin/env bash
# Poll until the PR reaches a state that needs the agent's judgement, then
# return pr-status.sh's output at that moment verbatim.
#
# The waiting ACTIONs in the pr skill's step 5 (`wait` = checks in progress /
# `review_required` = waiting on the approver) give the agent nothing to decide
# on. Driving them agent-side with `until ...; do sleep 30; done` makes a
# compound command that prompts every time, so the wait loop is shut inside this
# script as one atomic call.
#
# The ACTIONs that end the wait and return:
#   ready / merged / closed / conflict / check_failed / changes_requested /
#   draft / behind / blocked
#
# The default max_wait_sec is 240 (4 minutes). The Bash tool cuts off foreground
# execution at 6 minutes by default and 10 at most, so keep it shorter to always
# land on "the script returns exit 2 itself" (killed by the tool, the exit code
# is unobservable and cannot be branched on). max_wait_sec is the sum of the
# sleeps; real time adds pr-status.sh's runtime on top (a few seconds per call ×
# the number of waits). At 240 it measures out around 5 minutes, comfortably
# under the 6-minute default.
# To wait 30 minutes, start it with run_in_background: true and pass 1800 as the
# third argument.
#
# usage: wait-pr-actionable.sh <PR_number_or_url> [interval_sec=30] [max_wait_sec=240]
# exit code:
#   0 = reached a non-waiting ACTION (branch on the ACTION= line in the output)
#   1 = error (bad arguments / pr-status.sh failed)
#   2 = timeout (prints the last observed state to stdout; calling again is fine)
set -euo pipefail

PR="${1:?usage: wait-pr-actionable.sh <PR_number_or_url> [interval_sec] [max_wait_sec]}"
INTERVAL="${2:-30}"
MAX_WAIT="${3:-240}"

if ! [[ "$INTERVAL" =~ ^[1-9][0-9]*$ ]] || ! [[ "$MAX_WAIT" =~ ^[0-9]+$ ]]; then
	echo "wait-pr-actionable.sh: interval_sec must be a positive integer and max_wait_sec a non-negative integer (got '$INTERVAL' / '$MAX_WAIT')" >&2
	exit 1
fi

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

elapsed=0
while true; do
	out=$(bash "$script_dir/pr-status.sh" "$PR") || exit 1
	action=$(grep -E '^ACTION=' <<<"$out" | cut -d= -f2)
	case "$action" in
	wait | review_required) ;;
	*)
		printf '%s\n' "$out"
		exit 0
		;;
	esac
	if ((elapsed >= MAX_WAIT)); then
		echo "wait-pr-actionable.sh: timed out after ${MAX_WAIT}s (last ACTION=${action})" >&2
		printf '%s\n' "$out"
		exit 2
	fi
	echo "[elapsed ${elapsed}s] ACTION=${action}; retrying in ${INTERVAL}s" >&2
	sleep "$INTERVAL"
	elapsed=$((elapsed + INTERVAL))
done
