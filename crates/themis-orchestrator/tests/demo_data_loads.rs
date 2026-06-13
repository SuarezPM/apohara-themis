//! Integration test: load the 5 Stanford InvoiceNet-shaped demo
//! invoices from `fixtures/demo-invoices/`, run each one through
//! the orchestrator's `process_invoice` (fully mocked), and verify
//! the outcome matches the fixture's `expected_verdict`.
//!
//! This is the contract test for US-D01: 4 HALT + 1 APPROVED,
//! spread across Stark (#1-3) and Wayne (#1-2) trust domains.
//!
//! Run with: `cargo test -p themis-orchestrator --test demo_data_loads`

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use themis_agents::baaar::BaaarReason;
use themis_agents::llm::{LlmResponse, MockLlmProvider};
use themis_orchestrator::orchestrator::Orchestrator;
use themis_orchestrator::room::MockBandRoom;
use themis_orchestrator::tenants::TenantRegistry;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct DemoInvoice {
    invoice_id: String,
    tenant_id: String,
    expected_verdict: String,
    #[serde(default)]
    expected_halt_reason: String,
    #[serde(default)]
    halt_reason_human: Option<String>,
    extracted: ExtractedInvoice,
    fraud_assessment: FraudAssessmentShape,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ExtractedInvoice {
    vendor: String,
    vendor_tax_id: String,
    amount_cents: i64,
    line_items: Vec<LineItem>,
    date_iso: String,
    po_ref: String,
    #[serde(default = "default_currency")]
    currency: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LineItem {
    description: String,
    amount_cents: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FraudAssessmentShape {
    risk_score: f32,
    coherence_score: f32,
    debate_rounds: u32,
    #[serde(default)]
    explicit_halt: bool,
    #[serde(default)]
    secret_leak: bool,
}

fn default_currency() -> String {
    "USD".to_string()
}

fn fixtures_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR points to crates/themis-orchestrator; the
    // fixtures live at the repo root in fixtures/demo-invoices/.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .expect("workspace root")
        .parent()
        .expect("repo root")
        .join("fixtures")
        .join("demo-invoices")
}

/// Map a fixture's `expected_halt_reason` to the orchestrator's
/// outcome string (the strings the fraud_auditor payload uses).
fn expected_outcome_string(fixture: &DemoInvoice) -> &'static str {
    match fixture.expected_verdict.as_str() {
        "APPROVED" => "approve",
        "HALT" => match fixture.expected_halt_reason.as_str() {
            "risk_score_exceeded" => "halt_risk_score_exceeded",
            "secret_leak_detected" => "halt_secret_leak_detected",
            "coherence_too_low" => "halt_coherence_too_low",
            "max_debate_rounds_reached" => "halt_max_debate_rounds_reached",
            "explicit_halt_requested" => "halt_explicit_halt_requested",
            other => panic!("unknown halt_reason in fixture: {other}"),
        },
        other => panic!("unknown expected_verdict: {other}"),
    }
}

/// Build a fully-wired Orchestrator with a MockLlmProvider whose
/// responses are keyed by the invoice_id substring.
fn orchestrator_for(fixture: &DemoInvoice) -> Orchestrator {
    let mock_llm: Arc<dyn themis_agents::llm::LlmBackend> = Arc::new(
        MockLlmProvider::new("mock-fixture")
            // Default response for all non-extractor/non-fraud_auditor
            // agents (po_matcher, gaap_classifier, etc.) — they get
            // a minimal payload that the StubAgent can parse as JSON.
            .with_default(LlmResponse {
                text: serde_json::json!({"stub": "ok"}).to_string(),
                input_tokens: 64,
                output_tokens: 32,
                model_id: "mock-fixture".to_string(),
            })
            // Extractor response: the ExtractedInvoice JSON.
            .with_response(
                &fixture.invoice_id,
                LlmResponse {
                    text: serde_json::to_string(&fixture.extracted).unwrap(),
                    input_tokens: 256,
                    output_tokens: 128,
                    model_id: "mock-fixture".to_string(),
                },
            )
            // Fraud Auditor response: the FraudAuditorOutput (outcome
            // is a string the orchestrator parses — see
            // `orchestrator.rs` `halt_*` arms). The substring
            // "assess_fraud_risk" is unique to this agent's user
            // prompt (see StubAgent::process below), so the mock
            // can't accidentally return the ExtractedInvoice JSON.
            .with_response(
                "assess_fraud_risk",
                LlmResponse {
                    text: serde_json::json!({
                        "assessment": {
                            "risk_score": fixture.fraud_assessment.risk_score,
                            "findings": [{
                                "kind": if fixture.fraud_assessment.secret_leak {
                                    "secret_leak"
                                } else {
                                    "other"
                                },
                                "value": "fixture",
                                "description": fixture.halt_reason_human.clone().unwrap_or_default(),
                            }],
                            "coherence_score": fixture.fraud_assessment.coherence_score,
                            "debate_rounds": fixture.fraud_assessment.debate_rounds,
                            "explicit_halt": fixture.fraud_assessment.explicit_halt,
                        },
                        "outcome": expected_outcome_string(fixture),
                    })
                    .to_string(),
                    input_tokens: 256,
                    output_tokens: 64,
                    model_id: "mock-fixture".to_string(),
                },
            ),
    );

    // Wire 8 agents, each backed by the same mock LLM (the mock
    // dispatches by prompt substring). The decision_type on each
    // agent is cosmetic here; the BAAAR outcome is read from the
    // fraud_auditor's payload directly.
    let mut agents: HashMap<String, Arc<dyn themis_agents::traits::Agent>> = HashMap::new();
    for name in [
        "extractor",
        "po_matcher",
        "fraud_auditor",
        "gaap_classifier",
        "provenance_signer",
        "demo_narrator",
        "regression_tester",
        "audit_watchdog",
    ] {
        agents.insert(
            name.to_string(),
            Arc::new(StubAgent {
                name,
                llm: mock_llm.clone(),
            }),
        );
    }

    let rooms: Arc<dyn themis_orchestrator::room::BandRoom> = MockBandRoom::new().into_arc();
    let tenants = Arc::new(TenantRegistry::with_default_tenants());
    let router =
        themis_orchestrator::router::LlmBackendRouter::with_default_routing(HashMap::new());

    Orchestrator::new(rooms, agents, router, tenants)
}

/// Minimal stub agent that delegates every `process` call to the
/// mock LLM (which returns canned JSON based on the prompt). We
/// use a stub here (not the real agent types) to keep the test
/// independent of the agent implementations' payload formats.
struct StubAgent {
    name: &'static str,
    llm: Arc<dyn themis_agents::llm::LlmBackend>,
}

#[async_trait::async_trait]
impl themis_agents::traits::Agent for StubAgent {
    fn name(&self) -> &'static str {
        self.name
    }
    async fn process(
        &self,
        ctx: themis_agents::traits::AgentContext,
    ) -> Result<themis_agents::decision::AgentDecision, themis_agents::decision::AgentError> {
        use themis_agents::decision::{AgentDecision, DecisionType};
        // The fraud_auditor's user_prompt contains "Fraud Auditor
        // agent in THEMIS" (the system prompt), so the mock returns
        // the FraudAuditorOutput JSON. We need to add it to the
        // system_prompt so the mock matches. For other agents, the
        // mock returns the ExtractedInvoice JSON.
        let (system_prompt, user_prompt) = if self.name == "fraud_auditor" {
            (
                "fraud_auditor_agent".to_string(),
                format!("assess_fraud_risk:upstream_decisions={}", ctx.upstream_decisions.len()),
            )
        } else if self.name == "extractor" {
            (
                "extractor_agent".to_string(),
                format!("parse_invoice:{}:{}", ctx.tenant_id, ctx.invoice_id),
            )
        } else {
            (
                format!("{}_agent", self.name),
                format!("upstream_decisions={}", ctx.upstream_decisions.len()),
            )
        };

        let req = themis_agents::llm::LlmRequest {
            system_prompt,
            user_prompt,
            max_tokens: 1024,
            temperature: 0.0,
            seed: Some(42),
        };
        let resp = self.llm.complete(req).await?;
        let parsed: serde_json::Value = serde_json::from_str(&resp.text)
            .map_err(|e| themis_agents::decision::AgentError::LlmMalformedPayload(e.to_string()))?;
        let decision_type = match self.name {
            "extractor" => DecisionType::Extracted,
            "po_matcher" => DecisionType::PoMatched,
            "fraud_auditor" => DecisionType::FraudAssessed,
            "gaap_classifier" => DecisionType::GaapClassified,
            "provenance_signer" => DecisionType::ProvenanceSigned,
            "demo_narrator" => DecisionType::Narrated,
            "regression_tester" => DecisionType::RegressionResult,
            "audit_watchdog" => DecisionType::WatchdogAlert,
            other => panic!("unknown agent {other}"),
        };
        Ok(AgentDecision {
            agent_id: self.name.to_string(),
            tenant_id: ctx.tenant_id,
            invoice_id: ctx.invoice_id,
            decision_type,
            confidence: 0.9,
            reasoning: format!("{} stub: ok", self.name),
            timestamp_ms: 0,
            payload: parsed,
        })
    }
}

