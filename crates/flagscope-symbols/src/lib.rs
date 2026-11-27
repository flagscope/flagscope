//! Locating and downloading the debug-symbol (PDB) files of Windows binaries.
//!
//! A PE binary records which PDB it was built with in its CodeView debug
//! directory: the PDB file name, a GUID, and an age. Microsoft's public symbol
//! server stores each PDB under a path built from those three values.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Longest wait for the first answer of the server before a scan.
pub const REACHABILITY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

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
    /// The server answered, but not with the file (for example HTTP 404 when
    /// Microsoft does not publish the PDB).
    #[error("{url} returned HTTP status {status}")]
    Status {
        /// The requested URL.
        url: String,
        /// The HTTP status code.
        status: u16,
    },
    /// The server could not be reached: no network, DNS failure, refused or
    /// interrupted connection, or timeout.
    #[error("cannot reach {url}: {source}")]
    Connection {
        /// The requested URL.
        url: String,
        /// The underlying transport error.
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
    let data = fs::read(binary).map_err(|source| Error::Io {
        path: binary.to_owned(),
        source,
    })?;
    let file = object::File::parse(&*data).map_err(|source| Error::Parse {
        path: binary.to_owned(),
        source,
    })?;
    code_view_reference(&file).map_err(|source| Error::Parse {
        path: binary.to_owned(),
        source,
    })
}

/// Returns the PDB reference of an x64 PE image given its contents.
///
/// Returns `Ok(None)` for images of other architectures and for images
/// without a CodeView record.
///
/// # Errors
///
/// Returns the parser error if `contents` is not a readable PE image.
pub fn pdb_reference_of_x64(contents: &[u8]) -> Result<Option<PdbReference>, object::Error> {
    use object::Object as _;

    let file = object::File::parse(contents)?;
    if file.architecture() != object::Architecture::X86_64 {
        return Ok(None);
    }
    code_view_reference(&file)
}

