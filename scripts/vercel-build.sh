#!/usr/bin/env bash
# Vercel build for apohara-themis.
#
# Flattens crates/themis-frontend/static/ into public/ and rewrites
# the /static/* asset references inside the HTML to relative paths
# so they resolve at https://themis.apohara.dev/.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/crates/themis-frontend/static"
OUT="$ROOT/public"

rm -rf "$OUT"
mkdir -p "$OUT"

# Copy every asset, then rewrite path references in the HTML files.
cp "$SRC"/* "$OUT"/

# Rewrite /static/<file> → <file> in the HTML files only (assets keep
# their own internal references untouched).
for html in "$OUT"/*.html; do
  sed -i 's|href="/static/|href="./|g; s|src="/static/|src="./|g' "$html"
done

echo "[vercel-build] output:"
ls -la "$OUT"
