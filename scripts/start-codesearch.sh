#!/usr/bin/env bash
# scripts/start-codesearch.sh — launch codesearch-mcp on port 3000.
# Used by the agentgateway federation; profile=mcp in docker-compose.
set -euo pipefail
PORT="${CODESEARCH_PORT:-3000}"
if ss -ltn 2>/dev/null | grep -q ":$PORT "; then
  echo "codesearch-mcp already running on :$PORT"
  exit 0
fi
# Codesearch MCP is the Apohara open-source MCP server.
# In CI we mock it with a tiny Python shim that returns empty results.
if [ -f "$(dirname "$0")/../crates/themis-compliance/tests/codesearch_shim.py" ]; then
  python3 "$(dirname "$0")/../crates/themis-compliance/tests/codesearch_shim.py" --port "$PORT" &
  echo $! > /tmp/codesearch-mcp.pid
  echo "codesearch-mcp shim started on :$PORT (pid $(cat /tmp/codesearch-mcp.pid))"
else
  echo "ERROR: codesearch shim not found. See crates/themis-compliance/tests/codesearch_shim.py" >&2
  exit 1
fi
