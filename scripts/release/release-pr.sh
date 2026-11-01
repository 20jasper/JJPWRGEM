#!/usr/bin/env bash
# Open or update the release PR from pending changesets. sampo bumps versions
# and changelogs, then the npm package is regenerated to match the new version.
set -euo pipefail

branch=release/main

if ! compgen -G ".sampo/changesets/*.md" >/dev/null; then
  echo "no pending changesets"
  exit 0
fi

plan=$(sampo release --dry-run | sed -n '/^Planned releases:/,/^Dry-run/p' | sed '$d')
sampo release
mise run package-json

git switch -C "$branch"
git add -A
git -c user.name="github-actions[bot]" \
  -c user.email="41898282+github-actions[bot]@users.noreply.github.com" \
  commit -q -m "Release"
remote=origin
if [ -n "${GITHUB_ACTIONS:-}" ]; then
  remote="https://x-access-token:$GH_TOKEN@github.com/$GITHUB_REPOSITORY.git"
fi
# a bot branch rebuilt from main on every run
git push --force "$remote" "$branch"

body=$(
  echo '```'
  echo "$plan"
  echo '```'
  for changelog in $(git diff --name-only HEAD~ -- '*CHANGELOG.md'); do
    echo
    echo "# $changelog"
    git diff --no-ext-diff --no-color -U0 HEAD~ -- "$changelog" | sed -n 's/^+\([^+]\)/\1/p; s/^+$//p'
  done
)

number=$(gh pr list --head "$branch" --state open --json number -q '.[0].number')
if [ -n "$number" ]; then
  gh pr edit "$number" --body "$body"
else
  gh pr create --head "$branch" --base main --title "Release" --body "$body"
fi
