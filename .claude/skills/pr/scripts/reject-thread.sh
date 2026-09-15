#!/usr/bin/env bash
# Usage: reject-thread.sh <thread-id> <body>
# Reply to a review thread with a rejection reason, then resolve it.
set -euo pipefail

if [ $# -ne 2 ]; then
	echo "Usage: $0 <thread-id> <body>" >&2
	echo "Note: quote <body> (otherwise spaces and newlines are lost)" >&2
	exit 1
fi

dir="$(cd "$(dirname "$0")" && pwd)"
bash "$dir/reply-thread.sh" "$1" "$2"
bash "$dir/resolve-thread.sh" "$1"
