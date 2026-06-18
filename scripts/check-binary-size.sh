#!/usr/bin/env bash
# scripts/check-binary-size.sh — AC17 CI gate (binary size cap).
#
# Enforces the critic-amended 30 MB cap on the single-binary release
# artifact. Scans target/release/ for the workspace binaries and
# fails the build if any one of them exceeds 30 MB.
#
# The default cap (30 MB) is set per the critic amendment to the
# 96/100 score projection: a fat LTO + stripped single binary must
# fit in 30 MB or the deploy is rejected. Override with the
# THEMIS_BINARY_SIZE_CAP_MB env var if you need to investigate a
# specific build (e.g. `THEMIS_BINARY_SIZE_CAP_MB=40 bash
# scripts/check-binary-size.sh`).

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

CAP_MB="${THEMIS_BINARY_SIZE_CAP_MB:-30}"
CAP_BYTES=$((CAP_MB * 1024 * 1024))

RELEASE_DIR="target/release"
if [ ! -d "$RELEASE_DIR" ]; then
    echo "✗ $RELEASE_DIR/ not found; run 'cargo build --release' first" >&2
    exit 1
fi

# Binaries to check: the workspace's single-binary release
# targets. We list the ones the plan calls out explicitly so
# `themis-orchestrator` (the deployable binary) is gated, plus the
# secondary artifacts (`themis-verify`, `themis-aibom`,
# `themis-redteam`, `bench`) so regressions surface here.
EXPECTED_BINS=(
    "themis-orchestrator"
    "themis-verify"
    "themis-aibom"
    "themis-redteam"
    "bench"
)

VIOLATIONS=0
CHECKED=0

for bin in "${EXPECTED_BINS[@]}"; do
    bin_path="$RELEASE_DIR/$bin"
    if [ ! -x "$bin_path" ]; then
        # Missing binary is not a size violation; the build
        # itself would have failed. Skip silently to keep the
        # size-check focused on its single concern.
        continue
    fi
    CHECKED=$((CHECKED + 1))
    size_bytes=$(stat -c '%s' "$bin_path" 2>/dev/null || stat -f '%z' "$bin_path")
    size_mb=$(awk -v b="$size_bytes" 'BEGIN { printf "%.2f", b / 1024 / 1024 }')
    if [ "$size_bytes" -gt "$CAP_BYTES" ]; then
        echo "✗ $bin = ${size_mb} MB (cap: ${CAP_MB} MB)" >&2
        VIOLATIONS=$((VIOLATIONS + 1))
    else
        echo "  ✓ $bin = ${size_mb} MB (cap: ${CAP_MB} MB)"
    fi
done

if [ "$CHECKED" -eq 0 ]; then
    echo "✗ no release binaries found in $RELEASE_DIR; build first with 'cargo build --release'" >&2
    exit 1
fi

if [ "$VIOLATIONS" -gt 0 ]; then
    echo
    echo "AC17 binary-size gate: $VIOLATIONS violation(s) over the ${CAP_MB} MB cap"
    exit 1
fi

echo
echo "✓ AC17 binary-size gate: $CHECKED binary(ies) within ${CAP_MB} MB cap"
exit 0
