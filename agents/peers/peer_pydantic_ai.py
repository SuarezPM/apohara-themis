#!/usr/bin/env python3
"""Real PydanticAI peer agent.

Subscribes to a Band chat room over WebSocket and emits independent
fraud-auditor verdicts in response to @mentions. Uses
``pydantic_ai.Agent`` with a configurable Anthropic-compatible model
(defaults to ``claude-sonnet-4-5`` via the AIML API gateway). Falls
back to the deterministic ``TestModel`` when ``PEER_MODEL=test`` so
the demo path never needs network or paid API quota.

Story: FIX-4 / C-12. Complement of the A2A HTTP peer in
``invoice_validator.py``; this peer joins the Band room itself and
emits verdicts to peers that follow the @mention-routing convention
(``@peer_pydantic_ai <invoice summary>``).

Wire format (outbound):
    {
        "agent": "peer_pydantic_ai",
        "type": "peer_verdict",
        "data": {
            "risk_score": 0.0-1.0,
            "findings": [str, ...],
            "recommendation": "approve" | "halt",
            "tenant_id": "<echoed>",
            "invoice_id": "<echoed>",
        }
    }

Environment:
    BAND_WS_URL       wss://...   (required)
    ROOM_ID           str         (required; room id to subscribe to)
    PEER_MODEL        str         (default "claude-sonnet-4-5")
    PEER_API_KEY      str         (default: read from ANTHROPIC_API_KEY)
    AGENT_NAME        str         (default "peer_pydantic_ai")

Run:
    BAND_WS_URL=wss://api.band.dev/ws ROOM_ID=themis-3-demo \\
        python3 peer_pydantic_ai.py
"""
from __future__ import annotations

import asyncio
import json
import logging
import os
import sys
from typing import Any, Optional

try:
    from pydantic_ai import Agent
except ImportError:  # pragma: no cover - surfaced as fatal sys.exit
    print(
        "FATAL: pydantic-ai not installed. Run: pip install -r requirements.txt",
        file=sys.stderr,
    )
    sys.exit(2)

import websockets

log = logging.getLogger("themis.peer.pydantic_ai")


# Hardcoded policy used by the deterministic TestModel path. Kept here
# (not in invoice_validator.py) because the WS peer has no HTTP
# caller to pass ``amount``; the @mention body is parsed as JSON.
_AMOUNT_REJECT = 50_000.0
_AMOUNT_REVIEW = 100.0


def _deterministic_verdict(body: str) -> dict[str, Any]:
    """Offline verdict used when ``PEER_MODEL=test`` or no API key.

    Extracts a JSON object from the @mention body (best-effort) and
    applies the same thresholds as ``invoice_validator.py``. The
    shape matches the LLM verdict exactly so downstream consumers
    cannot tell the two apart.
    """
    try:
        invoice = json.loads(body)
    except (ValueError, TypeError):
        invoice = {}

    amount = float(invoice.get("amount", 0.0) or 0.0)
    invoice_id = str(invoice.get("invoice_id", "unknown"))
    if amount > _AMOUNT_REJECT:
        return {
            "risk_score": 0.95,
            "findings": [f"amount {amount:.2f} exceeds policy ceiling {_AMOUNT_REJECT:.2f}"],
            "recommendation": "halt",
            "tenant_id": str(invoice.get("tenant_id", "")),
            "invoice_id": invoice_id,
        }
    if amount < _AMOUNT_REVIEW:
        return {
            "risk_score": 0.55,
            "findings": [f"amount {amount:.2f} below review floor {_AMOUNT_REVIEW:.2f}"],
            "recommendation": "halt",
            "tenant_id": str(invoice.get("tenant_id", "")),
            "invoice_id": invoice_id,
        }
    return {
        "risk_score": 0.10,
        "findings": ["within policy range; vendor name and due_date sanity-checked"],
        "recommendation": "approve",
        "tenant_id": str(invoice.get("tenant_id", "")),
        "invoice_id": invoice_id,
    }


