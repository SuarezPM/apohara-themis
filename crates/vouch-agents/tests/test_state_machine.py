"""AC-1.6 mock test: the state machine runs end-to-end and emits
exactly one `thought` event per state (10 states = 10 emits).

We instantiate the compiled LangGraph state machine with a mock
``FakeAgentTools`` attached, run the initial state through, and
assert:
  * final state == 'DONE'
  * 10 transitions were recorded (one per node)
  * every transition has ``message_type='thought'``
  * the EVIDENCE transition includes the sealed packet
"""

from __future__ import annotations

import asyncio
import os
import sys
from pathlib import Path

import pytest

# Make `vouch_agents` importable when run from the repo root
THIS_DIR = Path(__file__).resolve().parent
SRC_DIR = THIS_DIR.parent / "src"
if str(SRC_DIR) not in sys.path:
    sys.path.insert(0, str(SRC_DIR))

# Ensure the in-process orchestrator skips the real /seal call.
os.environ.setdefault("VOUCH_SEAL_URL", "http://127.0.0.1:1/seal")  # unreachable

from orchestrator import (  # noqa: E402
    ORCHESTRATOR_STATES,
    OrchestratorState,
    compile_state_machine,
    build_state_graph,
    register_tools,
)


def test_state_machine_has_all_nine_states() -> None:
    """AC-1.2: every state IDLE..DONE is registered as a node."""
    g = build_state_graph()
    nodes = set(g.nodes.keys())
    expected = set(ORCHESTRATOR_STATES)
    assert expected.issubset(nodes), f"missing nodes: {expected - nodes}"


def test_state_machine_has_start_and_end_edges() -> None:
    g = build_state_graph()
    compiled = g.compile()
    graph = compiled.get_graph()
    nodes = {n for n in graph.nodes.keys()}
    edges = [(e.source, e.target) for e in graph.edges]
    start_targets = {v for (src, v) in edges if src == "__start__"}
    end_sources = {src for (src, v) in edges if v == "__end__"}
    assert "IDLE" in nodes, nodes
    assert "DONE" in nodes, nodes
    assert "IDLE" in start_targets, edges
    assert "DONE" in end_sources, edges


@pytest.mark.asyncio
async def test_state_machine_runs_to_done_with_mocked_tools() -> None:
    """AC-1.6: end-to-end run produces 10 transitions and DONE state."""
    from band.testing.fake_tools import FakeAgentTools  # type: ignore

    tools = FakeAgentTools(
        room_id="vouch-procurement-court",
        peers=[
            {"id": "5f710470-bcd6-428f-878c-e652664b0761", "handle": "@apohara-themis/extractor"},
            {"id": "05af1262-5431-4595-bca4-c42ff78ea471", "handle": "@apohara-themis/po-matcher"},
        ],
    )
    case_id = "case-test-001"
    register_tools(case_id, tools)
    sm = compile_state_machine()
    initial: OrchestratorState = {
        "state": "IDLE",
        "case_id": case_id,
        "tenant_id": "stark",
        "procurement_request": "wire $50,000 to vendor X for invoice #42",
        "transitions": [],
    }
    result = await sm.ainvoke(initial, {"configurable": {"thread_id": "t-1"}})

    assert result["state"] == "DONE", result
    transitions = result.get("transitions", [])
    # 11 nodes = 11 transitions (one thought emit per node).
    # S-07b added COMPLIANCE_ESCALATION between REDTEAM and EVIDENCE.
    assert len(transitions) == 11, f"expected 11, got {len(transitions)}: {transitions}"

    # Each transition should have come from one of the 11 states.
    seen = [t.get("from") for t in transitions]
    assert seen == list(ORCHESTRATOR_STATES), seen


@pytest.mark.asyncio
async def test_state_machine_emits_thought_events_via_fake_tools() -> None:
    """AC-1.3: every state transition emits thenvoi_send_event(thought)."""
    from band.testing.fake_tools import FakeAgentTools  # type: ignore

    tools = FakeAgentTools(room_id="vouch-procurement-court")
    case_id = "case-test-002"
    register_tools(case_id, tools)
    sm = compile_state_machine()
    initial: OrchestratorState = {
        "state": "IDLE",
        "case_id": case_id,
        "tenant_id": "wayne",
        "procurement_request": "approve vendor onboarding",
        "transitions": [],
    }
    await sm.ainvoke(initial, {"configurable": {"thread_id": "t-2"}})

    # The fake records every send_event call.
    # S-07b added COMPLIANCE_ESCALATION — total now 11.
    assert len(tools.events_sent) == 11, tools.events_sent
    for evt in tools.events_sent:
        assert evt["message_type"] == "thought", evt
        assert evt["content"], evt


@pytest.mark.asyncio
async def test_intake_recruits_specialists_via_tools() -> None:
    """INTAKE state must call lookup_peers + add_participant for each handle."""
    from band.testing.fake_tools import FakeAgentTools  # type: ignore

    tools = FakeAgentTools(room_id="vouch-procurement-court")
    case_id = "case-test-003"
    register_tools(case_id, tools)
    sm = compile_state_machine()
    initial: OrchestratorState = {
        "state": "IDLE",
        "case_id": case_id,
        "tenant_id": "stark",
        "procurement_request": "test intake",
        "transitions": [],
    }
    result = await sm.ainvoke(initial, {"configurable": {"thread_id": "t-3"}})

    # 8 specialists are listed in the shared chatroom: block.
    assert len(tools.participants_added) >= 1, tools.participants_added
    assert len(result.get("recruited_agents", [])) == len(
        tools.participants_added
    ), result.get("recruited_agents")


@pytest.mark.asyncio
async def test_evidence_state_attempts_seal_call() -> None:
    """EVIDENCE -> DECISION: the orchestrator POSTs to /seal (best-effort)."""
    from band.testing.fake_tools import FakeAgentTools  # type: ignore

    tools = FakeAgentTools(room_id="vouch-procurement-court")
    case_id = "case-test-004"
    register_tools(case_id, tools)
    sm = compile_state_machine()
    initial: OrchestratorState = {
        "state": "IDLE",
        "case_id": case_id,
        "tenant_id": "stark",
        "procurement_request": "test seal",
        "transitions": [],
    }
    result = await sm.ainvoke(initial, {"configurable": {"thread_id": "t-4"}})

    sealed = result.get("sealed_packet", {})
    # We don't require a successful HTTP call (no server in tests).
    # We DO require that the EVIDENCE node populated sealed_packet
    # with at least an 'error' key (or a real hash) and that the
    # decision memo was built from it.
    assert sealed, "EVIDENCE did not populate sealed_packet"
    assert "decision_memo" in result, "DECISION did not build memo"
    memo = result["decision_memo"]
    if "error" not in sealed:
        # If the seal server is up, the memo must include hash+sig.
        assert sealed.get("hash"), sealed
        assert "hash" in memo, memo
