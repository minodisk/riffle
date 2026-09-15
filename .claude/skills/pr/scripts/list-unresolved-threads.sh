#!/usr/bin/env bash
# Usage: list-unresolved-threads.sh <pr-number>
# Print the unresolved review threads as a JSON array.
# Each element: { id, path, line, author, body }
# reviewThreads is paginated so every thread is fetched (>100 supported).
set -euo pipefail

if [ $# -lt 1 ]; then
	echo "Usage: $0 <pr-number>" >&2
	exit 1
fi

pr=$1
owner=$(gh repo view --json owner -q .owner.login)
repo=$(gh repo view --json name -q .name)

cursor=""
all="[]"
# shellcheck disable=SC2016
query='
    query($owner:String!,$repo:String!,$number:Int!,$cursor:String) {
      repository(owner:$owner, name:$repo) {
        pullRequest(number:$number) {
          reviewThreads(first:100, after:$cursor) {
            pageInfo { hasNextPage endCursor }
            nodes {
              id
              isResolved
              comments(first:5) {
                nodes { author { login } body path line }
              }
            }
          }
        }
      }
    }'
while :; do
	args=(-f query="$query" -F owner="$owner" -F repo="$repo" -F number="$pr")
	if [ -n "$cursor" ]; then
		args+=(-F cursor="$cursor")
	fi
	page=$(gh api graphql "${args[@]}")

	nodes=$(echo "$page" | jq '.data.repository.pullRequest.reviewThreads.nodes')
	all=$(jq -n --argjson a "$all" --argjson b "$nodes" '$a + $b')

	has_next=$(echo "$page" | jq -r '.data.repository.pullRequest.reviewThreads.pageInfo.hasNextPage')
	if [ "$has_next" != "true" ]; then
		break
	fi
	cursor=$(echo "$page" | jq -r '.data.repository.pullRequest.reviewThreads.pageInfo.endCursor')
done

echo "$all" | jq '[.[]
     | select(.isResolved == false)
     | {
         id,
         path: .comments.nodes[0].path,
         line: .comments.nodes[0].line,
         author: .comments.nodes[0].author.login,
         body: .comments.nodes[0].body
       }]'
