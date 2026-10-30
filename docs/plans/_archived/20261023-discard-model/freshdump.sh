#!/bin/sh
# Step 4: the label-free dump of the six fresh folders, one folder at a time
# and resumable (an output that exists is skipped; each is written to .part
# and moved into place on exit 0). The library is only read: a size and mtime
# listing of the fresh folders is taken before the first run and after each.
# Usage: freshdump.sh <riffle-cli.exe> <threads>
cli=$1
threads=$2
folders="2026-10-11 2026-10-12a 2026-10-12b 2026-10-17 2026-09-13-a 2026-09-27-c"
lib=/d/Photos/2026
out=/d/Photos/tests/2026-10-23-discard-model/dump
model=/d/Photos/tests/2026-10-12-embed-models/siglip2-b32-256/vision_model.onnx
mkdir -p "$out/features" "$out/faces" "$out/embed" "$out/logs"
log="$out/logs/wall.log"

listing() {
  # shellcheck disable=SC2086
  (cd "$lib" && find $folders -type f -printf '%p\t%s\t%T@\n' | LC_ALL=C sort)
}

[ -e "$out/logs/listing.before.tsv" ] || listing > "$out/logs/listing.before.tsv"
[ -e "$out/genre_text.tsv" ] || cp /d/Photos/tests/2026-10-12-rank-app/genre_text.tsv "$out/genre_text.tsv"

# stage <name> <target> <stdout> <command...>: unless <target> exists, runs
# the command (which writes <target>.part, or stdout <target>.part when
# <stdout> is "-") with stderr in logs/<folder>.<name>.err, and moves
# <target>.part into place on exit 0.
stage() {
  name=$1
  target=$2
  stdout=$3
  shift 3
  if [ -e "$target" ]; then
    echo "$f $name skip" | tee -a "$log"
    return 0
  fi
  [ "$stdout" = "-" ] && stdout="$target.part"
  start=$(date +%s)
  "$@" > "$stdout" 2> "$out/logs/$f.$name.err"
  code=$?
  end=$(date +%s)
  [ $code -eq 0 ] && mv "$target.part" "$target"
  echo "$f $name exit $code wall $((end - start)) s" | tee -a "$log"
  return $code
}

for f in $folders; do
  dir="D:/Photos/2026/$f"
  t="$out/features/$f.tsv"
  stage features "$t" - "$cli" features "$dir" "$threads" || continue
  if [ -d "$lib/$f/Output" ] && [ ! -e "$out/features/$f.output.txt" ]; then
    ls "$lib/$f/Output" | sed 's/\.[^.]*$//' | sort > "$out/features/$f.output.txt"
  fi
  t="$out/embed/$f.tsv"
  stage embed "$t" "$out/logs/$f.embed.out" "$cli" embed "$model" siglip2 "$dir" "$threads" --near 404 --out "$t.part" || continue
  t="$out/faces/$f.tsv"
  stage faces "$t" "$out/logs/$f.faces.out" "$cli" facechoice "$dir" "$threads" --check --out "$t.part" || continue
done

cat "$out"/embed/*.tsv > "$out/embed.tsv"
listing > "$out/logs/listing.after.tsv"
if cmp -s "$out/logs/listing.before.tsv" "$out/logs/listing.after.tsv"; then
  echo "library unchanged" | tee -a "$log"
else
  echo "library CHANGED" | tee -a "$log"
fi
echo "freshdump done" | tee -a "$log"
