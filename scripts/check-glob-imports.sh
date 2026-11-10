#!/usr/bin/env bash
set -euo pipefail

if git grep -n 'gen_lsp_types::{' -- '*.rs'; then
  echo "error: glob import gen_lsp_types; use 'gen_lsp_types::*'" >&2
  exit 1
fi