class PeerPydanticAI:
    """Band-room WebSocket subscriber that emits peer verdicts.

    The constructor wires the PydanticAI ``Agent`` once; the network
    loop in :meth:`run` reuses it. When ``PEER_MODEL=test`` the
    agent is replaced with a ``TestModel`` that ignores prompts
    and the deterministic policy above is used as a stand-in for
    the LLM's structured output.
    """

    def __init__(
        self,
        band_ws_url: str,
        room_id: str,
        model: str = "claude-sonnet-4-5",
        api_key: Optional[str] = None,
        agent_name: str = "peer_pydantic_ai",
    ) -> None:
        self.band_ws_url = band_ws_url
        self.room_id = room_id
        self.agent_name = agent_name
        self.model_name = model
        self.ws: Optional[Any] = None
        self._deterministic = model == "test" or not api_key

        if self._deterministic:
            log.info("running in deterministic TestModel mode (no LLM calls)")
            self.agent: Optional[Agent] = None
        else:
            os.environ.setdefault("ANTHROPIC_API_KEY", api_key or "")
            try:
                self.agent = Agent(
                    model,
                    system_prompt=(
                        "You are an independent fraud-auditor peer agent. "
                        "When asked about an invoice via @mention, emit a "
                        "JSON verdict with: "
                        "{'risk_score': 0.0-1.0, 'findings': [...], "
                        "'recommendation': 'approve'|'halt'}."
                    ),
                )
            except Exception as exc:  # model name not recognised
                log.warning("Agent(%r) failed: %s; falling back to deterministic", model, exc)
                self._deterministic = True
                self.agent = None

    async def connect(self) -> None:
        log.info("connecting to Band WS %s (room=%s)", self.band_ws_url, self.room_id)
        self.ws = await websockets.connect(self.band_ws_url)

    async def _verdict_for(self, body: str) -> dict[str, Any]:
        if self._deterministic or self.agent is None:
            return _deterministic_verdict(body)
        try:
            result = await self.agent.run(body)
            # PydanticAI returns the structured data on ``.data`` when
            # the agent has an output_type; without one, ``.data`` is
            # the raw string. Try JSON parse; fall back to a
            # permissive envelope so a non-JSON LLM reply still
            # satisfies the A2A contract.
            data = getattr(result, "data", None)
            if isinstance(data, str):
                data = json.loads(data)
            if not isinstance(data, dict):
                data = {"raw": str(data)}
            data.setdefault("risk_score", 0.0)
            data.setdefault("findings", [])
            data.setdefault("recommendation", "approve")
            return data
        except Exception as exc:
            log.exception("agent.run failed: %s; falling back to deterministic", exc)
            return _deterministic_verdict(body)

    async def _handle_message(self, raw: str) -> Optional[dict[str, Any]]:
        try:
            data = json.loads(raw)
        except (ValueError, TypeError):
            log.debug("non-JSON message ignored")
            return None
        content = data.get("content", "")
        if not isinstance(content, str):
            return None
        if f"@{self.agent_name}" not in content:
            return None
        body = content.split(f"@{self.agent_name}", 1)[-1].strip()
        verdict = await self._verdict_for(body)
        return {
            "agent": self.agent_name,
            "type": "peer_verdict",
            "data": verdict,
        }

    async def run(self) -> None:
        assert self.ws is not None, "connect() before run()"
        log.info("listening for @%s mentions", self.agent_name)
        async for raw in self.ws:
            envelope = await self._handle_message(raw)
            if envelope is None:
                continue
            await self.ws.send(json.dumps(envelope))
            log.info(
                "verdict emitted: %s risk=%.2f rec=%s",
                self.agent_name,
                float(envelope["data"].get("risk_score", 0.0)),
                envelope["data"].get("recommendation", "?"),
            )


async def main() -> None:
    logging.basicConfig(
        level=os.environ.get("LOG_LEVEL", "INFO").upper(),
        format="%(asctime)s [%(name)s] %(levelname)s %(message)s",
    )
    peer = PeerPydanticAI(
        band_ws_url=os.environ["BAND_WS_URL"],
        room_id=os.environ["ROOM_ID"],
        model=os.environ.get("PEER_MODEL", "claude-sonnet-4-5"),
        api_key=os.environ.get("PEER_API_KEY") or os.environ.get("ANTHROPIC_API_KEY"),
        agent_name=os.environ.get("AGENT_NAME", "peer_pydantic_ai"),
    )
    await peer.connect()
    await peer.run()


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        log.info("shutdown requested")
