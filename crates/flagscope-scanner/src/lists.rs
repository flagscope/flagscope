//! Saved feature lists, one file per build (RF-07, AD-09).

use std::fs;
use std::io::{self, BufReader, BufWriter};
use std::path::{Path, PathBuf};

use flagscope_core::{Build, Feature};

use crate::Error;

/// The file that holds the saved list of `build`.
pub(crate) fn path(directory: &Path, build: Build) -> PathBuf {
    directory.join(format!("{build}.txt"))
}

/// Loads the list saved at `path`, or `None` if there is none.
pub(crate) fn load(path: &Path) -> Result<Option<Vec<Feature>>, Error> {
    let io_error = |source| Error::Io {
        path: path.to_owned(),
        source,
    };
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_error(error)),
    };
    flagscope_core::read_list(BufReader::new(file))
        .map(Some)
        .map_err(io_error)
}

/// Saves `features` at `path`, replacing any earlier list for the build.
///
/// The list is written to a temporary file first and then renamed, so an
/// interrupted save never leaves a partial list behind (RF-06.3).
pub(crate) fn save(path: &Path, features: &[Feature]) -> Result<(), Error> {
    let io_error = |source| Error::Io {
        path: path.to_owned(),
        source,
    };
    if let Some(directory) = path.parent() {
        fs::create_dir_all(directory).map_err(io_error)?;
    }
    // The process ID keeps two scans running at once from writing the same file.
    let temporary = path.with_extension(format!("txt.{}.partial", std::process::id()));
    let written = fs::File::create(&temporary)
        .and_then(|file| flagscope_core::write_list(BufWriter::new(file), features))
        .and_then(|()| fs::rename(&temporary, path));
    if let Err(error) = written {
        let _ = fs::remove_file(&temporary);
        return Err(io_error(error));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use flagscope_core::FeatureId;

    #[test]
    fn saved_list_is_loaded_back_and_missing_list_is_none() {
        let directory =
            std::env::temp_dir().join(format!("flagscope-lists-test-{}", std::process::id()));
        let build = Build {
            major: 26_300,
            revision: 10_075,
        };
        let file = path(&directory, build);
        assert!(file.ends_with("26300.10075.txt"));
        assert_eq!(load(&file).unwrap(), None);

        let features = vec![Feature {
            name: "Alpha".to_owned(),
            id: FeatureId(1),
        }];
        save(&file, &features).unwrap();
        assert_eq!(load(&file).unwrap(), Some(features));
        assert!(
            !file
                .with_extension(format!("txt.{}.partial", std::process::id()))
                .exists()
        );

        fs::remove_dir_all(&directory).unwrap();
    }
}
