//! Choice of the binaries a build scan downloads symbols for (AD-13).

use std::fs;
use std::path::{Path, PathBuf};

/// Byte strings that mark a binary as using the feature-staging API.
const MARKERS: [&[u8]; 2] = [b"featurestaging", b"RtlNotifyFeatureUsage"];

/// Returns the `.dll` and `.exe` files a build scan examines, sorted.
///
/// These are the files under `System32` and `SystemApps` (recursively, without
/// following links) and the executables directly in `system_root`.
pub(crate) fn system_binaries(system_root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect(&system_root.join("System32"), true, &mut files);
    collect(&system_root.join("SystemApps"), true, &mut files);
    collect(system_root, false, &mut files);
    files.sort();
    files
}

/// Adds the `.dll` and `.exe` files of `directory` to `files`.
///
/// Unreadable directories are skipped: protected folders are expected.
fn collect(directory: &Path, recursive: bool, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            if recursive {
                collect(&path, true, files);
            }
        } else if is_binary(&path) {
            files.push(path);
        }
    }
}

/// Whether `path` has a `.dll` or `.exe` extension, ignoring case.
fn is_binary(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("dll") || extension.eq_ignore_ascii_case("exe")
        })
}

/// Whether the contents of a binary reference the feature-staging API.
pub(crate) fn references_feature_staging(contents: &[u8]) -> bool {
    MARKERS.iter().any(|marker| {
        contents
            .windows(marker.len())
            .any(|window| window.eq_ignore_ascii_case(marker))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markers_are_found_ignoring_case() {
        assert!(references_feature_staging(
            b"..api-ms-win-core-FeatureStaging-l1-1-0.dll.."
        ));
        assert!(references_feature_staging(b"\0RtlNotifyFeatureUsage\0"));
        assert!(!references_feature_staging(b"kernel32.dll RtlNotify"));
    }

    #[test]
    fn only_dll_and_exe_are_binaries() {
        assert!(is_binary(Path::new(r"C:\Windows\explorer.EXE")));
        assert!(is_binary(Path::new("shell32.dll")));
        assert!(!is_binary(Path::new("shell32.dll.mui")));
        assert!(!is_binary(Path::new("drivers")));
    }
}
