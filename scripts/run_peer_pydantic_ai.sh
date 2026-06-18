#!/usr/bin/env bash
# Launch the real PydanticAI peer agent (Story FIX-4 / C-12).
#
# The peer subscribes to a Band chat room over WebSocket and emits
# independent fraud-auditor verdicts in response to @mentions. See
# ``agents/peers/peer_pydantic_ai.py`` for the protocol and
# ``crates/themis-orchestrator/src/a2a_handler.rs`` for the
# ``peer_verdict/attach`` ingestion method.
#
# Required env:
#   BAND_WS_URL   wss://...   Band room WebSocket endpoint
#   ROOM_ID       str         Band room id to subscribe to
#
# Optional env:
#   PEER_MODEL    default "claude-sonnet-4-5"; set to "test" for
#                 deterministic TestModel mode (no network, no cost).
#   PEER_API_KEY  default $ANTHROPIC_API_KEY
#   AGENT_NAME    default "peer_pydantic_ai"
#   LOG_LEVEL     default "INFO"
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PEER="${SCRIPT_DIR}/../agents/peers/peer_pydantic_ai.py"

if [[ ! -f "${PEER}" ]]; then
    echo "FATAL: peer script not found at ${PEER}" >&2
    exit 1
fi

if [[ -z "${BAND_WS_URL:-}" || -z "${ROOM_ID:-}" ]]; then
    echo "FATAL: BAND_WS_URL and ROOM_ID must be set" >&2
    exit 2
fi

# Use the repo's pinned interpreter if uv is on PATH, else fall back
# to system python3. The peer's PEP 723 script tag carries the deps.
if command -v uv >/dev/null 2>&1; then
    exec uv run --quiet "${PEER}"
fi

exec python3 "${PEER}"
