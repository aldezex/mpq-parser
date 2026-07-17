use crate::{
    block::BlockTableEntry,
    crypto::{CRYPT_TABLE_SIZE, MPQ_HASH_NAME_A, MPQ_HASH_NAME_B, hash_string},
    hash::HashTableEntry,
};
use std::io::Read;

/// Looks up a file by name within an MPQ archive, using its already
/// decrypted hash table and block table.
///
/// Computes both name hashes (`MPQ_HASH_NAME_A` and `MPQ_HASH_NAME_B`) for
/// `name` and scans `hash_entries` for a matching pair, skipping empty
/// slots (`file_block_index == 0xFFFFFFFF`) explicitly rather than relying
/// on the near-impossible chance that a real hash collides with that
/// sentinel value.
///
/// Returns `None` if no entry matches, or if the matching entry's
/// `file_block_index` falls outside `block_entries` (which would indicate
/// a corrupt or unsupported archive).
pub fn find_file<'a>(
    name: &str,
    hash_entries: &[HashTableEntry],
    block_entries: &'a [BlockTableEntry],
    crypt_table: &[u32; CRYPT_TABLE_SIZE],
) -> Option<&'a BlockTableEntry> {
    let hash_a = hash_string(name, MPQ_HASH_NAME_A, crypt_table);
    let hash_b = hash_string(name, MPQ_HASH_NAME_B, crypt_table);

    for entry in hash_entries {
        if entry.file_block_index == 0xFFFFFFFF {
            continue;
        };

        if entry.file_path_hash_a == hash_a && entry.file_path_hash_b == hash_b {
            let index = entry.file_block_index as usize;
            return block_entries.get(index);
        }
    }

    None
}

fn decompress_zlib(data: &[u8], uncompressed_size_hint: usize) -> std::io::Result<Vec<u8>> {
    let mut decoder = flate2::read::ZlibDecoder::new(data);
    let mut result = Vec::with_capacity(uncompressed_size_hint);
    decoder.read_to_end(&mut result)?;
    Ok(result)
}

fn decompress_bzip2(data: &[u8], uncompressed_size_hint: usize) -> std::io::Result<Vec<u8>> {
    let mut decoder = bzip2::read::BzDecoder::new(data);
    let mut result = Vec::with_capacity(uncompressed_size_hint);
    decoder.read_to_end(&mut result)?;
    Ok(result)
}

/// Decompresses a block of MPQ file data using the algorithm indicated by
/// `compression_flag` (the first byte of a compressed MPQ file block).
///
/// `uncompressed_size_hint` pre-sizes the output buffer — MPQ block
/// entries always carry the exact uncompressed size, so callers going
/// through [`extract_file`] get a single exact allocation instead of
/// `read_to_end`'s repeated grow-and-copy cycles (a real cost for
/// multi-hundred-KB event streams decompressed once per replay in batch
/// workloads). Passing `0` degrades gracefully to the old
/// grow-as-needed behavior; a wrong hint affects only performance,
/// never correctness.
///
/// Currently supported: `0x02` (zlib) and `0x10` (bzip2). Other MPQ
/// compression methods (e.g. PKWare implode, LZMA) are not yet
/// implemented and return an error.
pub fn decompress(
    compression_flag: u8,
    data: &[u8],
    uncompressed_size_hint: usize,
) -> std::io::Result<Vec<u8>> {
    match compression_flag {
        0x02 => decompress_zlib(data, uncompressed_size_hint),
        0x10 => decompress_bzip2(data, uncompressed_size_hint),
        other => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("unsupported compression method: {:#04x}", other),
        )),
    }
}

/// Extracts and, if necessary, decompresses the contents of a single file
/// stored inside an MPQ archive.
///
/// `mpq_header_offset` must be the offset of the MPQ header within
/// `replay_bytes` (i.e. [`MpqUserDataHeader::header_offset`]), since
/// `block_entry.file_pos` is relative to that header, not to the start
/// of the file.
///
/// If `block_entry.compressed_size` equals `block_entry.uncompressed_size`,
/// the file is stored as-is and is returned unchanged. Otherwise, the
/// first byte of the stored data is a compression-method flag (see
/// [`decompress`]), followed by the actual compressed stream.
pub fn extract_file(
    replay_bytes: &[u8],
    mpq_header_offset: u32,
    block_entry: BlockTableEntry,
) -> std::io::Result<Vec<u8>> {
    let start = (mpq_header_offset + block_entry.file_pos) as usize;
    let file_bytes = replay_bytes
        .get(start..start + block_entry.compressed_size as usize)
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "Error getting file bytes",
        ))?;

    if block_entry.compressed_size == block_entry.uncompressed_size {
        Ok(file_bytes.to_vec())
    } else {
        decompress(
            file_bytes[0],
            &file_bytes[1..],
            block_entry.uncompressed_size as usize,
        )
    }
}
