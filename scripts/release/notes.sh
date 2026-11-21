#!/usr/bin/env bash
# Print the section of changelog $1 under the heading of version $2.
set -euo pipefail

heading="^## \\[?${2//./\\.}([] ]|$)" awk '
  /^## / { if (found) exit; found = $0 ~ ENVIRON["heading"]; next }
  found
' "$1"
