//! The only module that reads PDB files (AD-05).
//!
//! Replacing the PDB reader (for example with the `pdb` crate) means replacing
//! this module only.
//!
//! A PDB can be read from a complete file, or from a [`PartialPdb`] that holds
//! only the blocks the extraction reads (light mode, RF-16.1).

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::Path;

use ms_pdb::codeview::parser::Parse as _;
use ms_pdb::syms::{Pub, SymKind};

use crate::{Error, descriptor};

/// Signature at the start of an MSF 7.0 file, the container of a PDB.
const MSF7_SIGNATURE: &[u8] = b"Microsoft C/C++ MSF 7.00\r\n\x1aDS\0\0\0";
/// Bytes read first: the MSF header lives in the first block.
const HEADER_BYTES: u64 = 4096;
/// Stream that holds the PDB information; the reader opens it.
const PDB_INFO_STREAM: usize = 1;
/// Stream that holds the debug information header, which names the symbol
/// record stream.
const DBI_STREAM: usize = 3;
/// Offset, in the debug information header, of the symbol record stream index.
const SYMBOL_RECORD_STREAM_OFFSET: usize = 20;

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

/// Returns every feature descriptor symbol among the public symbols of the
/// PDB file at `path`.
pub(crate) fn descriptor_symbols(path: &Path) -> Result<Vec<DescriptorSymbol>, Error> {
    let pdb = ms_pdb::Pdb::open(path).map_err(|error| pdb_error(path, &error))?;
    symbols_of(&pdb, path)
}

/// Returns every feature descriptor symbol among the public symbols of a
/// partly downloaded PDB. `label` names the PDB in error messages.
pub(crate) fn descriptor_symbols_partial(
    pdb: PartialPdb,
    label: &Path,
) -> Result<Vec<DescriptorSymbol>, Error> {
    let pdb = ms_pdb::Pdb::open_from_random_file(pdb).map_err(|error| pdb_error(label, &error))?;
    symbols_of(&pdb, label)
}

/// Converts an error of the PDB reader.
fn pdb_error(path: &Path, error: &anyhow::Error) -> Error {
    Error::Pdb {
        path: path.to_owned(),
        message: format!("{error:#}"),
    }
}

