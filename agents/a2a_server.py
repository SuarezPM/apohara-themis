"""Shared A2A 1.0 + JSON-RPC server used by all three Python peers.

Every peer (PydanticAI, LangGraph, CrewAI) hosts a tiny starlette app that
serves:

* `GET  /.well-known/agent-card.json` - the A2A 1.0 Agent Card.
* `POST /a2a`                        - JSON-RPC 2.0 `message/send` handler.
* `GET  /healthz`                    - liveness probe (returns 200 + JSON).

The verdict is produced by a peer-specific `verdict_fn(invoice: dict) -> str`
function. Per the C-12 PRD, the MVP is deterministic and LLM-free; a real
LLM call is a follow-up.

The Band room WebSocket join is opt-in via `BAND_WS_URL` + `BAND_API_KEY` +
`BAND_ROOM` (the WebSocket task runs in the background so the HTTP server
keeps serving even if the Band room is unreachable in dev).
"""
from __future__ import annotations

import asyncio
import inspect
import json
import logging
import os
import time
from contextlib import asynccontextmanager
from typing import Any, Awaitable, Callable

import httpx
import uvicorn
import websockets
from dotenv import load_dotenv
from starlette.applications import Starlette
from starlette.requests import Request
from starlette.responses import JSONResponse
from starlette.routing import Route

load_dotenv()

log = logging.getLogger("themis.peer")

# A2A 1.0 protocol fields shared by every peer. Per C-01, the orchestrator's
# card is the source of truth for `protocolVersion`, `capabilities`, and
# `authentication.schemes`. The peers mirror the same shape so the registry
# (`static/agents.json`) can list them as first-class A2A citizens.
A2A_PROTOCOL_VERSION = "1.0"
A2A_AUTH_SCHEMES = ["Ed25519Bearer"]
A2A_CAPABILITIES = {
    "streaming": False,
    "pushNotifications": False,
    "stateTransitionHistory": True,
}

VerdictFn = Callable[[dict[str, Any]], Awaitable[dict[str, Any]]]


def build_card(
    *,
    name: str,
    description: str,
    framework: str,
    port: int,
    skill_id: str,
    skill_name: str,
    skill_description: str,
) -> dict[str, Any]:
    """Return an A2A 1.0 Agent Card for one peer.

    Mirrors the orchestrator card's structure (protocolVersion, capabilities,
    defaultInputModes, defaultOutputModes, skills, authentication.schemes)
    so that the agentgateway sidecar treats every peer the same way.
    """
    return {
        "protocolVersion": A2A_PROTOCOL_VERSION,
        "name": name,
        "description": description,
        "url": f"http://localhost:{port}/a2a",
        "preferredTransport": "JSONRPC",
        "version": "3.0.0",
        "provider": {
            "organization": "Apohara",
            "url": "https://apohara.dev",
        },
        "capabilities": A2A_CAPABILITIES,
        "defaultInputModes": ["application/json"],
        "defaultOutputModes": ["application/json"],
        "skills": [
            {
                "id": skill_id,
                "name": skill_name,
                "description": skill_description,
                "inputModes": ["application/json"],
                "outputModes": ["application/json"],
                "examples": [
                    {
                        "input": {"invoice_id": "INV-001", "amount": 500.0},
                        "output": {"verdict": "APPROVED", "framework": framework},
                    }
                ],
            }
        ],
        "authentication": {"schemes": A2A_AUTH_SCHEMES},
        "x-themis": {
            "framework": framework,
            "story": "C-12",
            "sprint": 3,
            "gaps": ["G27"],
        },
    }


def _validate_invoice_shape(invoice: Any) -> dict[str, Any]:
    """Lightweight validation - the real Pydantic/LangGraph pipelines do the rest."""
    if not isinstance(invoice, dict):
        raise ValueError("invoice must be a JSON object")
    required = ("invoice_id", "vendor_name", "amount", "due_date", "line_items")
    missing = [k for k in required if k not in invoice]
    if missing:
        raise ValueError(f"missing fields: {', '.join(missing)}")
    if not isinstance(invoice["amount"], (int, float)):
        raise ValueError("amount must be numeric")
    return invoice