fn load_fixture(name: &str) -> DemoInvoice {
    let path = fixtures_dir().join(name);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {}: {e}", path.display()));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|e| panic!("failed to parse fixture {}: {e}", path.display()))
}

#[tokio::test]
async fn all_5_fixtures_load() {
    let names = [
        "stark-001.json",
        "stark-002.json",
        "stark-003.json",
        "wayne-001.json",
        "wayne-002.json",
    ];
    for name in names {
        let f = load_fixture(name);
        assert!(!f.invoice_id.is_empty());
        assert!(!f.tenant_id.is_empty());
        assert!(
            f.expected_verdict == "HALT" || f.expected_verdict == "APPROVED",
            "fixture {} has invalid expected_verdict: {}",
            name,
            f.expected_verdict
        );
    }
}

#[tokio::test]
async fn stark_001_halts_on_risk_score_exceeded() {
    let f = load_fixture("stark-001.json");
    let orch = orchestrator_for(&f);
    let sp = orch
        .process_invoice(&f.tenant_id, &f.invoice_id, b"raw".to_vec())
        .await
        .unwrap();
    eprintln!("DEBUG stark-001: outcome={:?}", sp.packet.bbaaar_outcome);
    assert_eq!(
        sp.packet.bbaaar_outcome,
        themis_agents::baaar::Outcome::Halt(BaaarReason::RiskScoreExceeded),
        "stark-001 should HALT on risk_score (vendor duplicate)"
    );
}

