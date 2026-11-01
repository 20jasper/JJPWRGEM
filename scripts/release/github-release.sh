#!/usr/bin/env bash
# Give every tagged library version a GitHub release with its changelog
# section. jjpwrgem is left to cargo-dist, which attaches the binaries.
set -euo pipefail

# the changelog section under the version's heading, in sampo or release-plz style
notes() {
  heading="^## \\[?${2//./\\.}([] ]|$)" awk '
    /^## / { if (found) exit; found = $0 ~ ENVIRON["heading"]; next }
    found
  ' "$1"
}

while read -r name version manifest; do
  [ "$name" = jjpwrgem ] && continue
  tag="$name-v$version"
  git ls-remote --exit-code --tags origin "refs/tags/$tag" >/dev/null || continue
  gh release view "$tag" >/dev/null 2>&1 && continue

  notes "$(dirname "$manifest")/CHANGELOG.md" "$version" >notes.md
  prerelease=()
  [[ "$version" == *-* ]] && prerelease=(--prerelease)
  gh release create "$tag" --verify-tag --latest=false --title "$name $version" \
    --notes-file notes.md "${prerelease[@]}"
done < <(cargo metadata --no-deps --format-version 1 |
  jq -r '.packages[] | select(.publish != []) | "\(.name) \(.version) \(.manifest_path)"')
