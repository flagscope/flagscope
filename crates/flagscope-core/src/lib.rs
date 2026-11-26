//! Domain types shared by every flagscope crate.

use std::fmt;

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

#[cfg(test)]
mod tests {
    use super::*;

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
