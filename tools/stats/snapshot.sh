#!/usr/bin/env bash
# snapshot.sh <dir> — record today's (UTC) download count of every release
# asset in <dir>/downloads.csv. Counts are the running totals the API returns.
# Today's rows are replaced, so a re-run on the same day does not duplicate
# them; older rows are left as they are.

set -euo pipefail

if [ $# -ne 1 ]; then
	echo "usage: $0 <dir>" >&2
	exit 2
fi

dir="$1"
repo="${GH_REPO:-minodisk/riffle}"
today="$(date -u +%F)"
file="$dir/downloads.csv"

# Fetch before touching the file, so a failed call leaves it intact.
# shellcheck disable=SC2016 # $t is a jq variable, not a shell one.
rows="$(gh api --paginate "repos/$repo/releases" \
	--jq '.[] | .tag_name as $t | .assets[] | [$t, .name, .download_count] | @csv')"

mkdir -p "$dir"
tmp="$(mktemp)"
{
	echo "date,tag,asset,download_count"
	if [ -f "$file" ]; then
		tail -n +2 "$file" | grep -v "^$today," || true
	fi
	if [ -n "$rows" ]; then
		printf '%s\n' "$rows" | sed "s/^/$today,/" | LC_ALL=C sort
	fi
} >"$tmp"
mv "$tmp" "$file"
