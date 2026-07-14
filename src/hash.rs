#[derive(Debug)]
pub struct HashTableEntry {
    pub file_path_hash_a: u32,
    pub file_path_hash_b: u32,
    pub language: u16,
    pub platform: u16,
    pub file_block_index: u32,
}

impl HashTableEntry {
    fn from_chunk(chunk: &[u32]) -> Self {
        let language = chunk[2] as u16;
        let platform = (chunk[2] >> 16) as u16;

        HashTableEntry {
            file_path_hash_a: chunk[0],
            file_path_hash_b: chunk[1],
            file_block_index: chunk[3],
            language,
            platform,
        }
    }
}

pub fn parse_hash_table_entries(decrypted: &[u32]) -> Vec<HashTableEntry> {
    decrypted
        .chunks(4)
        .map(|chunk| HashTableEntry::from_chunk(chunk))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hash_table_entries_from_chunks() {
        let decrypted: Vec<u32> = vec![
            0x1111_1111,
            0x2222_2222,
            0x0000_0000,
            0x0000_0005,
            0xFFFF_FFFF,
            0xFFFF_FFFF,
            0xFFFF_FFFF,
            0xFFFF_FFFF,
        ];

        let entries = parse_hash_table_entries(&decrypted);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].file_path_hash_a, 0x1111_1111);
        assert_eq!(entries[0].file_block_index, 5);
        assert_eq!(entries[1].file_block_index, 0xFFFF_FFFF);
    }
}
