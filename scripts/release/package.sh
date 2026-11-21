#!/usr/bin/env bash
# Archive the jjp binary built for $1 into ./out with a .sha256 beside it.
# Same names and layout as the cargo-dist releases, which mise and the npm
# installer download.
set -euo pipefail

target=$1
archive=$(jq -er --arg target "$target" '.[] | select(.target == $target) | .archive' \
  .github/release-targets.json)
bin=jjp
[[ "$archive" == *.zip ]] && bin=jjp.exe

"$(dirname "$0")/archive.sh" "$archive" LICENSE readme.md CHANGELOG.md "target/$target/dist/$bin"
