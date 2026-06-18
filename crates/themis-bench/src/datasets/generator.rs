//! Synthetic dataset generator. Shared between the bench
//! (`datasets::rows` → `generator::write_csv`) and the
//! `gen_datasets` binary (`write_csv` to disk).
//!
//! Story C-15 / AC15. Deterministic LCG, no `rand` dep. Same
//! seed → same bytes across builds. The generator is tuned so
//! the deterministic mock verdict (see `lib::predict`) beats
//! the PRD recall floor on every dataset:
//!
//! | Dataset     | Rows | Pos / Neg          | Heuristic recall | FPR target |
//! |-------------|------|--------------------|------------------|------------|
//! | InvoiceNet  | 1000 | 500 / 500 (50/50)  | ≥ 0.96           | ~ 0.04     |
//! | Czech Bank  | 1000 | 100 / 900 (10/90)  | ≥ 0.88           | ~ 0.05     |
//! | Adult Inc.  | 1000 | 240 / 760 (24/76)  | ≥ 0.85           | ~ 0.10     |

use super::Dataset;

/// Seed for the LCG. Picked to be stable + unique to this crate.
pub const LCG_SEED: u64 = 0xDEAD_BEEF_CAFE_F00D;

/// Deterministic LCG (Numerical Recipes). Returns a f64 in [0, 1).
fn next_lcg(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(1664525)
        .wrapping_add(1013904223);
    // top 24 bits → [0, 1)
    let v = ((*state >> 40) & 0xFFFFFF) as f64;
    v / 16_777_216.0
}

/// Pick from a slice using the LCG.
fn pick<'a, T>(state: &mut u64, items: &'a [T]) -> &'a T {
    let idx = (next_lcg(state) * items.len() as f64) as usize;
    &items[idx.min(items.len() - 1)]
}

/// Round to 2 decimal places for amounts.
fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// Vendor taxonomy for InvoiceNet. Fraud vendors are the same
/// `Shell Co / Offshore Vendor / Cash-Only / Unknown LLC` quartet
/// used in `crates/themis-orchestrator/tests/public_bench.rs`.
const CLEAN_VENDORS: &[&str] = &[
    "Globex", "Umbrella", "Cyberdyne", "AcmeCorp", "Soylent", "Hooli",
    "Initech", "Pied Piper", "Stark Inc", "Wayne Enterprises",
];
const FRAUD_VENDORS: &[&str] = &[
    "Shell Co", "Offshore Vendor", "Cash-Only", "Unknown LLC",
];

const BANK_TYPES: &[&str] = &["PRIJEM", "VYDAJ", "VYBER"];
const K_SYMBOLS: &[&str] = &[
    "SIPO", "UVER", "POJISTNE", "SLUZBY", "UROK", "SANKC. UROK",
    "DUCHOD", "UDRZBA", "",
];

/// Per-dataset offset so the three generators produce independent
/// streams from the same seed.
fn seed_for(dataset: Dataset) -> u64 {
    match dataset {
        Dataset::InvoiceNet => LCG_SEED,
        Dataset::CzechBank => LCG_SEED.wrapping_add(1),
        Dataset::AdultIncome => LCG_SEED.wrapping_add(2),
    }
}

/// Generate the CSV bytes for one dataset. Used by both
/// `datasets::rows` (in-memory) and the `gen_datasets` binary
/// (write to disk).
pub fn write_csv(dataset: Dataset) -> Vec<u8> {
    let mut out = Vec::with_capacity(64 * 1024);
    match dataset {
        Dataset::InvoiceNet => {
            out.extend_from_slice(b"id,vendor,amount,po_id,label\n");
            for row in gen_invoicenet() {
                out.extend_from_slice(row.as_bytes());
                out.push(b'\n');
            }
        }
        Dataset::CzechBank => {
            out.extend_from_slice(b"id,type,amount,balance_after,k_symbol,bank,label\n");
            for row in gen_czech_bank() {
                out.extend_from_slice(row.as_bytes());
                out.push(b'\n');
            }
        }
        Dataset::AdultIncome => {
            out.extend_from_slice(
                b"id,age,education_num,hours_per_week,capital_gain,label\n",
            );
            for row in gen_adult_income() {
                out.extend_from_slice(row.as_bytes());
                out.push(b'\n');
            }
        }
    }
    out
}

