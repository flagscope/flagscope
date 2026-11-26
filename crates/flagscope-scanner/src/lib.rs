//! Extraction of feature names and IDs from Windows binaries and their PDBs.
//!
//! Each feature compiled into a binary has a descriptor structure in the
//! binary's read-only data. The binary's public PDB names that structure
//! `Feature_<Name>__descriptor`; the feature ID is stored inside it (AD-12).

mod descriptor;
mod pdb_symbols;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use flagscope_core::{Feature, FeatureId};

/// Errors returned by this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A file could not be read.
    #[error("I/O error on {path}: {source}")]
    Io {
        /// The file the operation was working on.
        path: PathBuf,
        /// The underlying error.
        source: io::Error,
    },
    /// The PDB could not be read.
    #[error("cannot read PDB {path}: {message}")]
    Pdb {
        /// The PDB file.
        path: PathBuf,
        /// Description of the failure reported by the PDB reader.
        message: String,
    },
    /// The binary is not a PE image the parser understands.
    #[error("{path} is not a readable PE image: {source}")]
    Binary {
        /// The binary file.
        path: PathBuf,
        /// The underlying parser error.
        source: object::Error,
    },
}

/// The features found in one binary.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Extraction {
    /// Features with a name and an ID, sorted by name.
    pub features: Vec<Feature>,
    /// Names whose descriptor could not be read from the binary, sorted.
    pub unreadable: Vec<String>,
}

/// Extracts the features of `binary` using its public PDB at `pdb`.
///
/// # Errors
///
/// Returns [`Error::Pdb`] if the PDB cannot be read, [`Error::Io`] if the
/// binary cannot be read, and [`Error::Binary`] if it is not a PE image.
pub fn extract_features(pdb: &Path, binary: &Path) -> Result<Extraction, Error> {
    use object::Object as _;
    use object::ObjectSection as _;

    let symbols = pdb_symbols::descriptor_symbols(pdb)?;

    let data = fs::read(binary).map_err(|source| Error::Io {
        path: binary.to_owned(),
        source,
    })?;
    let image = object::File::parse(&*data).map_err(|source| Error::Binary {
        path: binary.to_owned(),
        source,
    })?;

    let mut extraction = Extraction::default();
    for symbol in symbols {
        // PDB segments and `object` section indices are both 1-based section-table positions.
        let section_data = image
            .section_by_index(object::SectionIndex(usize::from(symbol.segment)))
            .ok()
            .and_then(|section| section.data().ok());
        match section_data.and_then(|bytes| descriptor::read_id(bytes, symbol.offset)) {
            Some(id) => extraction.features.push(Feature {
                name: symbol.name,
                id: FeatureId(id),
            }),
            None => extraction.unreadable.push(symbol.name),
        }
    }
    extraction.features.sort();
    extraction.unreadable.sort();
    Ok(extraction)
}
