#!/usr/bin/env bash
# Usage: reply-thread.sh <thread-id> <body>
# Reply to a review thread.
set -euo pipefail

if [ $# -ne 2 ]; then
	echo "Usage: $0 <thread-id> <body>" >&2
	echo "Note: quote <body> (otherwise spaces and newlines are lost)" >&2
	exit 1
fi

# shellcheck disable=SC2016
gh api graphql -f query='
  mutation($id:ID!, $body:String!) {
    addPullRequestReviewThreadReply(input:{pullRequestReviewThreadId:$id, body:$body}) {
      comment { id }
    }
  }' -F id="$1" -f body="$2"
