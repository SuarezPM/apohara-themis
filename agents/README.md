# THEMIS 3.0 - Cross-Framework Python Peers (C-12 / G27)

Three independent Python processes that join the THEMIS Band room alongside
the Rust orchestrator, exposing the A2A 1.0 surface on ports 7001, 7002, 7003.

| Port | Script                  | Framework   | Status (2026-06-17)                |
|------|-------------------------|-------------|------------------------------------|
| 7001 | `invoice_validator.py`  | PydanticAI  | real, deterministic MVP            |
| 7002 | `compliance_reviewer.py`| LangGraph   | real state machine, deterministic   |
| 7003 | `bench_runner.py`       | CrewAI      | mock fallback (wheel build blocked) |

The mock fallback per peer is documented in each script's docstring and
returned in the verdict envelope's `mock_reason` / `mock` field, so the
demo UI/log can show it transparently.

## Why mock CrewAI?

`crewai 0.11.2` (latest on PyPI as of 2026-06-17) pins
`langchain-openai<0.0.6` and `langchain<0.2.0`. The pinned
`tiktoken<0.6.0` source wheel fails to build on Python 3.14
(upstream Rust toolchain pins are 3.12). Forcing the install requires
`--no-deps` plus manual incompatible-langchain juggling that breaks
pydantic-ai in the same process.

The C-12 PRD explicitly allows the mock fallback:

> "If `crewai` doesn't install cleanly, fall back to a function that
>  returns a hardcoded verdict."

The interface contract (A2A card shape, JSON-RPC `message/send`, verdict
envelope keys) is identical to the real peer path, so the orchestrator
and the agentgateway sidecar cannot tell the difference.

## Run locally

```bash
cd agents
python3 -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt
python3 invoice_validator.py        # :7001
python3 compliance_reviewer.py      # :7002
python3 bench_runner.py             # :7003
```

Each peer logs `[PydanticAI] serving A2A on 0.0.0.0:7001 (...)` and
attempts a WebSocket join to `BAND_WS_URL` (defaults to
`ws://localhost:8080/ws` for local dev; the join is best-effort and
non-fatal - A2A discovery is the primary join path).

## Run via Docker (profile `peers`)

The `peers` profile is opt-in so the default `docker compose up` stays
lightweight.

```bash
docker compose --profile peers up --build
```

This starts `themis-orchestrator` + `agentgateway` + the 3 peer
services on the internal `themis-net` bridge. The orchestrator
discovers the peers through A2A card fetch (not Band WS), so the
peers do not need port 7001-7003 published to the host.

## Run with the stdlib-only mock

If the heavy dependencies fail to install in a constrained environment
(small CI image, air-gapped runner, Py3.13 image before the wheels
land), use `MockPydanticAIAgent.py` on :7001 as the invoice validator
drop-in. It uses only the Python standard library.

```bash
python3 MockPydanticAIAgent.py             # :7001, MOCK
python3 MockPydanticAIAgent.py --port 7001 # same
```

The mock sets `x-themis.mock: true` in its A2A card and returns
`mock: true` + `mock_reason` in every verdict, so an operator can
spot the fallback at a glance.

## A2A contract

Every peer serves:

* `GET  /.well-known/agent-card.json` - A2A 1.0 Agent Card
* `POST /a2a`                        - JSON-RPC 2.0 `message/send`
* `GET  /healthz`                    - liveness probe

The card mirrors the orchestrator's card (C-01) for
`protocolVersion`, `capabilities`, `defaultInputModes`,
`defaultOutputModes`, `skills`, `authentication.schemes`. The
`x-themis` extension carries story metadata.

The `message/send` params envelope:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "message/send",
  "params": {
    "message": {
      "role": "user",
      "parts": [
        {
          "type": "data",
          "data": {
            "invoice_id": "INV-001",
            "vendor_name": "Acme Corp",
            "amount": 1500.0,
            "due_date": "2026-07-15",
            "line_items": []
          }
        }
      ]
    }
  }
}
```

The verdict envelope is identical across the three peers; the only
field that differs is `framework` and the pipeline metadata
(`pipeline_steps` for LangGraph, `crewai_available` for CrewAI).
