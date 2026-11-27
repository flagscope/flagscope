//! Every text the command-line interface shows to the user.
//!
//! Kept in one module so that the translation layer can replace it (RNF-02).

use std::path::Path;
use std::time::Duration;

use flagscope_core::Build;
use flagscope_scanner::{Error, Estimate, FeatureList, Progress, ScanMode, ScanReport};

use crate::CliError;

/// Bytes in one megabyte, for display.
const MEGABYTE: f64 = 1_000_000.0;

/// Converts bytes to megabytes for display.
// Display only: exact byte counts are not needed here.
#[allow(clippy::cast_precision_loss)]
fn megabytes(bytes: u64) -> f64 {
    bytes as f64 / MEGABYTE
}

/// A duration for people: `1 h 05 min`, `12 min 30 s`, or `45 s`.
fn duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let (hours, minutes, seconds) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    if hours > 0 {
        format!("{hours} h {minutes:02} min")
    } else if minutes > 0 {
        format!("{minutes} min {seconds:02} s")
    } else {
        format!("{seconds} s")
    }
}

/// Shown once before the first download (RF-13.1).
pub fn estimate(estimate: &Estimate) -> String {
    let planning = if estimate.planning_bytes > 0 {
        format!(
            "Read the structure of the symbol files ({:.1} MB). ",
            megabytes(estimate.planning_bytes)
        )
    } else {
        String::new()
    };
    format!(
        "{planning}About to download {files} symbol files, {size:.1} MB in total. \
         Estimated time: {time} at the current connection speed.",
        files = estimate.files,
        size = megabytes(estimate.bytes),
        time = duration(estimate.duration),
    )
}

/// The progress line while a scan runs (RF-13.2).
pub fn progress(progress: &Progress) -> String {
    let remaining = progress
        .remaining()
        .map_or_else(|| "calculating".to_owned(), duration);
    format!(
        "{percent:5.1}% | {done}/{total} files | {downloaded:.0} of {size:.0} MB | {remaining} left",
        percent = progress.fraction() * 100.0,
        done = progress.files_done,
        total = progress.files_total,
        downloaded = megabytes(progress.bytes_done),
        size = megabytes(progress.bytes_total),
    )
}

/// Actual download size and duration, next to the estimate (RF-13, CA-04.3).
pub fn actual(report: &ScanReport, estimate: Option<&Estimate>) -> String {
    let stats = &report.stats;
    let actual = format!(
        "Downloaded {size:.1} MB in {time}",
        size = megabytes(stats.downloaded_bytes),
        time = duration(stats.download_duration),
    );
    match estimate {
        Some(estimate) => format!(
            "{actual} (estimated: {size:.1} MB in {time}).",
            size = megabytes(estimate.bytes),
            time = duration(estimate.duration),
        ),
        None => format!("{actual}."),
    }
}

/// Shown before the list is loaded or scanned.
pub fn starting(
    build: Build,
    rescan: bool,
    mode: ScanMode,
    keep: bool,
    binaries: &[String],
) -> String {
    if !binaries.is_empty() {
        let mode = match mode {
            ScanMode::Light => "light",
            ScanMode::Full => "full",
        };
        return format!(
            "Targeted {mode} scan of {names} in Windows build {build}. The list covers only these \
             binaries and is not saved as the list of the build.",
            names = binaries.join(", "),
        );
    }
    if rescan {
        let mode = match (mode, keep) {
            (ScanMode::Light, _) => {
                "Light scan: only the needed parts of the symbol files are downloaded."
            }
            (ScanMode::Full, true) => {
                "Full scan: keeping symbol files needs complete files, so they are downloaded whole."
            }
            (ScanMode::Full, false) => "Full scan: complete symbol files are downloaded.",
        };
        format!(
            "Scanning Windows build {build}. This downloads symbol files and can take a while.\n{mode}"
        )
    } else {
        format!(
            "Getting the feature list of Windows build {build}. If no list is saved, this scans \
             the build, which downloads symbol files and can take a while."
        )
    }
}

/// The first line of a result.
pub fn build_line(list: &FeatureList) -> String {
    format!("Windows build {}", list.build)
}

/// Shown after a list that was loaded instead of scanned.
pub fn loaded(list: &FeatureList) -> String {
    format!(
        "{features} features in build {build}, from the saved list {path}.\n\
         Nothing was downloaded. Pass --rescan to scan the build again.",
        features = list.features.len(),
        build = list.build,
        path = list.saved_at.display(),
    )
}

