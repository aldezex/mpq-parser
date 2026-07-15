use crate::{
    block::BlockTableEntry,
    crypto::{CRYPT_TABLE_SIZE, MPQ_HASH_NAME_A, MPQ_HASH_NAME_B, hash_string},
    hash::HashTableEntry,
};

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
