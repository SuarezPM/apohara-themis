//! Integration test for US-A4: a sealed packet with a Rekor entry
//! round-trips through serde_json and the `rekor_entry` field is
//! preserved on the wire.
//!
//! This test lives in the orchestrator crate (rather than the
//! evidence crate's own `tests/`) because the orchestrator's
//! `process_invoice_sealed` is the production caller. The unit
//! test for the seal method itself lives in
//! `crates/themis-evidence/src/packet.rs::tests`.

use std::sync::Arc;

use themis_evidence::packet::EvidenceService;
use themis_evidence::rekor::{MockRekorClient, RekorClient};
use themis_evidence::timestamp::{MockTimestampAuthority, TimestampAuthority};
use themis_orchestrator::test_support::{
    build_orchestrator_with_evidence, fixtures_dir, DemoInvoice,
};

fn tsa() -> Arc<dyn TimestampAuthority> {
    Arc::new(MockTimestampAuthority::new("https://mock.tsa.local"))
}

fn load_fixture(name: &str) -> DemoInvoice {
    let path = fixtures_dir().join(name);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {}: {e}", path.display()));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|e| panic!("failed to parse fixture {}: {e}", path.display()))
}

/// Build a 2-tenant evidence-service map (stark + wayne) backed
/// by the per-tenant baked Ed25519 seeds. Mirrors the production
/// binary's wiring in `themis-orchestrator.rs::main`.
fn evidence_map() -> std::collections::HashMap<String, EvidenceService> {
    let mut m = std::collections::HashMap::new();
    for tenant in ["stark", "wayne"] {
        let svc = EvidenceService::for_tenant(tenant, tsa())
            .expect("baked tenant must have a key");
        m.insert(tenant.to_string(), svc);
    }
    m
}

#[tokio::test]
async fn sealed_packet_with_rekor_entry_round_trips_through_json() {
    let mut svc = EvidenceService::from_seed("stark", [0xA1; 32], tsa());
    let rekor = MockRekorClient::new();

    let payload = r#"{"invoice_id":"inv-rekor-1","tenant":"stark","vendor":"ACME","amount_cents":4242}"#;
    let hash_hex = blake3::hash(payload.as_bytes()).to_hex().to_string();
    let entry = rekor.anchor(&hash_hex, "stark").await.unwrap();

    let sealed = svc.seal("inv-rekor-1", payload, Some(entry)).await.unwrap();

    // The field is populated.
    let carried = sealed
        .rekor_entry
        .as_ref()
        .expect("rekor_entry should be Some");
    assert!(!carried.uuid.is_empty());
    assert!(!carried.bundle_url.is_empty());

    // Round-trip through JSON — the field must survive serialization.
    let json = serde_json::to_string(&sealed).expect("serialize");
    assert!(
        json.contains("\"rekor_entry\""),
        "serialized packet must include rekor_entry key (got: {json})"
    );

    let parsed: themis_evidence::packet::SealedPacket =
        serde_json::from_str(&json).expect("parse");
    assert_eq!(parsed.rekor_entry.as_ref().unwrap().uuid, carried.uuid);
    assert_eq!(parsed.rekor_entry.as_ref().unwrap().log_index, carried.log_index);
}

/// US-A5: `process_invoice_sealed` propagates the inner
/// `process_invoice`'s Rekor entry into the `SealedPacket`. With
/// a `MockRekorClient` wired in, the run produces a `SealedPacket`
/// whose `rekor_entry` is `Some`.
#[tokio::test]
async fn process_invoice_sealed_passes_rekor_entry_to_seal() {
    // Use an APPROVED fixture so the run completes (HALT fixtures
    // short-circuit before sealing).
    let f = load_fixture("wayne-001.json");
    let rekor: Arc<dyn RekorClient> = Arc::new(MockRekorClient::new());
    let orch = build_orchestrator_with_evidence(&f, None, Some(rekor), evidence_map());

    let (_signed, sealed) = orch
        .process_invoice_sealed("wayne", "wayne-001", br#"{"vendor":"ACME"}"#.to_vec())
        .await
        .expect("sealed run succeeds with mock rekor + mock tsa");
    let sealed = sealed.expect("orchestrator was built with evidence map");
    assert!(
        sealed.rekor_entry.is_some(),
        "process_invoice_sealed must propagate the inner Rekor entry to the SealedPacket"
    );
}

/// US-A5 graceful degradation: when no Rekor client is wired,
/// `process_invoice_sealed` still completes and the
/// `SealedPacket.rekor_entry` is `None`.
#[tokio::test]
async fn process_invoice_sealed_graceful_degrade_when_anchor_returns_none() {
    let f = load_fixture("wayne-001.json");
    let orch = build_orchestrator_with_evidence(&f, None, None, evidence_map());

    let (_signed, sealed) = orch
        .process_invoice_sealed("wayne", "wayne-001", br#"{"vendor":"ACME"}"#.to_vec())
        .await
        .expect("sealed run must succeed even without a Rekor client");
    let sealed = sealed.expect("orchestrator was built with evidence map");
    assert!(
        sealed.rekor_entry.is_none(),
        "rekor_entry must be None when no Rekor client is configured"
    );
}
