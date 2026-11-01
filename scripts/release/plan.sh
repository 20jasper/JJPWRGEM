#!/usr/bin/env bash
# Decide whether HEAD still has a release to finish: any publishable crate
# without its tag, or a library without its GitHub release, so reruns pick up
# where one stopped. jjpwrgem's own release comes from cargo-dist's workflow.
set -euo pipefail

pending=false
while read -r name version; do
  tag="$name-v$version"
  if ! git ls-remote --exit-code --tags origin "refs/tags/$tag" >/dev/null; then
    echo "$tag: not tagged"
    pending=true
  elif [ "$name" != jjpwrgem ] &&
    [ "$(gh release view "$tag" --json isDraft -q .isDraft 2>/dev/null || echo missing)" != false ]; then
    echo "$tag: no published GitHub release"
    pending=true
  fi
done < <(cargo metadata --no-deps --format-version 1 |
  jq -r '.packages[] | select(.publish != []) | "\(.name) \(.version)"')

echo "release=$pending" >>"${GITHUB_OUTPUT:-/dev/stdout}"
