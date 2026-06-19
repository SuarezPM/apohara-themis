//! Shared drawing helpers for the 1-page evidence receipt.
//!
//! Hallmark · macrostructure: Receipt (one-pager) · tone: technical-trust
//! · anchor hue: lime-green · theme: Synthex dark
//!
//! Design language: dark background, lime/green accent (Apahara brand
//! from the pitch deck), monospace for hashes and code, no hairlines
//! (use lime rule or 1.4mm black bar), generous whitespace.

use printpdf::{
    path::{PaintMode, WindingOrder},
    Color, IndirectFontRef, Line, Mm, PdfDocumentReference, PdfLayerReference, Point, Polygon, Rgb,
};

/// Synthex-style dark palette.
pub mod brand {
    use super::Rgb;

    pub const BG: (f64, f64, f64) = (0.020, 0.024, 0.031);
    pub const BG2: (f64, f64, f64) = (0.051, 0.067, 0.090);
    pub const INK: (f64, f64, f64) = (0.831, 0.843, 0.867);
    pub const MUTED: (f64, f64, f64) = (0.431, 0.463, 0.506);
    pub const LIME: (f64, f64, f64) = (0.702, 1.000, 0.227);
    pub const GREEN: (f64, f64, f64) = (0.180, 0.800, 0.443);
    pub const RED: (f64, f64, f64) = (0.906, 0.298, 0.235);
    pub const BLUE: (f64, f64, f64) = (0.431, 0.659, 0.996);

    /// Build a printpdf `Rgb` from a token triple.
    pub fn rgb(t: (f64, f64, f64)) -> Rgb {
        Rgb::new(t.0 as f32, t.1 as f32, t.2 as f32, None)
    }
}

pub struct Page {
    pub layer: PdfLayerReference,
    pub cursor_y: f32,
    pub line_h: f32,
}

impl Page {
    pub fn set_fill(&self, t: (f64, f64, f64)) {
        self.layer.set_fill_color(Color::Rgb(brand::rgb(t)));
    }

    pub fn reset_color(&self) {
        self.set_fill(brand::INK);
    }
}

pub struct Ctx<'a> {
    pub doc: &'a PdfDocumentReference,
    pub font_regular: &'a IndirectFontRef,
    pub font_bold: &'a IndirectFontRef,
}

impl<'a> Ctx<'a> {
    /// Build a single A4 portrait page with dark background.
    /// We create TWO layers: a "Background" layer (which fills
    /// the whole page dark) and the content layer (returned to
    /// the caller). Because printpdf 0.7 emits layers in creation
    /// order, the background is rendered first in the content
    /// stream and the text is drawn on top.
    pub fn add_a4_page(&self, layer_name: &str) -> Page {
        let (page_idx, _layer_idx) = self.doc.add_page(Mm(210.0), Mm(297.0), layer_name);

        // Layer 1: background. Use a different name to avoid
        // collision with the default "Layer 1" used by `PdfDocument::new`.
        let bg_layer = self
            .doc
            .get_page(page_idx)
            .add_layer("Background");
        // Fill the whole page with BG.
        bg_layer.set_fill_color(Color::Rgb(brand::rgb(brand::BG)));
        let ring = vec![
            (Point::new(Mm(0.0), Mm(0.0)), false),
            (Point::new(Mm(210.0), Mm(0.0)), false),
            (Point::new(Mm(210.0), Mm(297.0)), false),
            (Point::new(Mm(0.0), Mm(297.0)), false),
        ];
        let poly = Polygon {
            rings: vec![ring],
            mode: PaintMode::Fill,
            winding_order: WindingOrder::NonZero,
        };
        bg_layer.add_polygon(poly);

        // Layer 2 (the default "Layer 1"): the content layer the
        // caller will use. We do this by adding ANOTHER layer to the
        // same page — that puts the content layer AFTER the
        // background layer in creation order, which means printpdf
        // emits it after in the content stream, so the text is
        // drawn on top of the dark background.
        let content_layer = self
            .doc
            .get_page(page_idx)
            .add_layer("Content");
        // Default text color to INK (light) so text is visible on
        // the dark background.
        content_layer.set_fill_color(Color::Rgb(brand::rgb(brand::INK)));
        Page {
            layer: content_layer,
            cursor_y: 280.0,
            line_h: 7.0,
        }
    }

    pub fn write(&self, page: &Page, text: &str, x: f32, y: f32, size: f32, bold: bool) {
        let font = if bold { self.font_bold } else { self.font_regular };
        page.layer.use_text(text, size, Mm(x), Mm(y), font);
    }

    /// Filled rectangle in mm coordinates.
    pub fn rect(&self, page: &Page, x: f32, y: f32, w: f32, h: f32, color: (f64, f64, f64)) {
        page.set_fill(color);
        let ring = vec![
            (Point::new(Mm(x), Mm(y)), false),
            (Point::new(Mm(x + w), Mm(y)), false),
            (Point::new(Mm(x + w), Mm(y + h)), false),
            (Point::new(Mm(x), Mm(y + h)), false),
        ];
        let poly = Polygon {
            rings: vec![ring],
            mode: PaintMode::Fill,
            winding_order: WindingOrder::NonZero,
        };
        page.layer.add_polygon(poly);
        page.reset_color();
    }

    /// Lime rule (1.4mm) — section divider.
    pub fn lime_rule(&self, page: &Page, x: f32, y: f32, w: f32) {
        self.rect(page, x, y, w, 1.0, brand::LIME);
    }

    /// Card background (BG2 panel) with hairline lime border.
    pub fn card(&self, page: &Page, x: f32, y: f32, w: f32, h: f32) {
        self.rect(page, x, y, w, h, brand::BG2);
        // Top lime accent stripe (1mm).
        self.rect(page, x, y + h - 1.0, w, 1.0, brand::LIME);
    }
}
