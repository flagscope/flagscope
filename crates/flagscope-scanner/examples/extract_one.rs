//! HU-01 proof of concept: downloads the PDB of one binary and prints its features.
//!
//! Usage: `cargo run -p flagscope-scanner --example extract_one -- <binary> <symbol-directory>`

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let binary = PathBuf::from(args.next().ok_or("missing <binary>")?);
    let directory = PathBuf::from(args.next().ok_or("missing <symbol-directory>")?);

    let reference =
        flagscope_symbols::read_pdb_reference(&binary)?.ok_or("the binary has no PDB reference")?;
    let pdb = flagscope_symbols::SymbolServer::microsoft().download(&reference, &directory)?;
    println!("Downloaded {} ({} bytes)", pdb.path.display(), pdb.size);

    let extraction = flagscope_scanner::extract_features(&pdb.path, &binary)?;
    for feature in &extraction.features {
        println!("{feature}");
    }
    println!(
        "{} features, {} names without a readable descriptor",
        extraction.features.len(),
        extraction.unreadable.len()
    );
    Ok(())
}
