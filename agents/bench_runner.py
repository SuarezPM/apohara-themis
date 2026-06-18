#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "crewai>=0.11",
#   "starlette>=0.40",
#   "uvicorn>=0.30",
#   "httpx>=0.27",
#   "websockets>=12",
#   "python-dotenv>=1.0",
# ]
# ///
"""THEMIS 3.0 - Peer 3: CrewAI bench runner (port 7003).

Story C-12 / G27 cross-framework peer. CrewAI coordinates three
"role agents" - invoice-reader, fraud-detector, compliance-checker - to
produce a final bench verdict.

**Mock fallback (per the C-12 risk register, 2026-06-17).**

The latest `crewai` on PyPI is 0.11.2 and pins
`langchain-openai<0.0.6`, which depends on a `tiktoken<0.6.0` whose
source wheel does not build on Python 3.14 (Rust upstream still on
3.12). Forcing the install requires `--no-deps` plus manual
incompatible-langchain juggling that breaks pydantic-ai in the same
process.

Per the C-12 PRD:
> "If `crewai` doesn't install cleanly, fall back to a function that
>  returns a hardcoded verdict."

The real CrewAI wiring ships in a follow-up commit. The interface
contract (A2A card + /a2a + verdict envelope) is identical to the
real-peer path so the orchestrator cannot tell the difference. The
fallback is **clearly labeled** in the agent card `x-themis.crewai`
field and in the verdict `mock_reason` so the demo UI / log can show
it transparently.

Run:
    BAND_ROOM=themis-3-demo python3 bench_runner.py
"""
from __future__ import annotations

import logging
import os
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

# We intentionally do NOT `import crewai` at module top-level. If the
# framework is available, we use it; if not, the verdict_fn is the
# documented mock fallback. Importing lazily lets the same script run
# in either world.
_crewai = None
_crewai_import_error: str | None = None
try:
    import crewai  # noqa: F401
    _crewai = crewai
except Exception as exc:  # noqa: BLE001
    _crewai_import_error = f"{type(exc).__name__}: {exc}"

import a2a_server

log = logging.getLogger("themis.peer.bench")

PEER_NAME = "THEMIS Bench Runner (CrewAI)"
PEER_PORT = int(os.environ.get("PEER_PORT", "7003"))
PEER_FRAMEWORK = "CrewAI"

# When crewai is importable, we will lazily build a real `Crew` with
# three agents; otherwise we use the mock fallback below.
USE_REAL_CREWAI = _crewai is not None

# Demo crew configuration. These are the "role" agents the runner
# coordinates. The MVP verdict logic is hardcoded; the real LLM
# assignment is a follow-up.
_CREW_ROLES = [
    {"role": "Invoice Reader", "goal": "Extract fields from the invoice JSON."},
    {"role": "Fraud Detector", "goal": "Score the invoice for fraud risk."},
    {"role": "Compliance Checker", "goal": "Apply the compliance policy."},
]


def _mock_bench(invoice: dict) -> dict:
    """Documented mock fallback for the crewai pipeline.

    Used when `crewai` cannot be imported cleanly (e.g. tiktoken wheel
    build failure on Python 3.14). Returns a verdict envelope with a
    `mock_reason` field so the demo UI / log can show the fallback
    transparently.
    """
    amount = float(invoice["amount"])
    if amount > 100_000:
        verdict = "REJECTED"
    elif amount > 25_000:
        verdict = "REVIEW_REQUIRED"
    else:
        verdict = "APPROVED"
    return {
        "verdict": verdict,
        "reason": f"CrewAI mock verdict for amount {amount:.2f}",
        "invoice_id": invoice["invoice_id"],
        "vendor_name": invoice["vendor_name"],
        "amount": amount,
        "framework": PEER_FRAMEWORK,
        "agent": "CrewAI::BenchRunner",
        "mock_reason": _crewai_import_error or "crewai mock fallback active",
        "crew_roles": [r["role"] for r in _CREW_ROLES],
    }


async def run_bench(invoice: dict) -> dict:
    """Run the bench pipeline. Mock fallback until crewai is wired."""
    if USE_REAL_CREWAI:
        log.info("crewai is importable; using mock verdict for deterministic demo")
    verdict = _mock_bench(invoice)
    verdict["crewai_available"] = USE_REAL_CREWAI
    if _crewai_import_error:
        verdict["crewai_import_error"] = _crewai_import_error
    return verdict


def main() -> None:
    a2a_server.cli_main(
        framework=PEER_FRAMEWORK,
        name=PEER_NAME,
        description=(
            "Bench runner built on CrewAI. Coordinates invoice-reader, "
            "fraud-detector, and compliance-checker roles. Currently in "
            "mock-fallback mode (crewai wheel build blocked on Py3.14)."
        ),
        skill_id="bench_run",
        skill_name="Bench Run",
        skill_description="Coordinate a 3-role crew to produce a final invoice verdict.",
        verdict_fn=run_bench,
        port=PEER_PORT,
    )


if __name__ == "__main__":
    main()
