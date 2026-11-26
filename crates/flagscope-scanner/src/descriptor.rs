//! Layout of the feature descriptor and of its symbol name (AD-12).

/// Prefix of a feature descriptor's public symbol name.
const NAME_PREFIX: &str = "Feature_";
/// Suffix of a feature descriptor's public symbol name.
const NAME_SUFFIX: &str = "__descriptor";
/// Offset of the 32-bit feature ID inside the descriptor.
const ID_OFFSET: usize = 8;

/// Returns the feature name if `symbol` is a feature descriptor symbol.
pub(crate) fn feature_name(symbol: &str) -> Option<&str> {
    symbol
        .strip_prefix(NAME_PREFIX)?
        .strip_suffix(NAME_SUFFIX)
        .filter(|name| !name.is_empty())
}

/// Reads the feature ID of the descriptor at `offset` in `section`.
///
/// Returns `None` when the descriptor lies outside the section's data.
pub(crate) fn read_id(section: &[u8], offset: u32) -> Option<u32> {
    let start = usize::try_from(offset).ok()?.checked_add(ID_OFFSET)?;
    let bytes = section.get(start..start.checked_add(4)?)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_name_strips_prefix_and_suffix() {
        assert_eq!(
            feature_name("Feature_FileExplorerNavpaneFixes__descriptor"),
            Some("FileExplorerNavpaneFixes")
        );
        assert_eq!(
            feature_name("Feature_Servicing_Fix__descriptor"),
            Some("Servicing_Fix")
        );
    }

    #[test]
    fn feature_name_rejects_other_symbols() {
        assert_eq!(
            feature_name("?ReportUsage@?$FeatureImpl@U__WilFeatureTraits"),
            None
        );
        assert_eq!(feature_name("Feature___descriptor"), None);
        assert_eq!(feature_name("Feature_Name__private_descriptor_x"), None);
    }

    #[test]
    fn read_id_takes_little_endian_value_at_offset_8() {
        let mut section = vec![0u8; 16];
        section.extend_from_slice(&[0xAA; 8]);
        section.extend_from_slice(&57_437_827_u32.to_le_bytes());
        assert_eq!(read_id(&section, 16), Some(57_437_827));
    }

    #[test]
    fn read_id_rejects_descriptors_past_the_end() {
        let section = [0u8; 12];
        assert_eq!(read_id(&section, 0), Some(0));
        assert_eq!(read_id(&section, 1), None);
        assert_eq!(read_id(&section, u32::MAX), None);
    }
}
