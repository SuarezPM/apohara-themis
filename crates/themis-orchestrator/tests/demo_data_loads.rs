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
use std::sync::Arc;

use themis_agents::llm::{LlmResponse, MockLlmProvider};
use themis_orchestrator::orchestrator::Orchestrator;
use themis_orchestrator::room::MockBandRoom;
use themis_orchestrator::tenants::TenantRegistry;
use themis_orchestrator::test_support::{
    build_stub_agents, expected_outcome_string, fraud_auditor_payload, DemoInvoice,
};

fn orchestrator_for(fixture: &DemoInvoice) -> Orchestrator {
    let mock_llm: Arc<dyn themis_agents::llm::LlmBackend> = Arc::new(
        MockLlmProvider::new("mock-fixture")
            .with_response(
                &fixture.invoice_id,
                LlmResponse {
                    text: serde_json::to_string(&fixture.extracted).unwrap(),
                    input_tokens: 256,
                    output_tokens: 128,
                    model_id: "mock-fixture".to_string(),
                },
            )
            .with_response("assess_fraud_risk", {
                LlmResponse {
                    text: fraud_auditor_payload(fixture),
                    input_tokens: 256,
                    output_tokens: 64,
                    model_id: "mock-fixture".to_string(),
                }
            })
            .with_default(themis_orchestrator::test_support::stub_default_response(
                "mock-fixture",
            )),
    );

    let agents = build_stub_agents(mock_llm, None);
    let rooms: Arc<dyn themis_orchestrator::room::BandRoom> = MockBandRoom::new().into_arc();
    let tenants = Arc::new(TenantRegistry::with_default_tenants());
    let router = themis_orchestrator::router::LlmBackendRouter::with_default_routing(HashMap::new());
    Orchestrator::new(rooms, agents, router, tenants)
}

fn load_fixture(name: &str) -> DemoInvoice {
    let path = themis_orchestrator::test_support::fixtures_dir().join(name);
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
    assert_eq!(
        sp.packet.bbaaar_outcome,
        themis_agents::baaar::Outcome::Halt(themis_agents::baaar::BaaarReason::RiskScoreExceeded),
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
        themis_agents::baaar::Outcome::Halt(themis_agents::baaar::BaaarReason::RiskScoreExceeded),
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
        themis_agents::baaar::Outcome::Halt(themis_agents::baaar::BaaarReason::SecretLeakDetected),
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
        themis_agents::baaar::Outcome::Halt(themis_agents::baaar::BaaarReason::CoherenceTooLow),
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

// Reference the helper to keep it from being dead-code in case the
// only call site ever changes.
#[allow(dead_code)]
fn _exercise_expected_outcome_string() -> &'static str {
    let f = load_fixture("stark-001.json");
    expected_outcome_string(&f)
}
