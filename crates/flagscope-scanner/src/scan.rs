//! The build scan pipeline: choose binaries, download their PDBs, extract and
//! merge features.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use flagscope_core::{Build, Feature, FeatureId};
use flagscope_symbols::{PdbReference, SymbolServer};

use crate::progress::{Estimate, Progress, ScanObserver, expected_duration};
use crate::{Error, build, candidates, extract_features, lists};

/// Concurrent size requests while estimating (AD-18).
const SIZE_REQUESTS_IN_PARALLEL: usize = 16;
/// Bytes downloaded once to measure the transfer speed for the estimate.
const SPEED_SAMPLE_BYTES: u64 = 4_000_000;

/// Where and how a scan runs.
#[derive(Debug, Clone)]
pub struct ScanOptions {
    /// The Windows directory, usually `C:\Windows`.
    pub system_root: PathBuf,
    /// Directory for downloaded symbol files.
    pub symbol_directory: PathBuf,
    /// Keep downloaded symbol files instead of deleting each one once its
    /// features have been extracted (RF-12).
    pub keep_symbol_files: bool,
    /// Directory of saved lists, one `<build>.txt` per build (AD-09).
    pub lists_directory: PathBuf,
    /// The symbol server to download from.
    pub server: SymbolServer,
}

impl ScanOptions {
    /// Options for scanning this machine: `%SystemRoot%`, Microsoft's public
    /// symbol server, `%LOCALAPPDATA%\flagscope\symbols` for symbol files, and
    /// `%LOCALAPPDATA%\flagscope\lists` for saved lists (AD-09). Symbol files
    /// are deleted after use.
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
        let data = variable("LOCALAPPDATA")?.join("flagscope");
        Ok(Self {
            system_root: variable("SystemRoot")?,
            symbol_directory: data.join("symbols"),
            keep_symbol_files: false,
            lists_directory: data.join("lists"),
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
    /// Symbol files downloaded.
    pub symbol_files_downloaded: usize,
    /// Total bytes of symbol files downloaded.
    pub downloaded_bytes: u64,
    /// Largest amount of disk space the scan's symbol files took at one time.
    pub peak_disk_bytes: u64,
    /// Time from the first download to the end of the scan.
    pub download_duration: Duration,
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
    /// Where this scan stored its symbol files; it only exists afterwards when
    /// [`ScanOptions::keep_symbol_files`] was set.
    pub symbol_directory: PathBuf,
}

/// Scans the installed build and returns its feature list, without reading or
/// saving lists (see [`feature_list`] for that).
///
/// Symbol files are deleted as the scan goes unless
/// [`ScanOptions::keep_symbol_files`] is set. The scan stops at the first
/// download that fails for any reason other than the PDB not being published,
/// deletes the symbol files it downloaded, and returns no list.
///
/// # Errors
///
/// Returns [`Error::Build`] if the build cannot be detected,
/// [`Error::NoConnection`] if the symbol server cannot be reached, and
/// [`Error::Download`] or [`Error::Io`] if a download or temporary file fails.
pub fn scan_installed_build(
    options: &ScanOptions,
    observer: &mut dyn ScanObserver,
) -> Result<ScanReport, Error> {
    let build = build::installed_build()?;
    let mut stats = ScanStats::default();
    let mut found: BTreeMap<String, BTreeMap<FeatureId, Vec<PathBuf>>> = BTreeMap::new();
    // Directories this scan created, so that a failed scan removes only its own files.
    let mut created: Vec<PathBuf> = Vec::new();
    let mut disk_bytes: u64 = 0;
    // One directory per process, so that two scans running at once never share files.
    let run_directory = options
        .symbol_directory
        .join(format!("scan-{}", std::process::id()));

    let mut chosen = Vec::new();
    for binary in candidates::system_binaries(&options.system_root) {
        stats.binaries_examined += 1;
        if let Some(reference) = candidate_reference(&binary) {
            chosen.push((binary, reference));
        }
    }

    let (sizes, per_request) = probe_sizes(&options.server, &chosen)?;
    let mut downloads = Vec::new();
    for ((binary, reference), size) in chosen.into_iter().zip(sizes) {
        match size {
            Some(size) => downloads.push((binary, reference, size)),
            None => stats.without_public_pdb += 1,
        }
    }
    let bytes_total: u64 = downloads.iter().map(|(_, _, size)| size).sum();
    let largest = downloads.iter().max_by_key(|(_, _, size)| *size);
    let speed = match largest {
        Some((_, reference, _)) => measure_speed(&options.server, reference)?,
        None => 0.0,
    };
    observer.estimated(&Estimate {
        files: downloads.len(),
        bytes: bytes_total,
        duration: expected_duration(downloads.len(), bytes_total, per_request, speed),
    });

    let started = Instant::now();
    for (index, (binary, reference, _)) in downloads.iter().enumerate() {
        let directory = run_directory.join(reference.server_path().replace('/', "_"));
        let pdb = match options.server.download(reference, &directory) {
            Ok(pdb) => pdb,
            Err(flagscope_symbols::Error::Status { status: 404, .. }) => {
                stats.without_public_pdb += 1;
                continue;
            }
            Err(error) => {
                remove_all(&created, &run_directory, &options.symbol_directory);
                return Err(download_error(error));
            }
        };
        created.push(directory.clone());
        stats.symbol_files_downloaded += 1;
        stats.downloaded_bytes += pdb.size;
        disk_bytes += pdb.size;
        stats.peak_disk_bytes = stats.peak_disk_bytes.max(disk_bytes);
        let extraction = extract_features(&pdb.path, binary);
        if !options.keep_symbol_files {
            remove_directory(&directory);
            disk_bytes -= pdb.size;
        }
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
            Err(error) => {
                remove_all(&created, &run_directory, &options.symbol_directory);
                return Err(error);
            }
        }
        observer.progressed(&Progress {
            files_done: index + 1,
            files_total: downloads.len(),
            bytes_done: stats.downloaded_bytes,
            bytes_total,
            elapsed: started.elapsed(),
        });
    }
    stats.download_duration = started.elapsed();
    // Removes the scan's directories if nothing is left in them.
    let _ = fs::remove_dir(&run_directory);
    let _ = fs::remove_dir(&options.symbol_directory);

    let (features, conflicts) = merge(found);
    Ok(ScanReport {
        build,
        features,
        conflicts,
        stats,
        symbol_directory: run_directory,
    })
}

/// Asks the server for the size of every chosen PDB, several requests at a
/// time (AD-18). Returns the sizes in the order of `chosen` (`None` for a PDB
/// that is not published) and the average time per request.
fn probe_sizes(
    server: &SymbolServer,
    chosen: &[(PathBuf, PdbReference)],
) -> Result<(Vec<Option<u64>>, Duration), Error> {
    if chosen.is_empty() {
        return Ok((Vec::new(), Duration::ZERO));
    }
    let chunk = chosen.len().div_ceil(SIZE_REQUESTS_IN_PARALLEL);
    let results = std::thread::scope(|scope| {
        let workers: Vec<_> = chosen
            .chunks(chunk)
            .map(|part| {
                scope.spawn(move || {
                    let started = Instant::now();
                    let sizes: Result<Vec<_>, _> = part
                        .iter()
                        .map(|(_, reference)| server.size(reference))
                        .collect();
                    sizes.map(|sizes| (sizes, started.elapsed()))
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|worker| {
                worker
                    .join()
                    .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
            })
            .collect::<Vec<_>>()
    });

    let mut sizes = Vec::with_capacity(chosen.len());
    let mut busy = Duration::ZERO;
    for result in results {
        let (part, elapsed) = result.map_err(download_error)?;
        sizes.extend(part);
        busy += elapsed;
    }
    let per_request = busy / u32::try_from(chosen.len()).unwrap_or(u32::MAX);
    Ok((sizes, per_request))
}

/// Measures the transfer speed, in bytes per second, by downloading the start
/// of one PDB.
fn measure_speed(server: &SymbolServer, reference: &PdbReference) -> Result<f64, Error> {
    let started = Instant::now();
    let bytes = server
        .sample(reference, SPEED_SAMPLE_BYTES)
        .map_err(download_error)?;
    let seconds = started.elapsed().as_secs_f64();
    // Display only: the precision lost converting bytes to f64 does not matter.
    #[allow(clippy::cast_precision_loss)]
    let speed = if seconds > 0.0 {
        bytes as f64 / seconds
    } else {
        0.0
    };
    Ok(speed)
}

/// Maps a failed request to the scan error the user sees.
fn download_error(error: flagscope_symbols::Error) -> Error {
    match error {
        flagscope_symbols::Error::Connection { url, .. } => Error::NoConnection { url },
        source => Error::Download { source },
    }
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

/// Deletes the directories a failed scan created, then its run directory and
/// the symbol directory if they are left empty.
fn remove_all(created: &[PathBuf], run_directory: &Path, symbol_directory: &Path) {
    for directory in created {
        remove_directory(directory);
    }
    let _ = fs::remove_dir(run_directory);
    let _ = fs::remove_dir(symbol_directory);
}

/// Where a feature list came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListSource {
    /// Loaded from the list saved by an earlier scan; nothing was downloaded.
    Saved,
    /// Produced by a scan now, and saved for later runs.
    Scanned(Box<ScanReport>),
}

/// A feature list for the installed build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureList {
    /// The installed build.
    pub build: Build,
    /// The features, sorted by name.
    pub features: Vec<Feature>,
    /// Where the list is saved.
    pub saved_at: PathBuf,
    /// Whether the list was loaded or scanned.
    pub source: ListSource,
}

/// Returns the feature list of the installed build: the saved one when it
/// exists and `rescan` is false (RF-07.1), otherwise a new scan, which is then
/// saved (RF-07.2).
///
/// # Errors
///
/// Returns the errors of [`scan_installed_build`], and [`Error::Io`] if the
/// saved list cannot be read or the new one cannot be saved.
pub fn feature_list(
    options: &ScanOptions,
    rescan: bool,
    observer: &mut dyn ScanObserver,
) -> Result<FeatureList, Error> {
    let build = build::installed_build()?;
    let saved_at = lists::path(&options.lists_directory, build);
    if !rescan && let Some(features) = lists::load(&saved_at)? {
        return Ok(FeatureList {
            build,
            features,
            saved_at,
            source: ListSource::Saved,
        });
    }
    let report = scan_installed_build(options, observer)?;
    lists::save(&saved_at, &report.features)?;
    Ok(FeatureList {
        build: report.build,
        features: report.features.clone(),
        saved_at,
        source: ListSource::Scanned(Box::new(report)),
    })
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
