#!/usr/bin/env bash
# Fails if any AI-generated stand-in assets (ai_* files, or an ai/ directory
# with content) exist under the given directory. Run against any shipping
# build's asset tree before uploading to a storefront (see
# docs/DEION_STYLE.md §6, AI-asset quarantine rule).
#
# Usage: scripts/check_no_ai_assets.sh <assets-dir>
set -euo pipefail

dir="${1:?usage: check_no_ai_assets.sh <assets-dir>}"
[ -d "$dir" ] || { echo "check_no_ai_assets: no such directory: $dir" >&2; exit 2; }

found=$(find "$dir" \( -name 'ai_*' -o -path '*/ai/*' \) -type f)
if [ -n "$found" ]; then
    echo "FAIL: AI stand-in assets present — purge before shipping:" >&2
    echo "$found" >&2
    exit 1
fi
echo "OK: no AI stand-in assets under $dir"
