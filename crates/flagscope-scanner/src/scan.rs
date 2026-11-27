//! The build scan pipeline: choose binaries, download their PDBs, extract and
//! merge features.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError, mpsc};
use std::time::{Duration, Instant};

use flagscope_core::{Build, Feature, FeatureId};
use flagscope_symbols::{PdbReference, SymbolServer};

use crate::pdb_symbols::{self, PartialError, PartialPlan};
use crate::progress::{Estimate, Progress, ScanObserver, expected_duration};
use crate::{
    Error, Extraction, build, candidates, extract_features, lists, read_descriptors, results,
};

/// Concurrent requests while planning a scan (AD-18, AD-19).
const PLANNING_WORKERS: usize = 16;
/// Concurrent symbol files being downloaded and read (AD-19).
const DOWNLOAD_WORKERS: usize = 8;
/// Bytes downloaded once to measure the transfer speed for the estimate.
const SPEED_SAMPLE_BYTES: u64 = 4_000_000;

/// How a scan downloads symbol files (RF-16).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScanMode {
    /// Only the parts of each symbol file the scanner reads, held in memory
    /// (RF-16.1). The default.
    #[default]
    Light,
    /// Complete symbol files, written to disk (RF-16.3).
    Full,
}

/// Where and how a scan runs.
#[derive(Debug, Clone)]
pub struct ScanOptions {
    /// The Windows directory, usually `C:\Windows`.
    pub system_root: PathBuf,
    /// Directory for downloaded symbol files.
    pub symbol_directory: PathBuf,
    /// How symbol files are downloaded.
    pub mode: ScanMode,
    /// File names or full paths of the only binaries to scan (targeted mode,
    /// RF-16.4). Empty scans the whole build.
    pub binaries: Vec<String>,
    /// Keep downloaded symbol files instead of deleting each one once its
    /// features have been extracted (RF-12). Selects full mode.
    pub keep_symbol_files: bool,
    /// Directory of saved lists, one `<build>.txt` per build (AD-09).
    pub lists_directory: PathBuf,
    /// Directory of stored results, one file per distinct symbol file (AD-20).
    pub results_directory: PathBuf,
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
            mode: ScanMode::Light,
            binaries: Vec::new(),
            keep_symbol_files: false,
            lists_directory: data.join("lists"),
            results_directory: data.join("results"),
            server: SymbolServer::microsoft(),
        })
    }
}

