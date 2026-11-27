//! Every text the command-line interface shows to the user.
//!
//! Kept in one module so that the translation layer can replace it (RNF-02).

use std::path::Path;
use std::time::Duration;

use flagscope_core::Build;
use flagscope_scanner::{Error, Estimate, FeatureList, Progress, ScanReport};

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
    format!(
        "About to download {files} symbol files, {size:.1} MB in total. \
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
pub fn starting(build: Build, rescan: bool) -> String {
    if rescan {
        format!("Scanning Windows build {build}. This downloads symbol files and can take a while.")
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
pub fn summary(report: &ScanReport, saved_at: &Path, symbol_directory: Option<&Path>) -> String {
    let stats = &report.stats;
    let files = match symbol_directory {
        Some(directory) => format!(
            "{count} symbol files downloaded ({size:.1} MB) and kept in {path}.",
            count = stats.symbol_files_downloaded,
            size = megabytes(stats.downloaded_bytes),
            path = directory.display(),
        ),
        None => format!(
            "{count} symbol files downloaded ({size:.1} MB); all of them were deleted.",
            count = stats.symbol_files_downloaded,
            size = megabytes(stats.downloaded_bytes),
        ),
    };
    format!(
        "{features} features in build {build}.\n\
         {examined} binaries examined, {scanned} scanned, {missing} without a public symbol file, \
         {unreadable} with an unreadable symbol file.\n\
         {conflicts} names with conflicting IDs (left out of the list), \
         {descriptors} names without a readable descriptor.\n\
         {files}\n\
         Peak disk space used by symbol files: {peak:.1} MB.\n\
         List saved to {saved}.",
        features = report.features.len(),
        build = report.build,
        examined = stats.binaries_examined,
        scanned = stats.binaries_scanned,
        missing = stats.without_public_pdb,
        unreadable = stats.unreadable_pdbs,
        conflicts = report.conflicts.len(),
        descriptors = stats.unreadable_descriptors,
        peak = megabytes(stats.peak_disk_bytes),
        saved = saved_at.display(),
    )
}

/// Shown when a command fails.
pub fn error(error: &Error) -> String {
    match error {
        Error::NoConnection { url } => format!(
            "Error: no network connection. Cannot reach Microsoft's symbol server ({url}).\n\
             Check the network connection and run the scan again. No list was saved."
        ),
        other => format!("Error: {other}. No list was saved."),
    }
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
