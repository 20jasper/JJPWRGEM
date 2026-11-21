#!/usr/bin/env bash
# Archive the mise build-wasm output of the lsp crate for version $1 into ./out
# with a .sha256 beside it: tar.gz with a top level dir.
set -euo pipefail

name=jjpwrgem-lsp-wasm-$1
archive=$name.tar.gz

mkdir -p out "stage/$name"
cp LICENSE crates/lsp/readme.md crates/lsp/CHANGELOG.md target/wasm/jjpwrgem_lsp* "stage/$name/"
tar -C stage -czf "out/$archive" "$name"

cd out
if command -v sha256sum >/dev/null; then
  sha256sum -b "$archive" >"$archive.sha256"
else
  shasum -a 256 -b "$archive" >"$archive.sha256"
fi
cat "$archive.sha256"
