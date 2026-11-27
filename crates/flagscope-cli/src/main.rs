//! `flagscope`: find the hidden features of the installed Windows build.

mod messages;

use std::fs::{self, OpenOptions};
use std::io::{self, BufWriter};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use flagscope_core::Feature;
use flagscope_scanner::{ListSource, ScanOptions};

/// Command-line arguments.
#[derive(Debug, Parser)]
#[command(
    name = "flagscope",
    version,
    about = "Find the hidden features of the installed Windows build"
)]
struct Cli {
    /// The command to run.
    #[command(subcommand)]
    command: Command,
}

/// Available commands.
#[derive(Debug, Subcommand)]
enum Command {
    /// Print the feature list of the installed build as `Name: ID` lines.
    ///
    /// Uses the list saved by an earlier scan of the same build; scans the build
    /// only when no list is saved or when asked to.
    Scan {
        /// Also write the list to this file, one `Name: ID` line per feature.
        #[arg(long, value_name = "FILE")]
        output: Option<PathBuf>,
        /// Replace the output file if it already exists.
        #[arg(long, requires = "output")]
        overwrite: bool,
        /// Scan the build even if a list is saved for it, and save the new list.
        #[arg(long)]
        rescan: bool,
        /// Keep the downloaded symbol files instead of deleting them. Implies --rescan.
        #[arg(long)]
        keep_symbol_files: bool,
    },
}

/// Errors of the command-line interface.
#[derive(Debug)]
enum CliError {
    /// The scan failed.
    Scan(flagscope_scanner::Error),
    /// The output file exists and `--overwrite` was not given.
    OutputExists(PathBuf),
    /// The output file could not be written.
    Output(PathBuf, io::Error),
}

impl From<flagscope_scanner::Error> for CliError {
    fn from(error: flagscope_scanner::Error) -> Self {
        Self::Scan(error)
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Scan {
            output,
            overwrite,
            rescan,
            keep_symbol_files,
        } => scan(output.as_deref(), overwrite, rescan, keep_symbol_files),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", messages::cli_error(&error));
            ExitCode::FAILURE
        }
    }
}

/// Gets the feature list of the installed build (saved or scanned), prints it,
/// and writes it to `output` when given.
fn scan(
    output: Option<&Path>,
    overwrite: bool,
    rescan: bool,
    keep_symbol_files: bool,
) -> Result<(), CliError> {
    // Checked before scanning so that a scan of almost an hour is not lost at the end.
    if let Some(path) = output
        && !overwrite
        && path.exists()
    {
        return Err(CliError::OutputExists(path.to_owned()));
    }

    let mut options = ScanOptions::for_this_machine()?;
    options.keep_symbol_files = keep_symbol_files;
    // Keeping symbol files only makes sense for a scan that downloads them.
    let rescan = rescan || keep_symbol_files;
    let build = flagscope_scanner::installed_build()?;
    eprintln!("{}", messages::starting(build, rescan));
    let list = flagscope_scanner::feature_list(&options, rescan)?;

    if let Some(path) = output {
        write_output(path, &list.features, overwrite)?;
    }
    println!("{}", messages::build_line(&list));
    for feature in &list.features {
        println!("{feature}");
    }
    match &list.source {
        ListSource::Saved => eprintln!("{}", messages::loaded(&list)),
        ListSource::Scanned(report) => {
            let kept = keep_symbol_files.then_some(report.symbol_directory.as_path());
            eprintln!("{}", messages::summary(report, &list.saved_at, kept));
        }
    }
    if let Some(path) = output {
        eprintln!("{}", messages::written(path));
    }
    Ok(())
}

/// Writes the list to `path`, refusing to replace an existing file unless
/// `overwrite` is set.
fn write_output(path: &Path, features: &[Feature], overwrite: bool) -> Result<(), CliError> {
    let output_error = |error| CliError::Output(path.to_owned(), error);
    let mut options = OpenOptions::new();
    options.write(true);
    if overwrite {
        options.create(true).truncate(true);
    } else {
        // Fails if the file appeared during the scan.
        options.create_new(true);
    }
    let file = options.open(path).map_err(|error| {
        if error.kind() == io::ErrorKind::AlreadyExists {
            CliError::OutputExists(path.to_owned())
        } else {
            output_error(error)
        }
    })?;
    if let Err(error) = flagscope_core::write_list(BufWriter::new(file), features) {
        // A partly written list is worse than none.
        let _ = fs::remove_file(path);
        return Err(output_error(error));
    }
    Ok(())
}
