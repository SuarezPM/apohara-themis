#!/usr/bin/env bash
# THEMIS pre-commit hook — installed by scripts/install-pre-commit.sh.
#
# R11/US-X02: enforce AC11 + cargo-deny before every commit.
# Bypass: `git commit --no-verify` (use sparingly).

set -e

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

echo "[pre-commit] AC11 check (no apohara_* imports)..."
bash scripts/check-no-apohara.sh

if command -v cargo-deny >/dev/null 2>&1; then
    echo "[pre-commit] cargo-deny..."
    cargo deny check
else
    echo "[pre-commit] cargo-deny not installed; install with \`cargo install cargo-deny --locked\`"
    echo "           (skipping this check; the rest of the hook passed)"
fi

echo "[pre-commit] OK"
exit 0
