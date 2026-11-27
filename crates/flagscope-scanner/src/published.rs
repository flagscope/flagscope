//! Feature lists published in `flagscope/feature-lists` (RF-14, AD-16, AD-23).

use std::io::Read as _;
use std::time::Duration;

use flagscope_core::{Build, Feature, ListHeader};

/// Raw-file base URL of the `main` branch of `flagscope/feature-lists`.
pub const FEATURE_LISTS_URL: &str =
    "https://raw.githubusercontent.com/flagscope/feature-lists/main";
/// Architecture of the lists flagscope reads (RNF-01).
const ARCHITECTURE: &str = "x64";
/// Largest list accepted. Published lists are about 0.2 MB; anything far
/// larger is not a feature list.
const MAX_LIST_BYTES: u64 = 16 * 1024 * 1024;
/// Longest wait for the whole download of a list.
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(60);

/// A published list, checked against the installed build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PublishedList {
    pub header: ListHeader,
    pub features: Vec<Feature>,
}

/// Why a published list could not be used.
#[derive(Debug)]
pub(crate) enum PublishedError {
    /// The repository could not be reached, or answered with an error other
    /// than "not found".
    Unreachable { url: String, reason: String },
    /// The file is not a valid list for this build.
    Invalid { url: String, reason: String },
}

/// URL of the published list of `build`.
pub(crate) fn url(base: &str, build: Build) -> String {
    format!("{base}/{ARCHITECTURE}/{}/{build}.txt", build.major)
}

/// Downloads the published list of `build`. Returns `Ok(None)` when no list is
/// published for it (HTTP 404).
pub(crate) fn fetch(base: &str, build: Build) -> Result<Option<PublishedList>, PublishedError> {
    let url = url(base, build);
    let unreachable = |reason: String| PublishedError::Unreachable {
        url: url.clone(),
        reason,
    };
    // Windows' own TLS stack, as for the symbol server (AD-07).
    let tls = ureq::tls::TlsConfig::builder()
        .provider(ureq::tls::TlsProvider::NativeTls)
        .build();
    let agent = ureq::Agent::config_builder()
        .tls_config(tls)
        .timeout_global(Some(DOWNLOAD_TIMEOUT))
        .build()
        .new_agent();
    let response = match agent.get(&url).call() {
        Ok(response) => response,
        Err(ureq::Error::StatusCode(404)) => return Ok(None),
        Err(error) => return Err(unreachable(error.to_string())),
    };
    let mut bytes = Vec::new();
    response
        .into_body()
        .into_reader()
        .take(MAX_LIST_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| unreachable(error.to_string()))?;
    parse(&bytes, build)
        .map(Some)
        .map_err(|reason| PublishedError::Invalid { url, reason })
}

/// Checks downloaded bytes and turns them into a list for `build`.
///
/// The bytes come from the Internet, so they are accepted only if they are
/// UTF-8, not too large, start with an AD-16 header for this exact build and
/// architecture, and contain only `Name: ID` lines with unique names.
fn parse(bytes: &[u8], build: Build) -> Result<PublishedList, String> {
    if bytes.len() as u64 > MAX_LIST_BYTES {
        return Err(format!("larger than {MAX_LIST_BYTES} bytes"));
    }
    let text = std::str::from_utf8(bytes).map_err(|_| "not UTF-8 text".to_owned())?;
    let header = ListHeader::parse(text).ok_or("no flagscope list header")?;
    if header.build != build {
        return Err(format!(
            "the header names build {}, not {build}",
            header.build
        ));
    }
    if header.architecture != ARCHITECTURE {
        return Err(format!(
            "the header names architecture {}",
            header.architecture
        ));
    }
    let features = flagscope_core::read_list(text.as_bytes()).map_err(|error| error.to_string())?;
    let mut names: Vec<&str> = features
        .iter()
        .map(|feature| feature.name.as_str())
        .collect();
    names.sort_unstable();
    if let Some(pair) = names.windows(2).find(|pair| pair[0] == pair[1]) {
        return Err(format!("the name {} appears twice", pair[0]));
    }
    Ok(PublishedList { header, features })
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUILD: Build = Build {
        major: 26_300,
        revision: 10_075,
    };

    fn list(header_build: &str, architecture: &str, body: &str) -> String {
        format!(
            "# flagscope feature list\n# Build: {header_build}\n# Architecture: {architecture}\n\
             # Generator: flagscope 0.1.0, light scan\n# Scanned: 2026-11-27\n{body}"
        )
    }

    #[test]
    fn url_follows_the_repository_layout() {
        assert_eq!(
            url(FEATURE_LISTS_URL, BUILD),
            "https://raw.githubusercontent.com/flagscope/feature-lists/main/x64/26300/26300.10075.txt"
        );
    }

    #[test]
    fn valid_list_for_the_build_is_accepted() {
        let parsed = parse(
            list("26300.10075", "x64", "Alpha: 1\nBeta: 2\n").as_bytes(),
            BUILD,
        )
        .unwrap();
        assert_eq!(parsed.features.len(), 2);
        assert_eq!(parsed.header.build, BUILD);
    }

    #[test]
    fn lists_that_do_not_match_or_are_malformed_are_rejected() {
        let cases = [
            list("26300.10076", "x64", "Alpha: 1\n"),
            list("26300.10075", "arm64", "Alpha: 1\n"),
            list("26300.10075", "x64", "Alpha 1\n"),
            list("26300.10075", "x64", "Alpha: 1\nAlpha: 2\n"),
            "Alpha: 1\n".to_owned(),
        ];
        for case in cases {
            assert!(parse(case.as_bytes(), BUILD).is_err(), "{case}");
        }
        assert!(parse(&[0xFF, 0xFE], BUILD).is_err());
        let huge = vec![b'a'; usize::try_from(MAX_LIST_BYTES).unwrap() + 1];
        assert!(parse(&huge, BUILD).is_err());
    }
}
