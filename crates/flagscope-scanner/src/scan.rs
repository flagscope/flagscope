//! The build scan pipeline: choose binaries, download their PDBs, extract and
//! merge features.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use flagscope_core::{Build, Feature, FeatureId};
use flagscope_symbols::SymbolServer;

use crate::{Error, build, candidates, extract_features};

/// Where and how a scan runs.
#[derive(Debug, Clone)]
pub struct ScanOptions {
    /// The Windows directory, usually `C:\Windows`.
    pub system_root: PathBuf,
    /// Directory for downloaded symbol files. Each file is deleted once its
    /// features have been extracted.
    pub symbol_directory: PathBuf,
    /// The symbol server to download from.
    pub server: SymbolServer,
}

impl ScanOptions {
    /// Options for scanning this machine: `%SystemRoot%`, Microsoft's public
    /// symbol server, and `%LOCALAPPDATA%\flagscope\symbols` for temporary
    /// symbol files (AD-09).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Environment`] if `SystemRoot` or `LOCALAPPDATA` is not set.
    pub fn for_this_machine() -> Result<Self, Error> {
        let variable = |name: &'static str| {
            std::env::var_os(name)
                .map(PathBuf::from)
                .ok_or(Error::Environment { variable: name })
        };
        Ok(Self {
            system_root: variable("SystemRoot")?,
            symbol_directory: variable("LOCALAPPDATA")?.join("flagscope").join("symbols"),
            server: SymbolServer::microsoft(),
        })
    }
}

/// A feature name found with different IDs in different binaries (AD-14).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    /// The feature name.
    pub name: String,
    /// Each ID found, with the binaries it was found in.
    pub ids: BTreeMap<FeatureId, Vec<PathBuf>>,
}

/// Counters describing what a scan did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScanStats {
    /// `.dll` and `.exe` files examined.
    pub binaries_examined: usize,
    /// Binaries whose PDB was downloaded and read.
    pub binaries_scanned: usize,
    /// Candidate binaries whose PDB is not published on the symbol server.
    pub without_public_pdb: usize,
    /// Candidate binaries whose PDB could not be read.
    pub unreadable_pdbs: usize,
    /// Feature names whose descriptor could not be read from the binary.
    pub unreadable_descriptors: usize,
    /// Total bytes of symbol files downloaded.
    pub downloaded_bytes: u64,
}

/// The result of scanning a build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanReport {
    /// The build that was scanned.
    pub build: Build,
    /// Features with a single ID, sorted by name, without duplicate names.
    pub features: Vec<Feature>,
    /// Names found with more than one ID; they are not in `features`.
    pub conflicts: Vec<Conflict>,
    /// What the scan did.
    pub stats: ScanStats,
}

