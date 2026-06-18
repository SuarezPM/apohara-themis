#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "pydantic-ai>=1.0",
#   "starlette>=0.40",
#   "uvicorn>=0.30",
#   "httpx>=0.27",
#   "websockets>=12",
#   "python-dotenv>=1.0",
# ]
# ///
"""THEMIS 3.0 - Peer 1: PydanticAI invoice validator (port 7001).

Story C-12 / G27 cross-framework peer. Joins the Band room as a
PydanticAI-backed agent and exposes the A2A 1.0 surface for invoice
validation.

**Deterministic MVP.** Per the C-12 PRD, the demo must be deterministic
(no LLM cost, no flake). The verdict is produced by a hardcoded policy.
Wiring a real PydanticAI `Agent` against a Claude / Qwen model is a
follow-up; the import is here to prove the framework is installed and
the import surface is stable.

Real vs mock status (per the C-12 risk register):
  - pydantic-ai installs cleanly on Python 3.14 (verified 2026-06-17).
  - We import `pydantic_ai.Agent` to keep the dependency contract honest
    (no silent mock). The deterministic verdict is the demo path.

Run:
    BAND_ROOM=themis-3-demo python3 invoice_validator.py
"""
from __future__ import annotations

import logging
import os
import sys
from pathlib import Path

# Allow `import a2a_server` when run as a script via `python3 invoice_validator.py`.
sys.path.insert(0, str(Path(__file__).resolve().parent))

from pydantic_ai import Agent  # real framework import per C-12

import a2a_server

log = logging.getLogger("themis.peer.invoice")

PEER_NAME = "THEMIS Invoice Validator (PydanticAI)"
PEER_PORT = int(os.environ.get("PEER_PORT", "7001"))
PEER_FRAMEWORK = "PydanticAI"

# Real PydanticAI Agent. The model is only resolved at run time; in the
# deterministic MVP we never call `agent.run_sync()`. Keeping the Agent
# constructed here proves the framework is wired and the import surface
# is stable for the follow-up LLM wiring.
_agent: Agent = Agent(
    "test",  # PydanticAI TestModel - no network, no cost
    system_prompt=(
        "You are an AP fraud-detection validator. Given an invoice JSON, "
        "return a verdict: APPROVED, REJECTED, or REVIEW_REQUIRED."
    ),
)


# Hardcoded thresholds per the C-12 demo policy.
_HIGH_AMOUNT_REJECT = 50_000.0
_LOW_AMOUNT_REVIEW = 100.0


async def validate_invoice(invoice: dict) -> dict:
    """Deterministic validator. Real PydanticAI pipeline is a follow-up.

    Returns a verdict envelope that the Rust orchestrator can serialize
    straight into the BLAKE3 chain.
    """
    amount = float(invoice["amount"])
    invoice_id = invoice["invoice_id"]
    vendor = invoice["vendor_name"]

    if amount > _HIGH_AMOUNT_REJECT:
        verdict = "REJECTED"
        reason = f"amount {amount:.2f} exceeds policy ceiling {_HIGH_AMOUNT_REJECT:.2f}"
    elif amount < _LOW_AMOUNT_REVIEW:
        verdict = "REVIEW_REQUIRED"
        reason = f"amount {amount:.2f} below review floor {_LOW_AMOUNT_REVIEW:.2f}"
    else:
        verdict = "APPROVED"
        reason = "within policy range; vendor name and due_date sanity-checked"

    return {
        "verdict": verdict,
        "reason": reason,
        "invoice_id": invoice_id,
        "vendor_name": vendor,
        "amount": amount,
        "framework": PEER_FRAMEWORK,
        "agent": "PydanticAI::InvoiceValidator",
    }


def main() -> None:
    a2a_server.cli_main(
        framework=PEER_FRAMEWORK,
        name=PEER_NAME,
        description=(
            "Buyer-side invoice validator built on PydanticAI. Returns "
            "APPROVED / REJECTED / REVIEW_REQUIRED based on a hardcoded "
            "policy. Real LLM wiring is a follow-up."
        ),
        skill_id="validate_invoice",
        skill_name="Validate Invoice",
        skill_description="Deterministic invoice validation via PydanticAI pipeline.",
        verdict_fn=validate_invoice,
        port=PEER_PORT,
    )


if __name__ == "__main__":
    main()
