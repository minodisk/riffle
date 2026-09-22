#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset

# Decide whether the local review on the current branch has already APPROVED the
# latest HEAD. It cross-checks the newest review-*.md's STATUS and "Reviewed
# commit" against HEAD and reports whether another review is needed on the final
# line, "need_review=<true|false>".
#
# need_review=false when STATUS is APPROVED and either:
#   - the reviewed commit == HEAD
#   - the reviewed commit is an ancestor of HEAD and the diff from it is
#     confined to docs/plans/ (review history files, the Progress line in
#     plan.md — commits the develop skill always stacks after an APPROVED.
#     Treating those as needing review would loop review → commit → review
#     forever)
#
# It looks inside the newest file so it cannot miss a lingering NEEDS_FIX /
# MAX_ROUNDS_REACHED, or a reviewed commit gone stale behind extra commits
# outside docs/plans/. When the reviewed commit is not an ancestor of HEAD
# (rebase / amend) it errs on the safe side with need_review=true.

branch=$(git rev-parse --abbrev-ref HEAD)
review_dir="docs/plans/review-history/${branch}"
head_sha=$(git rev-parse HEAD)

need_review=true
if [[ -d "${review_dir}" ]]; then
	# Take the newest by string-sorting the filename (review-YYYYMMDD-HHmm.md).
	# With ls -t (mtime), merely editing an old file makes it "newest" and the
	# judgment becomes unstable.
	# shellcheck disable=SC2012 # sorting by filename is the point; find would need an extra sort spec
	latest=$(ls -1 "${review_dir}"/review-*.md 2>/dev/null | sort | tail -n1 || true)
	if [[ -n "${latest}" ]]; then
		# Check the final STATUS and the reviewed commit
		status=$(grep -E '^STATUS:' "${latest}" | tail -n1 | awk '{print $2}' || true)
		# shellcheck disable=SC2016 # the backtick is a literal in the sed regex, not command substitution
		target_sha_raw=$(grep -E '^\*\*Reviewed commit\*\*:' "${latest}" | tail -n1 | sed -E 's/.*`([0-9a-f]+)`.*/\1/' || true)
		# Normalize so a short SHA and a full SHA compare the same
		target_sha=$(git rev-parse --verify "${target_sha_raw}^{commit}" 2>/dev/null || echo "")
		if [[ "${status}" == "APPROVED" ]] && [[ -n "${target_sha}" ]]; then
			if [[ "${target_sha}" == "${head_sha}" ]]; then
				need_review=false
			elif git merge-base --is-ancestor "${target_sha}" "${head_sha}"; then
				# Without --no-renames a rename is listed by its new path only,
				# which would exempt a move in from outside docs/plans/
				non_docs=$(git diff --name-only --no-renames "${target_sha}" "${head_sha}" | grep -v '^docs/plans/' || true)
				if [[ -z "${non_docs}" ]]; then
					need_review=false
				fi
			fi
		fi
	fi
fi

if [[ "${need_review}" == "true" ]]; then
	# Guidance goes to stderr, so it does not break callers that mechanically
	# parse the final need_review= line on stdout (tail -n1 / $(...)).
	echo "Local review missing / outdated for ${branch}; running local-review-runner" >&2
fi
echo "need_review=${need_review}"
