#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset

# Create the plan folder docs/plans/{YYYYMMDD}-{feature-name}/ and print its
# path to stdout. The caller uses that path as the Write tool's destination.
#
# Getting the date, deciding the path, checking for an existing folder, and
# mkdir are all deterministic steps needing no LLM judgement, so they are shut
# inside a script rather than composed agent-side (which would be a compound
# command with substitutions and branches, prompting every time).
#
# The existence check looks at **both** the unarchived docs/plans/ and
# docs/plans/_archived/. Miss the archive side and it only fails at the very end
# as a destination collision in archive-plan.sh. And silently reusing an
# existing directory would overwrite an earlier run's plan when plan.md is
# written next, leaving learnings.md / design-decisions.md orphaned. That is why
# the plan folder itself is created with `mkdir`, not `mkdir -p`.
#
# Run it at the repository root (paths are relative to it). With a cwd anywhere
# else the `-e` checks are always false, the existence check misses, and it
# could create a fresh `docs/plans` right there — so the presence of
# `docs/plans` is verified up front.

function usage() {
	cat <<EOF
Usage: $0 <feature-name>

Create docs/plans/{YYYYMMDD}-{feature-name}/ and print its path to stdout.

Arguments:
  feature-name    Feature name in kebab-case (^[a-z0-9]+(-[a-z0-9]+)*\$)

Exit status:
  0    Created (the directory path on stdout)
  1    feature-name is not kebab-case, a plan folder of that name already
       exists, or this was not run at the repository root (docs/plans not found)
  2    Bad arguments
EOF
}

if [[ "${1:-}" == "-h" ]] || [[ "${1:-}" == "--help" ]]; then
	usage
	exit 0
fi

if [[ $# -ne 1 ]] || [[ -z "${1:-}" ]]; then
	usage >&2
	exit 2
fi

feature="$1"

if [[ ! "$feature" =~ ^[a-z0-9]+(-[a-z0-9]+)*$ ]]; then
	echo "Error: feature name must be kebab-case (^[a-z0-9]+(-[a-z0-9]+)*\$): ${feature}" >&2
	exit 1
fi

if [[ ! -d docs/plans ]]; then
	echo "Error: docs/plans not found. Run this at the repository root (cwd: $PWD)" >&2
	exit 1
fi

today="$(date +%Y%m%d)"
plan_dir="docs/plans/${today}-${feature}"
archived_dir="docs/plans/_archived/${today}-${feature}"

for existing in "$plan_dir" "$archived_dir"; do
	if [[ -e "$existing" ]]; then
		echo "Error: plan dir already exists: ${existing}" >&2
		echo "A plan with the same feature name already exists for today (a redo / parallel work / already archived)." >&2
		echo "Change the feature name, or tidy the earlier plan folder, and re-run." >&2
		exit 1
	fi
done

mkdir "$plan_dir"

printf '%s\n' "$plan_dir"
