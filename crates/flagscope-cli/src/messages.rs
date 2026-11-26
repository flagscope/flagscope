//! Every text the command-line interface shows to the user.
//!
//! Kept in one module so that the translation layer can replace it (RNF-02).

use flagscope_scanner::{Error, ScanReport};

/// Bytes in one megabyte, for display.
const MEGABYTE: f64 = 1_000_000.0;

/// Shown before a scan starts.
pub fn scanning(build: &str) -> String {
    format!("Scanning Windows build {build}. This downloads symbol files and can take a while.")
}

/// The first line of a scan result.
pub fn build_line(report: &ScanReport) -> String {
    format!("Windows build {}", report.build)
}

/// Summary printed after the feature list.
pub fn summary(report: &ScanReport) -> String {
    let stats = &report.stats;
    // Display only: exact byte counts are not needed here.
    #[allow(clippy::cast_precision_loss)]
    let megabytes = stats.downloaded_bytes as f64 / MEGABYTE;
    format!(
        "{features} features in build {build}.\n\
         {examined} binaries examined, {scanned} scanned, {missing} without a public symbol file, \
         {unreadable} with an unreadable symbol file.\n\
         {conflicts} names with conflicting IDs (left out of the list), \
         {descriptors} names without a readable descriptor.\n\
         {megabytes:.1} MB of symbol files downloaded and deleted.",
        features = report.features.len(),
        build = report.build,
        examined = stats.binaries_examined,
        scanned = stats.binaries_scanned,
        missing = stats.without_public_pdb,
        unreadable = stats.unreadable_pdbs,
        conflicts = report.conflicts.len(),
        descriptors = stats.unreadable_descriptors,
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
