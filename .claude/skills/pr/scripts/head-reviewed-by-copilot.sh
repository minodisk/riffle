#!/usr/bin/env bash
# Usage: head-reviewed-by-copilot.sh <PR-number>
# A helper for deciding how to nudge Copilot about the current head. It prints
# the two values below to stdout as KEY=VALUE. The caller (pr-runner's
# REVIEW_REQUIRED recovery path) greps out the values and branches on them.
#
#   headReviewedCount=N       How many of the commits Copilot reviewed could be
#                             judged equivalent to head (not a review count).
#                               0        → head is unreviewed
#                               1 or more → something equivalent to head was reviewed
#                             "Equivalent to head" covers commit_id == head sha
#                             plus any commit whose patch-id of the diff against
#                             base matches head's (right after a main merge or a
#                             rebase, where only the SHA changed).
#   copilotPending=(true|false)
#                             Whether Copilot is currently listed as a pending
#                             review request. Stuck pending,
#                             `requestReviews(union:true)` is a no-op and an
#                             ordinary re-request does nothing; that case needs
#                             `rerequest-review.sh --force --reset`.
#                             It is read via GraphQL's
#                             `reviewRequests(first:100)` (`gh pr view --json
#                             reviewRequests` does not return Bots, so it cannot
#                             be used).
#
#
# This is a supporting heuristic for deciding whether and how to re-request. It
# is not the approver's own decision logic (the approver issues approve/changes
# against the current head regardless of whether head was reviewed).
set -euo pipefail

if [ $# -lt 1 ]; then
	echo "Usage: $0 <PR-number>" >&2
	exit 1
fi

pr="$1"
repo_full=$(gh repo view --json nameWithOwner -q .nameWithOwner)
owner="${repo_full%%/*}"
name="${repo_full##*/}"

pr_info=$(gh pr view "$pr" --json headRefOid,baseRefName)
head=$(jq -r .headRefOid <<<"$pr_info")
base=$(jq -r .baseRefName <<<"$pr_info")
if [ -z "$head" ] || [ "$head" = "null" ]; then
	echo "cannot get the head commit of PR #$pr" >&2
	exit 1
fi

# Accept the same three Copilot login spellings the workflow does.
# --paginate --jq applies jq per page, so emit line by line and dedupe commit_id
# across all pages with sort -u.
review_shas=$(gh api "repos/$repo_full/pulls/$pr/reviews" --paginate \
	--jq '.[] | select(.user?.login == "Copilot" or .user?.login == "copilot-pull-request-reviewer" or .user?.login == "copilot-pull-request-reviewer[bot]") | .commit_id' |
	sort -u)

# Print the patch-id of the diff against base. Returns failure when the commit
# cannot be fetched and so on (the caller treats "cannot compare" as a
# mismatch).
function patch_id_of() {
	local sha="$1"
	if ! git cat-file -e "${sha}^{commit}" 2>/dev/null; then
		# A post-rebase old head can be dangling, so try fetching the SHA directly
		git fetch -q --no-tags origin "$sha" 2>/dev/null || return 1
	fi
	# The output is "<patch-id> <commit-id>"; an empty string for an empty diff.
	git diff "origin/${base}...${sha}" | git patch-id --stable | cut -d' ' -f1
}

matched=0
if [ -n "$review_shas" ]; then
	git fetch -q --no-tags origin "$base" "$head"

	has_other_sha=false
	while IFS= read -r sha; do
		if [ -z "$sha" ]; then
			continue
		fi
		if [ "$sha" = "$head" ]; then
			matched=$((matched + 1))
		else
			has_other_sha=true
		fi
	done <<<"$review_shas"

	if [ "$matched" -eq 0 ] && [ "$has_other_sha" = "true" ]; then
		if head_pid=$(patch_id_of "$head"); then
			while IFS= read -r sha; do
				if [ -z "$sha" ] || [ "$sha" = "$head" ]; then
					continue
				fi
				if ! pid=$(patch_id_of "$sha"); then
					continue
				fi
				if [ "$pid" = "$head_pid" ]; then
					matched=$((matched + 1))
				fi
			done <<<"$review_shas"
		fi
	fi
fi

# The Copilot bot's global node id (the same across all of GitHub). The login
# string is spelled differently depending on the API surface, so match on the
# id. The same constant as in rerequest-review.sh.
COPILOT_BOT_ID="BOT_kgDOCnlnWA"

# Determine whether Copilot is listed as a pending review request.
# `gh pr view --json reviewRequests` does not return Bots, so use GraphQL
# (first:100 is the same limit as rerequest-review.sh --reset's detection query).
# shellcheck disable=SC2016
pending_ids=$(gh api graphql -f query='query($o:String!,$n:String!,$pr:Int!){repository(owner:$o,name:$n){pullRequest(number:$pr){reviewRequests(first:100){nodes{requestedReviewer{... on Bot{id} ... on User{id}}}}}}}' \
	-F o="$owner" -F n="$name" -F pr="$pr" \
	--jq '.data.repository.pullRequest.reviewRequests.nodes[].requestedReviewer.id')

copilot_pending=false
if printf '%s\n' "$pending_ids" | grep -qxF "$COPILOT_BOT_ID"; then
	copilot_pending=true
fi

echo "headReviewedCount=$matched"
echo "copilotPending=$copilot_pending"