def _jsonrpc_envelope(id_: Any, result: dict[str, Any]) -> dict[str, Any]:
    return {"jsonrpc": "2.0", "id": id_, "result": result}


def _jsonrpc_error(id_: Any, code: int, message: str) -> dict[str, Any]:
    return {
        "jsonrpc": "2.0",
        "id": id_,
        "error": {"code": code, "message": message},
    }


def make_app(*, card: dict[str, Any], verdict_fn: VerdictFn, framework: str) -> Starlette:
    """Wire up the A2A HTTP surface for one peer."""

    async def card_endpoint(_: Request) -> JSONResponse:
        return JSONResponse(card)

    async def healthz(_: Request) -> JSONResponse:
        return JSONResponse(
            {
                "status": "ok",
                "framework": framework,
                "name": card["name"],
                "ts": int(time.time()),
            }
        )

    async def a2a_endpoint(request: Request) -> JSONResponse:
        try:
            payload = await request.json()
        except json.JSONDecodeError as exc:
            return JSONResponse(
                _jsonrpc_error(None, -32700, f"Parse error: {exc}"),
                status_code=400,
            )

        if not isinstance(payload, dict):
            return JSONResponse(
                _jsonrpc_error(None, -32600, "Invalid Request: top-level must be object"),
                status_code=400,
            )

        if payload.get("jsonrpc") != "2.0":
            return JSONResponse(
                _jsonrpc_error(payload.get("id"), -32600, "Invalid Request: jsonrpc must be '2.0'"),
                status_code=400,
            )

        method = payload.get("method")
        if method != "message/send":
            return JSONResponse(
                _jsonrpc_error(payload.get("id"), -32601, f"Method not found: {method}"),
                status_code=400,
            )

        params = payload.get("params") or {}
        message = params.get("message") or {}
        parts = message.get("parts") or []
        invoice_raw: Any = None
        for part in parts:
            if part.get("type") == "data":
                invoice_raw = part.get("data")
                break
            if part.get("type") == "text" and invoice_raw is None:
                try:
                    invoice_raw = json.loads(part["text"])
                except (KeyError, json.JSONDecodeError, TypeError):
                    invoice_raw = part.get("text")

        if invoice_raw is None:
            return JSONResponse(
                _jsonrpc_error(payload.get("id"), -32602, "Invalid params: no invoice data"),
                status_code=400,
            )

        try:
            invoice = _validate_invoice_shape(invoice_raw)
        except ValueError as exc:
            return JSONResponse(
                _jsonrpc_error(payload.get("id"), -32602, f"Invalid params: {exc}"),
                status_code=400,
            )

        try:
            verdict = await verdict_fn(invoice)
        except Exception as exc:  # noqa: BLE001
            log.exception("verdict_fn failed")
            return JSONResponse(
                _jsonrpc_error(payload.get("id"), -32000, f"verdict error: {exc}"),
                status_code=500,
            )

        result = {
            "kind": "message",
            "role": "agent",
            "parts": [{"type": "data", "data": verdict}],
            "metadata": {
                "framework": framework,
                "ts": int(time.time()),
                "agent_card": card["name"],
            },
        }
        return JSONResponse(_jsonrpc_envelope(payload.get("id"), result))

    @asynccontextmanager
    async def lifespan(app: Starlette):
        # Spawn the Band room WebSocket join as a background asyncio task.
        # Best-effort: if the room is unreachable, the HTTP server keeps
        # serving and the A2A discovery path still works.
        task = await spawn_band_task(framework)
        try:
            yield
        finally:
            if task and not task.done():
                task.cancel()

    app = Starlette(
        routes=[
            Route("/.well-known/agent-card.json", card_endpoint, methods=["GET"]),
            Route("/a2a", a2a_endpoint, methods=["POST"]),
            Route("/healthz", healthz, methods=["GET"]),
        ],
        lifespan=lifespan,
    )
    return app


def run_server(
    *,
    app: Starlette,
    host: str,
    port: int,
    log_level: str = "info",
) -> None:
    """Block on the uvicorn server."""
    uvicorn.run(app, host=host, port=port, log_level=log_level, access_log=False)


