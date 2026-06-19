//! Shared context + drawing helpers for the editorial audit PDF.
//!
//! Hallmark · macrostructure: Editorial Audit Brief · tone: editorial-audit
//! · anchor hue: warm-navy · theme: Atelier (warm paper)
//!
//! Design language:
//!   - Warm off-white paper (PAPER), ink-black typography
//!   - 7 pages, each with a distinct visual rhythm
//!   - No hairlines — section dividers are 4mm black bars or solid
//!     color blocks
//!   - Asymmetric layouts (left-aligned, ragged right, never centered)
//!   - Every "list" is a 2-col key-value table; numbers right-aligned
//!   - 60% of every page is whitespace, 40% content
//!
//! The 6 helpers (statement_hero, page_header, section_rule_heavy,
//! kpi_display, table_2col, margin_annotation, page_footer_centered)
//! are the design surface — every page composes them, never draws
//! raw text or shapes directly.

use printpdf::{
    path::{PaintMode, WindingOrder},
    Color, IndirectFontRef, Line, Mm, PdfDocumentReference, PdfLayerReference, Point, Polygon, Rgb,
};

/// Editorial palette — Atelier (warm paper, ink-black typography,
/// warm-navy display). All in 0.0..=1.0 sRGB.
pub mod brand {
    use super::Rgb;

    pub const PAPER: (f64, f64, f64) = (0.980, 0.969, 0.949);
    pub const INK: (f64, f64, f64) = (0.102, 0.102, 0.102);
    pub const MUTED: (f64, f64, f64) = (0.420, 0.420, 0.420);
    pub const BAND: (f64, f64, f64) = (0.957, 0.945, 0.918);

    pub const NAVY: (f64, f64, f64) = (0.039, 0.078, 0.157);
    pub const GOLD: (f64, f64, f64) = (0.722, 0.529, 0.043);
    pub const BLUE: (f64, f64, f64) = (0.290, 0.435, 0.647);

    pub const GREEN: (f64, f64, f64) = (0.039, 0.431, 0.227);
    pub const RED: (f64, f64, f64) = (0.701, 0.149, 0.118);
    pub const AMBER: (f64, f64, f64) = (0.604, 0.404, 0.000);

    pub const CRYPTO_BG: (f64, f64, f64) = (0.937, 0.925, 0.898);

    /// Build a printpdf `Rgb` from a token triple.
    pub fn rgb(t: (f64, f64, f64)) -> Rgb {
        Rgb::new(t.0 as f32, t.1 as f32, t.2 as f32, None)
    }
}

/// Per-page state. `cursor_y` is in millimetres, `line_h` is the
/// default line height in millimetres.
pub struct Page {
    pub layer: PdfLayerReference,
    pub cursor_y: f32,
    pub line_h: f32,
}

impl Page {
    /// Set the active fill color.
    pub fn set_fill(&self, t: (f64, f64, f64)) {
        self.layer.set_fill_color(Color::Rgb(brand::rgb(t)));
    }

    /// Reset fill color to ink.
    pub fn reset_color(&self) {
        self.set_fill(brand::INK);
    }
}

/// Document-wide state shared across all pages.
pub struct Ctx<'a> {
    pub doc: &'a PdfDocumentReference,
    pub font_regular: &'a IndirectFontRef,
    pub font_bold: &'a IndirectFontRef,
}

impl<'a> Ctx<'a> {
    /// Build a new A4 portrait page with warm paper background.
    pub fn add_a4_page(&self, layer_name: &str) -> Page {
        let (page_idx, layer_idx) = self.doc.add_page(Mm(210.0), Mm(297.0), layer_name);
        let layer = self.doc.get_page(page_idx).get_layer(layer_idx);
        layer.set_fill_color(Color::Rgb(brand::rgb(brand::INK)));
        Page {
            layer,
            cursor_y: 280.0,
            line_h: 7.0,
        }
    }

    /// Write one line of text at `(x, y)` on the given page layer.
    pub fn write(&self, page: &Page, text: &str, x: f32, y: f32, size: f32, bold: bool) {
        let font = if bold { self.font_bold } else { self.font_regular };
        page.layer.use_text(text, size, Mm(x), Mm(y), font);
    }

    // ===== Drawing primitives =====

    /// Filled rectangle in mm coordinates. The workhorse — used for
    /// color blocks, the verdict hero, alternating table rows, the
    /// hero QR container.
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

