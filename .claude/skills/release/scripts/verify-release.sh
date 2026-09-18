#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset

# Check that a published GitHub Release carries everything a user and the
# in-app updater need.
#
# Usage:
#   verify-release.sh <tag>        e.g. verify-release.sh v0.2.0
#
# Checks:
#   - the release exists and is neither a draft nor a prerelease
#   - one installer per platform: the macOS .dmg for aarch64 and x64, the
#     Windows -setup.exe and .msi, the Linux .deb, .rpm and .AppImage
#   - latest.json is attached, its version matches the tag, and the four
#     platforms the updater resolves (darwin-aarch64, darwin-x86_64,
#     windows-x86_64, linux-x86_64) each have a url and a signature
#
# Output: one `ok` / `missing` line per check, then `STATUS=ok|incomplete`.
# Exit: 0 when everything is present, 1 when anything is missing, 2 on bad
# arguments or when the release cannot be read at all.

if [ "$#" -ne 1 ]; then
  echo "usage: verify-release.sh <tag>" >&2
  exit 2
fi
tag="$1"
version="${tag#v}"

if ! release="$(gh release view "$tag" --json isDraft,isPrerelease,assets)"; then
  echo "cannot read release $tag" >&2
  exit 2
fi

missing=0
report() {
  if [ "$1" = ok ]; then
    echo "ok       $2"
  else
    echo "missing  $2"
    missing=1
  fi
}

if [ "$(jq -r '.isDraft or .isPrerelease' <<<"$release")" = false ]; then
  report ok "published (not a draft or prerelease)"
else
  report missing "published (it is a draft or prerelease)"
fi

assets="$(jq -r '.assets[].name' <<<"$release")"
for pattern in \
  "_aarch64\.dmg$" \
  "_x64\.dmg$" \
  "_x64-setup\.exe$" \
  "_x64_en-US\.msi$" \
  "_amd64\.deb$" \
  "\.x86_64\.rpm$" \
  "_amd64\.AppImage$" \
  "^latest\.json$"; do
  if grep -Eq "$pattern" <<<"$assets"; then
    report ok "asset $pattern"
  else
    report missing "asset $pattern"
  fi
done

if manifest="$(gh release download "$tag" --pattern latest.json --output - 2>/dev/null)"; then
  if [ "$(jq -r '.version' <<<"$manifest")" = "$version" ]; then
    report ok "latest.json version $version"
  else
    report missing "latest.json version $version (found $(jq -r '.version' <<<"$manifest"))"
  fi
  for platform in darwin-aarch64 darwin-x86_64 windows-x86_64 linux-x86_64; do
    if jq -e --arg p "$platform" \
      '.platforms[$p] | (.url // "") != "" and (.signature // "") != ""' \
      <<<"$manifest" >/dev/null; then
      report ok "latest.json $platform (url + signature)"
    else
      report missing "latest.json $platform (url + signature)"
    fi
  done
else
  report missing "latest.json readable"
fi

if [ "$missing" -eq 0 ]; then
  echo "STATUS=ok"
else
  echo "STATUS=incomplete"
  exit 1
fi
