//! End-to-end tests of the `--output` option that need no network: they stop
//! before any scan starts.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// A file in the system temporary directory, removed when dropped.
struct TempFile(PathBuf);

impl TempFile {
    fn with_contents(name: &str, contents: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("flagscope-test-{}-{name}", std::process::id()));
        fs::write(&path, contents).expect("create the test file");
        Self(path)
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn flagscope() -> Command {
    Command::new(env!("CARGO_BIN_EXE_flagscope"))
}

#[test]
fn existing_output_file_is_not_overwritten_without_the_option() {
    let file = TempFile::with_contents("existing.txt", "keep me\n");

    let result = flagscope()
        .args(["scan", "--output"])
        .arg(&file.0)
        .output()
        .expect("run flagscope");

    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("already exists"), "stderr: {stderr}");
    assert!(stderr.contains("--overwrite"), "stderr: {stderr}");
    assert_eq!(fs::read_to_string(&file.0).unwrap(), "keep me\n");
}

#[test]
fn overwrite_requires_an_output_file() {
    let result = flagscope()
        .args(["scan", "--overwrite"])
        .output()
        .expect("run flagscope");

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("--output"));
}
