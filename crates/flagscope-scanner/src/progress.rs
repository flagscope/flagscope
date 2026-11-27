//! Estimate and progress of a scan, reported through an observer (AD-10).

use std::time::Duration;

/// What a scan expects to download, computed before the first download.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Estimate {
    /// Symbol files to download.
    pub files: usize,
    /// Total bytes of those files.
    pub bytes: u64,
    /// Expected duration of the downloads.
    pub duration: Duration,
    /// Bytes already downloaded to plan the scan: in light mode, the headers
    /// and stream directories of the symbol files. Not part of `bytes`.
    pub planning_bytes: u64,
}

/// How far a scan has got.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    /// Symbol files handled so far.
    pub files_done: usize,
    /// Symbol files the scan will handle.
    pub files_total: usize,
    /// Bytes downloaded so far.
    pub bytes_done: u64,
    /// Bytes the scan will download.
    pub bytes_total: u64,
    /// Time since the first download started.
    pub elapsed: Duration,
}

impl Progress {
    /// Share of the work done, from 0.0 to 1.0. Files and bytes weigh the same,
    /// because per-file request time and transfer time both matter.
    #[must_use]
    pub fn fraction(&self) -> f64 {
        let ratio = |done: f64, total: f64| if total > 0.0 { done / total } else { 1.0 };
        // Display only: the precision lost converting counts to f64 does not matter.
        #[allow(clippy::cast_precision_loss)]
        let files = ratio(self.files_done as f64, self.files_total as f64);
        #[allow(clippy::cast_precision_loss)]
        let bytes = ratio(self.bytes_done as f64, self.bytes_total as f64);
        f64::midpoint(files, bytes).clamp(0.0, 1.0)
    }

    /// Estimated time remaining, recalculated from the speed so far (RF-13.2).
    /// `None` until some work is done.
    #[must_use]
    pub fn remaining(&self) -> Option<Duration> {
        let fraction = self.fraction();
        if fraction <= 0.0 {
            return None;
        }
        Some(self.elapsed.mul_f64((1.0 - fraction) / fraction))
    }
}

/// Receives the estimate and the progress of a scan.
pub trait ScanObserver {
    /// Called once, after the scan has chosen its binaries and asked the
    /// server for the sizes, before the first download.
    fn estimated(&mut self, estimate: &Estimate);
    /// Called after each symbol file has been handled.
    fn progressed(&mut self, progress: &Progress);
}

/// An observer that ignores everything, for callers that need no progress.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoObserver;

impl ScanObserver for NoObserver {
    fn estimated(&mut self, _estimate: &Estimate) {}
    fn progressed(&mut self, _progress: &Progress) {}
}

/// Expected duration of downloading `files` files of `bytes` bytes in total,
/// from the measured time per request and transfer speed.
pub(crate) fn expected_duration(
    files: usize,
    bytes: u64,
    per_request: Duration,
    bytes_per_second: f64,
) -> Duration {
    let requests = per_request.saturating_mul(u32::try_from(files).unwrap_or(u32::MAX));
    if bytes_per_second <= 0.0 {
        return requests;
    }
    // Display only: the precision lost converting bytes to f64 does not matter.
    #[allow(clippy::cast_precision_loss)]
    let transfer = Duration::from_secs_f64(bytes as f64 / bytes_per_second);
    requests + transfer
}

#[cfg(test)]
mod tests {
    use super::*;

    fn progress(files_done: usize, bytes_done: u64, elapsed: u64) -> Progress {
        Progress {
            files_done,
            files_total: 10,
            bytes_done,
            bytes_total: 1_000,
            elapsed: Duration::from_secs(elapsed),
        }
    }

    #[test]
    fn fraction_weighs_files_and_bytes_equally() {
        assert!((progress(5, 500, 10).fraction() - 0.5).abs() < 1e-9);
        assert!((progress(10, 0, 10).fraction() - 0.5).abs() < 1e-9);
        assert!((progress(10, 1_000, 10).fraction() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn remaining_follows_the_speed_so_far() {
        assert_eq!(progress(0, 0, 0).remaining(), None);
        assert_eq!(
            progress(5, 500, 10).remaining(),
            Some(Duration::from_secs(10))
        );
        assert_eq!(progress(10, 1_000, 30).remaining(), Some(Duration::ZERO));
    }

    #[test]
    fn expected_duration_adds_request_time_and_transfer_time() {
        let duration = expected_duration(10, 1_000_000, Duration::from_millis(400), 100_000.0);
        assert_eq!(duration, Duration::from_secs(14));
        assert_eq!(
            expected_duration(2, 10, Duration::from_secs(1), 0.0),
            Duration::from_secs(2)
        );
    }
}
