#!/bin/sh
# Usage: dump.sh <riffle-cli> <threads>
B="$1"; T="$2"
ROOT=/d/photos/2026
OUT=/d/Photos/tests/2026-10-08-burst-keep-score/dump
mkdir -p "$OUT"
for d in 2026-06-05 2026-06-13 2026-06-14 2026-06-20 2026-07-02 2026-07-05 2026-07-11 2026-07-18 2026-07-21 2026-07-22 2026-07-23 2026-07-24 2026-07-26 2026-07-30 2026-07-31 2026-08-01 2026-08-02 2026-08-08 2026-08-15 2026-08-16 2026-08-22 2026-08-29 2026-09-13-b 2026-09-19 2026-09-27-a 2026-09-27-b 2026-10-03 2026-08-29-l 2026-09-05 2026-01-08 2026-02-14 2026-03-29 2026-04-12 2026-04-18 2026-05-22 2026-05-26; do
  s=$SECONDS
  "$B" features "$ROOT/$d" "$T" > "$OUT/$d.tsv" 2> "$OUT/$d.log"
  e=$SECONDS
  if [ -d "$ROOT/$d/Output" ]; then ls "$ROOT/$d/Output" | sed 's/\.[^.]*$//' | sort > "$OUT/$d.output.txt"; fi
  echo "$d $(cat "$OUT/$d.log") wall $((e - s))s"
done
