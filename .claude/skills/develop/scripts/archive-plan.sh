#!/usr/bin/env bash
set -euo pipefail

# Usage: archive-plan.sh <plan-dir>
# Move a plan folder into docs/plans/_archived/ and commit & push.
# <plan-dir> is relative to the repository root (e.g. docs/plans/20260501-foo).
#
# Sandbox: this script assumes it is always called with the Bash tool's
# `dangerouslyDisableSandbox: true`. Setting an upstream writes to the shared
# `.git/config`, which under the sandbox always fails with
# `could not lock config file ...: Operation not permitted` (measured).

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=_lib.sh
source "$script_dir/_lib.sh"

if [ "$#" -ne 1 ]; then
	echo "Usage: $0 <plan-dir>" >&2
	exit 1
fi

src="$1"
if [ ! -d "$src" ]; then
	echo "Error: plan dir not found: $src" >&2
	exit 1
fi

base="$(basename "$src")"
dst="docs/plans/_archived/$base"

# Stop if the destination already exists (this prevents `git mv` from nesting it
# inside the existing directory and producing an unexpected structure, e.g. when
# the same feature name is reused).
if [ -e "$dst" ]; then
	echo "Error: archive destination already exists: $dst" >&2
	echo "Persist it under a different feature name, or tidy the existing archive by hand, and re-run." >&2
	exit 1
fi

mkdir -p docs/plans/_archived
wait_for_index_lock
# git mv stages both the deletion of src and the addition of dst, so no extra
# git add is needed (`git add "$src"` would be a pathspec error because src is
# already gone from the worktree).
git mv "$src" "$dst"

git commit -m "chore: archive completed plan $base"

branch="$(git rev-parse --abbrev-ref HEAD)"
if git rev-parse --abbrev-ref --symbolic-full-name "@{upstream}" >/dev/null 2>&1; then
	git push
else
	git push -u origin "$branch"
	# push -u returns exit 0 even when the config write fails, so verify and repair afterwards.
	ensure_upstream "$branch"
fi
