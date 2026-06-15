US-05 measurement gate — compressor wire feasibility
====================================================

Attempted to wire themis_compressor::compress_text into the
extractor's raw_invoice pipeline. The wire is mechanically
trivial (one call site, ~3 lines of glue), but the measurement
gate fails on substantive grounds:

1. themis_compressor::compress_text is a TEXT compressor
   (LLMLingua-2 port — word-level keep/drop, per the original
   ACL 2024 paper's design).
2. The Extractor agent receives AgentContext::raw_invoice as
   `Vec<u8>` with mime `application/octet-stream`. Real demo
   fixtures are PDF invoices (binary). compress_text expects
   `&str` and the LLMLingua-2 algorithm operates on word
   tokens, which requires text input.
3. Wiring compress_text on a UTF-8 lossy decode of a PDF
   would (a) corrupt the input (binary → UTF-8 lossy → LLM
   with bad tokens) and (b) produce near-zero token reduction
   because the input is structured bytes, not natural language.
4. The token-economy win that the project memory cites (-27.9%
   in the RCT) was on AGENT PROMPTS, not on raw invoice bytes.
   The agent prompt for the Extractor is small (the LLM
   request envelope, not the raw invoice) and the
   compression budget there is also small.

Decision: skip the wire. themis-compressor stays as a
crate, fully built and tested, but not integrated into the
demo path. This is the documented fallback per
.omc/plans/ralplan-repair.md §Decision 1.

For post-hackathon wiring, the right surface is the LLM
request envelope (compress the system prompt + recent
context before each LlmBackend::complete call). That
requires either:
  (a) wrapping LlmBackend with a CompressionBackend that
      intercepts LlmRequest, OR
  (b) adding a `compress: bool` field to LlmRequest that
      the orchestrator sets per-agent.

Both require LlmBackend trait changes — out of scope for
1-day sprint per the plan's P5 (no new architecture).

Outcome: US-05 closes with the crate isolated (same state
as pre-US-05) but with a recorded decision. The narrative
in the demo shifts from "custom compression" to "Rust port
of LLMLingua-2 staged for follow-up integration; current
cost-economy via prompt caching". The crate is recoverable
via git for the post-hackathon wiring sprint.
