//! Page 6 — General Counsel Legal Brief.
//!
//! Cursor-flow editorial:
//!   - Title + lead
//!   - Reporting triplet (24h/72h/15d) as side-by-side 32pt display
//!     (rendered at fixed x positions because it's a horizontal
//!     triplet, not a flow element)
//!   - Secondary KPI "€15M" (penalty exposure)
//!   - Heavy rule
//!   - Reporting summary as kv table
//!   - Margin annotation + footer

use crate::packet::SignedPacket;

use super::ctx::{brand, Ctx, Page};

pub fn render(ctx: &Ctx, packet: &SignedPacket, page: &mut Page, seal_id: &str, total: u32) {
    let p = &packet.packet;

    ctx.page_header(page, "06 / 07", seal_id, &p.tenant_id, &p.invoice_id);
    ctx.page_title(page, "General Counsel Brief");
    ctx.body_muted(
        page,
        "Reporting timeline and penalty exposure. The sealed log is the proof of timely response.",
    );
    page.cursor_y -= 6.0;
    ctx.section_rule_heavy(page, 20.0, page.cursor_y + 4.0, 170.0);
    page.cursor_y -= 14.0;

    // Reporting triplet (24h / 72h / 15d) — side-by-side 32pt display.
    // Each cell is 56mm wide with 2mm gap.
    let triplet_x = [20.0, 78.0, 136.0];
    let triplet_label = ["CRITICAL", "HIGH", "MEDIUM"];
    let triplet_value = ["24h", "72h", "15d"];
    let triplet_y_label = page.cursor_y;
    let triplet_y_value = page.cursor_y - 32.0;
    for i in 0..3 {
        page.set_fill(brand::MUTED);
        ctx.write(
            page,
            triplet_label[i],
            triplet_x[i],
            triplet_y_label,
            7.0,
            true,
        );
        page.reset_color();
        page.set_fill(brand::INK);
        ctx.write(
            page,
            triplet_value[i],
            triplet_x[i],
            triplet_y_value,
            32.0,
            true,
        );
        page.reset_color();
    }
    page.cursor_y = triplet_y_value - 10.0;
    page.set_fill(brand::MUTED);
    ctx.write(
        page,
        "EU AI Act Art. 73  \u{00B7}  reporting windows",
        20.0,
        page.cursor_y,
        7.0,
        false,
    );
    page.reset_color();
    page.cursor_y -= 14.0;

    // Secondary KPI: penalty exposure.
    page.cursor_y -= 4.0;
    ctx.kpi_display(page, "\u{20AC}15M", "PENALTY EXPOSURE  \u{00B7}  OR 3% GLOBAL TURNOVER");
    ctx.kpi_caption(
        page,
        "whichever is higher  \u{00B7}  per infringement  \u{00B7}  EU AI Act Art. 99",
    );

    // Reporting summary.
    page.cursor_y -= 4.0;
    ctx.section_rule_heavy(page, 20.0, page.cursor_y, 170.0);
    page.cursor_y -= 12.0;
    ctx.kv_row(
        page,
        "DORA Art. 17",
        "ICT-related incident reporting  \u{00B7}  72h window",
        false,
    );
    ctx.kv_row(
        page,
        "EU AI Act Art. 73",
        "Severity-based windows  \u{00B7}  24h / 72h / 15d",
        false,
    );
    ctx.kv_row(
        page,
        "SEALED EVIDENCE",
        "Eliminates the 72h rebuild  \u{00B7}  court-admissible",
        false,
    );

    ctx.margin_annotation(
        page,
        "Penalty figures are statutory maxima, not expected outcomes. The sealed evidence does not immunize against bad acts \u{2014} it proves timely response when one occurs.",
    );
    ctx.page_footer_centered(page, seal_id, 6, total);
}
