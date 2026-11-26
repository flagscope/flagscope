//! The only module that reads PDB files (AD-05).
//!
//! Replacing the PDB reader (for example with the `pdb` crate) means replacing
//! this module only.

use std::path::Path;

use ms_pdb::codeview::parser::Parse as _;
use ms_pdb::syms::{Pub, SymKind};

use crate::{Error, descriptor};

/// A feature descriptor symbol found in a PDB.
#[derive(Debug)]
pub(crate) struct DescriptorSymbol {
    /// Feature name taken from the symbol name.
    pub name: String,
    /// 1-based index of the binary's section that holds the descriptor.
    pub segment: u16,
    /// Offset of the descriptor within that section.
    pub offset: u32,
}

/// Returns every feature descriptor symbol among the PDB's public symbols.
pub(crate) fn descriptor_symbols(path: &Path) -> Result<Vec<DescriptorSymbol>, Error> {
    let pdb_error = |error: anyhow::Error| Error::Pdb {
        path: path.to_owned(),
        message: format!("{error:#}"),
    };
    let pdb = ms_pdb::Pdb::open(path).map_err(pdb_error)?;
    let globals = pdb.gss().map_err(pdb_error)?;

    let mut symbols = Vec::new();
    for symbol in globals.iter_syms() {
        if symbol.kind != SymKind::S_PUB32 {
            continue;
        }
        let Ok(public) = Pub::parse(symbol.data) else {
            continue;
        };
        let name = public.name.to_string();
        if let Some(feature) = descriptor::feature_name(&name) {
            let location = public.offset_segment();
            symbols.push(DescriptorSymbol {
                name: feature.to_owned(),
                segment: location.segment.get(),
                offset: location.offset(),
            });
        }
    }
    Ok(symbols)
}
