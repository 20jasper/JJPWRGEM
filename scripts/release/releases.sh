#!/usr/bin/env bash
# GitHub release for every publishable crate whose version is tagged on origin
# and not released yet, attaching the files in $1/<crate>/. The root crate (the
# cli) is the latest release and needs an archive and .sha256 for every target.
# Everything is checked before any release is created.
set -euo pipefail

assets=$1
here=$(dirname "$0")

root=$PWD

todo=()
failed=false
while read -r name version manifest; do
  tag="$name-v$version"
  git ls-remote --exit-code --tags origin "refs/tags/$tag" >/dev/null || continue
  state=$(gh release view "$tag" --json isDraft -q .isDraft 2>/dev/null || echo missing)
  [ "$state" = false ] && continue
  dir=$(dirname "$manifest")

  if [ "$dir" = "$root" ]; then
    while read -r file; do
      [ -e "$assets/$name/$file" ] || { echo "$tag is missing $file" >&2; failed=true; }
    done < <(jq -r '.[].archive' .github/release-targets.json | sed 'p; s/$/.sha256/')
  fi
  todo+=("$name $version $dir $state")
# name, version and manifest of each publishable crate
done < <(cargo metadata --no-deps --format-version 1 |
  jq -r '.packages[] | select(.publish != []) | "\(.name) \(.version) \(.manifest_path)"')
[ "$failed" = false ] || exit 1

for entry in "${todo[@]}"; do
  read -r name version dir state <<<"$entry"
  tag="$name-v$version"

  "$here/notes.sh" "$dir/CHANGELOG.md" "$version" >notes.md
  if [ -f "$dir/release-footer.md" ]; then
    { echo; version=$version envsubst "\$version" <"$dir/release-footer.md"; } >>notes.md
  fi

  files=()
  if [ -d "$assets/$name" ]; then
    (cd "$assets/$name" && cat ./*.sha256 >sha256.sum)
    files=("$assets/$name"/*)
  fi

  prerelease=()
  [[ "$version" == *-* ]] && prerelease=(--prerelease)
  latest=--latest=false
  [ "$dir" = "$root" ] && [ ${#prerelease[@]} -eq 0 ] && latest=--latest

  # a draft is left over from a failed run
  [ "$state" = true ] && gh release delete "$tag" --yes
  gh release create "$tag" --verify-tag --title "$name $version" --notes-file notes.md \
    "$latest" "${prerelease[@]}" "${files[@]}"
  echo "released $tag"
done
