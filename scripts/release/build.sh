#!/usr/bin/env bash
# Build the jjp release binary for $TARGET, against glibc $GLIBC when it is set.
set -euo pipefail

if [ -n "${GLIBC:-}" ]; then
  mise run zigbuild
else
  cargo build --locked --profile dist --bin jjp --target "$TARGET"
fi
