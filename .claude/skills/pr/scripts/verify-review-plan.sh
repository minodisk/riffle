#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset

# Verify the plan JSON returned by pr-review-planner against the PR's actual
# unresolved review threads. The planner is an LLM and can mismatch a thread_id
# with a path/plan, but path / line have a mechanical source of truth
# (list-unresolved-threads.sh's output), so cross-checking detects and rejects
# transcription errors.
#
# Output goes to stdout, one line per entry.
#   mismatch <id>: ...  path / line disagrees with the actual thread (a transcription error)
#   missing <id>        the id is not in the unresolved thread list (treated as a warning)
#
# exit code:
#   0 = everything matched (missing alone is still 0: a fabricated id from the
#       planner cannot be distinguished from an externally resolved thread, so
#       it stays a warning)
#   1 = one or more mismatches
#   2 = bad arguments / the plan JSON does not read as an array
#   3 = list-unresolved-threads.sh itself failed (API outage, expired auth, ...).
#       This is not a transcription error, so the caller must not confuse it
#       with a mismatch

function usage() {
	cat <<EOF
Usage: $0 <pr-number> <plan-json-file>

Cross-check pr-review-planner's plan JSON against the PR's unresolved review threads.

Arguments:
  pr-number        PR number
  plan-json-file   Path to the file holding the plan JSON array
EOF
}

if [[ $# -ne 2 ]]; then
	usage >&2
	exit 2
fi

pr=$1
plan_file=$2

if [[ ! -f "${plan_file}" ]]; then
	echo "Error: plan JSON file not found: ${plan_file}" >&2
	exit 2
fi

if ! jq -e 'type == "array"' "${plan_file}" >/dev/null 2>&1; then
	echo "Error: plan JSON must be a JSON array: ${plan_file}" >&2
	exit 2
fi

if ! jq -e '
  all(.[];
    (.thread_id | type) == "string"
    and (.path | type) == "string"
    and ((.line | type) == "number" or (.line | type) == "null")
  )
' "${plan_file}" >/dev/null 2>&1; then
	echo "Error: plan JSON entries must have thread_id/path as string and line as number or null: ${plan_file}" >&2
	exit 2
fi

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
if ! threads=$(bash "${script_dir}/list-unresolved-threads.sh" "${pr}"); then
	echo "Error: list-unresolved-threads.sh failed for PR ${pr}" >&2
	exit 3
fi

# line can be null, so render the comparison with tojson (so a null is never
# mistaken for a number and shown as a match). jq's == treats null vs number as
# unequal.
report=$(jq -r --argjson threads "${threads}" '
  .[]
  | . as $e
  | ($threads | map(select(.id == $e.thread_id)) | first) as $t
  | if $t == null then
      "missing \($e.thread_id)"
    elif ($e.path != $t.path) or ($e.line != $t.line) then
      "mismatch \($e.thread_id): path=\($e.path | tojson) want \($t.path | tojson), line=\($e.line | tojson) want \($t.line | tojson)"
    else
      empty
    end
' "${plan_file}")

if [[ -n "${report}" ]]; then
	printf '%s\n' "${report}"
fi

total=$(jq -r 'length' "${plan_file}")
mismatched=$(printf '%s\n' "${report}" | grep -c '^mismatch ' || true)
missing=$(printf '%s\n' "${report}" | grep -c '^missing ' || true)

echo "TOTAL=${total} MISMATCH=${mismatched} MISSING=${missing}"
[[ "${mismatched}" -eq 0 ]]
