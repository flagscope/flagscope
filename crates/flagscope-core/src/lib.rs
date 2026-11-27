//! Domain types shared by every flagscope crate.

use std::fmt;
use std::io;

/// A Windows build, identified as `major.revision` (for example `26300.10075`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Build {
    /// Build number (`CurrentBuildNumber`), for example `26300`.
    pub major: u32,
    /// Update build revision (`UBR`), for example `10075`.
    pub revision: u32,
}

impl fmt::Display for Build {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.revision)
    }
}

/// Identifier of a feature in the Windows feature configuration store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FeatureId(pub u32);

impl fmt::Display for FeatureId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A feature: its name as published in Microsoft's symbols, and its ID.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Feature {
    /// Feature name, for example `FileExplorerNavpaneFixes`.
    pub name: String,
    /// Feature ID in the configuration store.
    pub id: FeatureId,
}

impl fmt::Display for Feature {
    /// Formats the feature as one entry of a feature list: `Name: ID`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.id)
    }
}

/// Writes `features` in the mach2 list format: one `Name: ID` line each, in
/// the given order.
///
/// # Errors
///
/// Returns the first error reported by `writer`.
pub fn write_list<W: io::Write>(mut writer: W, features: &[Feature]) -> io::Result<()> {
    for feature in features {
        writeln!(writer, "{feature}")?;
    }
    writer.flush()
}

/// Reads a list in the mach2 format: one `Name: ID` line per feature.
///
/// Empty lines and lines that start with `#` are skipped, as are leading and
/// trailing spaces.
///
/// # Errors
///
/// Returns the reader's error, or [`io::ErrorKind::InvalidData`] naming the
/// first line that is not `Name: ID`.
pub fn read_list<R: io::BufRead>(reader: R) -> io::Result<Vec<Feature>> {
    let mut features = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let feature = line
            .split_once(": ")
            .and_then(|(name, id)| {
                let id = id.trim().parse().ok()?;
                (!name.is_empty()).then(|| Feature {
                    name: name.to_owned(),
                    id: FeatureId(id),
                })
            })
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("line {} is not `Name: ID`: {line}", index + 1),
                )
            })?;
        features.push(feature);
    }
    Ok(features)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_list_writes_one_line_per_feature_in_order() {
        let features = [
            Feature {
                name: "Beta".to_owned(),
                id: FeatureId(2),
            },
            Feature {
                name: "Alpha".to_owned(),
                id: FeatureId(1),
            },
        ];
        let mut out = Vec::new();
        write_list(&mut out, &features).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "Beta: 2\nAlpha: 1\n");
    }

    #[test]
    fn read_list_reads_what_write_list_writes() {
        let features = vec![
            Feature {
                name: "Alpha".to_owned(),
                id: FeatureId(1),
            },
            Feature {
                name: "Beta".to_owned(),
                id: FeatureId(40_767_182),
            },
        ];
        let mut out = Vec::new();
        write_list(&mut out, &features).unwrap();
        assert_eq!(read_list(out.as_slice()).unwrap(), features);
    }

    #[test]
    fn read_list_skips_comments_blank_lines_and_indentation() {
        let text = "# Build: 26300.10075\n\n  Alpha: 1\nBeta: 2\n";
        let features = read_list(text.as_bytes()).unwrap();
        assert_eq!(features.len(), 2);
        assert_eq!(features[0].name, "Alpha");
    }

    #[test]
    fn read_list_rejects_malformed_lines() {
        for text in ["Alpha 1\n", "Alpha: x\n", ": 1\n", "Alpha: 99999999999\n"] {
            let error = read_list(text.as_bytes()).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidData, "{text}");
        }
    }

    #[test]
    fn build_displays_as_major_dot_revision() {
        let build = Build {
            major: 26_300,
            revision: 10_075,
        };
        assert_eq!(build.to_string(), "26300.10075");
    }

    #[test]
    fn feature_displays_as_list_entry() {
        let feature = Feature {
            name: "FileExplorerNavpaneFixes".to_owned(),
            id: FeatureId(40_767_182),
        };
        assert_eq!(feature.to_string(), "FileExplorerNavpaneFixes: 40767182");
    }
}