/// Scans the installed build and returns its feature list.
///
/// Nothing is written except temporary symbol files, which are deleted as the
/// scan goes. The scan stops at the first download that fails for any reason
/// other than the PDB not being published, so it never returns a partial list.
///
/// # Errors
///
/// Returns [`Error::Build`] if the build cannot be detected,
/// [`Error::NoConnection`] if the symbol server cannot be reached, and
/// [`Error::Download`] or [`Error::Io`] if a download or temporary file fails.
pub fn scan_installed_build(options: &ScanOptions) -> Result<ScanReport, Error> {
    let build = build::installed_build()?;
    let mut stats = ScanStats::default();
    let mut found: BTreeMap<String, BTreeMap<FeatureId, Vec<PathBuf>>> = BTreeMap::new();

    for binary in candidates::system_binaries(&options.system_root) {
        stats.binaries_examined += 1;
        let Some(reference) = candidate_reference(&binary) else {
            continue;
        };
        // One directory per PDB identity: different binaries can share a PDB file name.
        let directory = options
            .symbol_directory
            .join(reference.server_path().replace('/', "_"));
        let pdb = match options.server.download(&reference, &directory) {
            Ok(pdb) => pdb,
            Err(flagscope_symbols::Error::Status { status: 404, .. }) => {
                stats.without_public_pdb += 1;
                continue;
            }
            Err(flagscope_symbols::Error::Connection { url, .. }) => {
                remove_directory(&options.symbol_directory);
                return Err(Error::NoConnection { url });
            }
            Err(source) => {
                remove_directory(&options.symbol_directory);
                return Err(Error::Download { source });
            }
        };
        stats.downloaded_bytes += pdb.size;
        let extraction = extract_features(&pdb.path, &binary);
        remove_directory(&directory);
        match extraction {
            Ok(extraction) => {
                stats.binaries_scanned += 1;
                stats.unreadable_descriptors += extraction.unreadable.len();
                for feature in extraction.features {
                    found
                        .entry(feature.name)
                        .or_default()
                        .entry(feature.id)
                        .or_default()
                        .push(binary.clone());
                }
            }
            Err(Error::Pdb { .. } | Error::Binary { .. }) => stats.unreadable_pdbs += 1,
            Err(error) => return Err(error),
        }
    }
    // Removes the scan's directory if the scan left it empty.
    let _ = fs::remove_dir(&options.symbol_directory);

    let (features, conflicts) = merge(found);
    Ok(ScanReport {
        build,
        features,
        conflicts,
        stats,
    })
}

/// Returns the PDB reference of `binary` if the scan should download its PDB.
fn candidate_reference(binary: &Path) -> Option<flagscope_symbols::PdbReference> {
    let contents = fs::read(binary).ok()?;
    if !candidates::references_feature_staging(&contents) {
        return None;
    }
    flagscope_symbols::pdb_reference_of_x64(&contents)
        .ok()
        .flatten()
}

/// Deletes a temporary directory; failures are ignored because the scan
/// result does not depend on them.
fn remove_directory(directory: &Path) {
    let _ = fs::remove_dir_all(directory);
}

/// Splits names into single-ID features and conflicts (AD-14).
fn merge(
    found: BTreeMap<String, BTreeMap<FeatureId, Vec<PathBuf>>>,
) -> (Vec<Feature>, Vec<Conflict>) {
    let mut features = Vec::new();
    let mut conflicts = Vec::new();
    for (name, ids) in found {
        let mut single = ids.keys().copied();
        match (single.next(), single.next()) {
            (Some(id), None) => features.push(Feature { name, id }),
            _ => conflicts.push(Conflict { name, ids }),
        }
    }
    (features, conflicts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(entries: &[(&str, u32, &str)]) -> BTreeMap<String, BTreeMap<FeatureId, Vec<PathBuf>>> {
        let mut map: BTreeMap<String, BTreeMap<FeatureId, Vec<PathBuf>>> = BTreeMap::new();
        for (name, id, binary) in entries {
            map.entry((*name).to_owned())
                .or_default()
                .entry(FeatureId(*id))
                .or_default()
                .push(PathBuf::from(binary));
        }
        map
    }

    #[test]
    fn same_name_and_id_in_several_binaries_is_one_feature() {
        let (features, conflicts) = merge(found(&[
            ("Alpha", 1, "a.dll"),
            ("Alpha", 1, "b.dll"),
            ("Beta", 2, "a.dll"),
        ]));
        assert_eq!(
            features,
            vec![
                Feature {
                    name: "Alpha".into(),
                    id: FeatureId(1)
                },
                Feature {
                    name: "Beta".into(),
                    id: FeatureId(2)
                },
            ]
        );
        assert!(conflicts.is_empty());
    }

    #[test]
    fn same_name_with_different_ids_is_a_conflict_not_a_feature() {
        let (features, conflicts) = merge(found(&[("Alpha", 1, "a.dll"), ("Alpha", 7, "b.dll")]));
        assert!(features.is_empty());
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].name, "Alpha");
        assert_eq!(conflicts[0].ids.len(), 2);
    }
}
