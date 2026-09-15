#!/usr/bin/env bash
# Usage: resolve-thread.sh <thread-id>
# Resolve a review thread.
set -euo pipefail

if [ $# -lt 1 ]; then
	echo "Usage: $0 <thread-id>" >&2
	exit 1
fi

# shellcheck disable=SC2016
gh api graphql -f query='
  mutation($id:ID!) {
    resolveReviewThread(input:{threadId:$id}) {
      thread { isResolved }
    }
  }' -F id="$1"
