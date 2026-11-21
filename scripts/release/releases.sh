#!/usr/bin/env bash
# GitHub release for every publishable crate whose version is tagged on origin
# and not released yet, attaching the files in $1/<crate>/. The root crate (the
# cli) is the latest release and needs an archive and .sha256 for every target.
# Everything is checked before any release is created.
set -euo pipefail
# shellcheck source=lib.sh
source "$(dirname "$0")/lib.sh"

assets=$1
here=$(dirname "$0")
root=$PWD

# the files crate $1 in $2 must have in its assets
required_assets() {
  [ "$2" = "$root" ] || return 0
  jq -r '.[].archive' .github/release-targets.json | sed 'p; s/$/.sha256/'
}

# the changelog section of version $1 in $2, then the install footer if it has one
notes() {
  "$here/notes.sh" "$2/CHANGELOG.md" "$1"
  if [ -f "$2/release-footer.md" ]; then
    echo
    version=$1 envsubst "\$version" <"$2/release-footer.md"
  fi
}

# create the release of crate $1 at version $2 in $3, replacing a draft when $4 is draft
release() {
  local name=$1 version=$2 dir=$3 state=$4 tag="$1-v$2"
  notes "$version" "$dir" >notes.md

  local files=()
  if [ -d "$assets/$name" ]; then
    (cd "$assets/$name" && cat ./*.sha256 >sha256.sum)
    files=("$assets/$name"/*)
  fi

  local prerelease=() latest=--latest=false
  [[ "$version" == *-* ]] && prerelease=(--prerelease)
  [ "$dir" = "$root" ] && [ ${#prerelease[@]} -eq 0 ] && latest=--latest

  if [ "$state" = draft ]; then gh release delete "$tag" --yes; fi
  gh release create "$tag" --verify-tag --title "$name $version" --notes-file notes.md \
    "$latest" "${prerelease[@]}" "${files[@]}"
  echo "released $tag"
}

todo=()
failed=false
while read -r name version dir; do
  tag="$name-v$version"
  [ -n "$(tag_commit "$tag")" ] || continue
  state=$(release_state "$tag")
  [ "$state" = published ] && continue

  while read -r file; do
    [ -e "$assets/$name/$file" ] || { echo "$tag is missing $file" >&2; failed=true; }
  done < <(required_assets "$name" "$dir")
  todo+=("$name $version $dir $state")
done < <(crates)
[ "$failed" = false ] || exit 1

for entry in "${todo[@]}"; do
  read -r name version dir state <<<"$entry"
  release "$name" "$version" "$dir" "$state"
done
