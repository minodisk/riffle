#!/usr/bin/env bash
# Stash the uncommitted changes under a unique tag and record that identifier in
# a marker file under `.git`. Used as a pair with `restore-stash.sh`.
#
# Composing it agent-side as `pr_stash_tag="pr-skill-prepare-$(date +%s)-..."`
# makes a compound command with command substitution, which triggers a
# permission prompt every time — so the whole procedure is shut inside this
# script as one atomic call.
#
# Whether a stash was created is judged by the change in stash count before and
# after (`git stash push` exits successfully even with no changes, so the exit
# code cannot tell you). If none was created, no marker is left, meaning
# `restore-stash.sh` restores nothing.
#
# usage: stash-work.sh
# exit code: 0=ok (see stdout for whether anything was stashed), 1=error
set -euo pipefail

git_dir=$(git rev-parse --git-dir)
marker="${git_dir}/PR_SKILL_STASH_TAG"

# Always clear a marker left by a previous run before judging
rm -f "$marker"

tag="pr-skill-prepare-$(date +%s)-$(git rev-parse --short HEAD 2>/dev/null || echo no-head)"

before=$(git stash list | wc -l | tr -d ' ')
git stash push --include-untracked -m "$tag" >/dev/null
after=$(git stash list | wc -l | tr -d ' ')

if ((after > before)); then
	printf '%s\n' "$tag" >"$marker"
	echo "STASHED=true"
	echo "TAG=${tag}"
else
	echo "STASHED=false"
fi
