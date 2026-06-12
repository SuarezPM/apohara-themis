# Apohara Themis

> **5-agent Rust system that processes buyer-side AP invoices on Band, detects fraud,
> and emits a signed Evidence Packet verifiable in &lt;30s with `cargo run --bin themis-verify`.**

Built for the **Band of Agents Hackathon** (12-19 jun 2026). Track 3 — Regulated & High-Stakes Workflows
(financial approvals, compliance).

---

## The pitch

Themis processes Accounts Payable invoices through a Band chat room populated by 5 specialized agents.
Each agent does one job, hands off to the next via `@mention`, and the chain is signed end-to-end.
When the final agent (Provenance Signer) seals the evidence packet, the result is a downloadable
artifact that satisfies **DORA Art. 9/10/17**, **EU AI Act Art. 12/26**, **NIST AI RMF** and
**OWASP Agentic 2026** — simultaneously, for two fictitious companies on two trust domains.

```
Invoice PDF (Stanford InvoiceNet)
  ↓
[Band Room per (tenant, invoice)]
  ↓
Extractor         (Claude Fable 5 via AI/ML API)
  → PO Matcher    (Featherless Qwen3-Coder-30B)
  → Fraud Auditor (Claude Fable 5, multimodal PDF)
  → GAAP Classifier (Claude Fable 5)
  → Provenance Signer (Ed25519 + RFC 3161 + Rekor v2)
  ↓
Evidence Packet (PDF + JSON, signed, offline-verifiable)
```

AGON is the Rust Band client (`crates/themis-band-client/`) — a thin subprocess wrapper over the
official Band Python SDK 0.2.11. Internal sub-crate, not a standalone publication.

---

## Stack

- **Runtime:** Rust 1.75+ stable, single Cargo workspace
- **Async:** Tokio + Axum 0.7 (HTTP + WebSocket)
- **Band client:** subprocess over official `band-sdk[langgraph]` 0.2.11
- **LLM:** `rig-core` 0.38 (Anthropic-compatible + OpenAI-compatible for Featherless)
- **Crypto:** `ed25519-dalek` 2 + `blake3` 1 + `rfc3161ng` 0.1
- **Tests:** `cargo test` (built-in)

---

## Workspace layout

```
crates/
  themis-band-client/   # subprocess wrapper around Band Python SDK
  themis-orchestrator/  # room lifecycle, state machine, BAAAR HALT
  themis-agents/        # 5 agent implementations (Extractor, PO Matcher, Fraud Auditor, GAAP Classifier, Provenance Signer)
  themis-evidence/      # Ed25519 + BLAKE3 + RFC 3161 + Rekor v2
  themis-compliance/    # DORA / EU AI Act / NIST AI RMF / OWASP Agentic mappers
  themis-frontend/      # HTML+vanilla JS UI, streams via EventSource
```

---

## Build

```bash
cargo build --release          # ~22 MB single static binary
cargo test                     # unit + integration
cargo clippy --all-targets     # lint
cargo run --bin themis-verify  # offline verification of any evidence packet
```

---

## Sponsors

- **Band** (thenvoi) — coordination substrate for the 5-agent chat room
- **AI/ML API** — Claude Fable 5 for the 3 reasoning-heavy agents
- **Featherless AI** — Qwen3-Coder-30B for the code/structure agent

---

## License

MIT. Pablo M. Suarez ([@SuarezPM](https://github.com/SuarezPM)). See [LICENSE](LICENSE).