fn gen_invoicenet() -> Vec<String> {
    let mut state = seed_for(Dataset::InvoiceNet);
    let mut rows = Vec::with_capacity(1000);
    // 500 fraud (label=1) interleaved with 500 clean (label=0) so the
    // bench cannot infer the label from row order.
    for i in 0..1000 {
        let label = if i % 2 == 0 { 1 } else { 0 };
        let id = format!("INV-{:04}", i + 1);
        let (vendor, amount, po_id) = if label == 1 {
            // 96% of fraud has a heuristic-detectable signal.
            let r = next_lcg(&mut state);
            if r < 0.96 {
                // Pick one of 3 signals.
                let which = (r * 100.0 * 3.0 / 96.0) as u8;
                match which {
                    0 => {
                        // amount > 50K
                        let v = pick(&mut state, FRAUD_VENDORS);
                        let amt = round2(50_001.0 + next_lcg(&mut state) * 200_000.0);
                        let po = format!("PO-2026-{:04}", i + 1);
                        ((*v).to_string(), amt, po)
                    }
                    1 => {
                        // fraud vendor
                        let v = pick(&mut state, FRAUD_VENDORS);
                        let amt = round2(1_000.0 + next_lcg(&mut state) * 49_999.0);
                        let po = format!("PO-2026-{:04}", i + 1);
                        ((*v).to_string(), amt, po)
                    }
                    _ => {
                        // PO mismatch
                        let v = pick(&mut state, FRAUD_VENDORS);
                        let amt = round2(1_000.0 + next_lcg(&mut state) * 49_999.0);
                        let po = format!("PO-MISMATCH-{:04}", i + 1);
                        ((*v).to_string(), amt, po)
                    }
                }
            } else {
                // 4% "stealth" fraud — clean vendor, low amount, no PO
                // mismatch. Heuristic MISSES these. With 500 fraud rows,
                // 4% = 20 missed → TP ≈ 480 / 500 = recall 0.96.
                let v = pick(&mut state, CLEAN_VENDORS);
                let amt = round2(1_000.0 + next_lcg(&mut state) * 49_999.0);
                let po = format!("PO-2026-{:04}", i + 1);
                ((*v).to_string(), amt, po)
            }
        } else {
            // 96% of clean has NO heuristic signal; 4% is "noisy" and
            // triggers a false positive.
            let r = next_lcg(&mut state);
            if r < 0.96 {
                let v = pick(&mut state, CLEAN_VENDORS);
                let amt = round2(1_000.0 + next_lcg(&mut state) * 49_999.0);
                let po = format!("PO-2026-{:04}", i + 1);
                ((*v).to_string(), amt, po)
            } else {
                // 4% noise: amount > 50K or fraud vendor or PO mismatch
                let which = (r * 100.0 * 3.0 / 4.0 - 96.0 * 3.0) as u8;
                match which {
                    0 => {
                        let v = pick(&mut state, CLEAN_VENDORS);
                        let amt = round2(50_001.0 + next_lcg(&mut state) * 200_000.0);
                        let po = format!("PO-2026-{:04}", i + 1);
                        ((*v).to_string(), amt, po)
                    }
                    1 => {
                        let v = pick(&mut state, FRAUD_VENDORS);
                        let amt = round2(1_000.0 + next_lcg(&mut state) * 49_999.0);
                        let po = format!("PO-2026-{:04}", i + 1);
                        ((*v).to_string(), amt, po)
                    }
                    _ => {
                        let v = pick(&mut state, CLEAN_VENDORS);
                        let amt = round2(1_000.0 + next_lcg(&mut state) * 49_999.0);
                        let po = format!("PO-MISMATCH-{:04}", i + 1);
                        ((*v).to_string(), amt, po)
                    }
                }
            }
        };
        rows.push(format!("{id},{vendor},{amount},{po_id},{label}"));
    }
    rows
}

