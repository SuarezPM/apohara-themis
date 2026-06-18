# THEMIS 3.0 — Video v5 Script (3–5 min, 30 fps)

> **Audience:** lablab.ai "Band of Agents Hackathon" judge.
> **Goal:** prove the sponsor pivot end-to-end (Band + AI/ML API +
> Featherless AI) in under 5 minutes.
> **Updated 2026-06-18** for the THEMIS 3.0 sponsor-pivot PRD (Story QW-5).
> **Recording is Pablo's manual step.** This file is the script —
> record the screen via OBS or `ffmpeg` + `xdotool`, upload to
> YouTube/Vimeo, then paste the URL in `docs/video-v5.md`.

## Recording setup

- Terminal A: `cargo run --release -p themis-orchestrator` (backend on `:8080`)
- Browser A: `https://themis.apohara.dev` (live demo)
- Browser B: `https://app.band.ai/rooms/themis-demo` (Band room, judges' view)
- Terminal B: `curl -s http://localhost:8080/metrics/aiml | jq`
- Terminal C: `curl -s http://localhost:8080/metrics/featherless | jq`
- Captions: OBS / `ffmpeg -f x11grab -r 30 -s 1920x1080 -i :0 ...`

## Shot 0 (0:00–0:20) — TITLE CARD

**Visual:** THEMIS wordmark on navy `#0a0e1a` background, gold
accent `#d4a017`. Tagline: "5 agents · 1 Band room · 3 sponsor
backends · 0 AI-slop". Cut to `docs/cover.svg` for 2 s, then
launch `https://themis.apohara.dev` in Browser A.

**Voiceover:**

> "THEMIS is a 5-agent Rust system for AP invoice fraud
> detection. Three production sponsors power it end-to-end:
> Band for multi-agent coordination, AI/ML API for multimodal
> reasoning, and Featherless AI for open-weight reasoning.
> Here's the proof — recorded live, no mocks."

## Shot 1 (0:20–1:00) — Section 1: live Band room (Band proof)

**Visual:** Split-screen. Left = Browser B
(`https://app.band.ai/rooms/themis-demo`). Right = THEMIS
frontend `/band-live` SSE feed.

Show the 6 connected agents with `@apohara-themis/<name>`
handles: `extractor`, `po_matcher`, `fraud_auditor`,
`gaap_classifier`, `provenance_signer`, `demo_narrator`.

Hit "Review this invoice" on Browser A. Watch the @mention
transcript flow live on both sides.

**Reference:** [`docs/band-room-screenshot.md`](band-room-screenshot.md)
— the canonical Band-room AC9 evidence. The room URL
`https://app.band.ai/rooms/themis-demo` is the judges' view;
the orchestrator's `/band-live` SSE mirrors it locally.

**Voiceover:**

> "Every THEMIS run opens a real Band room. Six agents connect
> over WebSocket to `wss://app.band.ai`. They coordinate by
> `@mention`. The full transcript is the audit trail of every
> Evidence Packet."

## Shot 2 (1:00–1:50) — Section 2: AIML metrics widget live (AI/ML API proof)

**Visual:** THEMIS frontend — "AI/ML API" widget on the
right column. Live ticks: `calls`, `successes`, `avg_latency_ms`,
`p95_latency_ms`, `total_tokens_in`, `total_tokens_out`,
`total_cost_usd`, `model: anthropic/claude-sonnet-4.5`.

Cut to Terminal B. Run:

```
curl -s http://localhost:8080/metrics/aiml | jq
```

Show JSON snapshot:

```json
{
  "calls": 50,
  "successes": 50,
  "success_rate": 1.0,
  "avg_latency_ms": 1240,
  "p95_latency_ms": 2180,
  "total_tokens_in": 2480,
  "total_tokens_out": 395,
  "total_cost_usd": 0.0823,
  "model": "anthropic/claude-sonnet-4.5"
}
```

**Reference:** `crates/themis-orchestrator/tests/aiml_50_real_e2e.rs`
— `fifty_real_calls_to_aimlapi`. Asserts `calls >= 50`,
`successes >= 45`, `total_cost_usd > 0`. The same `AimlApiMetricsHandle`
attached in production is read by the `/metrics/aiml` HTTP handler.

**Voiceover:**

> "AI/ML API is the multimodal backbone — 50+ real calls per
> demo run across Extractor, Fraud Auditor, and GAAP Classifier.
> Claude Sonnet 4.5 via the AIML API gateway. The live widget
> reads the same `AimlApiMetricsHandle` the backend exposes at
> `/metrics/aiml`."

## Shot 3 (1:50–2:40) — Section 3: Featherless metrics widget live (Featherless AI proof)

**Visual:** THEMIS frontend — "Featherless AI" widget.
Live ticks: `calls`, `successes`, `tokens_in`, `tokens_out`,
`cost_usd`, `model: Qwen3-Coder-30B-A3B-Instruct`.

Cut to Terminal C. Run:

```
curl -s http://localhost:8080/metrics/featherless | jq
```

Show JSON snapshot:

```json
{
  "calls": 50,
  "successes": 50,
  "total_tokens_in": 1920,
  "total_tokens_out": 480,
  "total_cost_usd": 0.0014,
  "model": "Qwen/Qwen3-Coder-30B-A3B-Instruct"
}
```

**Reference:** `crates/themis-orchestrator/tests/featherless_50_real_e2e.rs`
— `featherless_50_real_calls_e2e`. Asserts `calls >= 50`,
`successes >= 45`, `model == FRAUD_AUDITOR_FEATHERLESS_MODEL`.

**Voiceover:**

> "Featherless AI is the open-weight backbone — 50+ real
> calls per demo run. Qwen3-Coder-30B-A3B-Instruct handles
> PO matching, regression tests, and shadow reasoning. Same
> pattern: the live counter and the `/metrics/featherless`
> endpoint read the same `FeatherlessMetricsHandle`."

## Shot 4 (2:40–3:20) — Section 4: bench numbers (THEMIS proves itself)

**Visual:** Terminal running the public-bench eval. Cut to the
printed metrics block.

```
=== THEMIS public-bench (InvoiceNet sample 50) ===
TP=25 FP=0 FN=0 TN=25
precision = 1.000
recall    = 1.000  (target >= 0.85)
FPR       = 0.000  (target <= 0.05)
FP_reduction vs baseline = 100.0%  (target >= 20%)
==================================================
```

Then run the AC7 cost summary:

```
cargo test --release --features public-bench \
  -p themis-orchestrator --test public_bench -- --nocapture
```

Show the pass message and the per-agent cost breakdown.

**Reference:** `crates/themis-orchestrator/tests/public_bench.rs`
— `public_bench_meets_targets`. Loads
`fixtures/invoice_net_sample_50.csv` (25 fraud + 25 clean),
asserts `recall >= 0.85`, `FPR <= 0.05`, `FP_reduction >= 20%`.

**Voiceover:**

> "THEMIS proves itself on a balanced 50-row InvoiceNet sample:
> 25 fraud + 25 clean. Recall 100%, FPR 0%, FP reduction vs
> the worst-case baseline is 100%. Run the harness:

```
cargo test --release --features public-bench \
  -p themis-orchestrator --test public_bench -- --nocapture
```

> All assertions pass."

## Shot 5 (3:20–4:30) — Section 5: BAAAR HALT demo

**Visual:** THEMIS frontend. Submit the seeded Wayne
invoice that contains a secret-leak regex hit
(`AKIA[A-Z0-9]{16}` or `BEGIN PRIVATE KEY`).

Watch the BAAAR gate evaluate 5 conditions:
1. `risk_score > 0.85` ✓ (from Featherless Qwen3-Coder-30B)
2. `security_severity == CRITICAL` ✓ (secret-leak regex)
3. `coherence_score < 0.3` (not hit)
4. `debate_rounds >= 5` (not hit)
5. `explicit_halt_requested` (not hit)

Two of five conditions fire → BAAAR HALT.

The whole UI flashes a red `#dc2626` border. A modal
appears: **"WORKFLOW HALTED — secret detected — judge:
download the receipt to verify offline"**. The Band room
broadcasts `@everyone BAAAR HALT fired at <ts>`.

Click "Download PRC" — a signed Evidence Packet arrives
in <2s.

**Reference:** BAAAR HALT story from THEMIS 2.0 (already
in `main`). Determinism asserted by
`crates/themis-orchestrator/tests/baaar_z3_1210.rs`
(10/10 runs halt on a critical-severity input).

**Voiceover:**

> "The BAAAR kill-switch is the wow moment. Five conditions,
> any one fires the halt. In this run, two fire: the
> Featherless Qwen3-Coder-30B Fraud Auditor flagged
> `risk_score = 0.95`, and the secret-leak regex hit a
> hardcoded AWS access key. The UI flashes red. The Band
> room broadcasts the halt. The Evidence Packet downloads
> in under two seconds."

## Shot 6 (4:30–5:00) — CLOSING + URLs

**Visual:** Title card back. URLs in monospace:

- Demo: `https://themis.apohara.dev`
- Repo: `https://github.com/SuarezPM/apohara-themis`
- Band: `https://app.band.ai/rooms/themis-demo`
- Verify: `cargo run --bin themis-verify -- <packet.json>`

**Voiceover:**

> "Built for the Band of Agents Hackathon. Powered by Band,
> AI/ML API, and Featherless AI. MIT licensed. Pablo M. Suarez,
> @SuarezPM. Thank you."

---

## Recording instructions for Pablo

> **Video recording is Pablo's manual step — record screen via OBS
> or similar using this script, upload to YouTube, then paste URL
> in `docs/video-v5.md`.**

```bash
# 1. Source secrets
source ~/.config/apohara/secrets.env

# 2. Start backend (Terminal A)
cargo run --release -p themis-orchestrator

# 3. Start OBS scene capture (1920x1080, 30 fps, mkv or mp4)
obs --scene "THEMIS v5" --startrecording

# 4. Walk through Shots 0-6 in order. Use the demo URL
#    https://themis.apohara.dev for Browser A and
#    https://app.band.ai/rooms/themis-demo for Browser B.

# 5. Stop OBS, post-process to <50 MB MP4 (H.264, CRF 23).

# 6. Upload to YouTube (unlisted is fine for lablab.ai).

# 7. Paste the URL in docs/video-v5.md and reference it from
#    docs/submission-final.md.
```

## Screenshot / clip placeholders

| Shot | Start | Duration | Placeholder | Source of proof |
|---|---|---|---|---|
| 0 | 0:00 | 0:20 | `docs/cover.svg` rendered | `docs/cover.svg` |
| 1 | 0:20 | 0:40 | live Band room split-screen | `docs/band-room-screenshot.md` |
| 2 | 1:00 | 0:50 | AIML metrics widget + `curl /metrics/aiml` | `crates/themis-orchestrator/tests/aiml_50_real_e2e.rs` |
| 3 | 1:50 | 0:50 | Featherless metrics widget + `curl /metrics/featherless` | `crates/themis-orchestrator/tests/featherless_50_real_e2e.rs` |
| 4 | 2:40 | 0:40 | public-bench output | `crates/themis-orchestrator/tests/public_bench.rs` |
| 5 | 3:20 | 1:10 | BAAAR HALT modal + red border | THEMIS 2.0 story (`baaar_z3_1210.rs`) |
| 6 | 4:30 | 0:30 | closing card with 4 URLs | — |

## Acceptance Criteria — QW-5 / AC1

- [ ] YouTube/Vimeo URL recorded in `docs/video-v5.md`
- [ ] Section 1: live `app.band.ai` room visible
- [ ] Section 2: AIML metrics widget visible (live ticks)
- [ ] Section 3: Featherless metrics widget visible (live ticks)
- [ ] Section 4: bench numbers visible (recall/FPR/FP-reduction)
- [ ] Section 5: BAAAR HALT visible (red border + modal)