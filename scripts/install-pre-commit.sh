#!/usr/bin/env bash
# scripts/install-pre-commit.sh — install the THEMIS pre-commit hook.
#
# R11/US-X02: enforce AC11 (no apohara_* imports) + cargo-deny
# before every commit. Idempotent: re-running is a no-op if the
# hook is already installed and up-to-date.
#
# The hook source is the file YOU are reading (with the shebang
# stripped); we copy it to .git/hooks/pre-commit so git picks it up.
#
# Run once: `bash scripts/install-pre-commit.sh`

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

HOOK_PATH=".git/hooks/pre-commit"
HOOK_SOURCE="scripts/pre-commit.sh"

if [ ! -f "$HOOK_SOURCE" ]; then
    echo "✗ Expected hook source at $HOOK_SOURCE" >&2
    exit 1
fi

# If the hook is already installed and points at our source, no-op.
if [ -f "$HOOK_PATH" ] && grep -q "scripts/pre-commit.sh" "$HOOK_PATH" 2>/dev/null; then
    echo "✓ Pre-commit hook already installed at $HOOK_PATH"
    exit 0
fi

# Install: copy the source, mark executable.
cp "$HOOK_SOURCE" "$HOOK_PATH"
chmod +x "$HOOK_PATH"
echo "✓ Installed pre-commit hook at $HOOK_PATH"
echo
echo "What it does on every commit:"
echo "  1. Runs scripts/check-no-apohara.sh (AC11 guard)"
echo "  2. Runs cargo deny check (R11)"
echo
echo "Bypass with: git commit --no-verify  (use sparingly)"
