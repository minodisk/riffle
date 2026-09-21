#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset

# Tests for wait-post-merge-runs.sh's verdict paths.
#
# The merger branches on the exit code and the STATUS= line, so every case pins
# both, plus the counts and the results table rows. `gh` is replaced by a stub
# placed first on PATH that prints fixture JSON from env vars.
#
# Run it directly, or via `mise run ci`.

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
target="${script_dir}/wait-post-merge-runs.sh"

stub_dir="$(mktemp -d)"
trap 'rm -rf "${stub_dir}"' EXIT

cat >"${stub_dir}/gh" <<'STUB'
#!/usr/bin/env bash
case "$1 $2 ${3:-}" in
"run list --commit="*) printf '%s\n' "${STUB_COMMIT_RUNS}" ;;
"repo view --json"*) echo main ;;
"run list --workflow="*) printf '%s\n' "${STUB_RECENT_RUNS}" ;;
*)
	echo "stub gh: unexpected arguments: $*" >&2
	exit 1
	;;
esac
STUB
chmod +x "${stub_dir}/gh"

failures=0
cases=0

# Indent a multi-line block for the failure output.
function indent() {
	local text="$1"
	printf '  %s\n' "${text//$'\n'/$'\n'  }"
}

# expect <name> <expected exit> <commit runs json> <recent runs json> <expected line>...
# Runs the script against the fixtures and checks the exit code and that each
# expected line is present in stdout.
function expect() {
	local name="$1"
	local expected_status="$2"
	local commit_runs="$3"
	local recent_runs="$4"
	shift 4
	local output status=0

	cases=$((cases + 1))

	output=$(PATH="${stub_dir}:${PATH}" STUB_COMMIT_RUNS="${commit_runs}" STUB_RECENT_RUNS="${recent_runs}" \
		POLL_INTERVAL=1 INITIAL_GRACE=0 bash "${target}" --max-wait=0 deadbeef 2>&1) || status=$?

	if [[ "${status}" -ne "${expected_status}" ]]; then
		echo "FAIL: ${name}: expected exit ${expected_status}, got ${status}" >&2
		indent "${output}" >&2
		failures=$((failures + 1))
		return
	fi

	local line
	for line in "$@"; do
		if ! grep -qxF "${line}" <<<"${output}"; then
			echo "FAIL: ${name}: missing line: ${line}" >&2
			indent "${output}" >&2
			failures=$((failures + 1))
		fi
	done
}

function run() {
	printf '{"workflowName":"%s","status":"completed","conclusion":"%s","url":"https://example.com/%s","createdAt":"%s"}' "$1" "$2" "$1" "$3"
}

ci_ok="$(run CI success 2026-01-01T00:00:00Z)"
release_ok="$(run Release success 2026-01-01T00:00:00Z)"

expect "all success" 0 "[${ci_ok},${release_ok}]" '[]' \
	$'success\tCI\thttps://example.com/CI' \
	$'success\tRelease\thttps://example.com/Release' \
	'TOTAL_COUNT=2' 'FAILED_COUNT=0' 'SUPERSEDED_COUNT=0' 'STATUS=success'

expect "one failure" 1 "[${ci_ok},$(run Release failure 2026-01-01T00:00:00Z)]" '[]' \
	$'failure\tRelease\thttps://example.com/Release' \
	'TOTAL_COUNT=2' 'FAILED_COUNT=1' 'SUPERSEDED_COUNT=0' 'STATUS=failed'

expect "cancelled superseded" 0 "[${ci_ok},$(run Release cancelled 2026-01-01T00:00:00Z)]" \
	'[{"status":"completed","conclusion":"success","createdAt":"2026-01-01T00:05:00Z"}]' \
	$'superseded\tRelease\thttps://example.com/Release' \
	'TOTAL_COUNT=2' 'FAILED_COUNT=0' 'SUPERSEDED_COUNT=1' 'STATUS=success'

expect "no runs" 0 '[]' '[]' 'STATUS=no_runs'

if [[ "${failures}" -gt 0 ]]; then
	echo "wait-post-merge-runs: ${failures} failure(s) in ${cases} cases" >&2
	exit 1
fi

echo "wait-post-merge-runs: ${cases} cases passed"
