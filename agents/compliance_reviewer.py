#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "langgraph>=1.0",
#   "starlette>=0.40",
#   "uvicorn>=0.30",
#   "httpx>=0.27",
#   "websockets>=12",
#   "python-dotenv>=1.0",
# ]
# ///
"""THEMIS 3.0 - Peer 2: LangGraph compliance reviewer (port 7002).

Story C-12 / G27 cross-framework peer. Models a stateful compliance
review as a LangGraph state machine:

    vendor_check -> amount_check -> sanctions_check -> final_verdict

**Deterministic MVP.** The nodes are real LangGraph nodes (so the graph
is genuinely a state machine), but the verdict logic is hardcoded for
the demo. A real LLM-backed review is a follow-up.

Run:
    BAND_ROOM=themis-3-demo python3 compliance_reviewer.py
"""
from __future__ import annotations

import logging
import os
import sys
from pathlib import Path
from typing import TypedDict

sys.path.insert(0, str(Path(__file__).resolve().parent))

from langgraph.graph import END, START, StateGraph  # real framework import

import a2a_server

log = logging.getLogger("themis.peer.compliance")

PEER_NAME = "THEMIS Compliance Reviewer (LangGraph)"
PEER_PORT = int(os.environ.get("PEER_PORT", "7002"))
PEER_FRAMEWORK = "LangGraph"


class ReviewState(TypedDict, total=False):
    """State carried across the compliance pipeline."""
    invoice: dict
    vendor_ok: bool
    amount_ok: bool
    sanctions_ok: bool
    verdict: str
    reason: str
    steps: list[str]


# A small denylist for the demo sanctions screen. A real implementation
# pulls from OFAC / EU consolidated lists.
_SANCTIONS_DENYLIST = {"acme-sanctioned-llc", "bad-actor-inc"}


def vendor_check(state: ReviewState) -> ReviewState:
    """Node 1: vendor name must be present and non-empty."""
    vendor = (state.get("invoice") or {}).get("vendor_name", "")
    ok = bool(vendor) and len(vendor) >= 2
    return {
        "vendor_ok": ok,
        "steps": [*state.get("steps", []), f"vendor_check: {'ok' if ok else 'fail'}"],
    }


def amount_check(state: ReviewState) -> ReviewState:
    """Node 2: amount must be positive and not absurdly large."""
    amount = float((state.get("invoice") or {}).get("amount", 0))
    ok = 0 < amount < 1_000_000
    return {
        "amount_ok": ok,
        "steps": [*state.get("steps", []), f"amount_check: {amount} -> {'ok' if ok else 'fail'}"],
    }


def sanctions_check(state: ReviewState) -> ReviewState:
    """Node 3: vendor must not be on the demo sanctions denylist."""
    vendor = ((state.get("invoice") or {}).get("vendor_name") or "").lower()
    ok = vendor not in _SANCTIONS_DENYLIST
    return {
        "sanctions_ok": ok,
        "steps": [*state.get("steps", []), f"sanctions_check: {'ok' if ok else 'flagged'}"],
    }


def final_verdict(state: ReviewState) -> ReviewState:
    """Node 4: fold the three checks into a single verdict."""
    if not state.get("sanctions_ok", True):
        v, r = "REJECTED", "vendor on sanctions denylist"
    elif not state.get("vendor_ok", True):
        v, r = "REJECTED", "vendor name failed sanity check"
    elif not state.get("amount_ok", True):
        v, r = "REJECTED", "amount out of range"
    else:
        v, r = "APPROVED", "all three compliance checks passed"
    return {
        "verdict": v,
        "reason": r,
        "steps": [*state.get("steps", []), f"final_verdict: {v}"],
    }


def _build_graph():
    """Build and compile the LangGraph state machine."""
    g = StateGraph(ReviewState)
    g.add_node("vendor", vendor_check)
    g.add_node("amount", amount_check)
    g.add_node("sanctions", sanctions_check)
    g.add_node("final", final_verdict)
    g.add_edge(START, "vendor")
    g.add_edge("vendor", "amount")
    g.add_edge("amount", "sanctions")
    g.add_edge("sanctions", "final")
    g.add_edge("final", END)
    return g.compile()


_graph = _build_graph()


async def review_compliance(invoice: dict) -> dict:
    """Run the compliance review as a real LangGraph execution."""
    result = _graph.invoke({"invoice": invoice, "steps": []})
    return {
        "verdict": result.get("verdict", "REVIEW_REQUIRED"),
        "reason": result.get("reason", ""),
        "invoice_id": invoice["invoice_id"],
        "vendor_name": invoice["vendor_name"],
        "amount": float(invoice["amount"]),
        "framework": PEER_FRAMEWORK,
        "agent": "LangGraph::ComplianceReviewer",
        "pipeline_steps": result.get("steps", []),
        "checks": {
            "vendor_ok": result.get("vendor_ok"),
            "amount_ok": result.get("amount_ok"),
            "sanctions_ok": result.get("sanctions_ok"),
        },
    }


def main() -> None:
    a2a_server.cli_main(
        framework=PEER_FRAMEWORK,
        name=PEER_NAME,
        description=(
            "Stateful compliance review built on LangGraph. Pipeline: "
            "vendor_check -> amount_check -> sanctions_check -> final_verdict."
        ),
        skill_id="compliance_review",
        skill_name="Compliance Review",
        skill_description="Run the 4-node LangGraph compliance pipeline on an invoice.",
        verdict_fn=review_compliance,
        port=PEER_PORT,
    )


if __name__ == "__main__":
    main()
