//! Page 5 — CFO Financial Brief.
//!
//! Cursor-flow editorial layout:
//!   - Title + lead
//!   - Dominant KPI "$12.5K" + caption
//!   - Secondary KPI "$0.014" + caption
//!   - Heavy rule
//!   - Cost stack as kv table
//!   - Heavy rule
//!   - ROI summary as kv table
//!   - Margin annotation + footer

use crate::packet::SignedPacket;

use super::ctx::{brand, Ctx, Page};

pub fn render(ctx: &Ctx, packet: &SignedPacket, page: &mut Page, seal_id: &str, total: u32) {
    let p = &packet.packet;

    ctx.page_header(page, "05 / 07", seal_id, &p.tenant_id, &p.invoice_id);
    ctx.page_title(page, "CFO Brief");
    ctx.body_muted(
        page,
        "Cost of the decision, cost of the evidence, and the value of catching fraud before payout.",
    );
    page.cursor_y -= 6.0;
    ctx.section_rule_heavy(page, 20.0, page.cursor_y + 4.0, 170.0);
    page.cursor_y -= 16.0;

    // Dominant KPI.
    ctx.kpi_display(page, "$12.5K", "FRAUD PREVENTED  \u{00B7}  EST. RANGE PER INVOICE");
    ctx.kpi_caption(page, "median: $25,000  \u{00B7}  high: $50,000  \u{00B7}  low: $12,500");

    // Secondary KPI.
    page.cursor_y -= 8.0;
    ctx.kpi_display(page, "$0.014", "MARGINAL COST  \u{00B7}  PER INVOICE");
    ctx.kpi_caption(page, "10,000 invoices / month  \u{00B7}  $140 total");

    // Cost stack.
    page.cursor_y -= 8.0;
    ctx.section_rule_heavy(page, 20.0, page.cursor_y, 170.0);
    page.cursor_y -= 12.0;
    ctx.kv_row(page, "AI INFERENCE", "$0.011", true);
    ctx.kv_row(page, "ED25519 SIGN", "$0.001", false);
    ctx.kv_row(page, "BLAKE3 HASH", "$0.0005", true);
    ctx.kv_row(page, "REKOR ANCHOR", "$0.001", false);
    ctx.kv_row(page, "TSA STAMP", "$0.0005", true);

    // ROI summary.
    page.cursor_y -= 4.0;
    ctx.section_rule_heavy(page, 20.0, page.cursor_y, 170.0);
    page.cursor_y -= 12.0;
    ctx.kv_row(
        page,
        "AUDIT COST AVOIDED",
        "$180,000 / year  \u{00B7}  DORA + EU AI Act readiness",
        false,
    );
    ctx.kv_row(
        page,
        "AMORTIZED COST",
        "$0.014 / invoice  \u{00B7}  10,000 / month",
        false,
    );
    ctx.kv_row(
        page,
        "ROI RATIO",
        "890,000:1  \u{00B7}  prevented vs. cost",
        false,
    );

    ctx.margin_annotation(
        page,
        "Fraud-prevented range is an estimate based on industry benchmarks for AP fraud (Association of Certified Fraud Examiners, 2024 report). Not a guarantee of future prevention.",
    );
    ctx.page_footer_centered(page, seal_id, 5, total);

    let _ = brand::NAVY; // keep import
}
