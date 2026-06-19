//! Page 7 — Broker Insurance Brief.
//!
//! Cursor-flow editorial:
//!   - Title + lead
//!   - Dominant KPI "PRE-CLAIM EVIDENCE" + caption
//!   - Heavy rule
//!   - Eligibility conditions as kv table
//!   - Heavy rule
//!   - Coverage summary as kv table
//!   - Margin annotation + footer

use crate::packet::SignedPacket;
use themis_agents::baaar::Outcome;

use super::ctx::{brand, Ctx, Page};

pub fn render(ctx: &Ctx, packet: &SignedPacket, page: &mut Page, seal_id: &str, total: u32) {
    let p = &packet.packet;

    ctx.page_header(page, "07 / 07", seal_id, &p.tenant_id, &p.invoice_id);
    ctx.page_title(page, "Broker Brief");
    ctx.body_muted(
        page,
        "Cyber-liability eligibility. The pre-claim evidence packet is the proof that controls fired.",
    );
    page.cursor_y -= 6.0;
    ctx.section_rule_heavy(page, 20.0, page.cursor_y + 4.0, 170.0);
    page.cursor_y -= 16.0;

    // Dominant KPI: a single editorial statement as a giant number.
    let (verdict, color) = match &p.bbaaar_outcome {
        Outcome::Approve => ("PRE-CLAIM EVIDENCE", brand::GREEN),
        Outcome::Halt(_) => ("INCIDENT DETECTED", brand::RED),
    };
    ctx.kpi_display(page, verdict, "ELIGIBILITY  \u{00B7}  FOR COVERAGE");
    page.set_fill(color);
    ctx.kpi_caption(page, &format!("{verdict}  \u{00B7}  controls fired  \u{00B7}  sealed evidence"));
    page.reset_color();

    // Eligibility conditions.
    page.cursor_y -= 8.0;
    ctx.section_rule_heavy(page, 20.0, page.cursor_y, 170.0);
    page.cursor_y -= 12.0;
    ctx.kv_row(page, "AI-DRIVEN FRAUD LOSS", "covered", false);
    ctx.kv_row(page, "REGULATORY FINE", "reimbursed", false);
    ctx.kv_row(page, "PROOF OF CONTROL", "this PDF", false);

    // Coverage summary.
    page.cursor_y -= 4.0;
    ctx.section_rule_heavy(page, 20.0, page.cursor_y, 170.0);
    page.cursor_y -= 12.0;
    ctx.kv_row(
        page,
        "COVERAGE",
        "AI-driven fraud loss + regulatory fine reimbursement",
        false,
    );
    ctx.kv_row(
        page,
        "ELIGIBILITY",
        "Pre-claim evidence packet (this PDF) is the proof",
        false,
    );
    ctx.kv_row(
        page,
        "FAVORABLE RATING",
        "BAAAR HALT visible  \u{00B7}  EU AI Act Art. 12 satisfied",
        false,
    );

    ctx.margin_annotation(
        page,
        "Coverage eligibility is between the broker and the underwriter. This PDF documents what the system did; the broker decides what it is worth.",
    );
    ctx.page_footer_centered(page, seal_id, 7, total);
}
