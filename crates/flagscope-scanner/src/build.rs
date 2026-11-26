//! Detection of the running Windows build.

use flagscope_core::Build;

use crate::Error;

/// Registry key that records the installed Windows version.
const VERSION_KEY: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion";

/// Returns the build of the running system, as `winver` reports it.
///
/// # Errors
///
/// Returns [`Error::Build`] if the version cannot be read from the registry.
pub fn installed_build() -> Result<Build, Error> {
    let build_error = |message: String| Error::Build { message };
    let key = windows_registry::LOCAL_MACHINE
        .open(VERSION_KEY)
        .map_err(|error| build_error(error.to_string()))?;
    let major = key
        .get_string("CurrentBuildNumber")
        .map_err(|error| build_error(error.to_string()))?
        .parse()
        .map_err(|error: std::num::ParseIntError| build_error(error.to_string()))?;
    let revision = key
        .get_u32("UBR")
        .map_err(|error| build_error(error.to_string()))?;
    Ok(Build { major, revision })
}
