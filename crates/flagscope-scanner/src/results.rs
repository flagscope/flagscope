//! Results of reading each distinct symbol file, kept between scans so that
//! an unchanged symbol file is never downloaded again (RF-17.1, AD-20).

use std::fs;
use std::io::{self, BufWriter, Write as _};
use std::path::{Path, PathBuf};

use flagscope_symbols::PdbReference;

use crate::Extraction;

/// First line of a stored result.
const TITLE: &str = "# flagscope symbol file result";
/// Prefix of the lines that list names without a readable descriptor.
const UNREADABLE: &str = "# unreadable: ";

/// The file that holds the result of the symbol file `reference`.
pub(crate) fn path(directory: &Path, reference: &PdbReference) -> PathBuf {
    directory.join(format!("{}.txt", reference.server_path().replace('/', "_")))
}

/// Loads a stored result. Any missing or unreadable file gives `None`, so the
/// symbol file is simply read again.
pub(crate) fn load(directory: &Path, reference: &PdbReference) -> Option<Extraction> {
    let text = fs::read_to_string(path(directory, reference)).ok()?;
    if !text.starts_with(TITLE) {
        return None;
    }
    let unreadable = text
        .lines()
        .filter_map(|line| line.strip_prefix(UNREADABLE))
        .map(str::to_owned)
        .collect();
    let features = flagscope_core::read_list(text.as_bytes()).ok()?;
    Some(Extraction {
        features,
        unreadable,
    })
}

/// Stores a result, writing a temporary file first so that an interrupted
/// write never leaves a partial result.
pub(crate) fn save(
    directory: &Path,
    reference: &PdbReference,
    extraction: &Extraction,
) -> io::Result<()> {
    fs::create_dir_all(directory)?;
    let target = path(directory, reference);
    // The process ID keeps two scans running at once from writing the same file.
    let temporary = target.with_extension(format!("txt.{}.partial", std::process::id()));
    let written = (|| {
        let mut file = BufWriter::new(fs::File::create(&temporary)?);
        writeln!(file, "{TITLE}")?;
        for name in &extraction.unreadable {
            writeln!(file, "{UNREADABLE}{name}")?;
        }
        flagscope_core::write_list(file, &extraction.features)?;
        fs::rename(&temporary, &target)
    })();
    if written.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    written
}

#[cfg(test)]
mod tests {
    use super::*;
    use flagscope_core::{Feature, FeatureId};

    #[test]
    fn stored_result_is_loaded_back() {
        let directory =
            std::env::temp_dir().join(format!("flagscope-results-test-{}", std::process::id()));
        let reference = PdbReference {
            file_name: "shell32.pdb".to_owned(),
            guid: [7; 16],
            age: 1,
        };
        assert_eq!(load(&directory, &reference), None);

        let extraction = Extraction {
            features: vec![Feature {
                name: "Alpha".to_owned(),
                id: FeatureId(1),
            }],
            unreadable: vec!["Beta".to_owned()],
        };
        save(&directory, &reference, &extraction).unwrap();
        assert_eq!(load(&directory, &reference), Some(extraction));

        fs::write(path(&directory, &reference), "not a result\n").unwrap();
        assert_eq!(load(&directory, &reference), None);
        fs::remove_dir_all(&directory).unwrap();
    }
}
