# shellcheck shell=bash
# Helpers shared by the release scripts. Source it, do not run it.

crates() {
  cargo metadata --no-deps --format-version 1 |
    jq -r '.packages[] | select(.publish != []) |
      "\(.name) \(.version) \(.manifest_path | sub("/Cargo.toml$"; ""))"'
}

release_state() {
  case $(gh release view "$1" --json isDraft -q .isDraft 2>/dev/null) in
    false) echo published ;;
    true) echo draft ;;
    *) echo missing ;;
  esac
}

tag_commit() {
  git ls-remote --tags origin "refs/tags/$1" "refs/tags/$1^{}" | awk 'END { print $1 }'
}

notes() {
  local version=$1 dir=$2
  heading="^## \\[?${version//./\\.}([] ]|$)" awk '
    /^## / { if (found) exit; found = $0 ~ ENVIRON["heading"]; next }
    found
  ' "$dir/CHANGELOG.md"
  if [ -f "$dir/release-footer.md" ]; then
    echo
    version=$version envsubst "\$version" <"$dir/release-footer.md"
  fi
}