impl ScanOptions {
    /// The mode the scan uses: keeping symbol files needs complete files, so
    /// it selects full mode (RF-16.3).
    #[must_use]
    pub fn effective_mode(&self) -> ScanMode {
        if self.keep_symbol_files {
            ScanMode::Full
        } else {
            self.mode
        }
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
    /// Symbol files not downloaded because an earlier scan stored their
    /// result (RF-17.1).
    pub symbol_files_reused: usize,
    /// Total bytes of symbol files downloaded.
    pub downloaded_bytes: u64,
    /// Largest amount of disk space the scan's symbol files took at one time.
    pub peak_disk_bytes: u64,
    /// Time from the first download to the end of the scan.
    pub download_duration: Duration,
    /// Complete symbol files (full mode) whose size differs from the size the
    /// server reported before the download (RF-16.3).
    pub incomplete_files: usize,
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
/// Each distinct symbol file is downloaded once, even when several binaries
/// reference it (RF-16.2), and several are downloaded at a time (AD-19). In
/// light mode only the parts the scanner reads are downloaded, into memory;
/// in full mode complete files are written to disk and deleted after use
/// unless [`ScanOptions::keep_symbol_files`] is set. The scan stops at the
/// first request that fails for any reason other than a symbol file not being
/// published, deletes the files it wrote, and returns no list.
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
    let mode = options.effective_mode();
    let mut stats = ScanStats::default();
    // One directory per process, so that two scans running at once never share files.
    let run_directory = options
        .symbol_directory
        .join(format!("scan-{}", std::process::id()));

    let mut found: BTreeMap<String, BTreeMap<FeatureId, Vec<PathBuf>>> = BTreeMap::new();
    let mut targets = choose_targets(&options.system_root, &options.binaries, &mut stats)?;
    // Kept symbol files must be downloaded, so stored results are only used otherwise.
    if !options.keep_symbol_files {
        targets.retain(|target| {
            let Some(extraction) = results::load(&options.results_directory, &target.reference)
            else {
                return true;
            };
            stats.symbol_files_reused += 1;
            record(&mut found, &mut stats, extraction, &target.binaries);
            false
        });
    }
    let planning = plan_targets(&options.server, &targets, mode)?;
    stats.downloaded_bytes += planning.bytes;
    let mut work = Vec::new();
    for (index, plan) in planning.plans.into_iter().enumerate() {
        match plan {
            Plan::Missing => stats.without_public_pdb += targets[index].binaries.len(),
            plan => work.push((index, plan)),
        }
    }
    let bytes_total: u64 = work.iter().map(|(_, plan)| plan.remaining_bytes()).sum();
    let requests: usize = work.iter().map(|(_, plan)| plan.remaining_requests()).sum();
    let largest = work.iter().max_by_key(|(_, plan)| plan.remaining_bytes());
    let speed = match largest {
        Some((index, _)) => measure_speed(&options.server, &targets[*index].reference)?,
        None => 0.0,
    };
    observer.estimated(&Estimate {
        files: work.len(),
        bytes: bytes_total,
        duration: expected_duration(requests, bytes_total, planning.per_request, speed),
        planning_bytes: planning.bytes,
    });

    let started = Instant::now();
    let files_total = work.len();
    let disk = DiskUse::default();
    let stop = AtomicBool::new(false);
    // Directories this scan wrote, so that a failed scan removes only its own files.
    let mut created: Vec<PathBuf> = Vec::new();
    let mut failure: Option<Error> = None;
    let mut files_done = 0;
    let mut bytes_done = 0;
    let process = |(index, plan): (usize, Plan)| {
        let target: &Target = &targets[index];
        let outcome = process_target(options, target, plan, &run_directory, &disk);
        (index, outcome)
    };
    run_parallel(
        work,
        DOWNLOAD_WORKERS,
        &stop,
        process,
        |(index, outcome)| {
            files_done += 1;
            match outcome {
                Ok(processed) => {
                    stats.symbol_files_downloaded += 1;
                    bytes_done += processed.bytes;
                    created.extend(processed.directory);
                    if processed.complete == Some(false) {
                        stats.incomplete_files += 1;
                    }
                    let target = &targets[index];
                    match processed.extraction {
                        Ok(extraction) => {
                            // A result that cannot be stored is read again next time; the scan goes on.
                            let _ = results::save(
                                &options.results_directory,
                                &target.reference,
                                &extraction,
                            );
                            record(&mut found, &mut stats, extraction, &target.binaries);
                        }
                        Err(Error::Pdb { .. } | Error::Binary { .. }) => {
                            stats.unreadable_pdbs += target.binaries.len();
                        }
                        Err(error) => {
                            failure.get_or_insert(error);
                            stop.store(true, Ordering::Relaxed);
                        }
                    }
                }
                Err(error) => {
                    failure.get_or_insert(error);
                    stop.store(true, Ordering::Relaxed);
                }
            }
            observer.progressed(&Progress {
                files_done,
                files_total,
                bytes_done,
                bytes_total,
                elapsed: started.elapsed(),
            });
        },
    );
    if let Some(error) = failure {
        remove_all(&created, &run_directory, &options.symbol_directory);
        return Err(error);
    }
    stats.downloaded_bytes += bytes_done;
    stats.peak_disk_bytes = disk.peak.load(Ordering::Relaxed);
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

/// Adds the features read from one symbol file, for every binary that
/// references it.
fn record(
    found: &mut BTreeMap<String, BTreeMap<FeatureId, Vec<PathBuf>>>,
    stats: &mut ScanStats,
    extraction: Extraction,
    binaries: &[PathBuf],
) {
    stats.binaries_scanned += binaries.len();
    stats.unreadable_descriptors += extraction.unreadable.len();
    for feature in extraction.features {
        found
            .entry(feature.name)
            .or_default()
            .entry(feature.id)
            .or_default()
            .extend(binaries.iter().cloned());
    }
}

/// One distinct symbol file and the binaries that reference it (RF-16.2).
struct Target {
    reference: PdbReference,
    binaries: Vec<PathBuf>,
}

/// Chooses the binaries to scan and groups them by the symbol file they
/// reference, keeping the order in which each symbol file is first seen.
///
/// With `names`, only the system binaries whose file name or full path equals
/// one of them (ignoring case) are chosen, whether or not they reference the
/// feature-staging API (RF-16.4).
fn choose_targets(
    system_root: &Path,
    names: &[String],
    stats: &mut ScanStats,
) -> Result<Vec<Target>, Error> {
    let mut targets: Vec<Target> = Vec::new();
    let mut by_identity: HashMap<String, usize> = HashMap::new();
    let mut matched = vec![false; names.len()];
    for binary in candidates::system_binaries(system_root) {
        if !names.is_empty() {
            let mut any = false;
            for (name, hit) in names.iter().zip(matched.iter_mut()) {
                if names_binary(name, &binary) {
                    *hit = true;
                    any = true;
                }
            }
            if !any {
                continue;
            }
        }
        stats.binaries_examined += 1;
        let Some(reference) = candidate_reference(&binary, names.is_empty()) else {
            continue;
        };
        match by_identity.get(&reference.server_path()) {
            Some(&index) => targets[index].binaries.push(binary),
            None => {
                by_identity.insert(reference.server_path(), targets.len());
                targets.push(Target {
                    reference,
                    binaries: vec![binary],
                });
            }
        }
    }
    let unmatched: Vec<String> = names
        .iter()
        .zip(&matched)
        .filter(|(_, hit)| !**hit)
        .map(|(name, _)| name.clone())
        .collect();
    if !unmatched.is_empty() {
        return Err(Error::NoMatchingBinary { names: unmatched });
    }
    Ok(targets)
}

/// Whether `name` is the file name or the full path of `binary`, ignoring case
/// and the kind of slash.
fn names_binary(name: &str, binary: &Path) -> bool {
    let name = name.replace('/', "\\");
    let file_name = binary.file_name().and_then(|file| file.to_str());
    file_name.is_some_and(|file| file.eq_ignore_ascii_case(&name))
        || binary
            .to_string_lossy()
            .replace('/', "\\")
            .eq_ignore_ascii_case(&name)
}

/// What planning found out about one symbol file.
enum Plan {
    /// Not published on the server.
    Missing,
    /// Light mode: structure read, remaining blocks listed.
    Partial(PartialPlan),
    /// Full mode, or a file light mode cannot read in part: the whole file,
    /// of this size.
    Whole(u64),
}

impl Plan {
    /// Bytes still to download.
    fn remaining_bytes(&self) -> u64 {
        match self {
            Plan::Missing => 0,
            Plan::Partial(plan) => plan.missing_bytes(),
            Plan::Whole(size) => *size,
        }
    }

    /// Requests still to send.
    fn remaining_requests(&self) -> usize {
        match self {
            Plan::Missing => 0,
            Plan::Partial(plan) => plan.missing_requests(),
            Plan::Whole(_) => 1,
        }
    }
}

/// The plans of all targets, with what planning cost.
struct Planning {
    plans: Vec<Plan>,
    /// Bytes downloaded while planning (light mode reads file headers).
    bytes: u64,
    /// Average time of one request.
    per_request: Duration,
}

/// Plans every target, several at a time: in full mode by asking the server
/// for each size (AD-18), in light mode by reading each file's structure.
fn plan_targets(
    server: &SymbolServer,
    targets: &[Target],
    mode: ScanMode,
) -> Result<Planning, Error> {
    let stop = AtomicBool::new(false);
    let mut plans: Vec<Option<Plan>> = (0..targets.len()).map(|_| None).collect();
    let mut failure: Option<Error> = None;
    let mut busy = Duration::ZERO;
    let mut requests = 0;
    let plan_one = |index: usize| {
        let started = Instant::now();
        let reference = &targets[index].reference;
        let (plan, sent) = match mode {
            ScanMode::Full => (server.size(reference).map(whole_or_missing), 1),
            ScanMode::Light => plan_light(server, reference),
        };
        (index, plan, sent, started.elapsed())
    };
    run_parallel(
        (0..targets.len()).collect(),
        PLANNING_WORKERS,
        &stop,
        plan_one,
        |(index, plan, sent, elapsed)| {
            busy += elapsed;
            requests += sent;
            match plan {
                Ok(plan) => plans[index] = Some(plan),
                Err(error) => {
                    failure.get_or_insert(download_error(error));
                    stop.store(true, Ordering::Relaxed);
                }
            }
        },
    );
    if let Some(error) = failure {
        return Err(error);
    }
    let plans: Vec<Plan> = plans
        .into_iter()
        .map(|plan| plan.unwrap_or(Plan::Missing))
        .collect();
    let bytes = plans
        .iter()
        .map(|plan| match plan {
            Plan::Partial(plan) => plan.planned_bytes(),
            _ => 0,
        })
        .sum();
    let per_request = if requests == 0 {
        Duration::ZERO
    } else {
        busy / u32::try_from(requests).unwrap_or(u32::MAX)
    };
    Ok(Planning {
        plans,
        bytes,
        per_request,
    })
}

/// A size answer as a plan.
fn whole_or_missing(size: Option<u64>) -> Plan {
    size.map_or(Plan::Missing, Plan::Whole)
}

/// Plans one symbol file in light mode. Returns the plan and the number of
/// requests sent.
fn plan_light(
    server: &SymbolServer,
    reference: &PdbReference,
) -> (Result<Plan, flagscope_symbols::Error>, usize) {
    let mut sent = 0;
    let mut fetch = |start, length| {
        sent += 1;
        server.fetch_range(reference, start, length)
    };
    let plan = match pdb_symbols::plan_partial(&mut fetch) {
        Ok(plan) => Ok(Plan::Partial(plan)),
        Err(PartialError::Fetch(flagscope_symbols::Error::Status { status: 404, .. })) => {
            Ok(Plan::Missing)
        }
        Err(PartialError::Fetch(error)) => Err(error),
        // A file light mode cannot read in part is downloaded whole.
        Err(PartialError::NotMsf7 | PartialError::Format) => {
            sent += 1;
            server.size(reference).map(whole_or_missing)
        }
    };
    (plan, sent)
}

/// The result of downloading and reading one symbol file.
struct Processed {
    /// Features read, or why the file could not be read.
    extraction: Result<Extraction, Error>,
    /// Bytes downloaded for it in this step.
    bytes: u64,
    /// Directory written for it, in full mode.
    directory: Option<PathBuf>,
    /// In full mode, whether the file has the size the server reported.
    complete: Option<bool>,
}

/// Downloads and reads one symbol file. An error is returned only for
/// failures that stop the scan.
fn process_target(
    options: &ScanOptions,
    target: &Target,
    plan: Plan,
    run_directory: &Path,
    disk: &DiskUse,
) -> Result<Processed, Error> {
    // All binaries of a target are built from the same sources, so the first one is read.
    let binary = &target.binaries[0];
    match plan {
        Plan::Missing => Ok(Processed {
            extraction: Ok(Extraction::default()),
            bytes: 0,
            directory: None,
            complete: None,
        }),
        Plan::Partial(plan) => {
            let planned = plan.planned_bytes();
            let mut fetch =
                |start, length| options.server.fetch_range(&target.reference, start, length);
            let label = PathBuf::from(target.reference.server_path());
            let pdb = match plan.complete(&mut fetch) {
                Ok(pdb) => pdb,
                Err(PartialError::Fetch(error)) => return Err(download_error(error)),
                Err(PartialError::NotMsf7 | PartialError::Format) => {
                    return Ok(Processed {
                        extraction: Err(Error::Pdb {
                            path: label,
                            message: "inconsistent MSF structure".to_owned(),
                        }),
                        bytes: 0,
                        directory: None,
                        complete: None,
                    });
                }
            };
            let bytes = pdb.held_bytes().saturating_sub(planned);
            let extraction = pdb_symbols::descriptor_symbols_partial(pdb, &label)
                .and_then(|symbols| read_descriptors(symbols, binary));
            Ok(Processed {
                extraction,
                bytes,
                directory: None,
                complete: None,
            })
        }
        Plan::Whole(expected) => {
            let directory = run_directory.join(target.reference.server_path().replace('/', "_"));
            let pdb = options
                .server
                .download(&target.reference, &directory)
                .map_err(download_error)?;
            disk.add(pdb.size);
            let extraction = extract_features(&pdb.path, binary);
            if !options.keep_symbol_files {
                remove_directory(&directory);
                disk.remove(pdb.size);
            }
            Ok(Processed {
                extraction,
                bytes: pdb.size,
                directory: Some(directory),
                complete: Some(pdb.size == expected),
            })
        }
    }
}

/// Disk space taken by the scan's symbol files, now and at its largest.
#[derive(Debug, Default)]
struct DiskUse {
    current: AtomicU64,
    peak: AtomicU64,
}

impl DiskUse {
    /// Records a file written to disk.
    fn add(&self, bytes: u64) {
        let now = self.current.fetch_add(bytes, Ordering::Relaxed) + bytes;
        self.peak.fetch_max(now, Ordering::Relaxed);
    }

    /// Records a file deleted from disk.
    fn remove(&self, bytes: u64) {
        self.current.fetch_sub(bytes, Ordering::Relaxed);
    }
}

/// Runs `work` on every item with `workers` threads, and hands each result to
/// `on_result` on the calling thread, in the order they finish. Workers take
/// no new item once `stop` is set.
fn run_parallel<T, R, W, F>(
    items: Vec<T>,
    workers: usize,
    stop: &AtomicBool,
    work: W,
    mut on_result: F,
) where
    T: Send,
    R: Send,
    W: Fn(T) -> R + Sync,
    F: FnMut(R),
{
    let queue = Mutex::new(items.into_iter());
    let (sender, receiver) = mpsc::channel();
    std::thread::scope(|scope| {
        for _ in 0..workers {
            let sender = sender.clone();
            let (queue, work) = (&queue, &work);
            scope.spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    let next = queue.lock().unwrap_or_else(PoisonError::into_inner).next();
                    let Some(item) = next else { break };
                    if sender.send(work(item)).is_err() {
                        break;
                    }
                }
            });
        }
        drop(sender);
        for result in receiver {
            on_result(result);
        }
    });
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
/// With `require_marker`, only binaries that reference the feature-staging
/// API qualify (AD-13).
fn candidate_reference(
    binary: &Path,
    require_marker: bool,
) -> Option<flagscope_symbols::PdbReference> {
    let contents = fs::read(binary).ok()?;
    if require_marker && !candidates::references_feature_staging(&contents) {
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
    /// Produced by a targeted scan of some binaries only; not saved
    /// (RF-16.4).
    Targeted(Box<ScanReport>),
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
/// saved (RF-07.2). A targeted scan ([`ScanOptions::binaries`]) always scans
/// and never saves (RF-16.4).
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
    if !options.binaries.is_empty() {
        let report = scan_installed_build(options, observer)?;
        return Ok(FeatureList {
            build: report.build,
            features: report.features.clone(),
            saved_at,
            source: ListSource::Targeted(Box::new(report)),
        });
    }
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
    fn binaries_are_named_by_file_name_or_full_path_ignoring_case() {
        let binary = Path::new(r"C:\Windows\explorer.exe");
        assert!(names_binary("EXPLORER.EXE", binary));
        assert!(names_binary(r"c:\windows\explorer.exe", binary));
        assert!(names_binary("C:/Windows/explorer.exe", binary));
        assert!(!names_binary("explorer", binary));
        assert!(!names_binary(r"C:\Windows\System32\explorer.exe", binary));
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