    /// 4mm-tall horizontal ink-black bar. Section divider. NOT a
    /// hairline — editorial design uses heavy black rules.
    pub fn section_rule_heavy(&self, page: &Page, x: f32, y: f32, w: f32) {
        self.rect(page, x, y, w, 1.4, brand::INK);
    }

    /// Thin hairline (1pt) in muted gray. Used inside crypto chips
    /// or as a subtle internal rule. Never as a section divider.
    pub fn hairline(&self, page: &Page, x: f32, y: f32, w: f32) {
        self.rect(page, x, y, w, 0.3, brand::MUTED);
    }

    // ===== Editorial page-level helpers =====

    /// Page header strip. Draws a "01 / 05" numerator in the top-left
    /// margin and a thin meta line (seal id + tenant + invoice) on
    /// the top-right. Sits above the title block on every page.
    pub fn page_header(
        &self,
        page: &mut Page,
        page_numerator: &str,
        seal_id: &str,
        tenant: &str,
        invoice: &str,
    ) {
        // Heavy black bar at the very top of the page.
        self.rect(page, 0.0, 293.0, 210.0, 4.0, brand::NAVY);

        // Numerator (e.g. "01 / 05") — left, 8pt bold navy.
        page.set_fill(brand::NAVY);
        self.write(page, page_numerator, 20.0, 287.0, 8.0, true);
        page.reset_color();

        // Meta line on the right — 7pt ink, right-aligned at x=190.
        let meta = format!("{seal_id}  \u{00B7}  {tenant}  \u{00B7}  {invoice}");
        page.set_fill(brand::INK);
        self.write(page, &meta, 190.0, 287.0, 7.0, false);
        page.reset_color();

        // Hairline under the header.
        self.hairline(page, 20.0, 283.0, 170.0);

        // Cursor sits below the header.
        page.cursor_y = 278.0;
    }

    /// Statement hero (page 1 only). A solid color block covering
    /// the lower 60% of the page with the verdict rendered in 72pt
    /// display type in white. Above the block: identifiers as a
    /// 2-col table.
    pub fn statement_hero(
        &self,
        page: &mut Page,
        verdict: &str,
        verdict_color: (f64, f64, f64),
        verdict_label: &str,
    ) {
        // The block: starts at y=160, fills the bottom 137mm.
        self.rect(page, 0.0, 0.0, 210.0, 160.0, verdict_color);

        // Tiny eyebrow above the giant verdict.
        page.set_fill((1.0, 1.0, 1.0));
        self.write(page, verdict_label, 20.0, 145.0, 8.5, true);
        page.reset_color();

        // The verdict itself, 72pt bold, white.
        page.set_fill((1.0, 1.0, 1.0));
        self.write(page, verdict, 20.0, 90.0, 72.0, true);
        page.reset_color();

        // Sub-line under the verdict.
        page.set_fill((0.85, 0.87, 0.92));
        self.write(
            page,
            "BAAAR KILL-SWITCH VERDICT \u{2014} DORA + EU AI ACT + NIST AI RMF + OWASP AGENTIC + ISO 42001",
            20.0,
            38.0,
            7.5,
            false,
        );
        page.reset_color();
    }

    /// 2-column key-value table row. The label is rendered in
    /// 7.5pt bold uppercase tracked (Helvetica bold at 7.5 is the
    /// closest the builtin fonts can approximate small-caps
    /// tracking). The value is rendered at 9.5pt regular. The
    /// `value_right` flag right-aligns the value (use for numbers,
    /// hashes).
    pub fn kv_row(
        &self,
        page: &mut Page,
        label: &str,
        value: &str,
        value_right: bool,
    ) {
        // Label (left column).
        page.set_fill(brand::MUTED);
        self.write(page, label, 20.0, page.cursor_y - 4.5, 7.5, true);
        // Value (right column).
        page.set_fill(brand::INK);
        if value_right {
            // Right-align by estimating the value width at 9.5pt:
            // Helvetica 9.5pt averages ~2.4mm per char.
            let approx_w = value.chars().count() as f32 * 2.4;
            let x = 190.0 - approx_w;
            self.write(page, value, x, page.cursor_y - 4.5, 9.5, false);
        } else {
            self.write(page, value, 78.0, page.cursor_y - 4.5, 9.5, false);
        }
        page.reset_color();
        page.cursor_y -= 6.5;
    }

