"""THEMIS 3.0 cross-framework Python peers.

This package hosts three independent Python processes that join the THEMIS
Band room alongside the Rust orchestrator:

* invoice_validator.py  - PydanticAI  (port 7001)
* compliance_reviewer.py - LangGraph  (port 7002)
* bench_runner.py       - CrewAI     (port 7003)

All three expose the A2A 1.0 well-known card at `/.well-known/agent-card.json`
and accept JSON-RPC `message/send` at `POST /a2a`. They are intentionally
deterministic (no LLM call in the demo) - the real LLM wiring is a follow-up.

The story is described in the THEMIS 3.0 PRD under C-12 / G27.
"""
