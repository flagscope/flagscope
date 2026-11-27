//! Saved feature lists, one file per build (RF-07, AD-09), and their copy in
//! the layout of `flagscope/feature-lists` (RF-18, AD-16).

use std::fs;
use std::io::{self, BufReader, BufWriter, Write as _};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use flagscope_core::{Build, Feature, ListHeader};

use crate::Error;

/// Line that marks a saved list as produced by a scan on this machine. It is
/// not copied into contributions.
const SCANNED_HERE: &str = "# Source: scanned on this machine";

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

/// Saves a list produced by a scan on this machine at `path`, with its header
/// and the mark that it was scanned here, replacing any earlier list for the
/// build.
///
/// The list is written to a temporary file first and then renamed, so an
/// interrupted save never leaves a partial list behind (RF-06.3).
pub(crate) fn save(path: &Path, header: &ListHeader, features: &[Feature]) -> Result<(), Error> {
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
        .and_then(|file| {
            let mut file = BufWriter::new(file);
            for line in header.lines() {
                writeln!(file, "{line}")?;
            }
            writeln!(file, "{SCANNED_HERE}")?;
            flagscope_core::write_list(file, features)
        })
        .and_then(|()| fs::rename(&temporary, path));
    if let Err(error) = written {
        let _ = fs::remove_file(&temporary);
        return Err(io_error(error));
    }
    Ok(())
}

/// Copies the saved list at `saved_at` into `directory`, at
/// `<architecture>/<major>/<major>.<revision>.txt`, with the AD-16 header and
/// without the local source mark. Returns the path written.
///
/// # Errors
///
/// Returns [`Error::NoSavedList`] if no list is saved, [`Error::NotScannedHere`]
/// if the saved list has no header or was not scanned on this machine,
/// [`Error::ContributionExists`] if the target file exists, and [`Error::Io`]
/// if a file cannot be read or written.
pub(crate) fn prepare_contribution(saved_at: &Path, directory: &Path) -> Result<PathBuf, Error> {
    let text = match fs::read_to_string(saved_at) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(Error::NoSavedList {
                path: saved_at.to_owned(),
            });
        }
        Err(source) => {
            return Err(Error::Io {
                path: saved_at.to_owned(),
                source,
            });
        }
    };
    let header = ListHeader::parse(&text)
        .filter(|_| text.lines().any(|line| line == SCANNED_HERE))
        .ok_or_else(|| Error::NotScannedHere {
            path: saved_at.to_owned(),
        })?;
    let features = flagscope_core::read_list(text.as_bytes()).map_err(|source| Error::Io {
        path: saved_at.to_owned(),
        source,
    })?;

    let target = directory
        .join(&header.architecture)
        .join(header.build.major.to_string())
        .join(format!("{}.txt", header.build));
    let io_error = |source| Error::Io {
        path: target.clone(),
        source,
    };
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(io_error)?;
    }
    let file = match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&target)
    {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            return Err(Error::ContributionExists { path: target });
        }
        Err(error) => return Err(io_error(error)),
    };
    let mut file = BufWriter::new(file);
    let written = header
        .lines()
        .iter()
        .try_for_each(|line| writeln!(file, "{line}"))
        .and_then(|()| flagscope_core::write_list(file, &features));
    if let Err(error) = written {
        // A partly written contribution must not be submitted.
        let _ = fs::remove_file(&target);
        return Err(io_error(error));
    }
    Ok(target)
}

/// Today's date in UTC, `YYYY-MM-DD`.
pub(crate) fn today() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    utc_date(seconds)
}

/// The UTC date of a Unix time, `YYYY-MM-DD` (proleptic Gregorian calendar).
fn utc_date(seconds: u64) -> String {
    // Days since 1970-01-01 shifted to 0000-03-01, so leap days fall at the end of a year.
    let days = seconds / 86_400 + 719_468;
    let era = days / 146_097;
    let day_of_era = days % 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + u64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use flagscope_core::FeatureId;

    fn header() -> ListHeader {
        ListHeader {
            build: Build {
                major: 26_300,
                revision: 10_075,
            },
            architecture: "x64".to_owned(),
            generator: "flagscope 0.1.0, light scan".to_owned(),
            scanned: "2026-11-26".to_owned(),
        }
    }

    fn temporary(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("flagscope-{name}-{}", std::process::id()))
    }

    #[test]
    fn saved_list_is_loaded_back_and_missing_list_is_none() {
        let directory = temporary("lists-test");
        let file = path(&directory, header().build);
        assert!(file.ends_with("26300.10075.txt"));
        assert_eq!(load(&file).unwrap(), None);

        let features = vec![Feature {
            name: "Alpha".to_owned(),
            id: FeatureId(1),
        }];
        save(&file, &header(), &features).unwrap();
        assert_eq!(load(&file).unwrap(), Some(features));
        assert!(
            !file
                .with_extension(format!("txt.{}.partial", std::process::id()))
                .exists()
        );

        fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn contribution_copies_header_and_entries_without_the_source_mark() {
        let lists = temporary("contribution-lists");
        let out = temporary("contribution-out");
        let saved = path(&lists, header().build);
        let features = vec![Feature {
            name: "Alpha".to_owned(),
            id: FeatureId(1),
        }];
        save(&saved, &header(), &features).unwrap();

        let written = prepare_contribution(&saved, &out).unwrap();
        assert!(written.ends_with(Path::new("x64/26300/26300.10075.txt")));
        let text = fs::read_to_string(&written).unwrap();
        assert_eq!(text, header().lines().join("\n") + "\nAlpha: 1\n");
        assert!(matches!(
            prepare_contribution(&saved, &out),
            Err(Error::ContributionExists { .. })
        ));

        fs::write(&saved, "Alpha: 1\n").unwrap();
        assert!(matches!(
            prepare_contribution(&saved, &temporary("contribution-none")),
            Err(Error::NotScannedHere { .. })
        ));
        fs::remove_dir_all(&lists).unwrap();
        fs::remove_dir_all(&out).unwrap();
    }

    #[test]
    fn utc_dates_follow_the_gregorian_calendar() {
        assert_eq!(utc_date(0), "1970-01-01");
        assert_eq!(utc_date(951_782_400), "2000-02-29");
        assert_eq!(utc_date(1_795_737_599), "2026-11-26");
    }
}
