#!/usr/bin/env bash
# scripts/reconcile-lib-rs.sh — AC17 sanity check.
#
# Verifies that every `pub mod` declaration in a crate's `lib.rs`
# has a corresponding file on disk, and that every `*.rs` file
# under `src/` is either declared as a `pub mod` in `lib.rs` or
# is a `bin/` (binary) entry-point.
#
# Catches the most common drift bug in multi-crate Rust
# workspaces: a file added under `src/foo.rs` but never wired
# into `lib.rs` (it compiles only because `--all-targets`
# includes it indirectly via test refs, then disappears on a
# clean `cargo build`).
#
# Usage: bash scripts/reconcile-lib-rs.sh [crate-dir]
# Default crate-dir: crates/themis-orchestrator

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

CRATE_DIR="${1:-crates/themis-orchestrator}"
LIB_RS="$CRATE_DIR/src/lib.rs"

if [ ! -f "$LIB_RS" ]; then
    echo "✗ $LIB_RS not found; pass a crate-dir as arg 1" >&2
    exit 1
fi

echo "Reconciling $LIB_RS with files on disk..."

# 1. Collect declared mods.
#    `pub mod foo;` / `pub(crate) mod foo;` / `mod foo;`
DECLARED=$(grep -E '^[[:space:]]*(pub[[:space:]]*(\([[:space:]]*crate[[:space:]]*\))?[[:space:]]+)?mod[[:space:]]+[a-zA-Z_][a-zA-Z0-9_]*[[:space:]]*;' \
    "$LIB_RS" \
    | sed -E 's/^[[:space:]]*(pub[[:space:]]*(\([[:space:]]*crate[[:space:]]*\))?[[:space:]]+)?mod[[:space:]]+([a-zA-Z_][a-zA-Z0-9_]*)[[:space:]]*;.*/\3/' \
    | sort -u)

# 2. Collect .rs files under src/ (excluding bin/ and tests/).
ON_DISK=$(find "$CRATE_DIR/src" -maxdepth 1 -type f -name "*.rs" ! -name "lib.rs" \
    -printf '%f\n' 2>/dev/null \
    | sed -E 's/\.rs$//' \
    | sort -u)

# 3. Mismatch detection.
VIOLATIONS=0

# 3a. Declared but no file.
for mod in $DECLARED; do
    if [ ! -f "$CRATE_DIR/src/$mod.rs" ]; then
        echo "  ✗ declared mod '$mod' has no $mod.rs" >&2
        VIOLATIONS=$((VIOLATIONS + 1))
    fi
done

# 3b. File on disk but not declared.
for file in $ON_DISK; do
    if ! echo "$DECLARED" | grep -qx "$file"; then
        echo "  ✗ $file.rs exists but is not declared in lib.rs" >&2
        VIOLATIONS=$((VIOLATIONS + 1))
    fi
done

if [ "$VIOLATIONS" -gt 0 ]; then
    echo
    echo "lib.rs reconciliation: $VIOLATIONS drift violation(s) in $CRATE_DIR"
    exit 1
fi

echo "  ✓ $CRATE_DIR/src/lib.rs and src/*.rs are in sync"
echo "    declared: $(echo "$DECLARED" | wc -l) | on disk: $(echo "$ON_DISK" | wc -l)"
exit 0
