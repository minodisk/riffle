#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset

# Search for and report traces of another session working on the given feature in parallel.
#
# ensure-new-branch.sh's collision check only looks for an *exact* branch name
# match, so it cannot detect a parallel session working on the same feature
# under a different name (e.g. feature/foo-bar vs foo-bar-step-1). This has
# actually happened: a branch pushed by another session was used as a starting
# point, the same step was implemented twice, and one of the PRs was thrown away
# in a merge conflict. The other worktree was visible in `git worktree list` at
# the time, so the substring-based trace search is broken out here.
#
# What it searches:
#   1. Branches checked out in other worktrees (the current worktree excluded)
#   2. Branches on origin (the currently checked-out branch excluded)
#   3. Open PRs (via gh; the PR for the currently checked-out branch excluded)
# All three are matched on a substring of feature-name. Finding even one exits
# non-zero, so the caller always stops and checks with the user.
#
# You may pass the plan directory name (`YYYYMMDD-{feature-name}`) as
# feature-name directly. A leading `YYYYMMDD-` is stripped automatically before
# comparing (comparing it as-is would make every substring match against branch
# names fail because of the date, yielding zero traces and exit 0).

function usage() {
	cat <<EOF
Usage: $0 <feature-name>

Search for and report traces of parallel work whose name contains the given
feature-name (branches in other worktrees / branches on origin / open PRs).

Arguments:
  feature-name    Feature name in kebab-case (the same as the plan directory
                   name). A leading YYYYMMDD- date prefix is stripped
                   automatically before comparing, so you may pass the plan
                   directory name directly.

Exit status:
  0    No traces
  1    Traces found (listed by kind)
  2    Bad arguments
  3    The query to git failed
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

# Assumes the kebab-case used for a plan directory name (the same regex as ensure-new-branch.sh).
if ! [[ "$feature" =~ ^[a-z0-9]+(-[a-z0-9]+)*$ ]]; then
	echo "Error: feature name must be kebab-case (^[a-z0-9]+(-[a-z0-9]+)*\$): ${feature}" >&2
	exit 2
fi

# In case a plan directory name (YYYYMMDD-{feature-name}) was passed directly,
# strip the leading date prefix before comparing.
match_feature="$feature"
if [[ "$feature" =~ ^[0-9]{8}-(.+)$ ]]; then
	match_feature="${BASH_REMATCH[1]}"
fi

current_branch="$(git rev-parse --abbrev-ref HEAD)"

# 1. Branches in other worktrees
# --porcelain prints worktree / HEAD / branch (or detached) lines separated by
# blank lines. A detached-HEAD worktree has no branch line, so it is skipped
# naturally.
current_worktree="$(git rev-parse --show-toplevel)"
worktree_hits=()
worktree_path=""
while IFS= read -r line; do
	case "$line" in
	"worktree "*)
		worktree_path="${line#worktree }"
		;;
	"branch refs/heads/"*)
		worktree_branch="${line#branch refs/heads/}"
		if [[ "$worktree_path" != "$current_worktree" ]] && [[ "$worktree_branch" == *"$match_feature"* ]]; then
			worktree_hits+=("${worktree_branch} (${worktree_path})")
		fi
		;;
	esac
done < <(git worktree list --porcelain)

# 2. Branches on origin
# Local tracking refs (refs/remotes/origin/...) miss unfetched branches, so query
# origin directly. Fetch them all and narrow by substring rather than passing a
# pattern.
set +e
origin_refs="$(git ls-remote --heads origin 2>&1)"
ls_remote_status=$?
set -e
if [[ "${ls_remote_status}" -ne 0 ]]; then
	echo "Error: failed to query origin (git ls-remote exit ${ls_remote_status})" >&2
	echo "${origin_refs}" >&2
	exit 3
fi

