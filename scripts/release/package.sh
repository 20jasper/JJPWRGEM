#!/usr/bin/env bash
# Archive the jjp binary built for $1 into ./out with a .sha256 beside it.
# Same names and layout as the cargo-dist releases, which mise and the npm
# installer download: tar.xz with a top level dir, or a flat zip on windows.
set -euo pipefail

target=$1
archive=$(jq -er --arg target "$target" '.[] | select(.target == $target) | .archive' \
  .github/release-targets.json)
name=${archive%%.*}

mkdir -p out "stage/$name"
cp LICENSE readme.md CHANGELOG.md "stage/$name/"
case "$archive" in
  *.zip)
    cp "target/$target/dist/jjp.exe" "stage/$name/"
    (cd "stage/$name" && 7z a -tzip -bd "../../out/$archive" . >/dev/null)
    ;;
  *.tar.xz)
    cp "target/$target/dist/jjp" "stage/$name/"
    tar -C stage -cJf "out/$archive" "$name"
    ;;
esac

cd out
if command -v sha256sum >/dev/null; then
  sha256sum -b "$archive" >"$archive.sha256"
else
  shasum -a 256 -b "$archive" >"$archive.sha256"
fi
cat "$archive.sha256"