    /// Crypto chip: a 2-line block with a label above the value,
    /// both rendered inside a warm-gray rectangle. The value is
    /// rendered in INK at 8.5pt (the most-readable monospace-style
    /// size in Helvetica regular).
    pub fn crypto_chip(&self, page: &mut Page, label: &str, value: &str) {
        let chip_h = 14.0;
        self.rect(page, 20.0, page.cursor_y - chip_h, 170.0, chip_h, brand::CRYPTO_BG);

        page.set_fill(brand::MUTED);
        self.write(page, label, 22.0, page.cursor_y - 4.0, 6.5, true);
        page.set_fill(brand::INK);
        self.write(page, value, 22.0, page.cursor_y - 11.0, 8.5, false);
        page.reset_color();
        page.cursor_y -= chip_h + 2.0;
    }

    /// KPI display: a giant number (36pt) with a unit label (7.5pt)
    /// underneath. Renders at the current cursor position, advances
    /// the cursor by 42mm so the next element can sit below.
    pub fn kpi_display(
        &self,
        page: &mut Page,
        number: &str,
        unit: &str,
    ) {
        let y = page.cursor_y - 36.0;
        page.set_fill(brand::INK);
        self.write(page, number, 20.0, y, 36.0, true);
        page.set_fill(brand::MUTED);
        self.write(page, unit, 20.0, y - 6.0, 7.5, true);
        page.reset_color();
        page.cursor_y = y - 14.0;
    }

    /// Sub-KPI caption: 8pt MUTED italic-style tracking surrogate.
    /// Renders at the current cursor, advances by 6mm.
    pub fn kpi_caption(&self, page: &mut Page, text: &str) {
        page.set_fill(brand::MUTED);
        self.write(page, text, 20.0, page.cursor_y, 8.0, false);
        page.reset_color();
        page.cursor_y -= 6.0;
    }

    /// Margin annotation: a 7pt italic-simulated (tracking + size
    /// differential) note in the bottom-left margin. Helvetica has
    /// no real italic, so we render in regular with a wide tracking
    /// surrogate.
    pub fn margin_annotation(&self, page: &Page, text: &str) {
        page.set_fill(brand::MUTED);
        self.write(page, text, 20.0, 18.0, 6.5, false);
        page.reset_color();
    }

    /// Centered page footer. Format: "page X of Y" centered with
    /// the seal id on the left and the disclaimer on the right.
    pub fn page_footer_centered(&self, page: &Page, seal_id: &str, page_n: u32, total: u32) {
        self.hairline(page, 20.0, 14.0, 170.0);
        page.set_fill(brand::MUTED);
        self.write(page, seal_id, 20.0, 10.0, 7.0, false);
        self.write(
            page,
            &format!("{page_n} / {total}"),
            105.0,
            10.0,
            7.0,
            true,
        );
        self.write(
            page,
            "vouch.apohara.dev",
            190.0,
            10.0,
            7.0,
            false,
        );
        page.reset_color();
    }

    /// Page title (h1): 22pt bold INK, left-aligned, no underline.
    pub fn page_title(&self, page: &mut Page, text: &str) {
        self.write(page, text, 20.0, page.cursor_y, 22.0, true);
        page.cursor_y -= 14.0;
        // Lead paragraph slot: 9.5pt MUTED subtitle, 1 line under title.
    }

    /// Section title (h2): 10pt bold INK uppercase tracked.
    pub fn section_title(&self, page: &mut Page, text: &str) {
        page.set_fill(brand::INK);
        self.write(page, text, 20.0, page.cursor_y, 10.0, true);
        page.reset_color();
        page.cursor_y -= 7.0;
    }

    /// Body line: 9.5pt INK, left-aligned.
    pub fn body(&self, page: &mut Page, text: &str) {
        page.set_fill(brand::INK);
        self.write(page, text, 20.0, page.cursor_y, 9.5, false);
        page.reset_color();
        page.cursor_y -= 6.0;
    }

    /// Body line in MUTED (sub-text, secondary).
    pub fn body_muted(&self, page: &mut Page, text: &str) {
        page.set_fill(brand::MUTED);
        self.write(page, text, 20.0, page.cursor_y, 9.5, false);
        page.reset_color();
        page.cursor_y -= 6.0;
    }
}
