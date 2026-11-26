//! Domain types shared by every flagscope crate.

use std::fmt;

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
    fn feature_displays_as_list_entry() {
        let feature = Feature {
            name: "FileExplorerNavpaneFixes".to_owned(),
            id: FeatureId(40_767_182),
        };
        assert_eq!(feature.to_string(), "FileExplorerNavpaneFixes: 40767182");
    }
}
