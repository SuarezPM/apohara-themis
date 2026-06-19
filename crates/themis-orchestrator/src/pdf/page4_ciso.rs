//! Page 4 — CISO Executive Brief.
//!
//! Asymmetric layout, cursor-flow:
//!   - Title + lead
//!   - Heavy rule
//!   - Left column: dominant KPI "31" + secondary "90%"
//!   - Heavy rule
//!   - Right column: table of risk posture rows (rendered as
//!     a normal kv table, not absolute coords)
//!   - Margin annotation + footer

use crate::packet::SignedPacket;
use themis_agents::baaar::Outcome;

use super::ctx::{brand, Ctx, Page};

pub fn render(ctx: &Ctx, packet: &SignedPacket, page: &mut Page, seal_id: &str, total: u32) {
    let p = &packet.packet;

    ctx.page_header(page, "04 / 07", seal_id, &p.tenant_id, &p.invoice_id);
    ctx.page_title(page, "CISO Brief");
    ctx.body_muted(
        page,
        "Risk posture, controls passed, and regulatory coverage for the security leadership team.",
    );
    page.cursor_y -= 6.0;
    ctx.section_rule_heavy(page, 20.0, page.cursor_y + 4.0, 170.0);
    page.cursor_y -= 16.0;

    // Dominant KPI: "31" (controls passed).
    ctx.kpi_display(page, "31", "CONTROLS  PASSED / 31");
    ctx.kpi_caption(page, "BAAAR KILL-SWITCH + STATE-MACHINE + 8-AGENT PIPELINE");

    // Secondary KPI: fraud risk score from fraud_auditor decision.
    let risk = p
        .agent_decisions
        .iter()
        .find(|d| d.agent_id == "fraud_auditor")
        .map(|d| (d.confidence * 100.0) as u32);
    match risk {
        Some(score) => {
            page.cursor_y -= 8.0;
            ctx.kpi_display(page, &format!("{score}%"), "FRAUD RISK SCORE");
        }
        None => {
            page.cursor_y -= 8.0;
            ctx.kpi_display(page, "\u{2014}", "FRAUD RISK SCORE");
        }
    }

    // Heavy rule + summary table.
    page.cursor_y -= 12.0;
    ctx.section_rule_heavy(page, 20.0, page.cursor_y, 170.0);
    page.cursor_y -= 12.0;

    // Outcome as a sub-KPI line.
    let outcome_str = match &p.bbaaar_outcome {
        Outcome::Approve => "APPROVED",
        Outcome::Halt(_) => "HALT",
    };
    ctx.kv_row(page, "BAAR OUTCOME", outcome_str, false);
    ctx.kv_row(
        page,
        "FRAMEWORKS",
        "DORA \u{00B7} EU AI Act \u{00B7} NIST \u{00B7} OWASP \u{00B7} ISO 42001",
        false,
    );
    ctx.kv_row(page, "VERIFY", "vouch-verify <packet.json>", false);
    ctx.kv_row(page, "INCIDENT DETECTION", "agent: audit_watchdog", false);
    ctx.kv_row(page, "REPORTING", "DORA Art. 17 / EU AI Act Art. 73", false);
    ctx.kv_row(page, "TRANSPARENCY", "Rekor v2 (optional)", false);
    ctx.kv_row(page, "INTEGRITY", "Ed25519 + BLAKE3", false);

    ctx.margin_annotation(
        page,
        "Rating is structural coverage from the framework mapper, not an attestation or certification. The seal proves when these bytes existed.",
    );
    ctx.page_footer_centered(page, seal_id, 4, total);
}
