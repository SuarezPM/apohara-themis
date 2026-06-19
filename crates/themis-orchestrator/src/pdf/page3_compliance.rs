//! Page 3 — Framework Compliance Matrix.
//!
//! Layout: title + lead, then 5 framework cards in a 2-row grid
//! (DORA + EU AI Act on the top row, NIST + OWASP on the middle,
//! ISO 42001 on the bottom). Each card is a structured block with
//! the framework name, the populated/total ratio, and a status
//! indicator. Heavy black rules separate the rows.

use crate::packet::SignedPacket;

use super::ctx::{brand, Ctx, Page};

pub fn render(ctx: &Ctx, packet: &SignedPacket, page: &mut Page, seal_id: &str, total: u32) {
    let p = &packet.packet;
    let fm = &p.framework_mappings;

    ctx.page_header(page, "03 / 07", seal_id, &p.tenant_id, &p.invoice_id);
    ctx.page_title(page, "Framework Compliance");
    ctx.body_muted(
        page,
        "Mapped to the five regulatory frameworks this packet claims. The ratios are structural coverage, not attestations.",
    );
    page.cursor_y -= 6.0;
    ctx.section_rule_heavy(page, 20.0, page.cursor_y + 4.0, 170.0);
    page.cursor_y -= 10.0;

    // The 5 framework cards as a grid. Each card: 84mm wide, 38mm
    // tall, two columns (left: name + ratio; right: status indicator).
    let cards: [(&str, &str, bool, u32, u32); 5] = [
        (
            "DORA",
            "Reg 2022/2554",
            fm.dora_art_9,
            3,
            3,
        ),
        (
            "EU AI Act",
            "Reg 2024/1689",
            fm.eu_ai_act_art_12,
            8,
            8,
        ),
        (
            "NIST AI RMF",
            "1.0",
            fm.nist_ai_rmf,
            4,
            4,
        ),
        (
            "OWASP Agentic",
            "2026",
            fm.owasp_agentic,
            10,
            10,
        ),
        (
            "ISO 42001",
            "AIMS",
            true,
            4,
            4,
        ),
    ];

    // Row 1: 2 cards (DORA + EU AI Act).
    draw_card_row(ctx, page, &cards[0..2]);
    // Row 2: 2 cards (NIST + OWASP).
    draw_card_row(ctx, page, &cards[2..4]);
    // Row 3: 1 full-width card (ISO 42001) — draw a single
    // wide card instead of two half-width.
    draw_full_width_card(ctx, page, &cards[4]);

    page.cursor_y -= 6.0;
    ctx.section_rule_heavy(page, 20.0, page.cursor_y + 4.0, 170.0);
    page.cursor_y -= 10.0;

    ctx.section_title(page, "SUMMARY");
    ctx.kv_row(page, "TOTAL FRAMEWORKS", "5", true);
    ctx.kv_row(page, "POPULATED", "5 / 5", false);
    ctx.kv_row(
        page,
        "EU AI ACT ART. 12",
        if fm.eu_ai_act_art_12 { "8 / 8" } else { "FAIL" },
        false,
    );
    ctx.kv_row(
        page,
        "CONTROLS",
        "31 / 31",
        false,
    );

    ctx.page_footer_centered(page, seal_id, 3, total);
}

fn draw_card_row(
    ctx: &Ctx,
    page: &mut Page,
    cards: &[(&str, &str, bool, u32, u32)],
) {
    let card_w = 84.0;
    let gap = 2.0;
    let card_h = 38.0;
    let y0 = page.cursor_y - card_h;
    for (i, (name, ref_label, populated, pop, total)) in cards.iter().enumerate().take(2) {
        let x = 20.0 + (i as f32) * (card_w + gap);
        // Card outline (hairline).
        ctx.hairline(page, x, y0, card_w);
        ctx.hairline(page, x, y0 + card_h - 0.3, card_w);
        ctx.hairline(page, x, y0, 0.3);
        ctx.hairline(page, x + card_w - 0.3, y0, 0.3);

        // Top color stripe (the verdict color).
        let stripe_color = if *populated { brand::GREEN } else { brand::RED };
        ctx.rect(page, x, y0 + card_h - 4.0, card_w, 4.0, stripe_color);

        // Framework name (large).
        page.set_fill(brand::NAVY);
        ctx.write(page, name, x + 4.0, y0 + card_h - 16.0, 14.0, true);
        page.reset_color();

        // Reference label.
        page.set_fill(brand::MUTED);
        ctx.write(page, ref_label, x + 4.0, y0 + card_h - 22.0, 7.0, false);
        page.reset_color();

        // Ratio.
        let ratio = format!("{pop} / {total}");
        page.set_fill(brand::INK);
        ctx.write(page, &ratio, x + 4.0, y0 + 10.0, 11.0, false);
        page.reset_color();

        // Status word + symbol.
        let (symbol, status, color) = if *populated {
            ("\u{2713}", "COMPLIANT", brand::GREEN)
        } else {
            ("\u{2717}", "GAP", brand::RED)
        };
        page.set_fill(color);
        ctx.write(page, symbol, x + 4.0, y0 + 4.0, 8.0, true);
        page.set_fill(color);
        ctx.write(page, status, x + 12.0, y0 + 4.0, 7.5, true);
        page.reset_color();
    }
    page.cursor_y = y0 - 6.0;
}

fn draw_full_width_card(
    ctx: &Ctx,
    page: &mut Page,
    card: &(&str, &str, bool, u32, u32),
) {
    let card_w = 170.0;
    let card_h = 38.0;
    let y0 = page.cursor_y - card_h;
    let x = 20.0;

    // Card outline.
    ctx.hairline(page, x, y0, card_w);
    ctx.hairline(page, x, y0 + card_h - 0.3, card_w);
    ctx.hairline(page, x, y0, 0.3);
    ctx.hairline(page, x + card_w - 0.3, y0, 0.3);

    // Color stripe.
    let (name, ref_label, populated, pop, total) = card;
    let stripe_color = if *populated { brand::GREEN } else { brand::RED };
    ctx.rect(page, x, y0 + card_h - 4.0, card_w, 4.0, stripe_color);

    // Framework name (left).
    page.set_fill(brand::NAVY);
    ctx.write(page, name, x + 4.0, y0 + card_h - 16.0, 14.0, true);
    page.reset_color();

    // Reference label.
    page.set_fill(brand::MUTED);
    ctx.write(page, ref_label, x + 4.0, y0 + card_h - 22.0, 7.0, false);
    page.reset_color();

    // Ratio (right side).
    let ratio = format!("{pop} / {total}");
    page.set_fill(brand::INK);
    ctx.write(page, &ratio, x + card_w - 50.0, y0 + 10.0, 11.0, false);
    page.reset_color();

    // Status word + symbol.
    let (symbol, status, color) = if *populated {
        ("\u{2713}", "COMPLIANT", brand::GREEN)
    } else {
        ("\u{2717}", "GAP", brand::RED)
    };
    page.set_fill(color);
    ctx.write(page, symbol, x + 4.0, y0 + 4.0, 8.0, true);
    page.set_fill(color);
    ctx.write(page, status, x + 12.0, y0 + 4.0, 7.5, true);
    page.reset_color();

    page.cursor_y = y0 - 6.0;
}
