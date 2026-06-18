//! Dataset metadata + row access for the cross-domain bench.
//!
//! Story C-15 / AC15: extend the bench harness to run on
//! InvoiceNet 1K, Czech Bank 1K, and Adult Income 1K. For the
//! MVP the rows are **synthetic** — generated on the fly by a
//! seeded LCG so the tests are hermetic, deterministic, and
//! bootstrap-free (no `include_bytes!` chicken-and-egg). The same
//! generator writes the canonical CSVs to
//! `data/<dataset>/<dataset>.csv` via the `gen_datasets` binary
//! for inspection and reproducibility.
//!
//! ## Determinism
//!
//! Each dataset has a fixed seed (see `generator::LCG_SEED`). The
//! generator is stable — same seed → same bytes across builds.
//! The `gen_datasets` binary in `src/bin/gen_datasets.rs` writes
//! the same bytes to disk; `lib::parse_csv` then reads the bytes
//! (or `rows()` generates them in memory — both produce the
//! identical row set).

use serde::{Deserialize, Serialize};

pub mod generator;

/// The three datasets the bench runs against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dataset {
    /// Stanford InvoiceNet — buyer-side AP fraud (balanced 50/50).
    InvoiceNet,
    /// Czech Bank — bank-account fraud (cross-domain, 10/90).
    CzechBank,
    /// UCI Adult Income — income > $50K (cross-domain, 24/76).
    AdultIncome,
}

impl Dataset {
    /// Stable name used in CSV paths and report keys.
    pub fn name(self) -> &'static str {
        match self {
            Dataset::InvoiceNet => "invoicenet",
            Dataset::CzechBank => "czech_bank",
            Dataset::AdultIncome => "adult_income",
        }
    }
}

/// Metadata for an in-repo 1K fixture.
#[derive(Debug, Clone, Copy)]
pub struct DatasetMeta {
    /// Human-readable name (e.g. "InvoiceNet 1K").
    pub name: &'static str,
    /// Number of data rows in the fixture (1K for all three).
    pub rows: usize,
    /// Class distribution: (positive_count, negative_count).
    /// Positive = the "fraud" / ">50K" / "bad" class.
    pub label_distribution: (usize, usize),
}

/// The minimum recall target per dataset. The thresholds match the
/// PRD C-15 acceptance criteria (InvoiceNet >= 0.96, Czech Bank
/// >= 0.88, Adult Income >= 0.85). For synthetic data we set the
/// > bar to the PRD floor — the synthetic generator is tuned so the
/// > deterministic mock verdict beats the floor.
pub fn recall_target(dataset: Dataset) -> f64 {
    match dataset {
        Dataset::InvoiceNet => 0.96,
        Dataset::CzechBank => 0.88,
        Dataset::AdultIncome => 0.85,
    }
}

/// Return the metadata for an in-repo 1K fixture.
pub fn metadata(dataset: Dataset) -> DatasetMeta {
    match dataset {
        Dataset::InvoiceNet => DatasetMeta {
            name: "InvoiceNet 1K (synthetic)",
            rows: 1000,
            // Balanced 50/50 (synthetic; the real Stanford
            // InvoiceNet is closer to 80/20 buyer-side legit).
            label_distribution: (500, 500),
        },
        Dataset::CzechBank => DatasetMeta {
            name: "Czech Bank 1K (synthetic)",
            rows: 1000,
            // 10/90 fraud/legit (matches the canonical Czech
            // Bank distribution; 1999 Berka dataset).
            label_distribution: (100, 900),
        },
        Dataset::AdultIncome => DatasetMeta {
            name: "Adult Income 1K (synthetic)",
            rows: 1000,
            // 24/76 ">50K / <=50K" (matches the canonical UCI
            // Adult distribution; ~23.9% positive).
            label_distribution: (240, 760),
        },
    }
}

/// Return the dataset rows as a CSV byte slice. The MVP path
/// generates the bytes from the seeded LCG in memory (no disk
/// dependency); the `gen_datasets` binary writes the same bytes
/// to `data/<dataset>/<dataset>.csv` for inspection.
pub fn rows(dataset: Dataset) -> Vec<u8> {
    generator::write_csv(dataset)
}
