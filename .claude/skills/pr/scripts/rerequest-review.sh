#!/usr/bin/env bash
# Usage: rerequest-review.sh [--force] [--reset] <PR-number> [<reviewer-login>...]
# Re-request review from the given reviewers.
# With reviewer-login omitted, it targets Copilot.
#
# Copilot is a GitHub Bot account, so REST's requested_reviewers rejects it with
# "not a collaborator". It has to go through GraphQL's requestReviews mutation,
# passing the Copilot bot's global id in botIds.
#
# Reviewers who already reviewed the current head SHA are not re-requested.
# Re-requesting when the head has not changed induces a duplicate review of the
# same commit, which can flip an approval that was already given into
# changes_requested. --force disables this skip and always re-requests.
#
# --reset is only for pr-runner's REVIEW_REQUIRED recovery path (head
# unreviewed + Copilot stuck pending). When a reviewer is already listed as
# pending, requestReviews(union:true) is a no-op and Copilot never moves, so the
# request is cleared and re-registered. The clearing replaces the set with
# "everything except the targets" (union:false), so there is never a moment when
# a human reviewer drops off the list. Do not use it on the routine re-request
# path.
#
# Re-requesting while unresolved Copilot threads remain can make Copilot return
# REQUEST_CHANGES. The caller (pr-runner's REVIEW_REQUIRED recovery path) does
# not check for that state beforehand, so it costs one extra round trip.
set -euo pipefail

force=false
reset=false
while [ $# -gt 0 ]; do
	case "$1" in
	--force)
		force=true
		shift
		;;
	--reset)
		reset=true
		shift
		;;
	*)
		break
		;;
	esac
done

if [ $# -lt 1 ]; then
	echo "Usage: $0 [--force] [--reset] <PR-number> [<reviewer-login>...]" >&2
	exit 1
fi

pr="$1"
shift

if [ $# -eq 0 ]; then
	set -- Copilot
fi

repo_full=$(gh repo view --json nameWithOwner -q .nameWithOwner)
owner="${repo_full%%/*}"
name="${repo_full##*/}"

# Fetch it with command substitution first: via process substitution `< <(...)`
# a gh api failure escapes set -e and execution continues with an empty value.
# shellcheck disable=SC2016
pr_head=$(gh api graphql -f query='query($o:String!,$n:String!,$pr:Int!){repository(owner:$o,name:$n){pullRequest(number:$pr){id headRefOid}}}' \
	-F o="$owner" -F n="$name" -F pr="$pr" \
	--jq '.data.repository.pullRequest | "\(.id) \(.headRefOid)"')
read -r pr_id head_sha <<<"$pr_head"
if [ -z "$pr_id" ] || [ "$pr_id" = "null" ]; then
	echo "PR #$pr not found in $repo_full" >&2
	exit 1
fi
if [ -z "$head_sha" ] || [ "$head_sha" = "null" ]; then
	echo "could not get the head SHA of PR #$pr" >&2
	exit 1
fi

# Logins of reviewers who already reviewed the current head SHA (to prevent duplicate reviews).
reviewed_logins=$(gh api "repos/$owner/$name/pulls/$pr/reviews" --paginate \
	--jq ".[] | select(.commit_id == \"$head_sha\") | .user.login")

# Drop "already reviewed the current head" from the reviewers passed to
# requestReviews. Copilot's review login is copilot-pull-request-reviewer[bot].
reviewers=()
for r in "$@"; do
	case "$r" in
	Copilot | copilot | copilot-pull-request-reviewer | copilot-pull-request-reviewer\[bot\])
		review_login="copilot-pull-request-reviewer[bot]"
		;;
	*)
		review_login="$r"
		;;
	esac
	if [ "$force" = false ] && printf '%s\n' "$reviewed_logins" | grep -qxF "$review_login"; then
		echo "skip: $r already reviewed the current head ($head_sha) (use --force to override)" >&2
		continue
	fi
	reviewers+=("$r")
done

