//! Downloads the PDB of one binary and prints its size and container signature.
//!
//! Usage: `cargo run -p flagscope-symbols --example fetch_pdb -- <binary> <directory>`

use std::io::Read as _;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let binary = PathBuf::from(args.next().ok_or("missing <binary>")?);
    let directory = PathBuf::from(args.next().ok_or("missing <directory>")?);

    let reference = flagscope_symbols::read_pdb_reference(&binary)?.ok_or("no CodeView record")?;
    println!("server path: {}", reference.server_path());
    let file = flagscope_symbols::download(
        &reference,
        flagscope_symbols::MICROSOFT_SYMBOL_SERVER,
        &directory,
    )?;
    println!("downloaded: {} ({} bytes)", file.path.display(), file.size);

    let mut signature = [0u8; 32];
    std::fs::File::open(&file.path)?.read_exact(&mut signature)?;
    println!("signature: {:?}", String::from_utf8_lossy(&signature));
    Ok(())
}