# A valid git ls-remote line is only ever "<sha>\trefs/heads/<branch>". Ignoring
# any line that does not match keeps a stray credential helper warning on stderr
# from being mistaken for a trace of parallel work.
origin_hits=()
valid_ref_count=0
while IFS=$'\t' read -r sha ref; do
	if [[ ! "$sha" =~ ^[0-9a-f]{40}$ ]] || [[ "$ref" != refs/heads/* ]]; then
		continue
	fi
	valid_ref_count=$((valid_ref_count + 1))
	branch="${ref#refs/heads/}"
	if [[ "$branch" == "$current_branch" ]]; then
		continue
	fi
	if [[ "$branch" == *"$match_feature"* ]]; then
		origin_hits+=("$branch")
	fi
done < <(printf '%s\n' "${origin_refs}")

# If origin_refs has content yet not one line matched the expected format, the
# filter may be broken by an output format change or broken line boundaries. Err
# on the safe side: stop with exit 3 rather than pretending there are no traces.
if [[ -n "${origin_refs}" ]] && [[ "${valid_ref_count}" -eq 0 ]]; then
	echo "Error: git ls-remote output did not match the expected sha/refs-heads format" >&2
	echo "${origin_refs}" >&2
	exit 3
fi

# 3. Open PRs
# A gh failure alone (unauthenticated, network down) should not block starting
# work, so this stays a warning and the git-based judgement continues.
# `gh pr list --search` is GitHub full-text search (token matching on title and
# body) and does not index head branch names. Post-filtering on headRefName
# therefore only produces the conjunction "title/body match AND headRefName
# substring match", dropping parallel PRs whose title and body never mention the
# feature-name. So --search is not used at all: list every open PR and narrow
# mechanically on a headRefName substring alone.
pr_hits=()
pr_raw_count=0
gh_warning=""
set +e
pr_lines="$(gh pr list --state open --limit 100 \
	--json number,title,headRefName \
	--jq '.[] | "\(.number)\t\(.headRefName)\t\(.title)"' 2>&1)"
gh_status=$?
set -e
if [[ "${gh_status}" -ne 0 ]]; then
	gh_warning="skipped the PR check because gh pr list failed (exit ${gh_status}): ${pr_lines}"
else
	# Every line jq emits has exactly the three fields
	# "<number>\t<headRefName>\t<title>". A version notice from gh on stderr
	# does not match the tab-separated format, so headRefName comes out empty
	# and it is never wrongly listed as a PR.
	while IFS=$'\t' read -r number headRefName title; do
		if [[ -z "$headRefName" ]]; then
			continue
		fi
		pr_raw_count=$((pr_raw_count + 1))
		if [[ "$headRefName" == "$current_branch" ]]; then
			continue
		fi
		if [[ "$headRefName" == *"$match_feature"* ]]; then
			pr_hits+=("#${number} [${headRefName}] ${title}")
		fi
	done < <(printf '%s\n' "${pr_lines}")
fi

if [[ -n "${gh_warning}" ]]; then
	echo "Warning: ${gh_warning}" >&2
fi

if [[ "${pr_raw_count}" -ge 100 ]]; then
	echo "Warning: hit gh pr list --limit 100, so some PRs may not be shown" >&2
fi

total=$((${#worktree_hits[@]} + ${#origin_hits[@]} + ${#pr_hits[@]}))

if [[ "${total}" -eq 0 ]]; then
	echo "No parallel work found for '${feature}'."
	exit 0
fi

echo "Parallel work traces found for '${feature}':"

if [[ "${#worktree_hits[@]}" -gt 0 ]]; then
	echo ""
	echo "Branches checked out in other worktrees:"
	for hit in "${worktree_hits[@]}"; do
		echo "  - ${hit}"
	done
fi

if [[ "${#origin_hits[@]}" -gt 0 ]]; then
	echo ""
	echo "Branches on origin:"
	for hit in "${origin_hits[@]}"; do
		echo "  - ${hit}"
	done
fi

if [[ "${#pr_hits[@]}" -gt 0 ]]; then
	echo ""
	echo "Open pull requests:"
	for hit in "${pr_hits[@]}"; do
		echo "  - ${hit}"
	done
fi

exit 1
