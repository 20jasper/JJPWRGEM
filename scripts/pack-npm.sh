#!/usr/bin/env bash
# Copy repo-level docs into the npm package, pack it into ./npm, and check the
# tarball contains them. Run from the repo root.
set -euo pipefail

pkg=npm-packages/jjpwrgem
required=(README.md CHANGELOG.md LICENSE)

# npm pack skips symlinks, so the license must be a real copy
cp -f readme.md "$pkg/README.md"
cp -f CHANGELOG.md "$pkg/CHANGELOG.md"
cp -f LICENSE "$pkg/LICENSE"

rm -rf ./npm
mkdir -p ./npm
npm pack "./$pkg" --pack-destination ./npm

tarballs=(./npm/*.tgz)
[[ ${#tarballs[@]} -eq 1 ]] || { echo "expected 1 tarball, found ${#tarballs[@]}" >&2; exit 1; }
contents=$(tar -tzf "${tarballs[0]}")

for f in "${required[@]}"; do
  grep -qx "package/$f" <<<"$contents" || { echo "missing $f in ${tarballs[0]}" >&2; exit 1; }
done
echo "ok: ${tarballs[0]} contains ${required[*]}"
