#!/usr/bin/env bash
# Create a GitHub release for each tagged, unreleased crate, attaching $1/<crate>/*.
# Every crate is checked before any release is created.
set -euo pipefail
# shellcheck source=lib.sh
source "$(dirname "$0")/lib.sh"

assets=$1
root=$PWD

todo=()
failed=false
while read -r name version dir; do
  tag="$name-v$version"
  [ -n "$(tag_commit "$tag")" ] || continue
  state=$(release_state "$tag")
  [ "$state" = published ] && continue

  if [ "$dir" = "$root" ]; then
    while read -r file; do
      [ -e "$assets/$name/$file" ] || { echo "$tag is missing $file" >&2; failed=true; }
    done < <(jq -r '.[].archive' .github/release-targets.json | sed 'p; s/$/.sha256/')
  fi
  todo+=("$name $version $dir $state")
done < <(crates)
[ "$failed" = false ] || exit 1

for entry in "${todo[@]}"; do
  read -r name version dir state <<<"$entry"
  tag="$name-v$version"
  notes "$version" "$dir" >notes.md

  files=()
  if [ -d "$assets/$name" ]; then
    (cd "$assets/$name" && cat ./*.sha256 >sha256.sum)
    files=("$assets/$name"/*)
  fi

  latest=--latest=false
  prerelease=()
  if [[ "$version" == *-* ]]; then
    prerelease=(--prerelease)
  elif [ "$dir" = "$root" ]; then
    latest=--latest
  fi

  if [ "$state" = draft ]; then gh release delete "$tag" --yes; fi
  gh release create "$tag" --verify-tag --title "$name $version" --notes-file notes.md \
    "$latest" "${prerelease[@]}" "${files[@]}"
  echo "released $tag"
done
