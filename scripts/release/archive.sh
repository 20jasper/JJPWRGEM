#!/usr/bin/env bash
# Archive the files $2.. into ./out/$1 (.tar.xz, .tar.gz or flat .zip) with a .sha256 beside it.
set -euo pipefail

archive=$1
shift
name=${archive%.tar.*}
name=${name%.zip}

mkdir -p out "stage/$name"
cp "$@" "stage/$name/"
case "$archive" in
  *.zip) (cd "stage/$name" && 7z a -tzip -bd "../../out/$archive" . >/dev/null) ;;
  *.tar.xz) tar -C stage -cJf "out/$archive" "$name" ;;
  *.tar.gz) tar -C stage -czf "out/$archive" "$name" ;;
  *)
    echo "unsupported archive format: $archive" >&2
    exit 1
    ;;
esac

cd out
if command -v sha256sum >/dev/null; then
  sha256sum -b "$archive" >"$archive.sha256"
else
  shasum -a 256 -b "$archive" >"$archive.sha256"
fi
cat "$archive.sha256"