/// Collects the feature descriptor symbols of an opened PDB.
fn symbols_of<F: ms_pdb::ReadAt>(
    pdb: &ms_pdb::Pdb<F>,
    label: &Path,
) -> Result<Vec<DescriptorSymbol>, Error> {
    let globals = pdb.gss().map_err(|error| pdb_error(label, &error))?;
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

/// The parts of a PDB held in memory. Everything else reads as zeros, which
/// the extraction never looks at.
#[derive(Debug, Default)]
pub(crate) struct PartialPdb {
    /// Size of the whole file on the server.
    size: u64,
    /// Downloaded parts, by start offset. They do not overlap.
    chunks: BTreeMap<u64, Vec<u8>>,
}

impl PartialPdb {
    /// Bytes held in memory, which is what was downloaded.
    pub(crate) fn held_bytes(&self) -> u64 {
        self.chunks.values().map(|chunk| chunk.len() as u64).sum()
    }

    /// Reads `length` bytes at `offset` into a new buffer.
    fn read(&self, offset: u64, length: usize) -> Vec<u8> {
        let mut buffer = vec![0; length];
        // Infallible: `read_at` on memory only fills the buffer.
        let _ = ms_pdb::ReadAt::read_at(self, &mut buffer, offset);
        buffer
    }
}

impl ms_pdb::ReadAt for PartialPdb {
    fn read_at(&self, buffer: &mut [u8], offset: u64) -> io::Result<usize> {
        if offset >= self.size {
            return Ok(0);
        }
        let available = usize::try_from(self.size - offset).unwrap_or(usize::MAX);
        let length = buffer.len().min(available);
        let wanted = &mut buffer[..length];
        wanted.fill(0);
        let end = offset + length as u64;
        for (&start, chunk) in self.chunks.range(..end) {
            let chunk_end = start + chunk.len() as u64;
            if chunk_end <= offset {
                continue;
            }
            let from = offset.max(start);
            let to = end.min(chunk_end);
            // Both differences are bounded by `length`, which fits in usize.
            let source = &chunk[(from - start) as usize..(to - start) as usize];
            wanted[(from - offset) as usize..(to - offset) as usize].copy_from_slice(source);
        }
        Ok(length)
    }
}

/// Why a PDB could not be downloaded in part.
#[derive(Debug)]
pub(crate) enum PartialError {
    /// A request failed.
    Fetch(flagscope_symbols::Error),
    /// The file is not an MSF 7.0 container (for example a compressed PDB),
    /// so it must be downloaded whole.
    NotMsf7,
    /// The MSF structures are inconsistent.
    Format,
}

/// Requests one range: `(start, length)` to `(bytes, size of the whole file)`.
pub(crate) type FetchRange<'a> =
    dyn FnMut(u64, u64) -> Result<(Vec<u8>, u64), flagscope_symbols::Error> + 'a;

/// A PDB whose structure is known and whose remaining blocks are listed but
/// not downloaded yet.
#[derive(Debug)]
pub(crate) struct PartialPlan {
    pdb: PartialPdb,
    held: BTreeSet<u32>,
    block_size: u32,
    needed: Vec<u32>,
}

impl PartialPlan {
    /// Bytes downloaded while planning.
    pub(crate) fn planned_bytes(&self) -> u64 {
        self.pdb.held_bytes()
    }

    /// Bytes that completing the PDB will download.
    pub(crate) fn missing_bytes(&self) -> u64 {
        let missing = self
            .needed
            .iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter(|block| !self.held.contains(block))
            .count();
        missing as u64 * u64::from(self.block_size)
    }

    /// Requests that completing the PDB will send.
    pub(crate) fn missing_requests(&self) -> usize {
        let missing: BTreeSet<u32> = self
            .needed
            .iter()
            .copied()
            .filter(|block| !self.held.contains(block))
            .collect();
        runs(&missing).len()
    }

    /// Downloads the remaining blocks and returns the PDB ready to read.
    pub(crate) fn complete(self, fetch: &mut FetchRange<'_>) -> Result<PartialPdb, PartialError> {
        let needed = self.needed;
        let mut blocks = Blocks {
            pdb: self.pdb,
            held: self.held,
            block_size: self.block_size,
            fetch,
        };
        blocks.read(&needed)?;
        Ok(blocks.pdb)
    }
}

/// Reads the structure of a PDB with a few small requests and lists the
/// blocks the extraction reads: the MSF header, the block map and stream
/// directory, the PDB information stream, the first block of the debug
/// information stream, and the symbol record stream.
pub(crate) fn plan_partial(fetch: &mut FetchRange<'_>) -> Result<PartialPlan, PartialError> {
    let (header, size) = fetch(0, HEADER_BYTES).map_err(PartialError::Fetch)?;
    if !header.starts_with(MSF7_SIGNATURE) {
        return Err(PartialError::NotMsf7);
    }
    let field = |offset: usize| -> Result<u32, PartialError> {
        header
            .get(offset..offset + 4)
            .and_then(|bytes| bytes.try_into().ok())
            .map(u32::from_le_bytes)
            .ok_or(PartialError::Format)
    };
    let block_size = field(32)?;
    let directory_bytes = field(44)?;
    let block_map = field(52)?;
    if block_size == 0 || u64::from(block_size) > HEADER_BYTES * 16 {
        return Err(PartialError::Format);
    }

    let mut held = BTreeSet::new();
    let header_blocks = u32::try_from(header.len() as u64 / u64::from(block_size)).unwrap_or(0);
    held.extend(0..header_blocks);
    let mut chunks = BTreeMap::new();
    chunks.insert(0, header);
    let mut blocks = Blocks {
        pdb: PartialPdb { size, chunks },
        held,
        block_size,
        fetch,
    };

    let directory_blocks = directory_bytes.div_ceil(block_size);
    let map = blocks.read(&[block_map])?;
    let directory_block_list = u32_list(&map, 0, directory_blocks)?;
    let mut directory = blocks.read(&directory_block_list)?;
    directory.truncate(usize::try_from(directory_bytes).map_err(|_| PartialError::Format)?);

    let streams = parse_directory(&directory, block_size)?;
    let dbi = streams.get(DBI_STREAM).ok_or(PartialError::Format)?;
    let dbi_header = blocks.read(dbi.get(..1).ok_or(PartialError::Format)?)?;
    let symbol_records = dbi_header
        .get(SYMBOL_RECORD_STREAM_OFFSET..SYMBOL_RECORD_STREAM_OFFSET + 2)
        .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
        .ok_or(PartialError::Format)?;
    let info = streams.get(PDB_INFO_STREAM).ok_or(PartialError::Format)?;
    let records = streams
        .get(usize::from(symbol_records))
        .ok_or(PartialError::Format)?;
    let needed = info.iter().chain(records).copied().collect();
    Ok(PartialPlan {
        pdb: blocks.pdb,
        held: blocks.held,
        block_size,
        needed,
    })
}

/// The block list of every stream in an MSF stream directory.
fn parse_directory(directory: &[u8], block_size: u32) -> Result<Vec<Vec<u32>>, PartialError> {
    let count = u32_list(directory, 0, 1)?[0];
    let sizes = u32_list(directory, 4, count)?;
    let mut offset = 4 + 4 * usize::try_from(count).map_err(|_| PartialError::Format)?;
    let mut streams = Vec::with_capacity(sizes.len());
    for size in sizes {
        // A size of u32::MAX marks a deleted stream, which has no blocks.
        let blocks = if size == u32::MAX {
            0
        } else {
            size.div_ceil(block_size)
        };
        streams.push(u32_list(directory, offset, blocks)?);
        offset += 4 * usize::try_from(blocks).map_err(|_| PartialError::Format)?;
    }
    Ok(streams)
}

/// Reads `count` little-endian `u32` values at `offset`.
fn u32_list(bytes: &[u8], offset: usize, count: u32) -> Result<Vec<u32>, PartialError> {
    let count = usize::try_from(count).map_err(|_| PartialError::Format)?;
    let end = count
        .checked_mul(4)
        .and_then(|length| length.checked_add(offset))
        .ok_or(PartialError::Format)?;
    let slice = bytes.get(offset..end).ok_or(PartialError::Format)?;
    Ok(slice
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| u32::from_le_bytes(*chunk))
        .collect())
}

