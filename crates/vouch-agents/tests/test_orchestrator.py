"""AC-1.5, AC-1.7, AC-1.8 tests for the Orchestrator class + secrets + config."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

import pytest

THIS_DIR = Path(__file__).resolve().parent
SRC_DIR = THIS_DIR.parent / "src"
if str(SRC_DIR) not in sys.path:
    sys.path.insert(0, str(SRC_DIR))

from orchestrator import (  # noqa: E402
    build_chat_completions_llm,
    load_agent_config,
    load_chatroom,
    load_secrets,
    make_graph_factory,
    Orchestrator,
)


ORCHESTRATOR_UUID = "c963ea72-65fb-4388-ad8f-75dfd0043250"


def test_agent_config_yaml_is_git_ignored() -> None:
    """AC-1.4: `git check-ignore` returns a path for agent_config.yaml."""
    repo = Path(__file__).resolve().parents[3]
    result = subprocess.run(
        [
            "git",
            "check-ignore",
            "crates/themis-band-client/agent-config/agent_config.yaml",
        ],
        cwd=repo,
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, (
        f"git check-ignore returned {result.returncode}, "
        f"stderr={result.stderr!r} stdout={result.stdout!r}"
    )
    assert "agent_config.yaml" in result.stdout, result.stdout


def test_load_agent_config_returns_orchestrator_uuid() -> None:
    """AC-1.5: load_agent_config('themis-orchestrator') returns c963ea72..."""
    rec = load_agent_config("themis-orchestrator")
    assert rec["agent_id"] == ORCHESTRATOR_UUID, rec
    assert rec["api_key"], rec
    assert rec["api_key"].startswith("band_"), rec
    assert "orchestrator" in rec["handle"], rec


def test_load_chatroom_has_shared_block() -> None:
    """AC-1.8: the chatroom: block is shared (not per-agent)."""
    chatroom = load_chatroom()
    assert chatroom["slug"] == "vouch-procurement-court", chatroom
    assert chatroom["chatroom_id"], chatroom
    assert len(chatroom["participants"]) == 8, chatroom["participants"]


def test_load_secrets_reads_aiml_keys() -> None:
    """AC-1.7: secrets come from secrets.env, never from source."""
    secrets = load_secrets()
    assert "AIML_API_KEY" in secrets
    assert "AIML_API_BASE_URL" in secrets
    assert "FEATHERLESS_API_KEY" in secrets
    assert "FEATHERLESS_API_BASE_URL" in secrets
    # At least one is non-empty (in CI / dev the AIML key is set).
    assert any(secrets[k] for k in secrets), secrets


def test_load_secrets_does_not_include_source_hardcoded_keys() -> None:
    """AC-1.7: no hardcoded keys in orchestrator.py source.

    The orchestrator source MUST NOT contain any value starting with
    ``band_a_`` or any AIML-style API key. We assert by reading the
    source and grepping for those patterns.
    """
    src = (SRC_DIR / "orchestrator.py").read_text(encoding="utf-8")
    assert "band_a_" not in src, "hardcoded band api key in source"
    # AIML keys are hex; the real one is 32 chars. We just assert no
    # long hex-only string assignment to a key variable.
    bad_patterns = [
        'AIML_API_KEY = "',
        "AIML_API_KEY = '",
    ]
    for pat in bad_patterns:
        assert pat not in src, f"hardcoded key pattern {pat!r} in source"


def test_build_chat_completions_llm_uses_aiml_base() -> None:
    """AC-1.1: build_chat_completions_llm returns a ChatOpenAI aimed at AIML."""
    llm = build_chat_completions_llm(
        secrets={
            "AIML_API_KEY": "test-key",
            "AIML_API_BASE_URL": "https://api.aimlapi.com/v1",
            "FEATHERLESS_API_KEY": "",
            "FEATHERLESS_API_BASE_URL": "",
        }
    )
    # ChatOpenAI stores the kwargs on `openai_api_key`, `openai_api_base`,
    # `model_name`. We assert the model name and base URL.
    assert "gpt-5.4" in llm.model_name, llm.model_name
    assert "aimlapi.com" in str(llm.openai_api_base), llm.openai_api_base


def test_orchestrator_class_constructs_with_real_config() -> None:
    """The Orchestrator class wires the state machine and loads config."""
    orch = Orchestrator()
    assert orch.config["agent_id"] == ORCHESTRATOR_UUID
    assert orch.chatroom["slug"] == "vouch-procurement-court"
    assert orch.llm is not None
    assert orch.state_machine is not None


def test_make_graph_factory_returns_callable() -> None:
    """AC-1.1: graph_factory builds a state machine for LangGraphAdapter."""
    factory = make_graph_factory(
        secrets={
            "AIML_API_KEY": "test",
            "AIML_API_BASE_URL": "https://api.aimlapi.com/v1",
            "FEATHERLESS_API_KEY": "",
            "FEATHERLESS_API_BASE_URL": "",
        }
    )
    assert callable(factory)
    # The factory takes a list of tools; we pass an empty list and
    # confirm it returns a compiled Pregel.
    from langgraph.pregel import Pregel

    sm = factory([])
    assert isinstance(sm, Pregel)
