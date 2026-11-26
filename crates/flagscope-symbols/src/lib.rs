//! Locating and downloading the debug-symbol (PDB) files of Windows binaries.
//!
//! A PE binary records which PDB it was built with in its CodeView debug
//! directory: the PDB file name, a GUID, and an age. Microsoft's public symbol
//! server stores each PDB under a path built from those three values.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Base URL of Microsoft's public symbol server.
pub const MICROSOFT_SYMBOL_SERVER: &str = "https://msdl.microsoft.com/download/symbols";

/// Errors returned by this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A file could not be read or written.
    #[error("I/O error on {path}: {source}")]
    Io {
        /// The file the operation was working on.
        path: PathBuf,
        /// The underlying error.
        source: io::Error,
    },
    /// The file is not a PE image the parser understands.
    #[error("{path} is not a readable PE image: {source}")]
    Parse {
        /// The file that failed to parse.
        path: PathBuf,
        /// The underlying parser error.
        source: object::Error,
    },
    /// The request to the symbol server failed.
    #[error("download of {url} failed: {source}")]
    Download {
        /// The requested URL.
        url: String,
        /// The underlying HTTP error.
        source: Box<ureq::Error>,
    },
}

/// The identity of the PDB a binary was built with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdbReference {
    /// File name of the PDB, without directories (for example `shell32.pdb`).
    pub file_name: String,
    /// GUID bytes exactly as stored in the CodeView record.
    pub guid: [u8; 16],
    /// Age of the PDB.
    pub age: u32,
}

impl PdbReference {
    /// Path of this PDB on a symbol server: `<name>/<GUID><age>/<name>`.
    ///
    /// The GUID is written as its four fields in hexadecimal, uppercase, without
    /// separators; the age follows in hexadecimal without padding.
    #[must_use]
    pub fn server_path(&self) -> String {
        let g = &self.guid;
        let data1 = u32::from_le_bytes([g[0], g[1], g[2], g[3]]);
        let data2 = u16::from_le_bytes([g[4], g[5]]);
        let data3 = u16::from_le_bytes([g[6], g[7]]);
        let data4: String = g[8..].iter().map(|b| format!("{b:02X}")).collect();
        format!(
            "{name}/{data1:08X}{data2:04X}{data3:04X}{data4}{age:X}/{name}",
            name = self.file_name,
            age = self.age
        )
    }
}

/// Reads the PDB reference from the CodeView debug directory of a PE binary.
///
/// Returns `Ok(None)` when the binary has no CodeView record.
///
/// # Errors
///
/// Returns [`Error::Io`] if the file cannot be read and [`Error::Parse`] if it
/// is not a PE image.
pub fn read_pdb_reference(binary: &Path) -> Result<Option<PdbReference>, Error> {
    use object::Object as _;

    let data = fs::read(binary).map_err(|source| Error::Io {
        path: binary.to_owned(),
        source,
    })?;
    let parse_error = |source| Error::Parse {
        path: binary.to_owned(),
        source,
    };
    let file = object::File::parse(&*data).map_err(parse_error)?;
    let Some(code_view) = file.pdb_info().map_err(parse_error)? else {
        return Ok(None);
    };
    // The record may hold a full build-machine path; the server uses only the file name.
    let path = String::from_utf8_lossy(code_view.path());
    let file_name = path.rsplit(['\\', '/']).next().unwrap_or(&path).to_owned();
    Ok(Some(PdbReference {
        file_name,
        guid: code_view.guid(),
        age: code_view.age(),
    }))
}

/// A symbol file saved on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadedFile {
    /// Where the file was saved.
    pub path: PathBuf,
    /// Size of the file in bytes.
    pub size: u64,
}

/// Downloads the PDB from `server` into `directory`, keeping its file name.
///
/// # Errors
///
/// Returns [`Error::Download`] if the server does not return the file and
/// [`Error::Io`] if it cannot be written.
pub fn download(
    reference: &PdbReference,
    server: &str,
    directory: &Path,
) -> Result<DownloadedFile, Error> {
    let url = format!("{server}/{}", reference.server_path());
    // Windows' own TLS stack (AD-07), so no C toolchain is needed for TLS.
    let tls = ureq::tls::TlsConfig::builder()
        .provider(ureq::tls::TlsProvider::NativeTls)
        .build();
    let agent = ureq::Agent::config_builder()
        .tls_config(tls)
        .build()
        .new_agent();
    let response = agent.get(&url).call().map_err(|source| Error::Download {
        url: url.clone(),
        source: Box::new(source),
    })?;

    let path = directory.join(&reference.file_name);
    let io_error = |source| Error::Io {
        path: path.clone(),
        source,
    };
    fs::create_dir_all(directory).map_err(io_error)?;
    let mut file = fs::File::create(&path).map_err(io_error)?;
    let size = io::copy(&mut response.into_body().into_reader(), &mut file).map_err(io_error)?;
    Ok(DownloadedFile { path, size })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_path_formats_guid_fields_and_age() {
        // GUID {109096DE-2CF2-44BD-8D03-0E5473C5253A}, stored little-endian in its first three fields.
        let reference = PdbReference {
            file_name: "netbios.pdb".to_owned(),
            guid: [
                0xDE, 0x96, 0x90, 0x10, 0xF2, 0x2C, 0xBD, 0x44, 0x8D, 0x03, 0x0E, 0x54, 0x73, 0xC5,
                0x25, 0x3A,
            ],
            age: 1,
        };
        assert_eq!(
            reference.server_path(),
            "netbios.pdb/109096DE2CF244BD8D030E5473C5253A1/netbios.pdb"
        );
    }
}
