//! `flagscope`: find the hidden features of the installed Windows build.

mod messages;

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use flagscope_scanner::{Error, ScanOptions};

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
    /// Scan the installed build and print its feature list as `Name: ID` lines.
    Scan,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Scan => scan(),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", messages::error(&error));
            ExitCode::FAILURE
        }
    }
}

/// Runs a scan of the installed build and prints the result.
fn scan() -> Result<(), Error> {
    let options = ScanOptions::for_this_machine()?;
    let build = flagscope_scanner::installed_build()?;
    eprintln!("{}", messages::scanning(&build.to_string()));
    let report = flagscope_scanner::scan_installed_build(&options)?;
    println!("{}", messages::build_line(&report));
    for feature in &report.features {
        println!("{feature}");
    }
    eprintln!("{}", messages::summary(&report));
    Ok(())
}
