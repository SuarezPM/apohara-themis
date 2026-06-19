//! Page 2 — Cryptographic Ledger.
//!
//! Layout: identifiers (reminder) + cryptographic integrity as a
//! 2-col table. No bullets, no `[x]` markers — just label/value
//! rows. The Rekor anchor + agent decision summary live here as
//! secondary structured data.

use crate::packet::SignedPacket;
use themis_agents::decision::AgentDecision;

use super::ctx::{brand, Ctx, Page};

pub fn render(ctx: &Ctx, packet: &SignedPacket, page: &mut Page, seal_id: &str, total: u32) {
    let p = &packet.packet;

    ctx.page_header(page, "02 / 07", seal_id, &p.tenant_id, &p.invoice_id);

    ctx.page_title(page, "Cryptographic Ledger");
    page.cursor_y -= 4.0;
    ctx.body_muted(
        page,
        "Every byte in this packet is signed, hashed, and chained. The BLAKE3 hash is what was signed; the Ed25519 signature is the proof; the public key is the anchor.",
    );
    page.cursor_y -= 6.0;
    ctx.section_rule_heavy(page, 20.0, page.cursor_y + 4.0, 170.0);
    page.cursor_y -= 8.0;

    ctx.section_title(page, "INTEGRITY");
    ctx.crypto_chip(page, "BLAKE3 HASH", &packet.blake3_hash_hex);

    let sig_preview: String = if packet.signature_hex.len() >= 24 {
        format!("{}\u{2026}", &packet.signature_hex[..24])
    } else {
        packet.signature_hex.clone()
    };
    ctx.crypto_chip(
        page,
        "ED25519 SIGNATURE",
        &format!("{sig_preview}  ({} chars, full on file)", packet.signature_hex.len()),
    );
    ctx.crypto_chip(page, "PUBLIC KEY", &packet.public_key_hex);

    if let Some(entry) = &packet.rekor_entry {
        page.cursor_y -= 4.0;
        ctx.section_title(page, "REKOR TRANSPARENCY LOG");
        ctx.kv_row(page, "REKOR UUID", &entry.uuid, false);
        ctx.kv_row(page, "LOG INDEX", &entry.log_index.to_string(), true);
        ctx.kv_row(
            page,
            "INTEGRATED TIME",
            &format!("{} s", entry.integrated_time),
            true,
        );
        ctx.kv_row(page, "BUNDLE", &entry.bundle_url, false);
    }

    page.cursor_y -= 4.0;
    ctx.section_title(page, "AGENT DECISIONS");
    for (i, d) in p.agent_decisions.iter().enumerate() {
        if page.cursor_y < 30.0 {
            ctx.body_muted(
                page,
                &format!("... and {} more (see JSON packet)", p.agent_decisions.len() - i),
            );
            break;
        }
        let conf_pct = (d.confidence * 100.0) as u32;
        let line = format!(
            "{:>2}.  {}  \u{00B7}  conf={}%  \u{00B7}  {:?}",
            i + 1,
            d.agent_id,
            conf_pct,
            d.decision_type
        );
        ctx.body(page, &line);
    }

    // QR code in the top-right of the page, smaller and muted.
    let qr_x = 165.0;
    let qr_y = 245.0;
    let qr_size = 22.0;
    render_qr_png(ctx, page, packet, qr_x, qr_y, qr_size);

    ctx.page_footer_centered(page, seal_id, 2, total);
}

/// Render a QR code (PNG bitmap) at (x, y) with the given size in
/// mm. Shared by the cover and ledger pages.
pub fn render_qr_png(
    _ctx: &Ctx,
    page: &Page,
    packet: &SignedPacket,
    x: f32,
    y: f32,
    size_mm: f32,
) {
    let verify_url = format!(
        "https://vouch.apohara.dev/verify?packet={}&tenant={}",
        packet.packet.packet_id, packet.packet.tenant_id
    );
    let qr = match qrcode::QrCode::new(verify_url.as_bytes()) {
        Ok(qr) => qr,
        Err(_) => return,
    };
    let w = qr.width();
    let colors = qr.to_colors();
    let mut img = image::GrayImage::new(w as u32, w as u32);
    for y in 0..w {
        for x in 0..w {
            let is_dark = colors[y * w + x] == qrcode::Color::Dark;
            let luma = if is_dark { 0u8 } else { 255u8 };
            img.put_pixel(x as u32, y as u32, image::Luma([luma]));
        }
    }
    let scaled = image::imageops::resize(
        &img,
        (w as u32) * 8,
        (w as u32) * 8,
        image::imageops::Nearest,
    );
    let dyn_img = image::DynamicImage::ImageLuma8(scaled);
    let (w_px, h_px) = (dyn_img.width() as usize, dyn_img.height() as usize);
    let pixels: Vec<u8> = dyn_img.to_luma8().pixels().map(|p| p.0[0]).collect();
    let xobject = printpdf::ImageXObject {
        width: printpdf::Px(w_px),
        height: printpdf::Px(h_px),
        color_space: printpdf::ColorSpace::Greyscale,
        bits_per_component: printpdf::ColorBits::Bit8,
        interpolate: true,
        image_data: pixels,
        image_filter: None,
        clipping_bbox: None,
        smask: None,
    };
    let pdf_image: printpdf::Image = xobject.into();
    let qr_pt = size_mm * 2.834_645_7_f32;
    let pdf_w_pt = w_px as f32;
    let scale = qr_pt / pdf_w_pt;
    let transform = printpdf::ImageTransform {
        translate_x: Some(printpdf::Mm(x)),
        translate_y: Some(printpdf::Mm(y)),
        scale_x: Some(scale),
        scale_y: Some(scale),
        ..Default::default()
    };
    pdf_image.add_to_layer(page.layer.clone(), transform);
}

// Suppress unused warning on AgentDecision import (kept for future use).
#[allow(dead_code)]
fn _typed(_d: &AgentDecision) {}
