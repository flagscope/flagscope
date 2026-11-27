//! Integration test: reads the PDB reference of a real Windows binary, without
//! network access.

use std::path::PathBuf;

#[test]
fn ntdll_names_its_public_pdb() {
    let system_root = std::env::var_os("SystemRoot").expect("SystemRoot is set on Windows");
    let ntdll = PathBuf::from(system_root)
        .join("System32")
        .join("ntdll.dll");

    let reference = flagscope_symbols::read_pdb_reference(&ntdll)
        .expect("ntdll.dll is a readable PE image")
        .expect("ntdll.dll has a CodeView record");

    assert!(reference.file_name.eq_ignore_ascii_case("ntdll.pdb"));
    let path = reference.server_path();
    assert!(path.starts_with(&format!("{}/", reference.file_name)));
    assert!(path.ends_with(&format!("/{}", reference.file_name)));
}
