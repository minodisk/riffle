#!/usr/bin/env bash
# Fetch the reviews and comments on a PR.
# Emits reviews / comments / inline (review thread) comments as one JSON document.
set -euo pipefail

PR_INPUT="${1:?usage: pr-review-comments.sh <PR_number_or_url>}"

# Resolve a number even when given a URL (gh api does not accept URLs)
PR_NUMBER=$(gh pr view "$PR_INPUT" --json number --jq '.number')
# A PR number belongs to the base repository, so use the base repo's owner/name (the current gh context)
repo=$(gh repo view --json owner,name --jq '"\(.owner.login)/\(.name)"')

pr_data=$(gh pr view "$PR_NUMBER" --json reviews,comments,headRefOid,author)
# --paginate runs jq per page, so emit JSONL objects with `.[]` and gather them into one array with -s
inline_comments=$(gh api --paginate "repos/$repo/pulls/$PR_NUMBER/comments" \
	--jq '.[] | {author: .user.login, path: .path, line: (.line // .original_line), body: .body, createdAt: .created_at}' |
	jq -s '.')

jq -n \
	--argjson pr "$pr_data" \
	--argjson inlineComments "$inline_comments" '
  {
    changesRequestedCount: ($pr.reviews | map(select(.state == "CHANGES_REQUESTED")) | length),
    reviews: [$pr.reviews[] | {author: .author.login, state: .state, body: .body, submittedAt: .submittedAt}],
    comments: [$pr.comments[] | {author: .author.login, body: .body, createdAt: .createdAt}],
    inlineComments: $inlineComments
  }
'
