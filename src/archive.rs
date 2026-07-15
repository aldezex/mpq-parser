use crate::{
    block::BlockTableEntry,
    crypto::{CRYPT_TABLE_SIZE, MPQ_HASH_NAME_A, MPQ_HASH_NAME_B, hash_string},
    hash::HashTableEntry,
};
use std::io::Read;

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

pub fn decompress_zlib(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decoder = flate2::read::ZlibDecoder::new(data);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result)?;
    Ok(result)
}

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
        decompress_zlib(&file_bytes[1..])
    }
}
