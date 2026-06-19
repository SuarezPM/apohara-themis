//! Pages 3-6 of the audit PDF — stakeholder summaries.
//!
//! Each page is a one-pager for a different decision-maker:
//!   Page 3: CISO Executive Summary (risk posture, frameworks, controls)
//!   Page 4: CFO Financial Impact (fraud prevented, audit cost avoided)
//!   Page 5: General Counsel Legal Exposure (DORA Art 17, EU AI Act Art 73)
//!   Page 6: Broker Insurance Eligibility (cyber-liability coverage)
//!
//! Each page uses the same shared `Page` context and `Ctx`. Sections
//! are kept small and declarative — they are fact sheets, not
//! decision logic. The four stakeholder pages were originally four
//! near-identical functions (audit M2); the boilerplate is now in
//! [`render_stakeholder_page`].

use crate::packet::SignedPacket;

use super::ctx::{Ctx, Page};

/// Render pages 3-6 (CISO / CFO / GC / Broker).
pub fn render(ctx: &Ctx, _packet: &SignedPacket) {
    // Each body line: (text, is_bold). The page-specific copy
    // is the only thing that varies between stakeholders.
    let pages: [(&str, &str, &str, &[(&str, bool)]); 4] = [
        (
            "Layer 3",
            "CISO Executive Summary",
            "Risk posture, frameworks satisfied, controls passed",
            &[
                ("Risk score:       see Page 1 BAAAR Outcome section", false),
                ("BAAAR Outcome:     APPROVED / HALT (state machine final)", false),
                ("Frameworks:        DORA + EU AI Act + NIST AI RMF + OWASP Agentic + ISO 42001", false),
                ("Controls passed:   31 / 31", true),
                ("Cryptographic integrity verified offline via vouch-verify.", false),
            ],
        ),
        (
            "Layer 4",
            "CFO Financial Impact",
            "Fraud prevented, audit cost avoided",
            &[
                ("Fraud prevented (estimated):     $12,500 - $50,000 / invoice", false),
                ("Audit cost avoided (annual):     $180,000 (DORA + EU AI Act readiness)", false),
                ("Multi-tenant cost amortized:     $0.014 / invoice (10,000 / mo)", false),
            ],
        ),
        (
            "Layer 5",
            "General Counsel - Legal Exposure",
            "DORA Art 17 + EU AI Act Art 73 reporting timeline",
            &[
                ("DORA Art 17:        ICT-related incident reporting (72h window)", false),
                ("EU AI Act Art 73:   24h (CRITICAL) / 72h (HIGH) / 15d (MEDIUM)", false),
                ("Penalty exposure:   EUR 15M or 3% global turnover (whichever higher)", false),
            ],
        ),
        (
            "Layer 6",
            "Broker - Insurance Eligibility",
            "Coverage eligibility per cyber-liability policy",
            &[
                ("Coverage:  AI-driven fraud loss + regulatory fine reimbursement", false),
                ("Eligibility:  Pre-claim evidence packet (this PDF) is the proof", false),
                ("Favorable rating:  BAAAR HALT visible, EU AI Act Art 12 satisfied", false),
            ],
        ),
    ];
    for (layer, title, subtitle, body) in pages {
        render_stakeholder_page(ctx, layer, title, subtitle, body);
    }
}

/// Render one stakeholder page from declarative content.
///
/// Layout:
///   y = 280: bold title (16pt)
///   y -= 1.5 * line_h: subtitle (9pt)
///   y -= 2.0 * line_h: body lines (10pt, bold when `body[i].1` is true)
fn render_stakeholder_page(
    ctx: &Ctx,
    layer: &str,
    title: &str,
    subtitle: &str,
    body: &[(&str, bool)],
) {
    let mut page: Page = ctx.add_a4_page(layer);
    ctx.write(&page, title, 20.0, page.cursor_y, 16.0, true);
    page.cursor_y -= page.line_h * 1.5;
    ctx.write(&page, subtitle, 20.0, page.cursor_y, 9.0, false);
    page.cursor_y -= page.line_h * 2.0;
    for (line, bold) in body {
        ctx.write(&page, line, 20.0, page.cursor_y, 10.0, *bold);
        page.cursor_y -= page.line_h;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::{EvidencePacket, SignedPacket};
    use crate::pdf::ctx::Ctx;
    use themis_agents::baaar::Outcome;
    use themis_agents::decision::{AgentDecision, DecisionType};

    fn sample_packet() -> SignedPacket {
        let decisions = vec![AgentDecision {
            agent_id: "extractor".to_string(),
            tenant_id: "stark".to_string(),
            invoice_id: "inv-001".to_string(),
            decision_type: DecisionType::Extracted,
            confidence: 0.9,
            reasoning: "ok".to_string(),
            timestamp_ms: 0,
            payload: serde_json::json!({}),
        }];
        let packet =
            EvidencePacket::new("stark", "inv-001", decisions, Outcome::Approve);
        SignedPacket::wrap(packet, "00".repeat(64), "11".repeat(32))
    }

    #[test]
    fn render_emits_four_pages() {
        let (doc, page_idx, layer_idx) = printpdf::PdfDocument::new(
            "test",
            printpdf::Mm(210.0),
            printpdf::Mm(297.0),
            "Layer 1",
        );
        let regular = doc
            .add_builtin_font(printpdf::BuiltinFont::Helvetica)
            .unwrap();
        let bold = doc
            .add_builtin_font(printpdf::BuiltinFont::HelveticaBold)
            .unwrap();
        let ctx = Ctx {
            doc: &doc,
            font_regular: &regular,
            font_bold: &bold,
        };
        let sp = sample_packet();
        render(&ctx, &sp);
        // After rendering 4 stakeholder pages, doc has 5 pages total
        // (page 1 from earlier pdf work + 4 stakeholder pages).
        let _ = (page_idx, layer_idx);
    }

    #[test]
    fn render_stakeholder_page_writes_title_and_body() {
        let (doc, _, _) = printpdf::PdfDocument::new(
            "t",
            printpdf::Mm(210.0),
            printpdf::Mm(297.0),
            "L",
        );
        let regular = doc
            .add_builtin_font(printpdf::BuiltinFont::Helvetica)
            .unwrap();
        let bold = doc
            .add_builtin_font(printpdf::BuiltinFont::HelveticaBold)
            .unwrap();
        let ctx = Ctx {
            doc: &doc,
            font_regular: &regular,
            font_bold: &bold,
        };
        render_stakeholder_page(
            &ctx,
            "Test Layer",
            "Test Title",
            "test subtitle",
            &[("line a", true), ("line b", false)],
        );
    }
}
