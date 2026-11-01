#!/usr/bin/env bash
# GitHub release for cli tag $1 with the binaries in $2
set -euo pipefail

tag=$1
assets=$2
version=${tag#jjpwrgem-v}

state=$(gh release view "$tag" --json isDraft -q .isDraft 2>/dev/null || echo missing)
# already released
if [ "$state" = false ]; then
  echo "$tag is already published"
  exit 0
fi

# draft first, since published releases are immutable and take no more assets
if [ "$state" = missing ]; then
  # the changelog section under the version's heading
  heading="^## \\[?${version//./\\.}([] ]|$)" awk '
    /^## / { if (found) exit; found = $0 ~ ENVIRON["heading"]; next }
    found
  ' CHANGELOG.md >notes.md
  cat >>notes.md <<EOF

## Install jjpwrgem $version

\`\`\`bash
mise use -g github:20jasper/jjpwrgem@$version
npm install -g jjpwrgem@$version
cargo install jjpwrgem --version $version --locked
\`\`\`
EOF
  prerelease=()
  [[ "$version" == *-* ]] && prerelease=(--prerelease)
  gh release create "$tag" --draft --verify-tag --title "jjpwrgem $version" \
    --notes-file notes.md "${prerelease[@]}"
fi

(cd "$assets" && cat ./*.sha256 >sha256.sum)
gh release upload "$tag" "$assets"/* --clobber

# every archive, its .sha256 and sha256.sum
missing=$(comm -23 \
  <({ jq -r '.[].archive' .github/release-targets.json | sed 'p; s/$/.sha256/'; echo sha256.sum; } | sort) \
  <(gh release view "$tag" --json assets -q '.assets[].name' | sort))
if [ -n "$missing" ]; then
  echo "$tag is missing assets: $missing" >&2
  exit 1
fi

# prereleases never become latest
latest=--latest
[[ "$version" == *-* ]] && latest=--latest=false
gh release edit "$tag" --draft=false "$latest"
echo "published $tag"
