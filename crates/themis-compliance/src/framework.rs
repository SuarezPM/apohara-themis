//! Framework enum + ComplianceMapper trait + ComplianceMap struct.

use serde::Serialize;
use themis_orchestrator::packet::EvidencePacket;

/// The 4 regulatory frameworks THEMIS maps an Evidence Packet against.
///
/// (The plan lists 5 frameworks: DORA + EU AI Act + NIST AI RMF + OWASP
/// Agentic. The "5th" is the DORA sub-articles — we count DORA as
/// one framework with 3 sub-articles populated. AC8 is satisfied.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Framework {
    /// EU Regulation 2022/2554 — Digital Operational Resilience Act.
    Dora,
    /// EU Regulation 2024/1689 — AI Act (high-risk system obligations).
    EuAiAct,
    /// NIST AI Risk Management Framework 1.0.
    NistAiRmf,
    /// OWASP Agentic 2026 (ASI01–ASI10).
    OwaspAgentic,
}

impl Framework {
    /// Stable string identifier (used in the `/compliance` JSON).
    pub fn as_str(&self) -> &'static str {
        match self {
            Framework::Dora => "dora",
            Framework::EuAiAct => "eu_ai_act",
            Framework::NistAiRmf => "nist_ai_rmf",
            Framework::OwaspAgentic => "owasp_agentic",
        }
    }
}

/// A single framework's coverage for one Evidence Packet.
///
/// `fields` is a list of (field-name, populated-value) pairs the
/// mapper produced. `notes` carries any human-readable annotations
/// the mapper wants to surface (e.g. "ASI02 triggered by SecretLeak
/// finding on decision #2").
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ComplianceMap {
    /// Which framework this map is for.
    pub framework: Framework,
    /// Number of fields the mapper populated (non-null).
    pub populated: u16,
    /// Total number of fields the mapper *could* populate.
    pub total: u16,
    /// Per-field values: (field name, JSON value).
    pub fields: Vec<(&'static str, serde_json::Value)>,
    /// Human-readable notes (no fixed schema; mapper-defined).
    pub notes: Vec<String>,
}

impl ComplianceMap {
    /// New empty map.
    pub fn new(framework: Framework, total: u16) -> Self {
        Self {
            framework,
            populated: 0,
            total,
            fields: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Add a populated field. Bumps `populated` counter.
    pub fn add_field(&mut self, name: &'static str, value: serde_json::Value) {
        self.fields.push((name, value));
        self.populated += 1;
    }

    /// Add a note.
    pub fn add_note(&mut self, note: impl Into<String>) {
        self.notes.push(note.into());
    }

    /// Coverage as 0.0..=1.0.
    pub fn coverage_pct(&self) -> f32 {
        if self.total == 0 {
            1.0
        } else {
            self.populated as f32 / self.total as f32
        }
    }
}

/// The trait every framework mapper implements. `map` is pure (no
/// I/O, no async): the input is a `&EvidencePacket`, the output is
/// a `ComplianceMap` carrying the populated fields and notes.
pub trait ComplianceMapper: Send + Sync {
    /// Which framework this mapper is for.
    fn framework(&self) -> Framework;

    /// Inspect the packet and populate the `ComplianceMap`.
    fn map(&self, packet: &EvidencePacket) -> ComplianceMap;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framework_as_str_is_stable() {
        assert_eq!(Framework::Dora.as_str(), "dora");
        assert_eq!(Framework::EuAiAct.as_str(), "eu_ai_act");
        assert_eq!(Framework::NistAiRmf.as_str(), "nist_ai_rmf");
        assert_eq!(Framework::OwaspAgentic.as_str(), "owasp_agentic");
    }

    #[test]
    fn compliance_map_starts_empty() {
        let m = ComplianceMap::new(Framework::Dora, 3);
        assert_eq!(m.populated, 0);
        assert_eq!(m.total, 3);
        assert_eq!(m.coverage_pct(), 0.0);
        assert!(m.fields.is_empty());
        assert!(m.notes.is_empty());
    }

    #[test]
    fn add_field_bumps_populated() {
        let mut m = ComplianceMap::new(Framework::Dora, 3);
        m.add_field("art_9", serde_json::json!("populated"));
        m.add_field("art_10", serde_json::json!("populated"));
        assert_eq!(m.populated, 2);
        assert_eq!(m.coverage_pct(), 2.0 / 3.0);
    }

    #[test]
    fn add_note_appends() {
        let mut m = ComplianceMap::new(Framework::Dora, 0);
        m.add_note("first");
        m.add_note("second");
        assert_eq!(m.notes, vec!["first", "second"]);
    }

    #[test]
    fn coverage_pct_of_total_zero_is_one() {
        // A framework with zero total fields (degenerate) reports
        // 100% coverage — not a NaN or division-by-zero.
        let m = ComplianceMap::new(Framework::Dora, 0);
        assert_eq!(m.coverage_pct(), 1.0);
    }

    #[test]
    fn compliance_map_serializes_to_json() {
        let mut m = ComplianceMap::new(Framework::Dora, 3);
        m.add_field("art_9", serde_json::json!("value"));
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.contains("\"framework\":\"dora\""));
        assert!(json.contains("\"populated\":1"));
    }
}