/// Reads the CodeView record of a parsed image.
fn code_view_reference(file: &object::File<'_>) -> Result<Option<PdbReference>, object::Error> {
    use object::Object as _;

    let Some(code_view) = file.pdb_info()? else {
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

/// A symbol server client. One client reuses its connections across downloads.
#[derive(Debug, Clone)]
pub struct SymbolServer {
    base_url: String,
    agent: ureq::Agent,
}

impl SymbolServer {
    /// A client for the server at `base_url` (without a trailing slash).
    #[must_use]
    pub fn new(base_url: &str) -> Self {
        // Windows' own TLS stack (AD-07).
        let tls = ureq::tls::TlsConfig::builder()
            .provider(ureq::tls::TlsProvider::NativeTls)
            .build();
        let agent = ureq::Agent::config_builder()
            .tls_config(tls)
            .build()
            .new_agent();
        Self {
            base_url: base_url.to_owned(),
            agent,
        }
    }

    /// A client for [`MICROSOFT_SYMBOL_SERVER`].
    #[must_use]
    pub fn microsoft() -> Self {
        Self::new(MICROSOFT_SYMBOL_SERVER)
    }

    /// Checks that the server answers at all, waiting at most
    /// [`REACHABILITY_TIMEOUT`]. Any HTTP answer, even an error status, counts.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the server cannot be reached.
    pub fn check_reachable(&self) -> Result<(), Error> {
        let url = format!("{}/", self.base_url);
        let answer = self
            .agent
            .head(&url)
            .config()
            .timeout_global(Some(REACHABILITY_TIMEOUT))
            .build()
            .call();
        match answer {
            Ok(_) | Err(ureq::Error::StatusCode(_)) => Ok(()),
            Err(source) => Err(Error::Connection {
                url,
                source: Box::new(source),
            }),
        }
    }

    /// Returns the size in bytes of the PDB on the server, without downloading
    /// it, or `None` if the server does not publish it (HTTP 404).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Status`] for other unexpected answers and
    /// [`Error::Connection`] if the server cannot be reached.
    pub fn size(&self, reference: &PdbReference) -> Result<Option<u64>, Error> {
        let url = format!("{}/{}", self.base_url, reference.server_path());
        let response = match self.agent.head(&url).call() {
            Ok(response) => response,
            Err(ureq::Error::StatusCode(404)) => return Ok(None),
            Err(source) => return Err(request_error(&url, source)),
        };
        Ok(response
            .headers()
            .get("content-length")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse().ok()))
    }

    /// Downloads `length` bytes of the PDB starting at `start`, and returns
    /// them with the size of the whole file.
    ///
    /// When the server ignores the range and sends the whole file, the
    /// requested part is cut from it.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Status`] if the server answers without the file and
    /// [`Error::Connection`] if it cannot be reached or the transfer fails.
    pub fn fetch_range(
        &self,
        reference: &PdbReference,
        start: u64,
        length: u64,
    ) -> Result<(Vec<u8>, u64), Error> {
        let url = format!("{}/{}", self.base_url, reference.server_path());
        let range = format!("bytes={start}-{}", start + length.saturating_sub(1));
        let response = self
            .agent
            .get(&url)
            .header("Range", &range)
            .call()
            .map_err(|source| request_error(&url, source))?;
        let partial = response.status().as_u16() == 206;
        // "bytes <first>-<last>/<total>"
        let total = response
            .headers()
            .get("content-range")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.rsplit('/').next())
            .and_then(|value| value.parse::<u64>().ok());
        let mut body = Vec::new();
        io::copy(&mut response.into_body().into_reader(), &mut body).map_err(|source| {
            Error::Connection {
                url,
                source: Box::new(ureq::Error::Io(source)),
            }
        })?;
        if partial {
            let total = total.unwrap_or(start + body.len() as u64);
            return Ok((body, total));
        }
        let total = body.len() as u64;
        let first = usize::try_from(start.min(total)).unwrap_or(usize::MAX);
        let last = usize::try_from((start + length).min(total)).unwrap_or(usize::MAX);
        Ok((body[first..last].to_vec(), total))
    }

    /// Downloads the first `length` bytes of the PDB and discards them, to
    /// measure the transfer speed. Returns the number of bytes received.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Status`] if the server answers without the file and
    /// [`Error::Connection`] if it cannot be reached or the transfer fails.
    pub fn sample(&self, reference: &PdbReference, length: u64) -> Result<u64, Error> {
        let url = format!("{}/{}", self.base_url, reference.server_path());
        let range = format!("bytes=0-{}", length.saturating_sub(1));
        let response = self
            .agent
            .get(&url)
            .header("Range", &range)
            .call()
            .map_err(|source| request_error(&url, source))?;
        io::copy(&mut response.into_body().into_reader(), &mut io::sink()).map_err(|source| {
            Error::Connection {
                url,
                source: Box::new(ureq::Error::Io(source)),
            }
        })
    }

    /// Downloads the PDB into `directory`, keeping its file name.
    ///
    /// A partially written file is removed when the download fails.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Status`] if the server answers without the file,
    /// [`Error::Connection`] if it cannot be reached, and [`Error::Io`] if the
    /// file cannot be written or the transfer is interrupted.
    pub fn download(
        &self,
        reference: &PdbReference,
        directory: &Path,
    ) -> Result<DownloadedFile, Error> {
        let url = format!("{}/{}", self.base_url, reference.server_path());
        let response = self
            .agent
            .get(&url)
            .call()
            .map_err(|source| request_error(&url, source))?;

        let path = directory.join(&reference.file_name);
        let io_error = |source| Error::Io {
            path: path.clone(),
            source,
        };
        fs::create_dir_all(directory).map_err(io_error)?;
        let mut file = fs::File::create(&path).map_err(io_error)?;
        match io::copy(&mut response.into_body().into_reader(), &mut file) {
            Ok(size) => Ok(DownloadedFile { path, size }),
            Err(source) => {
                drop(file);
                // Best effort: the caller is told about the original failure either way.
                let _ = fs::remove_file(&path);
                Err(io_error(source))
            }
        }
    }
}

/// Classifies a failed request: an HTTP status from the server, or a failure
/// to reach it.
fn request_error(url: &str, source: ureq::Error) -> Error {
    match source {
        ureq::Error::StatusCode(status) => Error::Status {
            url: url.to_owned(),
            status,
        },
        source => Error::Connection {
            url: url.to_owned(),
            source: Box::new(source),
        },
    }
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
