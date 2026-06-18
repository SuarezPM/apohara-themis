#!/usr/bin/env python3
"""THEMIS 3.0 - Mock PydanticAI agent fallback (stdlib-only).

This is the documented mock fallback for Story C-12. It exists so the
demo can run even when the `pydantic-ai` package fails to install in
the runtime image (e.g. Py3.14 wheel not yet available, or a CI image
that pins an older Python).

**This script uses only the Python standard library** (`http.server`,
`json`, `argparse`). No pip dependency. Swap it for
`invoice_validator.py` once the pydantic-ai wheel is available.

The mock serves the **same** A2A 1.0 surface as the real peer
(`/.well-known/agent-card.json`, `POST /a2a`, `GET /healthz`) so the
orchestrator and the agentgateway sidecar cannot tell the difference.
The verdict is hardcoded to `APPROVED` for any well-formed invoice.

The card's `x-themis.mock` field is set to `true` so an operator
inspecting the card can see at a glance that this peer is the mock.

Run:
    python3 MockPydanticAIAgent.py            # serves on :7001
    python3 MockPydanticAIAgent.py --port 8001 # custom port
"""
from __future__ import annotations

import argparse
import json
import logging
import os
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Any

log = logging.getLogger("themis.peer.mock")

DEFAULT_PORT = 7001
PEER_NAME = "THEMIS Invoice Validator (PydanticAI - MOCK)"
PEER_FRAMEWORK = "PydanticAI-Mock"

# Match the real peer's card shape so a swap is a no-op for the
# orchestrator and the agentgateway sidecar.
A2A_CARD: dict[str, Any] = {
    "protocolVersion": "1.0",
    "name": PEER_NAME,
    "description": (
        "Mock PydanticAI peer for the 18-jun-2026 EOD fallback path. "
        "Stdlib-only implementation; verdict is hardcoded to APPROVED "
        "for any well-formed invoice."
    ),
    "url": f"http://localhost:{DEFAULT_PORT}/a2a",
    "preferredTransport": "JSONRPC",
    "version": "3.0.0",
    "provider": {"organization": "Apohara", "url": "https://apohara.dev"},
    "capabilities": {
        "streaming": False,
        "pushNotifications": False,
        "stateTransitionHistory": True,
    },
    "defaultInputModes": ["application/json"],
    "defaultOutputModes": ["application/json"],
    "skills": [
        {
            "id": "validate_invoice",
            "name": "Validate Invoice (MOCK)",
            "description": "Hardcoded APPROVED verdict. Mock fallback only.",
            "inputModes": ["application/json"],
            "outputModes": ["application/json"],
            "examples": [
                {
                    "input": {"invoice_id": "INV-001", "amount": 500.0},
                    "output": {"verdict": "APPROVED", "framework": PEER_FRAMEWORK},
                }
            ],
        }
    ],
    "authentication": {"schemes": ["Ed25519Bearer"]},
    "x-themis": {
        "framework": PEER_FRAMEWORK,
        "story": "C-12",
        "sprint": 3,
        "gaps": ["G27"],
        "mock": True,
        "mock_reason": "stdlib-only fallback for 18-jun-2026 EOD",
    },
}


def _validate_invoice_shape(invoice: Any) -> dict[str, Any]:
    if not isinstance(invoice, dict):
        raise ValueError("invoice must be a JSON object")
    required = ("invoice_id", "vendor_name", "amount", "due_date", "line_items")
    missing = [k for k in required if k not in invoice]
    if missing:
        raise ValueError(f"missing fields: {', '.join(missing)}")
    if not isinstance(invoice["amount"], (int, float)):
        raise ValueError("amount must be numeric")
    return invoice


def _make_verdict(invoice: dict[str, Any]) -> dict[str, Any]:
    """Hardcoded mock verdict - APPROVED for any well-formed invoice."""
    return {
        "verdict": "APPROVED",
        "reason": "MockPydanticAIAgent hardcoded APPROVED (stdlib-only fallback)",
        "invoice_id": invoice["invoice_id"],
        "vendor_name": invoice["vendor_name"],
        "amount": float(invoice["amount"]),
        "framework": PEER_FRAMEWORK,
        "agent": "MockPydanticAIAgent",
        "mock": True,
    }


