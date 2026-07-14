/// A single entry of an MPQ block table, describing the location and
/// size of one file stored inside the archive.
///
/// Retrieved by looking up a file's `file_block_index` in a
/// [`crate::block::BlockTableEntry`].
#[derive(Debug)]
pub struct BlockTableEntry {
    /// Offset of the file's data, relative to the start of the archive
    /// (i.e. relative to the MPQ header, same base as
    /// `MpqHeader::hash_table_position` or `MpqHeader::block_table_position`).
    pub file_pos: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    /// Bit flags describing how the file is stored (compressed,
    /// encrypted, etc.). Not yet decoded into named flags.
    pub flags: u32,
}

impl BlockTableEntry {
    fn from_chunk(chunk: &[u32]) -> Self {
        BlockTableEntry {
            file_pos: chunk[0],
            compressed_size: chunk[1],
            uncompressed_size: chunk[2],
            flags: chunk[3],
        }
    }
}

/// Parses a fully decrypted block table into a list of typed
/// [`BlockTableEntry`] values.
///
/// `decrypted` is expected to contain `4 * block_table_size` `u32` words —
/// one 4-word chunk per entry, as produced by decrypting the raw block
/// table bytes located at `MpqHeader::block_table_position`.
pub fn parse_block_table_entries(decrypted: &[u32]) -> Vec<BlockTableEntry> {
    decrypted
        .chunks(4)
        .map(|chunk| BlockTableEntry::from_chunk(chunk))
        .collect()
}
