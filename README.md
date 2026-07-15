# mpq-parser

A pure-Rust parser for the **MPQ (Mo'PaQ)** file container, the generic archive format used by several Blizzard Entertainment games (StarCraft II, WarCraft III, World of Warcraft, Diablo) to package their assets and data.

Written **from scratch**, without relying on existing MPQ-parsing crates — the main goal is to serve as a Rust learning project (binary parsing, simple cryptography, error handling, library design), not to compete in functionality with more mature implementations in the ecosystem (`wow-mpq`, `ceres-mpq`).

## Origin

This crate started out as part of [sc2reader-rs](https://github.com/aldezex/sc2reader-rs), a learning port of [sc2reader](https://github.com/ggtracker/sc2reader) (Python) to Rust. Since the MPQ format isn't specific to StarCraft II, it was extracted into its own independent library — with its own versioning and publishing cycle — instead of staying coupled to a project that *is* game-specific.

## Current status

✅ **Full, end-to-end functional MPQ container:**
- [x] `MPQUserData` (the wrapper preceding the real MPQ header in files such as SC2 replays).
- [x] `MpqHeader` — the real MPQ header, format V1-V4 (sizes, version, hash table / block table positions and sizes).
- [x] MPQ's own cryptography (`crypto`): crypt table generation, the multi-purpose hash function (`hash_string`), and stream decryption (`decrypt`).
- [x] Reading and decrypting the **hash table**, typed as `HashTableEntry`.
- [x] Reading and decrypting the **block table**, typed as `BlockTableEntry`.
- [x] File lookup by name (`find_file`), cross-referencing the hash table and block table.
- [x] File extraction (`extract_file`), with automatic decompression when needed.
- [x] Decompression: **zlib** and **bzip2** (the two methods seen in real data so far). Other MPQ compression methods (PKWare implode, LZMA) return an explicit error instead of failing silently.

Not supported (yet, and possibly never, given the learning scope of this project): writing/creating MPQ archives, very old protocol versions, protected/signed archives, compression methods other than zlib/bzip2.

## Usage

```rust
use mpq_parser::{MpqUserDataHeader, MpqHeader};
use mpq_parser::crypto::{build_crypt_table, hash_string, decrypt, MPQ_HASH_FILE_KEY};
use mpq_parser::hash::parse_hash_table_entries;
use mpq_parser::block::parse_block_table_entries;
use mpq_parser::archive::{find_file, extract_file};

let replay = std::fs::read("replay.SC2Replay")?;

let user_header = MpqUserDataHeader::parse(&replay)?;
let offset = user_header.header_offset as usize;
let mpq_header = MpqHeader::parse(&replay[offset..])?;

let crypt_table = build_crypt_table();

// Hash table
let ht_start = offset + mpq_header.hash_table_position as usize;
let ht_size = mpq_header.hash_table_size as usize * 16;
let ht_key = hash_string("(hash table)", MPQ_HASH_FILE_KEY, &crypt_table);
let ht_decrypted = decrypt(&replay[ht_start..ht_start + ht_size], ht_key, &crypt_table);
let hash_entries = parse_hash_table_entries(&ht_decrypted);

// Block table
let bt_start = offset + mpq_header.block_table_position as usize;
let bt_size = mpq_header.block_table_size as usize * 16;
let bt_key = hash_string("(block table)", MPQ_HASH_FILE_KEY, &crypt_table);
let bt_decrypted = decrypt(&replay[bt_start..bt_start + bt_size], bt_key, &crypt_table);
let block_entries = parse_block_table_entries(&bt_decrypted);

// Look up and extract an internal file
if let Some(entry) = find_file("replay.details", &hash_entries, &block_entries, &crypt_table) {
    let contents = extract_file(&replay, offset as u32, *entry)?;
    println!("{} bytes extracted", contents.len());
}
```

## Error handling

Header-parsing functions return `Result<T, MpqParseError>` instead of panicking — the caller decides what to do with corrupt or incomplete data. Extraction/decompression functions use `std::io::Result<T>`, reusing the standard error type instead of a custom one, since most failures there already map naturally onto `std::io::ErrorKind` variants.

## Tests

- `cargo test` — unit tests for the core primitives (crypt table, hashing, header parsing) using hand-built data.
- Integration tests under `tests/`, which require real replays in `tests/fixtures/` (git-ignored — not distributed with the crate). To run them, place one or more of your own `.SC2Replay` files in that folder.

## License

Dual-licensed under MIT or Apache-2.0, at your option. See `LICENSE-MIT.md` and `LICENSE-APACHE.md`.

## Contributing

Personal learning project — not actively looking for external contributions, but issues and suggestions are welcome.