/// Blocks of a PDB being downloaded in part.
struct Blocks<'a, 'f> {
    pdb: PartialPdb,
    held: BTreeSet<u32>,
    block_size: u32,
    fetch: &'a mut FetchRange<'f>,
}

impl Blocks<'_, '_> {
    /// Returns the contents of `list`, in order, downloading the blocks not
    /// held yet. Runs of consecutive blocks are requested together.
    fn read(&mut self, list: &[u32]) -> Result<Vec<u8>, PartialError> {
        let missing: BTreeSet<u32> = list
            .iter()
            .copied()
            .filter(|block| !self.held.contains(block))
            .collect();
        for (first, last) in runs(&missing) {
            let size = u64::from(self.block_size);
            let start = u64::from(first) * size;
            let length = u64::from(last - first + 1) * size;
            let (bytes, _) = (self.fetch)(start, length).map_err(PartialError::Fetch)?;
            self.pdb.chunks.insert(start, bytes);
            self.held.extend(first..=last);
        }
        let block = self.block_size as usize;
        let mut contents = Vec::with_capacity(list.len() * block);
        for &index in list {
            contents.extend(
                self.pdb
                    .read(u64::from(index) * u64::from(self.block_size), block),
            );
        }
        Ok(contents)
    }
}

/// Groups sorted block numbers into runs of consecutive blocks.
fn runs(blocks: &BTreeSet<u32>) -> Vec<(u32, u32)> {
    let mut runs: Vec<(u32, u32)> = Vec::new();
    for &block in blocks {
        match runs.last_mut() {
            Some((_, last)) if *last + 1 == block => *last = block,
            _ => runs.push((block, block)),
        }
    }
    runs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_group_consecutive_blocks() {
        let blocks: BTreeSet<u32> = [1, 2, 3, 7, 9, 10].into_iter().collect();
        assert_eq!(runs(&blocks), vec![(1, 3), (7, 7), (9, 10)]);
        assert!(runs(&BTreeSet::new()).is_empty());
    }

    #[test]
    fn partial_pdb_reads_held_bytes_and_zeros_elsewhere() {
        let mut pdb = PartialPdb {
            size: 12,
            chunks: BTreeMap::new(),
        };
        pdb.chunks.insert(2, vec![1, 2, 3]);
        pdb.chunks.insert(8, vec![9, 9]);
        assert_eq!(pdb.read(0, 12), vec![0, 0, 1, 2, 3, 0, 0, 0, 9, 9, 0, 0]);
        assert_eq!(pdb.read(3, 2), vec![2, 3]);
        let mut past_end = [7u8; 4];
        assert_eq!(ms_pdb::ReadAt::read_at(&pdb, &mut past_end, 10).unwrap(), 2);
        assert_eq!(pdb.held_bytes(), 5);
    }

    #[test]
    fn non_msf7_files_are_rejected_before_any_other_request() {
        let mut requests = 0;
        let mut fetch = |_: u64, _: u64| {
            requests += 1;
            Ok((
                b"Microsoft C/C++ MSF 7.00\r\n\x1aDS\0\0\0".to_vec()[..10].to_vec(),
                10,
            ))
        };
        assert!(matches!(
            plan_partial(&mut fetch),
            Err(PartialError::NotMsf7)
        ));
        assert_eq!(requests, 1);
    }
}
