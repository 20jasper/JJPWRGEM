#!/usr/bin/env bash
# Archive the jjp binary built for $1 into ./out with a .sha256 beside it.
# Same names and layout as the cargo-dist releases, which mise and the npm
# installer download: tar.xz with a top level dir, or a flat zip on windows.
# "wasm $version" instead archives the mise build-wasm output as a tar.gz.
set -euo pipefail

target=$1
if [ "$target" = wasm ]; then
  name=jjpwrgem-lsp-wasm-$2
  archive=$name.tar.gz
  changelog=crates/lsp/CHANGELOG.md
else
  archive=$(jq -er --arg target "$target" '.[] | select(.target == $target) | .archive' \
    .github/release-targets.json)
  name=${archive%%.*}
  changelog=CHANGELOG.md
fi

mkdir -p out "stage/$name"
cp LICENSE readme.md "$changelog" "stage/$name/"
case "$archive" in
  *.zip)
    cp "target/$target/dist/jjp.exe" "stage/$name/"
    (cd "stage/$name" && 7z a -tzip -bd "../../out/$archive" . >/dev/null)
    ;;
  *.tar.xz)
    cp "target/$target/dist/jjp" "stage/$name/"
    tar -C stage -cJf "out/$archive" "$name"
    ;;
  *.tar.gz)
    cp target/wasm/jjpwrgem_lsp* "stage/$name/"
    tar -C stage -czf "out/$archive" "$name"
    ;;
esac

cd out
if command -v sha256sum >/dev/null; then
  sha256sum -b "$archive" >"$archive.sha256"
else
  shasum -a 256 -b "$archive" >"$archive.sha256"
fi
cat "$archive.sha256"