/// Summary printed after a scan.
pub fn summary(
    report: &ScanReport,
    saved_at: Option<&Path>,
    mode: ScanMode,
    symbol_directory: Option<&Path>,
) -> String {
    let stats = &report.stats;
    let files = match (mode, symbol_directory) {
        (ScanMode::Light, _) => format!(
            "Light scan: the needed parts of {count} symbol files were downloaded ({size:.1} MB) \
             and read in memory; nothing was written to disk.",
            count = stats.symbol_files_downloaded,
            size = megabytes(stats.downloaded_bytes),
        ),
        (ScanMode::Full, Some(directory)) => format!(
            "{count} symbol files downloaded ({size:.1} MB) and kept in {path}; {complete}",
            count = stats.symbol_files_downloaded,
            size = megabytes(stats.downloaded_bytes),
            path = directory.display(),
            complete = completeness(stats.symbol_files_downloaded, stats.incomplete_files),
        ),
        (ScanMode::Full, None) => format!(
            "{count} symbol files downloaded ({size:.1} MB) and deleted after use; {complete}",
            count = stats.symbol_files_downloaded,
            size = megabytes(stats.downloaded_bytes),
            complete = completeness(stats.symbol_files_downloaded, stats.incomplete_files),
        ),
    };
    format!(
        "{features} features in build {build}.\n\
         {examined} binaries examined, {scanned} scanned, {missing} without a public symbol file, \
         {unreadable} with an unreadable symbol file.\n\
         {reused} symbol files taken from earlier scans, {downloaded} downloaded.\n\
         {conflicts} names with conflicting IDs (left out of the list), \
         {descriptors} names without a readable descriptor.\n\
         {files}\n\
         Peak disk space used by symbol files: {peak:.1} MB.\n\
         {saved}",
        features = report.features.len(),
        build = report.build,
        examined = stats.binaries_examined,
        scanned = stats.binaries_scanned,
        missing = stats.without_public_pdb,
        unreadable = stats.unreadable_pdbs,
        reused = stats.symbol_files_reused,
        downloaded = stats.symbol_files_downloaded,
        conflicts = report.conflicts.len(),
        descriptors = stats.unreadable_descriptors,
        peak = megabytes(stats.peak_disk_bytes),
        saved = match saved_at {
            Some(path) => format!("List saved to {}.", path.display()),
            None => "Targeted scan: the list covers only the named binaries and was not saved."
                .to_owned(),
        },
    )
}

/// Whether full-mode downloads have the size the server reported (RF-16.3).
fn completeness(files: usize, incomplete: usize) -> String {
    if incomplete == 0 {
        format!("all {files} have the size the server reported.")
    } else {
        format!("{incomplete} of {files} do NOT have the size the server reported.")
    }
}

/// Shown when a command fails.
pub fn error(error: &Error) -> String {
    match error {
        Error::NoSavedList { path } => format!(
            "Error: no list is saved for this build ({}). Run `flagscope scan` first. \
             Nothing was written.",
            path.display()
        ),
        Error::NotScannedHere { path } => format!(
            "Error: the saved list {} does not record a scan on this machine, so it cannot be \
             contributed: flagscope/feature-lists only accepts lists scanned by the contributor. \
             Run `flagscope scan --rescan`, then try again. Nothing was written.",
            path.display()
        ),
        Error::ContributionExists { path } => format!(
            "Error: {} already exists. The list may already be in flagscope/feature-lists; \
             to replace it, delete the file first. Nothing was written.",
            path.display()
        ),
        Error::NoMatchingBinary { names } => format!(
            "Error: no system binary matches {}. Use a file name such as explorer.exe, or a full \
             path under C:\\Windows. Nothing was scanned.",
            names.join(", ")
        ),
        Error::NoConnection { url } => format!(
            "Error: no network connection. Cannot reach Microsoft's symbol server ({url}).\n\
             Check the network connection and run the scan again. No list was saved."
        ),
        other => format!("Error: {other}. No list was saved."),
    }
}

/// Shown after a contribution file was written.
pub fn contribution_written(path: &Path) -> String {
    format!(
        "Contribution written to {}.\n\
         Check it, then commit it with `git commit -s` and open a pull request to \
         flagscope/feature-lists.",
        path.display()
    )
}

/// Shown after the list was written to a file.
pub fn written(path: &Path) -> String {
    format!("List written to {}.", path.display())
}

/// Shown when the command-line interface fails.
pub fn cli_error(error: &CliError) -> String {
    match error {
        CliError::Scan(error) => self::error(error),
        CliError::OutputExists(path) => format!(
            "Error: {} already exists. Choose another file, or pass --overwrite to replace it.\n\
             Nothing was scanned or written.",
            path.display()
        ),
        CliError::Output(path, error) => {
            format!(
                "Error: cannot write {}: {error}. No list was saved.",
                path.display()
            )
        }
    }
}
