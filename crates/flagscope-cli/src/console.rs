//! Shows the estimate and the progress of a scan on the console.

use std::io::{IsTerminal as _, Write as _};
use std::time::{Duration, Instant};

use flagscope_scanner::{Estimate, Progress, ScanObserver};

use crate::messages;

/// How often the progress line is refreshed on an interactive console.
const TERMINAL_REFRESH: Duration = Duration::from_secs(1);
/// How often a progress line is written when the output is redirected.
const LOG_REFRESH: Duration = Duration::from_secs(60);

/// Writes the estimate and the progress to standard error.
pub struct ConsoleProgress {
    /// Whether standard error is an interactive console.
    terminal: bool,
    /// When the progress was last shown.
    last_shown: Option<Instant>,
    /// Whether a progress line is still open on the console.
    line_open: bool,
    /// The estimate, kept to compare it with the result.
    pub estimate: Option<Estimate>,
}

impl ConsoleProgress {
    /// A progress display for the current standard error.
    pub fn new() -> Self {
        Self {
            terminal: std::io::stderr().is_terminal(),
            last_shown: None,
            line_open: false,
            estimate: None,
        }
    }

    /// Ends the progress line, so that the next message starts on a new line.
    pub fn finish(&mut self) {
        if self.line_open {
            eprintln!();
            self.line_open = false;
        }
    }
}

impl ScanObserver for ConsoleProgress {
    fn estimated(&mut self, estimate: &Estimate) {
        self.estimate = Some(*estimate);
        eprintln!("{}", messages::estimate(estimate));
    }

    fn progressed(&mut self, progress: &Progress) {
        let refresh = if self.terminal {
            TERMINAL_REFRESH
        } else {
            LOG_REFRESH
        };
        let done = progress.files_done == progress.files_total;
        if !done
            && self
                .last_shown
                .is_some_and(|shown| shown.elapsed() < refresh)
        {
            return;
        }
        self.last_shown = Some(Instant::now());
        let line = messages::progress(progress);
        if self.terminal {
            // Rewrites the same console line; the padding clears a longer previous line.
            eprint!("\r{line:<100}");
            let _ = std::io::stderr().flush();
            self.line_open = true;
        } else {
            eprintln!("{line}");
        }
    }
}
