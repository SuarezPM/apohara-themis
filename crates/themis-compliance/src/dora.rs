//! DORA (EU Regulation 2022/2554) Art 9/10/17 mapper.

use themis_agents::baaar::Outcome;
use themis_agents::decision::DecisionType;
use themis_orchestrator::packet::EvidencePacket;

use crate::framework::{ComplianceMap, ComplianceMapper, Framework};

/// Maps an Evidence Packet to DORA's 3 sub-articles.
pub struct DoraMapper;

impl ComplianceMapper for DoraMapper {
    fn framework(&self) -> Framework {
        Framework::Dora
    }

    fn map(&self, packet: &EvidencePacket) -> ComplianceMap {
        let mut m = ComplianceMap::new(self.framework(), 3);

        // Art 9 — ICT risk management. The state machine and the
        // BAAAR gate (5 conditions, deterministic thresholds)
        // together constitute the risk-management process.
        let has_risk_management = packet
            .agent_decisions
            .iter()
            .any(|d| matches!(d.decision_type, DecisionType::FraudAssessed | DecisionType::WatchdogAlert));
        if has_risk_management {
            m.add_field(
                "art_9_ict_risk_management",
                serde_json::json!({
                    "mechanism": "BaaarGate 5-condition kill-switch + StateMachine traversal",
                    "populated_from_decisions": packet.agent_decisions.len(),
                }),
            );
        }

        // Art 10 — Incident detection. The Audit Watchdog is the
        // detection agent; its WatchdogAlert decision captures the
        // incident.
        let watchdog_alert = packet
            .agent_decisions
            .iter()
            .find(|d| d.decision_type == DecisionType::WatchdogAlert);
        if let Some(alert) = watchdog_alert {
            m.add_field(
                "art_10_incident_detection",
                serde_json::json!({
                    "agent": "audit_watchdog",
                    "coherence_score": alert.confidence,
                    "reasoning": alert.reasoning,
                }),
            );
        } else {
            m.add_note("no WatchdogAlert decision in chain; Art 10 detection evidence missing");
        }

        // Art 17 — Incident reporting. A HALT outcome is the
        // incident; the Evidence Packet itself is the report.
        if matches!(packet.bbaaar_outcome, Outcome::Halt(_)) {
            m.add_field(
                "art_17_incident_reporting",
                serde_json::json!({
                    "outcome": "halt",
                    "evidence_packet_id": packet.packet_id.to_string(),
                    "tenant_id": packet.tenant_id,
                    "invoice_id": packet.invoice_id,
                }),
            );
        } else {
            m.add_field(
                "art_17_incident_reporting",
                serde_json::json!({
                    "outcome": "no_incident",
                    "note": "no HALT in this run; Art 17 reporting N/A",
                }),
            );
        }

        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use themis_agents::baaar::BaaarReason;
    use themis_agents::decision::AgentDecision;
    use themis_orchestrator::packet::EvidencePacket;
    use themis_agents::decision::DecisionType;

    fn dec(dt: DecisionType, conf: f32) -> AgentDecision {
        AgentDecision {
            agent_id: "x".to_string(),
            tenant_id: "stark".to_string(),
            invoice_id: "inv-001".to_string(),
            decision_type: dt,
            confidence: conf,
            reasoning: "x".to_string(),
            timestamp_ms: 0,
            payload: serde_json::json!({}),
        }
    }

    #[test]
    fn all_three_art_fields_populated_on_clean_packet() {
        let m = DoraMapper.map(&EvidencePacket::new(
            "stark",
            "inv-001",
            vec![
                dec(DecisionType::Extracted, 0.9),
                dec(DecisionType::FraudAssessed, 0.85),
                dec(DecisionType::WatchdogAlert, 0.92),
            ],
            Outcome::Approve,
        ));
        assert_eq!(m.populated, 3);
        assert_eq!(m.total, 3);
    }

    #[test]
    fn halt_outcome_populates_art_17_with_incident_metadata() {
        let m = DoraMapper.map(&EvidencePacket::new(
            "stark",
            "inv-001",
            vec![dec(DecisionType::FraudAssessed, 0.95)],
            Outcome::Halt(BaaarReason::RiskScoreExceeded),
        ));
        let art_17 = m.fields.iter().find(|(n, _)| *n == "art_17_incident_reporting");
        assert!(art_17.is_some());
        let val = &art_17.unwrap().1;
        assert_eq!(val["outcome"], "halt");
    }

    #[test]
    fn missing_watchdog_adds_a_note() {
        let m = DoraMapper.map(&EvidencePacket::new(
            "stark",
            "inv-001",
            vec![dec(DecisionType::Extracted, 0.9)],
            Outcome::Approve,
        ));
        // Art 10 absent, but the mapper added a note.
        assert!(m.notes.iter().any(|n| n.contains("Art 10")));
    }
}
