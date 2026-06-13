#!/usr/bin/env bash
# scripts/check-no-apohara.sh — AC11 hardening (R11).
#
# THEMIS must NOT depend on any other `apohara_*` crate. This script
# greps the workspace (excluding this script itself + the legacy
# `.archive/` dir + vendored fixtures) and exits 1 if any
# `use apohara_` import, `apohara_` Cargo path-dep, or `apohara-*`
# binary name is found.
#
# Installed automatically by scripts/install-pre-commit.sh; runs in
# the pre-commit hook before every commit.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

# Patterns we ban:
#   - use apohara_xxx::...
#   - use apohara::...
#   - apohara_xxx = ... (in Cargo.toml, but only in path/git
#     deps, since the workspace itself is themis_* and the
#     apohara- branding appears in the apohara-themis repo
#     name; this catches cross-crate pollution)
# We allow:
#   - The repo's own name (apohara-themis, apohara-dev, etc.)
#     appearing in URLs, .claude/, .gitignore, etc.
#   - The legacy `.archive/pre-themis/` (pre-hackathon dev snapshot)

VIOLATIONS=0

# 1. Rust source: any `use apohara_` import
echo "Checking Rust source for 'use apohara_' imports..."
# `apohara_` followed by an identifier char or `::`. The trailing
# `_` in apohara_ is a literal character class [a-zA-Z0-9_] so we
# match the prefix literally; \s with the literal `_` is what
# broke bash's word-splitting. Use fixed-string with grep -F first,
# then a regex pass to surface the actual matches.
if grep -rEn '(use[[:space:]]+apohara)|(apohara[_a-zA-Z0-9]*[[:space:]]*::)' crates/ 2>/dev/null; then
    echo "✗ Found apohara_ imports in Rust source" >&2
    VIOLATIONS=$((VIOLATIONS + 1))
else
    echo "  ✓ no apohara_ imports in crates/"
fi

# 2. Cargo.toml path/git dependencies on apohara-*
echo "Checking Cargo.toml for apohara- path dependencies..."
if grep -rEn 'apohara-?[a-zA-Z0-9_-]*\s*=\s*\{' crates/ 2>/dev/null; then
    echo "✗ Found apohara-* path deps in Cargo.toml" >&2
    VIOLATIONS=$((VIOLATIONS + 1))
else
    echo "  ✓ no apohara-* path deps"
fi

# 3. Binary names matching apohara-*
echo "Checking for apohara-* binaries in workspace Cargo.toml..."
if grep -rEn 'name\s*=\s*"apohara-' crates/ 2>/dev/null; then
    echo "✗ Found apohara-* binary name" >&2
    VIOLATIONS=$((VIOLATIONS + 1))
else
    echo "  ✓ no apohara-* binary names"
fi

if [ "$VIOLATIONS" -gt 0 ]; then
    echo
    echo "AC11 violation: $VIOLATIONS check(s) failed"
    exit 1
fi

echo
echo "✓ AC11 clean: no apohara_* imports, path deps, or binary names"
exit 0