# ---------------------------------------------------------------------------
# Band room WebSocket join (background task, never blocks the HTTP server).
# ---------------------------------------------------------------------------


async def _band_room_loop(framework: str) -> None:
    """Connect to the Band room via WebSocket and emit a hello message.

    This is a best-effort join. If `BAND_WS_URL` is unset or the room is
    unreachable, we log and exit the task; the HTTP server keeps serving.
    The Rust orchestrator (C-01) discovers peers through A2A
    `/.well-known/agent-card.json`, not Band WS, so a missing room is
    non-fatal for the demo.
    """
    ws_url = os.environ.get("BAND_WS_URL")
    api_key = os.environ.get("BAND_API_KEY", "")
    room = os.environ.get("BAND_ROOM", "themis-3-demo")
    agent_id = os.environ.get(
        "BAND_AGENT_ID", f"themis-peer-{framework.lower()}"
    )

    if not ws_url:
        log.info("[%s] BAND_WS_URL not set; skipping Band room join (HTTP-only mode)", framework)
        return

    headers = [("x-api-key", api_key)] if api_key else []
    log.info("[%s] joining Band room %s at %s", framework, room, ws_url)
    try:
        # The `websockets` library changed the kwarg name for custom
        # headers at v13. We probe both names so the same script works
        # on the pinned SDK (v12) and the current PyPI release (>=13).
        connect_sig = inspect.signature(websockets.connect)
        if "extra_headers" in connect_sig.parameters:
            ws_ctx = websockets.connect(ws_url, extra_headers=headers)
        elif "additional_headers" in connect_sig.parameters:
            ws_ctx = websockets.connect(ws_url, additional_headers=headers)
        else:
            ws_ctx = websockets.connect(ws_url)
        async with ws_ctx as ws:
            hello = {
                "type": "agent_hello",
                "agent_id": agent_id,
                "framework": framework,
                "room": room,
                "ts": int(time.time()),
            }
            await ws.send(json.dumps(hello))
            try:
                async for raw in ws:
                    msg = json.loads(raw)
                    if msg.get("type") == "mention" and msg.get("to") == agent_id:
                        await ws.send(
                            json.dumps(
                                {
                                    "type": "ack",
                                    "agent_id": agent_id,
                                    "mention_id": msg.get("mention_id"),
                                }
                            )
                        )
            except websockets.ConnectionClosed:
                log.info("[%s] Band WS closed", framework)
    except Exception as exc:  # noqa: BLE001
        log.warning("[%s] Band WS join failed: %s", framework, exc)


async def spawn_band_task(framework: str) -> asyncio.Task[None]:
    """Spawn the Band room join as a background asyncio task."""
    return asyncio.create_task(_band_room_loop(framework), name=f"band-{framework}")


# ---------------------------------------------------------------------------
# CLI entry point.
# ---------------------------------------------------------------------------


def cli_main(
    *,
    framework: str,
    name: str,
    description: str,
    skill_id: str,
    skill_name: str,
    skill_description: str,
    verdict_fn: VerdictFn,
    port: int,
    host: str = "0.0.0.0",
) -> None:
    """Standard CLI: build the app, optionally join Band, run uvicorn."""
    logging.basicConfig(
        level=os.environ.get("LOG_LEVEL", "INFO").upper(),
        format="%(asctime)s [%(name)s] %(levelname)s %(message)s",
    )
    card = build_card(
        name=name,
        description=description,
        framework=framework,
        port=port,
        skill_id=skill_id,
        skill_name=skill_name,
        skill_description=skill_description,
    )
    app = make_app(card=card, verdict_fn=verdict_fn, framework=framework)
    log.info("[%s] serving A2A on %s:%d (card=/.well-known/agent-card.json)", framework, host, port)
    run_server(app=app, host=host, port=port, log_level=os.environ.get("UVICORN_LOG", "info"))


# Optional helper for peers that want to verify a remote A2A card (used in tests).
async def fetch_card(base_url: str) -> dict[str, Any]:
    """GET /.well-known/agent-card.json from another A2A peer."""
    async with httpx.AsyncClient(timeout=5.0) as client:
        r = await client.get(f"{base_url.rstrip('/')}/.well-known/agent-card.json")
        r.raise_for_status()
        return r.json()
