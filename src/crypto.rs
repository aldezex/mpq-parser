const CRYPT_TABLE_SEED: u32 = 0x0010_0001;
const CRYPT_TABLE_SIZE: usize = 0x500;
pub const MPQ_HASH_TABLE_OFFSET: u32 = 0;
pub const MPQ_HASH_NAME_A: u32 = 1;
pub const MPQ_HASH_NAME_B: u32 = 2;
pub const MPQ_HASH_FILE_KEY: u32 = 3;
pub const MPQ_HASH_KEY2_MIX: u32 = 4;

pub fn build_crypt_table() -> [u32; CRYPT_TABLE_SIZE] {
    let mut table = [0u32; CRYPT_TABLE_SIZE];
    let mut seed: u32 = CRYPT_TABLE_SEED;

    for index1 in 0..0x100 {
        let mut index2 = index1;

        for _ in 0..5 {
            seed = next_seed(seed);
            let temp1 = (seed & 0xFFFF) << 16;

            seed = next_seed(seed);
            let temp2 = seed & 0xFFFF;

            table[index2] = temp1 | temp2;
            index2 += 0x100;
        }
    }

    table
}

fn next_seed(seed: u32) -> u32 {
    let temp: u64 = seed as u64;
    let temp = (temp * 125 + 3) % 0x2AAAAB;
    temp as u32
}

/// Computes one of MPQ's multi-purpose hashes for a given string, using
/// the algorithm described in the MPQ format specification.
///
/// The same algorithm produces different, unrelated results depending on
/// `hash_type`, which selects a distinct 256-value "zone" within the
/// crypt table:
/// - `MPQ_HASH_TABLE_OFFSET` (0): index of a file name within the hash table.
/// - `MPQ_HASH_NAME_A` (1) / `MPQ_HASH_NAME_B` (2): the two independent
///   hashes stored in a `HashTableEntry`, used together to avoid collisions.
/// - `MPQ_HASH_FILE_KEY` (3): decryption key for a file or table. For
///   example, the hash table itself is decrypted using the key
///   `hash_string("(hash table)", MPQ_HASH_FILE_KEY, &crypt_table)`.
///
/// `text` is uppercased internally (MPQ hashing is case-insensitive).
pub fn hash_string(text: &str, hash_type: u32, crypt_table: &[u32; CRYPT_TABLE_SIZE]) -> u32 {
    let mut seed1: u32 = 0x7FED_7FED;
    let mut seed2: u32 = 0xEEEE_EEEE;

    for ch in text.chars() {
        let upp_ch = ch.to_ascii_uppercase() as u32;
        let table_value = crypt_table[((hash_type << 8) as usize).wrapping_add(upp_ch as usize)];
        seed1 = table_value ^ seed1.wrapping_add(seed2);
        seed2 = upp_ch
            .wrapping_add(seed1)
            .wrapping_add(seed2)
            .wrapping_add(seed2 << 5)
            .wrapping_add(3);
    }

    seed1
}

/// Decrypts a block of MPQ data (e.g. the raw hash table or block table
/// bytes) in place, returning the decrypted content as `u32` words.
///
/// `key` must be the value returned by [`hash_string`] with
/// `MPQ_HASH_FILE_KEY`, computed from the name MPQ uses to identify the
/// thing being decrypted (e.g. `"(hash table)"`, `"(block table)"`, or a
/// file's own name).
///
/// Internally, this is a stream cipher: `key` evolves after each 4-byte
/// block, mixed with a distinct region of the crypt table
/// (`MPQ_HASH_KEY2_MIX`) reserved specifically for decryption, separate
/// from the regions used by [`hash_string`].
pub fn decrypt(data: &[u8], mut key: u32, crypt_table: &[u32; CRYPT_TABLE_SIZE]) -> Vec<u32> {
    let mut seed: u32 = 0xEEEE_EEEE;
    let mut result = Vec::with_capacity(data.len() / 4);

    for chunk in data.chunks(4) {
        let encrypted = u32::from_le_bytes(chunk.try_into().unwrap());

        let table_index = ((MPQ_HASH_KEY2_MIX << 8) as usize).wrapping_add((key & 0xFF) as usize);
        seed = seed.wrapping_add(crypt_table[table_index]);

        let decrypted = encrypted ^ key.wrapping_add(seed);
        result.push(decrypted);

        key = (!key).wrapping_shl(0x15).wrapping_add(0x1111_1111) | key.wrapping_shr(0x0B);

        seed = decrypted
            .wrapping_add(seed)
            .wrapping_add(seed.wrapping_shl(5))
            .wrapping_add(3);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crypt_table_first_values() {
        let table = build_crypt_table();
        println!("{:#010x} {:#010x} {:#010x}", table[0], table[1], table[2]);
    }

    #[test]
    fn hash_table_key() {
        let table = build_crypt_table();
        let key = hash_string("(hash table)", MPQ_HASH_FILE_KEY, &table);
        println!("{:#010x}", key);
    }
}