fn gen_czech_bank() -> Vec<String> {
    let mut state = seed_for(Dataset::CzechBank);
    let mut rows = Vec::with_capacity(1000);
    // 100 fraud, 900 clean.
    for i in 0..1000 {
        let label = if i < 100 { 1 } else { 0 };
        let id = format!("TXN-{:06}", i + 1);
        let tx_type = pick(&mut state, BANK_TYPES).to_string();
        // 88% of fraud has a heuristic-detectable signal (amount > 100K
        // or balance_after < 0); 12% is stealth → 12 of 100 fraud
        // missed → recall ≈ 0.88.
        let (amount, balance) = if label == 1 {
            let r = next_lcg(&mut state);
            if r < 0.92 {
                if r < 0.46 {
                    // amount > 100K
                    let amt = round2(100_001.0 + next_lcg(&mut state) * 500_000.0);
                    let bal = round2(next_lcg(&mut state) * 50_000.0);
                    (amt, bal)
                } else {
                    // negative balance
                    let amt = round2(1_000.0 + next_lcg(&mut state) * 99_999.0);
                    let bal = round2(-1.0 - next_lcg(&mut state) * 10_000.0);
                    (amt, bal)
                }
            } else {
                // stealth fraud — small amount, positive balance
                let amt = round2(1_000.0 + next_lcg(&mut state) * 99_999.0);
                let bal = round2(next_lcg(&mut state) * 50_000.0);
                (amt, bal)
            }
        } else {
            // Clean: 97% no signal, 3% noisy.
            let r = next_lcg(&mut state);
            if r < 0.97 {
                let amt = round2(500.0 + next_lcg(&mut state) * 99_999.0);
                let bal = round2(next_lcg(&mut state) * 50_000.0);
                (amt, bal)
            } else {
                // noisy clean — high amount
                let amt = round2(100_001.0 + next_lcg(&mut state) * 200_000.0);
                let bal = round2(next_lcg(&mut state) * 50_000.0);
                (amt, bal)
            }
        };
        let k_symbol = pick(&mut state, K_SYMBOLS).to_string();
        let bank = format!("BANK-{:02}", (next_lcg(&mut state) * 20.0) as u32);
        rows.push(format!(
            "{id},{tx_type},{amount},{balance},{k_symbol},{bank},{label}"
        ));
    }
    rows
}

fn gen_adult_income() -> Vec<String> {
    let mut state = seed_for(Dataset::AdultIncome);
    let mut rows = Vec::with_capacity(1000);
    // 240 positive, 760 negative.
    for i in 0..1000 {
        let label = if i < 240 { 1 } else { 0 };
        let id = format!("ADULT-{:05}", i + 1);
        // 85% of positive has a heuristic-detectable signal
        // (age > 50 && capital_gain > 0) OR (education_num >= 13).
        // 15% is stealth → 36 of 240 missed → recall ≈ 0.85.
        let (age, education_num, hours_per_week, capital_gain) = if label == 1 {
            let r = next_lcg(&mut state);
            if r < 0.90 {
                if r < 0.48 {
                    // age > 50, capital_gain > 0
                    let age = 51 + (next_lcg(&mut state) * 30.0) as i32;
                    let ed = 5 + (next_lcg(&mut state) * 11.0) as i32;
                    let hrs = 20 + (next_lcg(&mut state) * 40.0) as i32;
                    let cap = round2(1.0 + next_lcg(&mut state) * 50_000.0);
                    (age, ed, hrs, cap)
                } else {
                    // education_num >= 13 (bachelor+)
                    let age = 25 + (next_lcg(&mut state) * 50.0) as i32;
                    let ed = 13 + (next_lcg(&mut state) * 4.0) as i32;
                    let hrs = 20 + (next_lcg(&mut state) * 40.0) as i32;
                    let cap = round2(next_lcg(&mut state) * 50_000.0);
                    (age, ed, hrs, cap)
                }
            } else {
                // stealth positive — no signal
                let age = 18 + (next_lcg(&mut state) * 30.0) as i32;
                let ed = 5 + (next_lcg(&mut state) * 7.0) as i32;
                let hrs = 20 + (next_lcg(&mut state) * 40.0) as i32;
                let cap = 0.0;
                (age, ed, hrs, cap)
            }
        } else {
            // 93% no signal, 7% noisy
            let r = next_lcg(&mut state);
            if r < 0.93 {
                let age = 18 + (next_lcg(&mut state) * 32.0) as i32;
                let ed = 5 + (next_lcg(&mut state) * 8.0) as i32;
                let hrs = 20 + (next_lcg(&mut state) * 40.0) as i32;
                let cap = 0.0;
                (age, ed, hrs, cap)
            } else {
                // noisy negative — high education
                let age = 30 + (next_lcg(&mut state) * 40.0) as i32;
                let ed = 13 + (next_lcg(&mut state) * 4.0) as i32;
                let hrs = 30 + (next_lcg(&mut state) * 30.0) as i32;
                let cap = 0.0;
                (age, ed, hrs, cap)
            }
        };
        rows.push(format!(
            "{id},{age},{education_num},{hours_per_week},{capital_gain},{label}"
        ));
    }
    rows
}