def _jsonrpc_envelope(id_: Any, result: dict[str, Any]) -> dict[str, Any]:
    return {"jsonrpc": "2.0", "id": id_, "result": result}


def _jsonrpc_error(id_: Any, code: int, message: str) -> dict[str, Any]:
    return {"jsonrpc": "2.0", "id": id_, "error": {"code": code, "message": message}}


class _Handler(BaseHTTPRequestHandler):
    """stdlib HTTP request handler for the mock peer."""

    server_version = "THEMIS-MockPydanticAIAgent/3.0"

    def _send_json(self, status: int, body: dict[str, Any]) -> None:
        payload = json.dumps(body).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def log_message(self, fmt: str, *args: Any) -> None:
        log.info("%s - - %s", self.address_string(), fmt % args)

    def do_GET(self) -> None:  # noqa: N802
        if self.path == "/.well-known/agent-card.json":
            self._send_json(200, A2A_CARD)
            return
        if self.path == "/healthz":
            self._send_json(
                200,
                {
                    "status": "ok",
                    "framework": PEER_FRAMEWORK,
                    "name": PEER_NAME,
                    "ts": int(time.time()),
                    "mock": True,
                },
            )
            return
        self._send_json(404, {"error": "not found"})

    def do_POST(self) -> None:  # noqa: N802
        if self.path != "/a2a":
            self._send_json(404, {"error": "not found"})
            return
        length = int(self.headers.get("Content-Length", "0") or "0")
        raw = self.rfile.read(length) if length > 0 else b""
        try:
            payload = json.loads(raw)
        except json.JSONDecodeError as exc:
            self._send_json(400, _jsonrpc_error(None, -32700, f"Parse error: {exc}"))
            return

        if not isinstance(payload, dict) or payload.get("jsonrpc") != "2.0":
            self._send_json(
                400,
                _jsonrpc_error(
                    payload.get("id") if isinstance(payload, dict) else None,
                    -32600,
                    "Invalid Request: jsonrpc must be '2.0'",
                ),
            )
            return

        if payload.get("method") != "message/send":
            self._send_json(
                400,
                _jsonrpc_error(payload.get("id"), -32601, f"Method not found: {payload.get('method')}"),
            )
            return

        params = payload.get("params") or {}
        message = params.get("message") or {}
        parts = message.get("parts") or []
        invoice_raw: Any = None
        for part in parts:
            if part.get("type") == "data":
                invoice_raw = part.get("data")
                break
        if invoice_raw is None:
            self._send_json(
                400,
                _jsonrpc_error(payload.get("id"), -32602, "Invalid params: no invoice data"),
            )
            return

        try:
            invoice = _validate_invoice_shape(invoice_raw)
        except ValueError as exc:
            self._send_json(
                400,
                _jsonrpc_error(payload.get("id"), -32602, f"Invalid params: {exc}"),
            )
            return

        verdict = _make_verdict(invoice)
        result = {
            "kind": "message",
            "role": "agent",
            "parts": [{"type": "data", "data": verdict}],
            "metadata": {
                "framework": PEER_FRAMEWORK,
                "ts": int(time.time()),
                "agent_card": PEER_NAME,
                "mock": True,
            },
        }
        self._send_json(200, _jsonrpc_envelope(payload.get("id"), result))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=int(os.environ.get("MOCK_PORT", DEFAULT_PORT)))
    parser.add_argument("--host", default=os.environ.get("MOCK_HOST", "0.0.0.0"))
    args = parser.parse_args()

    A2A_CARD["url"] = f"http://localhost:{args.port}/a2a"

    logging.basicConfig(
        level=os.environ.get("LOG_LEVEL", "INFO").upper(),
        format="%(asctime)s [%(name)s] %(levelname)s %(message)s",
    )

    server = ThreadingHTTPServer((args.host, args.port), _Handler)
    log.info("[%s] serving A2A on %s:%d (MOCK - stdlib only)", PEER_FRAMEWORK, args.host, args.port)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        log.info("shutting down")
        server.server_close()


if __name__ == "__main__":
    main()
