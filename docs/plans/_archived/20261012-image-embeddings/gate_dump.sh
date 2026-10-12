#!/bin/sh
# Usage: gate_dump.sh <riffle-cli> <siglip2 vision_model.onnx> <out-dir> [threads]
# Writes, for the 27 sidecar-labeled ARW folders of burst-keep-score's data.md:
# <out-dir>/features/<folder>.tsv (+ .log, + .output.txt, the stems of Output/)
# and <out-dir>/embed.tsv (SigLIP 2 vectors of the preview decoded near 404 px).
B="$1"; M="$2"; OUT="$3"; T="${4:-24}"
ROOT=/d/photos/2026
FOLDERS="2026-06-05 2026-06-13 2026-06-14 2026-06-20 2026-07-02 2026-07-05 2026-07-11 2026-07-18 2026-07-21 2026-07-22 2026-07-23 2026-07-24 2026-07-26 2026-07-30 2026-07-31 2026-08-01 2026-08-02 2026-08-08 2026-08-15 2026-08-16 2026-08-22 2026-08-29 2026-09-13-b 2026-09-19 2026-09-27-a 2026-09-27-b 2026-10-03"
mkdir -p "$OUT/features"
for d in $FOLDERS; do
  "$B" features "$ROOT/$d" "$T" > "$OUT/features/$d.tsv" 2> "$OUT/features/$d.log"
  if [ -d "$ROOT/$d/Output" ]; then ls "$ROOT/$d/Output" | sed 's/\.[^.]*$//' | sort > "$OUT/features/$d.output.txt"; fi
  echo "$d $(cat "$OUT/features/$d.log")"
done
dirs=""
for d in $FOLDERS; do dirs="$dirs $ROOT/$d"; done
# shellcheck disable=SC2086
"$B" embed "$M" siglip2 $dirs "$T" --near 404 --out "$OUT/embed.tsv" > "$OUT/embed.log" 2>&1
tail -5 "$OUT/embed.log"
