#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset

# Create a PR and print its URL to stdout (gh pr create's own output).
#
# The caller writes the body to a file with the Write tool and passes the path.
# Composing `gh pr create --body $'...'` directly agent-side catches on the
# permission analyzer because of the `$`, prompting every time, and it also
# leaks a literal `\n` into the PR body. Going through a file makes newlines and
# quoting a non-issue, and closes the permission down to this one script entry.
#
# Usage: create-pr.sh <title> <body-file>

function usage() {
	cat <<EOF
Usage: $0 <title> <body-file>

Create a PR with gh pr create, and print the PR URL to stdout.

Arguments:
  title        PR title (English, Conventional Commits)
  body-file    Path to the file holding the PR body (write it with the Write tool first)

Exit status:
  0    Created (PR URL on stdout)
  1    The body file does not exist
  2    Bad arguments
  any other non-zero  Passes through gh pr create's exit code
EOF
}

if [[ "${1:-}" == "-h" ]] || [[ "${1:-}" == "--help" ]]; then
	usage
	exit 0
fi

if [[ $# -ne 2 ]] || [[ -z "${1:-}" ]] || [[ -z "${2:-}" ]]; then
	usage >&2
	exit 2
fi

title="$1"
body_file="$2"

if [[ ! -f "$body_file" ]]; then
	echo "Error: body file not found: ${body_file}" >&2
	exit 1
fi

gh pr create --title "$title" --body-file "$body_file"
