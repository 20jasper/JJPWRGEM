# shellcheck shell=bash
# Helpers shared by the release scripts. Source it, do not run it.

# name, version and directory of each publishable crate
crates() {
  cargo metadata --no-deps --format-version 1 |
    jq -r '.packages[] | select(.publish != []) |
      "\(.name) \(.version) \(.manifest_path | sub("/Cargo.toml$"; ""))"'
}

# missing, draft or published
release_state() {
  case $(gh release view "$1" --json isDraft -q .isDraft 2>/dev/null) in
    false) echo published ;;
    true) echo draft ;;
    *) echo missing ;;
  esac
}

# the commit tag $1 points at on origin, nothing when it is not pushed
tag_commit() {
  git ls-remote --tags origin "refs/tags/$1" "refs/tags/$1^{}" | awk 'END { print $1 }'
}
