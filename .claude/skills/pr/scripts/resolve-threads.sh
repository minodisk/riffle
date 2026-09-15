#!/usr/bin/env bash
# Resolve several review threads at once.
#
# Driving the individual `resolve-thread.sh` with `if ! ...; then echo ...; fi`
# or `for id in ...` makes a compound command with branches and loops, which
# triggers a permission prompt every time — so "process them all and report
# failures without swallowing them" is shut inside this script.
#
# An individual failure does not stop the loop (a thread may have been resolved
# externally since the planner ran). Results go to stdout, one line per thread.
#
# usage: resolve-threads.sh <thread-id> [<thread-id>...]
# exit code: 0=all succeeded, 1=one or more failed (see the failed lines), 2=bad arguments
set -euo pipefail

if [[ $# -lt 1 ]]; then
	echo "Usage: $0 <thread-id> [<thread-id>...]" >&2
	exit 2
fi

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

failed=0
for id in "$@"; do
	# Keep stderr on failure so a total wipeout (expired auth, say) is traceable
	if err=$(bash "$script_dir/resolve-thread.sh" "$id" 2>&1); then
		echo "resolved ${id}"
	else
		echo "failed ${id}: $(printf '%s' "$err" | tr '\n' ' ' | cut -c1-200)"
		failed=$((failed + 1))
	fi
done

echo "TOTAL=$# FAILED=${failed}"
[[ "$failed" -eq 0 ]]
