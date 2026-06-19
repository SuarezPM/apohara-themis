//! PDF rendering of a `SignedPacket`.
//!
//! Hallmark · macrostructure: Editorial Audit Brief · tone: editorial-audit
//! · anchor hue: warm-navy · theme: Atelier (warm paper)
//!
//! 7 pages, each with a distinct visual rhythm:
//!   1. Cover — Statement hero (verdict in 72pt color block, identifiers left)
//!   2. Ledger — Cryptographic integrity (kv table, no bullets)
//!   3. Matrix — Framework compliance grid (5 framework cards)
//!   4. CISO brief — Asymmetric KPI
//!   5. CFO brief — Dollar-value display
//!   6. GC brief — Timeline triplet
//!   7. Broker brief — Eligibility verdict
//!
//! Every page composes from the editorial helpers in `ctx` — no raw
//! text or shapes outside the helpers.

use thiserror::Error;

use crate::packet::SignedPacket;

mod baaar;
mod ctx;
mod page1_cover;
mod page2_ledger;
mod page3_compliance;
mod page4_ciso;
mod page5_cfo;
mod page6_gc;
mod page7_broker;

pub use ctx::{Ctx, Page};

#[derive(Debug, Error)]
pub enum PdfError {
    #[error("font error: {0}")]
    Font(String),
    #[error("save error: {0}")]
    Save(String),
}

/// Render a `SignedPacket` to PDF bytes (7-page A4 editorial brief).
pub fn render_packet_pdf(packet: &SignedPacket) -> Result<Vec<u8>, PdfError> {
    use printpdf::{Mm, PdfDocument};

    let (doc, page1, layer1) = PdfDocument::new(
        "Apohara VOUCH Evidence Packet",
        Mm(210.0),
        Mm(297.0),
        "Layer 1",
    );
    let font_regular = doc
        .add_builtin_font(printpdf::BuiltinFont::Helvetica)
        .map_err(|e| PdfError::Font(format!("{e:?}")))?;
    let font_bold = doc
        .add_builtin_font(printpdf::BuiltinFont::HelveticaBold)
        .map_err(|e| PdfError::Font(format!("{e:?}")))?;
    let ctx = Ctx {
        doc: &doc,
        font_regular: &font_regular,
        font_bold: &font_bold,
    };

    // Stable seal id (first 8 hex of BLAKE3 hash).
    let seal_id = format!("VOUCH-{}", &packet.blake3_hash_hex[..8]);

    // Resolve the page-1 layer into a `Page`.
    let layer1 = doc.get_page(page1).get_layer(layer1);
    let mut p1 = Page {
        layer: layer1,
        cursor_y: 280.0,
        line_h: 7.0,
    };
    page1_cover::render(&ctx, packet, &mut p1, &seal_id, 7)?;
    page2_ledger::render(&ctx, packet, &mut ctx.add_a4_page("Layer 2"), &seal_id, 7);
    page3_compliance::render(&ctx, packet, &mut ctx.add_a4_page("Layer 3"), &seal_id, 7);
    page4_ciso::render(&ctx, packet, &mut ctx.add_a4_page("Layer 4"), &seal_id, 7);
    page5_cfo::render(&ctx, packet, &mut ctx.add_a4_page("Layer 5"), &seal_id, 7);
    page6_gc::render(&ctx, packet, &mut ctx.add_a4_page("Layer 6"), &seal_id, 7);
    page7_broker::render(&ctx, packet, &mut ctx.add_a4_page("Layer 7"), &seal_id, 7);

    let mut buf: Vec<u8> = Vec::new();
    {
        let mut writer = std::io::BufWriter::new(&mut buf);
        doc.save(&mut writer)
            .map_err(|e| PdfError::Save(format!("{e:?}")))?;
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
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
            crate::packet::EvidencePacket::new("stark", "inv-001", decisions, Outcome::Approve);
        SignedPacket::wrap(packet, "00".repeat(64), "11".repeat(32))
    }

    #[test]
    fn renders_to_non_empty_bytes() {
        let sp = sample_packet();
        let bytes = render_packet_pdf(&sp).expect("render");
        assert!(bytes.len() > 4096, "PDF should be >4KB, got {}", bytes.len());
        assert_eq!(&bytes[..5], b"%PDF-");
    }
}
