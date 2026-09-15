#!/usr/bin/env bash
set -euo pipefail

# Update origin/main, confirm the branch exists neither locally nor on origin,
# and cut a new branch from origin/main.
# Local tracking refs (refs/remotes/origin/...) miss unfetched remote branches,
# so origin is queried directly.
#
# The main sync is built in, so there is no need to sync to main beforehand.
# Calling it when already synced is harmless: fetch is idempotent, and
# `git checkout --no-track -b <branch> origin/main` gives the same result as
# branching from a HEAD already at origin/main.
#
# Sandbox: when there is a diff under `.claude/` between the checkout target
# origin/main and the current HEAD, `git checkout` tries to update `.claude/`
# (`settings.json`, the skills) and always fails with
# `unable to unlink old '.claude/...': Operation not permitted` under the
# harness sandbox's `denyWithinAllow` (measured).
# The nasty part is that `git checkout` still returns exit 0 when that unlink
# error happens. HEAD/index advance to the new branch (= origin/main) while only
# `.claude/` in the working tree keeps the previous branch's content — a
# half-done state the caller cannot detect from the exit code alone. So below,
# the state of `.claude/` before and after the checkout is compared, and it
# exits with an error if a diff the checkout should have resolved is still
# there. (A diff that predates the checkout is merely carried over and has
# nothing to do with the sandbox, so looking only at the post-checkout state
# without the before/after comparison would false-positive and discard those
# uncommitted changes via the `-f` in the recovery procedure below.)
# This check is a safety net so the failure is noticeable — not a reason to skip
# disabling the sandbox. It is not a fallback: always make this call with the
# Bash tool's `dangerouslyDisableSandbox: true`.
#
# Recovering when that check exits with an error: with
# `dangerouslyDisableSandbox: true`, run `git checkout -f <original branch>`,
# delete the leftover ref with `git branch -D <branch>`, and re-run this script
# with `dangerouslyDisableSandbox: true`. (Do not use `git reset --hard`; it
# only rewinds to origin/main and does not return you to the original branch.)
#
# When <feature-name> is given, validate up front that it works as a plan
# directory name (the same kebab-case regex as ensure-plan-dir.sh). Writing
# `if [[ ! ... =~ ... ]]` caller-side makes a compound command that prompts
# every time, so the validation is shut in here too.
#
# Usage: ensure-new-branch.sh <branch-name> [<feature-name>]

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=_lib.sh
source "$script_dir/_lib.sh"

if [[ $# -lt 1 ]] || [[ $# -gt 2 ]] || [[ -z "${1:-}" ]]; then
	echo "Usage: $0 <branch-name> [<feature-name>]" >&2
	exit 2
fi

branch="$1"
feature="${2:-}"

if [[ -n "$feature" ]] && ! [[ "$feature" =~ ^[a-z0-9]+(-[a-z0-9]+)*$ ]]; then
	echo "Error: feature name must be kebab-case (^[a-z0-9]+(-[a-z0-9]+)*\$): ${feature}" >&2
	exit 1
fi

git fetch origin main

if git rev-parse --verify --quiet "refs/heads/${branch}" >/dev/null; then
	echo "Error: branch ${branch} already exists locally" >&2
	exit 1
fi

# git ls-remote --exit-code: 0=found, 2=not found, anything else=error
set +e
git ls-remote --exit-code --heads origin "${branch}" >/dev/null 2>&1
ls_remote_status=$?
set -e
case "${ls_remote_status}" in
0)
	echo "Error: branch ${branch} already exists on origin" >&2
	exit 1
	;;
2)
	# Absent — as expected, continue
	;;
*)
	echo "Error: failed to query origin for branch ${branch} (git ls-remote exit ${ls_remote_status})" >&2
	exit 1
	;;
esac

previous_ref="$(git symbolic-ref --quiet --short HEAD || git rev-parse HEAD)"

# Record the state of `.claude/` before the checkout. A diff that predates it
# (uncommitted changes, untracked files) is carried over afterwards, so the
# check below judges only on the before/after difference. (Without that
# comparison, a `.claude/` that was merely dirty before the call would
# false-positive and the `-f` in the recovery procedure would discard those
# uncommitted changes.)
claude_status_before="$(git status --porcelain -- .claude)"

# --no-track: branching from origin/main sets the upstream to origin/main by
# default, which sends the "push if there is an upstream, otherwise
# `git push -u origin <branch>`" branch in commit-push.sh / archive-plan.sh /
# commit-review-history.sh down the former path. push.default=simple refuses a
# push when the upstream name differs from the branch name, so all three would
# fail at their final push. Creating it with no upstream keeps them on the `-u`
# path as before.
wait_for_index_lock

git checkout --no-track -b "${branch}" origin/main

# `git checkout` returns exit 0 even when the sandbox blocks the unlink (see the
# header comment), so check the `.claude/` diff explicitly here. The before/after
# comparison is what keeps a pre-existing diff (unrelated to the sandbox) from
# false-positiving.
if [[ "$(git status --porcelain -- .claude)" != "${claude_status_before}" ]]; then
	echo "Error: git checkout succeeded (exit 0) but the status of '.claude/' changed across the checkout." >&2
	echo "The sandbox likely blocked the unlink: HEAD/index moved to '${branch}' (= origin/main)," >&2
	echo "but the '.claude/' working tree still holds '${previous_ref}' content." >&2
	echo "Recovery (requires dangerouslyDisableSandbox: true):" >&2
	echo "  git checkout -f ${previous_ref} && git branch -D ${branch}" >&2
	echo "Then re-run this script with dangerouslyDisableSandbox: true." >&2
	exit 1
fi
