//! Page 1 — Statement Cover.
//!
//! Layout (asymmetric, editorial):
//!   - Top quarter: page header (numerator + meta) + identifiers as
//!     a 2-col kv table
//!   - Bottom 60%: solid color block (green for APPROVED, red for
//!     HALT, amber for REVIEW) with the verdict in 72pt display type
//!   - QR code in the top-right corner of the cover, in a hairline
//!     border

use crate::packet::SignedPacket;
use themis_agents::baaar::Outcome;

use super::ctx::{brand, Ctx, Page};
use super::page2_ledger::render_qr_png;

pub fn render(
    ctx: &Ctx,
    packet: &SignedPacket,
    page: &mut Page,
    seal_id: &str,
    total: u32,
) -> Result<(), super::PdfError> {
    let p = &packet.packet;
    let (verdict, color, label) = match p.bbaaar_outcome {
        Outcome::Approve => (
            "APPROVED",
            brand::GREEN,
            "ALL 5 BAAAR CONDITIONS PASSED",
        ),
        Outcome::Halt(_) => ("HALT", brand::RED, "KILL-SWITCH TRIGGERED"),
    };

    // Header.
    ctx.page_header(
        page,
        "01 / 07",
        seal_id,
        &p.tenant_id,
        &p.invoice_id,
    );

    // Eyebrow over the identifiers.
    page.set_fill(brand::MUTED);
    ctx.write(
        page,
        "APOHARA VOUCH \u{00B7} EVIDENCE PACKET",
        20.0,
        page.cursor_y,
        7.0,
        true,
    );
    page.reset_color();
    page.cursor_y -= 6.0;

    // Sub-eyebrow: the dataset.
    page.set_fill(brand::MUTED);
    ctx.write(
        page,
        "stanford-invoicenet-50 \u{00B7} BAAAR kill-switch \u{00B7} EU AI Act Art. 12 \u{00B7} DORA Art. 17",
        20.0,
        page.cursor_y,
        7.0,
        false,
    );
    page.reset_color();
    page.cursor_y -= 14.0;

    // Heavy black rule.
    ctx.section_rule_heavy(page, 20.0, page.cursor_y + 6.0, 170.0);
    page.cursor_y -= 8.0;

    // Identifiers as a 2-col table (right-aligned values for the IDs).
    ctx.kv_row(page, "TENANT", &p.tenant_id.to_uppercase(), false);
    ctx.kv_row(page, "INVOICE", &p.invoice_id, false);
    ctx.kv_row(page, "PACKET ID", &p.packet_id.to_string(), true);
    ctx.kv_row(
        page,
        "GENERATED",
        &format!("{} ms", p.generated_at_ms),
        true,
    );
    ctx.kv_row(
        page,
        "POLICY",
        "apohara-vouch-1",
        false,
    );

    page.cursor_y -= 4.0;

    // Statement hero (the big color block with the verdict).
    ctx.statement_hero(page, verdict, color, label);

    // QR code in the top-right corner of the cover, in a hairline
    // border (so it doesn't compete with the verdict block).
    let qr_x = 158.0;
    let qr_y = 240.0;
    let qr_size = 32.0;
    ctx.hairline(page, qr_x - 1.5, qr_y - 1.5, qr_size + 3.0);
    render_qr_png(ctx, page, packet, qr_x, qr_y, qr_size);

    // QR caption.
    page.set_fill(brand::MUTED);
    ctx.write(page, "SCAN TO VERIFY", qr_x, qr_y - 5.0, 6.5, true);
    page.reset_color();

    // No page footer on the cover — the hero IS the footer.
    let _ = total;
    Ok(())
}
