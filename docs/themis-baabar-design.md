# themis-baabar — design document

> Extract THEMIS 3.0's BAAAR deterministic kill-switch as a standalone
> Rust crate. Publishable to crates.io. Compatible with any agent
> framework (LangGraph, CrewAI, Pydantic AI, custom Rust).

| Field | Value |
|---|---|
| Status | Design (post-hackathon work, week 1) |
| Author | Pablo M. Suarez (@SuarezPM) |
| License | MIT |
| Target version | `0.1.0-alpha` |
| Last updated | 2026-06-18 |

## Why extract

The audit of 2026-06-18 identified BAAAR as the **only genuinely
differentiating asset** in THEMIS 3.0. The 5-condition deterministic
gate + SAC integration (arXiv [2605.09076](https://arxiv.org/abs/2605.09076)) is:

- Domain-agnostic (works on any LLM output)
- Reproducible (1210 proptest cases already in THEMIS)
- Critical for agent safety (10/10 deterministic halting in tests)

Extracting it as a crate lets us:

1. Build a community around deterministic agent guardrails
2. Enable LangGraph / CrewAI / AutoGen integrations (the audit's gap)
3. Position `themis-baabar` as **"the Llama-Guard of deterministic gates"** —
   probabilistic classifiers for inputs vs. deterministic guards for outputs

### Adjacent ecosystem (research summary, 2026-06-18)

A scan of crates.io (via EXA web search) shows the guardrails space is
crowded and rapidly consolidating. `themis-baabar` must position clearly:

| Crate | License | Approach | Gap we fill |
|---|---|---|---|
| [`nannyd`](https://crates.io/crates/nannyd) | Open-source | Per-agent budget + tool allowlist + hard kill | No multi-condition `Verdict` enum, no deterministic `5/5` gate |
| [`bastion-ai`](https://crates.io/crates/bastion-ai) | Open-source | Multi-model consensus + SHA-256 audit chain | Consensus needs ≥2 LLMs (cost), not deterministic single-pass |
| [`adk-guardrail`](https://crates.io/crates/adk-guardrail) | Apache-2.0 | PII redaction + content filter + schema | ADK-Rust only; no domain-agnostic gate |
| [`enact-core`](https://crates.io/crates/enact-core) | Open-source | Graph-first agent runtime + `enact-guardrails` | Runtime is the product; guardrails are coupled |
| [`agent-ruler`](https://github.com/steadeepanda/agent-ruler) | MIT | Deterministic reference monitor for local agents | Zones/boundaries model; no 5-condition logic |
| [`portcullis`](https://crates.io/crates/portcullis) | MIT/Apache-2.0 | Quotient-lattice permissions + `Obligations` | Type-system approach; complements our guard |
| [`llm-guard-rs`](https://github.com/marirs/llm-guard-rs) / [`shield`](https://github.com/LLM-Dev-Ops/shield) | Open-source | 22 input/output scanners (rewrite of Python `llm-guard`) | Input scanners, not action-time gates |
| [`coding-guardrails`](https://github.com/stawils/coding-guardrails) | Open-source | Two-layer proxy (Forge + 11 rules) for coding agents | Coding-specific (path safety, secret masking) |
| [`hanzo-guard`](https://crates.io/crates/hanzo-guard) | Open-source | PII/injection detection, sub-ms latency | Detection only; no action gating |

**Our wedge:** THEMIS has the only *deterministic* 5-condition gate
that combines a `Verdict` enum (5 named reasons), a hash-chained audit
trail, and a SAC-based trust controller in a single package. None of
the above ships the full triplet.

### Research gap to flag

The plan referenced "SAC — Secure Agentic Control (arXiv 2605.09076)".
The actual paper at that arXiv ID is **"Robust Multi-Agent LLMs under
Byzantine Faults"** by Haejoon Lee et al. (2026-05-09), which proposes
**SAC = Self-Anchored Consensus** — a decentralized iterative
filter-and-refine protocol with $(F+1)$-robustness conditions on the
communication graph. This is closer to what BAAAR's SAC controller
should implement (trust downgrade + action filtering) than the
"SAC = Secure Agentic Control" framing in the plan. **The plan's
naming is wrong**; this design uses **Self-Anchored Consensus** with
a footnote acknowledging the plan's misattribution. A future pass
should re-search arXiv for the original "Secure Agentic Control"
paper; if it exists at a different ID, update the citation.

## Public API

```rust
pub trait HaltingPolicy: Send + Sync {
    fn evaluate(&self, input: &GateInput) -> Verdict;
    fn name(&self) -> &str;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Approve,
    Halt(HaltReason),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive] // Adding a 6th halt reason is a minor, not major, bump
pub enum HaltReason {
    RiskScoreExceeded { score: f64, threshold: f64 },
    SecretLeakDetected { pattern: String, line: usize },
    CoherenceTooLow { score: f64, threshold: f64 },
    MaxDebateRoundsReached { rounds: u32, max: u32 },
    ExplicitHaltRequested { operator: String },
}

#[derive(Debug, Clone)]
pub struct GateInput {
    pub risk_score: f64,
    pub findings: Vec<String>,
    pub coherence_score: f64,
    pub debate_rounds: u32,
    pub explicit_halt: bool,
    pub raw_llm_output: Option<String>,
}

pub struct BaabarGate<P: HaltingPolicy> {
    policy: P,
    sac_controller: SacController,
    metrics: Arc<BaabarMetrics>,
}

impl<P: HaltingPolicy> BaabarGate<P> {
    pub fn with_policy(policy: P) -> Self;
    pub fn evaluate(&self, input: &GateInput) -> Verdict;
    pub fn verify_determinism(&self, cases: u32) -> DeterminismReport;
    pub fn metrics(&self) -> BaabarMetrics;
}
```

### API design notes

- `Verdict` and `HaltReason` are `#[non_exhaustive]` (per Cargo SemVer
  guidance) so adding a 6th halt condition in v0.2.x is non-breaking.
- `GateInput` is `Clone` (cheap) so callers can retry evaluation under
  different policies without rebuilding the struct.
- `BaabarGate<P>` is **generic over the policy trait**, not a concrete
  type — this is what makes the crate framework-agnostic. LangGraph,
  CrewAI, and AutoGen each implement `HaltingPolicy` against their
  own findings schema.
- `metrics()` returns `Arc<BaabarMetrics>` for cheap clone-and-share
  with observability stacks (Prometheus exporter, OpenTelemetry).

## Default policy (built-in)

```rust
pub struct DefaultPolicy {
    pub risk_threshold: f64,         // default 0.85
    pub coherence_threshold: f64,    // default 0.3
    pub max_debate_rounds: u32,      // default 5
    pub secret_patterns: Vec<Regex>,
}

impl Default for DefaultPolicy {
    fn default() -> Self {
        Self {
            risk_threshold: 0.85,
            coherence_threshold: 0.3,
            max_debate_rounds: 5,
            secret_patterns: vec![
                Regex::new(r"(?i)aws_secret_access_key").unwrap(),
                Regex::new(r"(?i)api[_-]?key\s*[:=]").unwrap(),
                Regex::new(r"-----BEGIN (RSA |EC )?PRIVATE KEY-----").unwrap(),
            ],
        }
    }
}

impl HaltingPolicy for DefaultPolicy { /* short-circuit OR of all 5 conditions */ }
```

## SAC controller (arXiv 2605.09076)

> Note: the original plan called this "Secure Agentic Control". The
> actual paper at arXiv 2605.09076 is Lee et al., "Robust Multi-Agent
> LLMs under Byzantine Faults", which proposes **Self-Anchored
> Consensus (SAC)** — a $(F+1)$-robust decentralized filter. We use
> that definition; see the research-gap note above.

```rust
pub struct SacController {
    /// Current trust level. Downgraded on every Halt.
    pub trust_level: TrustLevel,
    /// Allowed actions per trust level (HashMap keeps the example simple).
    pub allowed_actions: HashMap<TrustLevel, ActionSet>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrustLevel { High, Medium, Low }

impl SacController {
    /// Downgrade trust on a halt event.
    pub fn downgrade(&mut self, reason: HaltReason);
    /// Check whether an action is allowed at the current trust level.
    pub fn can_perform(&self, action: &Action) -> bool;
}
```

The SAC controller integrates with the gate: every `Verdict::Halt`
triggers a `downgrade()`, and every `evaluate()` consults
`can_perform()` before producing a verdict. This is what makes the
crate *deterministic + Byzantine-aware*: the $(F+1)$-robustness
condition from arXiv 2605.09076 §3 means we tolerate up to $F$
Byzantine (lying) agents in a peer set of size $N \geq F+1$.

## Crate manifest (Cargo.toml for `themis-baabar`)

```toml
[package]
name = "themis-baabar"
version = "0.1.0-alpha"
edition = "2021"
rust-version = "1.75"           # MSRV (matches THEMIS workspace)
license = "MIT"
description = "Deterministic 5-condition kill-switch for AI agents. SAC paper integration."
repository = "https://github.com/SuarezPM/themis-baabar"
readme = "README.md"
keywords = ["agent", "guardrails", "kill-switch", "deterministic", "llm-safety"]
categories = ["api-bindings", "asynchronous", "no-std"]
include = ["src/**/*.rs", "Cargo.toml", "README.md", "LICENSE", "CHANGELOG.md"]

[dependencies]
ed25519-dalek = "2"
blake3 = "1"
regex = "1"
tokio = { version = "1", features = ["full"], optional = true }

[dev-dependencies]
proptest = "1"
criterion = "0.5"

[features]
default = []
async = ["tokio"]
serde = ["dep:serde"]

[[example]]
name = "baabar_cli"
path = "examples/baabar_cli.rs"
```

### Manifest notes

- `rust-version = "1.75"` matches the THEMIS workspace MSRV; raising
  it is a minor bump per Cargo SemVer guidance.
- `categories = ["no-std"]` is aspirational (the SAC controller uses
  `HashMap`; a future v0.2 release swaps to `hashbrown` for no_std).
- `serde` is feature-gated so non-Rust consumers (WASM via
  `wasm-bindgen`, Python via PyO3) can opt out.
- `include = [...]` prevents `target/` and `.git/` from being
  accidentally published (cargo does this by default, but explicit
  is safer).

## Tests (already in THEMIS)

- **1210 proptest cases** cover the 5 conditions + edge cases. All 1210
  must pass before publishing `0.1.0-alpha`.
- **Determinism test**: `verify_determinism(10_000)` runs the gate
  against a seeded input set and asserts byte-identical output.
- **Compliance test**: every `Verdict::Halt` reason must produce a
  corresponding Sec 4 audit log entry (the hash chain references it).
- **Snapshot test** (`insta`): the JSON shape of a halt event is
  frozen so EU AI Act Art. 12 field-set coverage can be diffed in CI.

### Test matrix

| Test | Count | Wall-clock target |
|---|---|---|
| 5-condition proptest (each reason) | 1210 | <5s |
| Determinism (10k seeded runs) | 1 | <30s |
| SAC controller downgrade + can_perform | 50 | <1s |
| Hash chain audit (compliance) | 1 | <2s |
| Snapshot (insta) | 1 | <1s |

Total: <30s, well within the 70% test budget rule.

## Example binary (`baabar-cli`)

```rust
// examples/baabar_cli.rs
use themis_baabar::{BaabarGate, DefaultPolicy, GateInput};

fn main() {
    let policy = DefaultPolicy::default();
    let gate = BaabarGate::with_policy(policy);
    let input = GateInput {
        risk_score: 0.92,
        findings: vec!["price_gouge".to_string()],
        coherence_score: 0.8,
        debate_rounds: 1,
        explicit_halt: false,
        raw_llm_output: None,
    };
    let verdict = gate.evaluate(&input);
    println!("Verdict: {:?}", verdict);
}
```

Run: `cargo run --example baabar_cli`

## crates.io publication checklist

- [ ] MIT license file (compatible with THEMIS repo)
- [ ] API documented (rustdoc for every public item; `#![warn(missing_docs)]`)
- [ ] Example binary (`baabar-cli`) in `examples/`
- [ ] 1210 proptest cases pass + determinism + SAC tests
- [ ] README with quick-start + architecture diagram
- [ ] CHANGELOG.md (Keep a Changelog format, semver)
- [ ] GitHub Actions CI (`fmt + clippy + test` on `ubuntu-latest`,
      `macos-latest`, `windows-latest`)
- [ ] Git tag `v0.1.0-alpha` (annotated)
- [ ] `cargo publish --dry-run` clean (no warnings)
- [ ] Marketing post draft: "BAAAR vs Llama-Guard: deterministic vs
      probabilistic"

## Timeline (post-hackathon)

| Week | Milestone |
|---|---|
| 1 | Extract crate from THEMIS, polish API, publish `v0.1.0-alpha` |
| 2 | SAC controller refinement (use the real SAC paper definition); write 4 example integrations |
| 3 | Benchmark on standard halting datasets (HAL, HaluEval) |
| 4 | Write "BAAAR vs Llama-Guard" comparison blog post |
| 5 | Outreach to LangGraph + CrewAI + AutoGen maintainers |
| 6 | `v0.2.0` stable, with Python bindings via PyO3 |

## What does NOT go in the crate

- THEMIS-specific types (`Invoice`, `EvidencePacket`, the 5 BAAAR
  conditions are made generic via the `HaltingPolicy` trait; the
  invoice fraud domain lives in a THEMIS wrapper crate).
- Band WebSocket client (that is `themis-band-client`'s domain).
- Per-tenant signing (the crate is single-tenant; multi-tenancy is a
  downstream wrapper crate).
- LLM provider adapters (rig-core, Anthropic SDK, Featherless) —
  those are dependency choices the consumer makes.

## Out of scope (hackathon deadline 2026-06-19)

- Actual extraction (this doc only; code stays in THEMIS for now).
- crates.io account creation (Pablo has one already, name
  `themis-baabar` to be reserved before extraction).
- Marketing site (`themis-baabar.dev` is reserved but not built).
- The "Secure Agentic Control" citation correction (a separate
  research task — the arXiv search for the original paper name).

## References

1. Lee, H. et al. **"Robust Multi-Agent LLMs under Byzantine Faults"**.
   arXiv [2605.09076](https://arxiv.org/abs/2605.09076), 2026-05-09.
   Introduces **Self-Anchored Consensus (SAC)** with $(F+1)$-robustness
   on the communication graph.
2. EXA web search results, 2026-06-18:
   - [`nannyd`](https://crates.io/crates/nannyd) — adjacent deterministic-kill-switch crate.
   - [`bastion-ai`](https://crates.io/crates/bastion-ai) — consensus + SHA-256 audit chains.
   - [`adk-guardrail`](https://crates.io/crates/adk-guardrail) — ADK-Rust guardrails framework.
   - [`enact-core`](https://crates.io/crates/enact-core) — graph-first agent runtime.
   - [`agent-ruler`](https://github.com/steadeepanda/agent-ruler) — deterministic reference monitor.
   - [`portcullis`](https://crates.io/crates/portcullis) — quotient-lattice permissions.
   - [`hanzo-guard`](https://crates.io/crates/hanzo-guard) — LLM I/O sanitization.
   - [`llm-guard-rs`](https://github.com/marirs/llm-guard-rs) / [`shield`](https://github.com/LLM-Dev-Ops/shield) — Rust rewrites of llm-guard.
   - [`coding-guardrails`](https://github.com/stawils/coding-guardrails) — local coding-agent proxy.
   - [The Cargo Book — Publishing on crates.io](https://doc.rust-lang.org/cargo/reference/publishing.html).
   - [The Cargo Book — SemVer Compatibility](https://doc.rust-lang.org/cargo/reference/semver.html).
3. THEMIS 3.0 audit, 2026-06-18 (referenced from the kickoff plan).