if [ ${#reviewers[@]} -eq 0 ]; then
	echo "no re-request needed: every target has already reviewed the current head" >&2
	exit 0
fi
set -- "${reviewers[@]}"

# The Copilot bot's global node id (the same across all of GitHub)
COPILOT_BOT_ID="BOT_kgDOCnlnWA"

bot_ids=()
user_ids=()
target_ids=()
for r in "$@"; do
	case "$r" in
	Copilot | copilot | copilot-pull-request-reviewer | copilot-pull-request-reviewer\[bot\])
		bot_ids+=("\"$COPILOT_BOT_ID\"")
		target_ids+=("$COPILOT_BOT_ID")
		;;
	*)
		# shellcheck disable=SC2016
		uid=$(gh api graphql -f query='query($l:String!){user(login:$l){id}}' \
			-F l="$r" --jq '.data.user.id')
		if [ -z "$uid" ] || [ "$uid" = "null" ]; then
			echo "reviewer '$r' not found" >&2
			exit 1
		fi
		user_ids+=("\"$uid\"")
		target_ids+=("$uid")
		;;
	esac
done

join_csv() {
	local IFS=,
	echo "$*"
}

bot_csv=""
user_csv=""
if [ ${#bot_ids[@]} -gt 0 ]; then
	bot_csv=$(join_csv "${bot_ids[@]}")
fi
if [ ${#user_ids[@]} -gt 0 ]; then
	user_csv=$(join_csv "${user_ids[@]}")
fi

# --reset: clear the pending request by replacing the set with everything except
# the targets (union:false). Human reviewers remain in the replacement set, so
# there is never a moment when they drop off the list.
if [ "$reset" = true ]; then
	# shellcheck disable=SC2016
	current_requests=$(gh api graphql -f query='query($o:String!,$n:String!,$pr:Int!){repository(owner:$o,name:$n){pullRequest(number:$pr){reviewRequests(first:100){nodes{requestedReviewer{__typename ... on Bot{id login} ... on User{id login} ... on Team{id name}}}}}}}' \
		-F o="$owner" -F n="$name" -F pr="$pr" \
		--jq '.data.repository.pullRequest.reviewRequests.nodes[].requestedReviewer | "\(.__typename) \(.id) \(.login // .name)"')

	keep_bot_ids=()
	keep_user_ids=()
	target_pending=false
	while read -r req_type req_id req_login; do
		if [ -z "$req_type" ]; then
			continue
		fi
		if [ "$req_type" = "Team" ]; then
			echo "--reset: aborted because a Team reviewer ($req_login) is pending. Restoring a Team is unverified, so handle it by hand" >&2
			exit 1
		fi
		if printf '%s\n' "${target_ids[@]}" | grep -qxF "$req_id"; then
			target_pending=true
			continue
		fi
		case "$req_type" in
		Bot)
			keep_bot_ids+=("\"$req_id\"")
			;;
		User)
			keep_user_ids+=("\"$req_id\"")
			;;
		*)
			echo "--reset: aborted on an unknown reviewer type $req_type ($req_login)" >&2
			exit 1
			;;
		esac
	done <<<"$current_requests"

	if [ "$target_pending" = false ]; then
		echo "--reset: the targets are not listed as pending, so skipping the clear" >&2
	else
		keep_bot_csv=""
		keep_user_csv=""
		if [ ${#keep_bot_ids[@]} -gt 0 ]; then
			keep_bot_csv=$(join_csv "${keep_bot_ids[@]}")
		fi
		if [ ${#keep_user_ids[@]} -gt 0 ]; then
			keep_user_csv=$(join_csv "${keep_user_ids[@]}")
		fi
		reset_mutation=$(
			cat <<EOF
mutation {
  requestReviews(input:{
    pullRequestId:"$pr_id",
    botIds:[$keep_bot_csv],
    userIds:[$keep_user_csv],
    union:false
  }){ clientMutationId }
}
EOF
		)
		gh api graphql -f query="$reset_mutation" >/dev/null
		echo "--reset: cleared the pending requests (kept ${#keep_bot_ids[@]} bot / ${#keep_user_ids[@]} user, excluding $*)" >&2
	fi
fi

mutation=$(
	cat <<EOF
mutation {
  requestReviews(input:{
    pullRequestId:"$pr_id",
    botIds:[$bot_csv],
    userIds:[$user_csv],
    union:true
  }){
    pullRequest{
      reviewRequests(first:20){
        nodes{ requestedReviewer{ __typename ... on Bot{login} ... on User{login} } }
      }
    }
  }
}
EOF
)

gh api graphql -f query="$mutation" \
	--jq '.data.requestReviews.pullRequest.reviewRequests.nodes[].requestedReviewer.login'
