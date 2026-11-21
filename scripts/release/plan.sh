#!/usr/bin/env bash
# Write the release plan to $GITHUB_OUTPUT: the build matrix, the cli and lsp
# tags, whether each still needs a GitHub release (tagged on this commit, since
# its binaries build from it, and not published), and whether the cli needs its
# npm package (tagged, and that version not on npm).
set -euo pipefail
# shellcheck source=lib.sh
source "$(dirname "$0")/lib.sh"

unreleased() {
  [ "$(tag_commit "$1")" = "$GITHUB_SHA" ] && [ "$(release_state "$1")" != published ]
}

version=$(crates | awk '$1 == "jjpwrgem" { print $2 }')
tag="jjpwrgem-v$version"
lsp_tag="jjpwrgem-lsp-v$(crates | awk '$1 == "jjpwrgem-lsp" { print $2 }')"

cli=false
lsp=false
npm=false
unreleased "$tag" && cli=true
unreleased "$lsp_tag" && lsp=true
if [ -n "$(tag_commit "$tag")" ]; then
  [ -n "$(npm view "jjpwrgem@$version" version 2>/dev/null)" ] || npm=true
fi

{
  echo "matrix=$(jq -c '{include: .}' .github/release-targets.json)"
  echo "tag=$tag"
  echo "cli=$cli"
  echo "npm=$npm"
  echo "lsp_tag=$lsp_tag"
  echo "lsp=$lsp"
} >>"$GITHUB_OUTPUT"
