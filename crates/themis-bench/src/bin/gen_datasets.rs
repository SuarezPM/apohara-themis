//! `gen_datasets` — write the three 1K synthetic CSV fixtures to
//! `crates/themis-bench/data/<dataset>/<dataset>.csv`.
//!
//! Story C-15 / AC15. Run once (committed to the repo):
//!
//! ```text
//! cargo run -p themis-bench --bin gen_datasets
//! ```
//!
//! The bytes written here are **byte-for-byte identical** to
//! what `lib::datasets::rows` produces at runtime — both paths
//! call `datasets::generator::write_csv(dataset)`. The CSVs are
//! committed for inspection and reproducibility; the bench itself
//! doesn't read them (it generates in memory to avoid the
//! `include_bytes!` chicken-and-egg on a fresh checkout).

use std::fs;
use std::path::PathBuf;

use themis_bench::datasets::Dataset;

fn out_path(dataset: Dataset) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("data");
    p.push(dataset.name());
    p.push(format!("{}.csv", dataset.name()));
    p
}

fn write_dataset(dataset: Dataset) {
    let path = out_path(dataset);
    fs::create_dir_all(path.parent().unwrap()).expect("create dir");
    let bytes = themis_bench::datasets::rows(dataset);
    fs::write(&path, &bytes).expect("write file");
    let n_lines = bytes.iter().filter(|b| **b == b'\n').count();
    println!(
        "wrote {} ({} lines) to {}",
        dataset.name(),
        n_lines,
        path.display()
    );
}

fn main() {
    write_dataset(Dataset::InvoiceNet);
    write_dataset(Dataset::CzechBank);
    write_dataset(Dataset::AdultIncome);
    println!("all 3 datasets regenerated from seed");
}