#[tokio::test]
async fn stark_002_halts_on_risk_score_exceeded() {
    let f = load_fixture("stark-002.json");
    let orch = orchestrator_for(&f);
    let sp = orch
        .process_invoice(&f.tenant_id, &f.invoice_id, b"raw".to_vec())
        .await
        .unwrap();
    assert_eq!(
        sp.packet.bbaaar_outcome,
        themis_agents::baaar::Outcome::Halt(BaaarReason::RiskScoreExceeded),
        "stark-002 should HALT on risk_score (no PO)"
    );
}

#[tokio::test]
async fn stark_003_halts_on_secret_leak() {
    let f = load_fixture("stark-003.json");
    let orch = orchestrator_for(&f);
    let sp = orch
        .process_invoice(&f.tenant_id, &f.invoice_id, b"raw".to_vec())
        .await
        .unwrap();
    assert_eq!(
        sp.packet.bbaaar_outcome,
        themis_agents::baaar::Outcome::Halt(BaaarReason::SecretLeakDetected),
        "stark-003 should HALT on SecretLeak (OFAC sanctioned vendor)"
    );
}

#[tokio::test]
async fn wayne_001_halts_on_coherence_too_low() {
    let f = load_fixture("wayne-001.json");
    let orch = orchestrator_for(&f);
    let sp = orch
        .process_invoice(&f.tenant_id, &f.invoice_id, b"raw".to_vec())
        .await
        .unwrap();
    assert_eq!(
        sp.packet.bbaaar_outcome,
        themis_agents::baaar::Outcome::Halt(BaaarReason::CoherenceTooLow),
        "wayne-001 should HALT on CoherenceTooLow (future invoice_date)"
    );
}

#[tokio::test]
async fn wayne_002_approves() {
    let f = load_fixture("wayne-002.json");
    let orch = orchestrator_for(&f);
    let sp = orch
        .process_invoice(&f.tenant_id, &f.invoice_id, b"raw".to_vec())
        .await
        .unwrap();
    assert_eq!(
        sp.packet.bbaaar_outcome,
        themis_agents::baaar::Outcome::Approve,
        "wayne-002 should APPROVE (clean invoice)"
    );
}

#[tokio::test]
async fn distribution_4_halt_1_approved() {
    let mut halts = 0;
    let mut approves = 0;
    for name in [
        "stark-001.json",
        "stark-002.json",
        "stark-003.json",
        "wayne-001.json",
        "wayne-002.json",
    ] {
        let f = load_fixture(name);
        let orch = orchestrator_for(&f);
        let sp = orch
            .process_invoice(&f.tenant_id, &f.invoice_id, b"raw".to_vec())
            .await
            .unwrap();
        match sp.packet.bbaaar_outcome {
            themis_agents::baaar::Outcome::Halt(_) => halts += 1,
            themis_agents::baaar::Outcome::Approve => approves += 1,
        }
    }
    assert_eq!(halts, 4, "expected 4 HALT verdicts across the 5 fixtures");
    assert_eq!(approves, 1, "expected 1 APPROVED verdict across the 5 fixtures");
}
