//! MPQ container parsing for StarCraft II replays.
//!
//! A `.SC2Replay` file is an MPQ archive wrapped in an `MPQUserData`
//! header. This module handles only the container layer: locating the
//! real MPQ header and its associated tables (hash table, block table).
//! It does not yet interpret the contents of the internal files
//! (`replay.details`, `replay.tracker.events`, etc.) — that lives in
//! another module further down the line.
pub mod block;
pub mod crypto;
pub mod hash;

/// Errors that can occur while parsing the MPQ container of a replay.
///
/// A distinction is made between "the file is too short to contain the
/// field we're looking for" and "the signature isn't the expected one",
/// since these are failures with different causes and remedies for
/// whoever uses this library.
#[derive(Debug, thiserror::Error)]
pub enum MpqParseError {
    #[error("needed {needed} bytes at offset {offset}, but only {available} are available")]
    UnexpectedEof {
        needed: usize,
        offset: usize,
        available: usize,
    },
    #[error("invalid signature: expected {expected:02x?}, found {found:02x?}")]
    InvalidSignature { expected: [u8; 4], found: [u8; 4] },
}

/// Short-hand result type for this module.
pub type Result<T> = std::result::Result<T, MpqParseError>;

// --- Layout constants ---------------------------------------------
//
// Named explicitly instead of using magic numbers in slice ranges.
// Offsets are relative to the start of each structure, not to the
// start of the file.

const USER_DATA_SIGNATURE: [u8; 4] = *b"MPQ\x1b";
const MPQ_HEADER_SIGNATURE: [u8; 4] = *b"MPQ\x1a";

mod user_data_offsets {
    pub const SIGNATURE: (usize, usize) = (0, 4);
    pub const USER_DATA_SIZE: (usize, usize) = (4, 8);
    pub const HEADER_OFFSET: (usize, usize) = (8, 12);
    pub const USER_DATA_HEADER_SIZE: (usize, usize) = (12, 16);
}

mod header_offsets {
    pub const SIGNATURE: (usize, usize) = (0, 4);
    pub const HEADER_SIZE: (usize, usize) = (4, 8);
    pub const ARCHIVE_SIZE: (usize, usize) = (8, 12);
    pub const FORMAT_VERSION: (usize, usize) = (12, 14);
    pub const BLOCK_SIZE: (usize, usize) = (14, 16);
    pub const HASH_TABLE_POSITION: (usize, usize) = (16, 20);
    pub const BLOCK_TABLE_POSITION: (usize, usize) = (20, 24);
    pub const HASH_TABLE_SIZE: (usize, usize) = (24, 28);
    pub const BLOCK_TABLE_SIZE: (usize, usize) = (28, 32);
}

// --- Reading helpers --------------------------------------------------

/// Reads a little-endian `u32` from the `(start, end)` range within `bytes`.
///
/// Centralizes the `slice -> try_into -> from_le_bytes` pattern that was
/// repeated for every field, and turns a possible size failure into an
/// `MpqParseError` instead of a panic.
fn read_u32(bytes: &[u8], range: (usize, usize)) -> Result<u32> {
    let (start, end) = range;
    let slice = bytes.get(start..end).ok_or(MpqParseError::UnexpectedEof {
        needed: end - start,
        offset: start,
        available: bytes.len().saturating_sub(start),
    })?;
    // The slice is guaranteed to have length 4 by the constant's range,
    // so this unwrap is safe: if it fails, it's an internal bug on our
    // side, not a condition of the input file.
    Ok(u32::from_le_bytes(slice.try_into().unwrap()))
}

/// Reads a little-endian `u16` from the `(start, end)` range within `bytes`.
fn read_u16(bytes: &[u8], range: (usize, usize)) -> Result<u16> {
    let (start, end) = range;
    let slice = bytes.get(start..end).ok_or(MpqParseError::UnexpectedEof {
        needed: end - start,
        offset: start,
        available: bytes.len().saturating_sub(start),
    })?;
    Ok(u16::from_le_bytes(slice.try_into().unwrap()))
}

fn read_signature(bytes: &[u8], range: (usize, usize)) -> Result<[u8; 4]> {
    let (start, end) = range;
    let slice = bytes.get(start..end).ok_or(MpqParseError::UnexpectedEof {
        needed: end - start,
        offset: start,
        available: bytes.len().saturating_sub(start),
    })?;
    Ok(slice.try_into().unwrap())
}

// --- Domain types -----------------------------------------------------

/// `MPQUserData` wrapper that precedes the real MPQ header in a
/// StarCraft II replay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MpqUserDataHeader {
    pub user_data_size: u32,
    /// Offset, relative to the start of the file, where the real MPQ
    /// header (`MPQ\x1A`) begins.
    pub header_offset: u32,
    pub user_data_header_size: u32,
}

impl MpqUserDataHeader {
    /// Parses the `MPQUserData` starting at the beginning of a replay file.
    ///
    /// # Errors
    /// Returns [`MpqParseError::InvalidSignature`] if the first 4 bytes
    /// aren't `MPQ\x1B`, and [`MpqParseError::UnexpectedEof`] if `bytes`
    /// is too short.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let signature = read_signature(bytes, user_data_offsets::SIGNATURE)?;
        if signature != USER_DATA_SIGNATURE {
            return Err(MpqParseError::InvalidSignature {
                expected: USER_DATA_SIGNATURE,
                found: signature,
            });
        }

        Ok(Self {
            user_data_size: read_u32(bytes, user_data_offsets::USER_DATA_SIZE)?,
            header_offset: read_u32(bytes, user_data_offsets::HEADER_OFFSET)?,
            user_data_header_size: read_u32(bytes, user_data_offsets::USER_DATA_HEADER_SIZE)?,
        })
    }
}

/// Real MPQ header (format V1-V4). Only the fields needed to locate the
/// hash table and the block table are exposed; the rest of the V4
/// extended header (hi-block table, checksums) is out of scope for now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MpqHeader {
    pub archive_size: u32,
    pub format_version: u16,
    /// Hash table offset, relative to the start of the MPQ header
    /// (not to the start of the file).
    pub hash_table_position: u32,
    /// Block table offset, relative to the start of the MPQ header.
    pub block_table_position: u32,
    /// Number of entries in the hash table (not bytes).
    pub hash_table_size: u32,
    /// Number of entries in the block table (not bytes).
    pub block_table_size: u32,
}

impl MpqHeader {
    /// Parses the MPQ header starting at the offset indicated by
    /// [`MpqUserDataHeader::header_offset`].
    ///
    /// `bytes` must be the slice of the full file *starting from* that
    /// offset (i.e. already trimmed by the caller).
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let signature = read_signature(bytes, header_offsets::SIGNATURE)?;
        if signature != MPQ_HEADER_SIGNATURE {
            return Err(MpqParseError::InvalidSignature {
                expected: MPQ_HEADER_SIGNATURE,
                found: signature,
            });
        }

        Ok(Self {
            archive_size: read_u32(bytes, header_offsets::ARCHIVE_SIZE)?,
            format_version: read_u16(bytes, header_offsets::FORMAT_VERSION)?,
            hash_table_position: read_u32(bytes, header_offsets::HASH_TABLE_POSITION)?,
            block_table_position: read_u32(bytes, header_offsets::BLOCK_TABLE_POSITION)?,
            hash_table_size: read_u32(bytes, header_offsets::HASH_TABLE_SIZE)?,
            block_table_size: read_u32(bytes, header_offsets::BLOCK_TABLE_SIZE)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_user_data() -> Vec<u8> {
        // MPQ\x1B, user_data_size=512, header_offset=1024, header_size=114
        let mut bytes = vec![0x4d, 0x50, 0x51, 0x1b];
        bytes.extend_from_slice(&512u32.to_le_bytes());
        bytes.extend_from_slice(&1024u32.to_le_bytes());
        bytes.extend_from_slice(&114u32.to_le_bytes());
        bytes
    }

    #[test]
    fn parses_valid_user_data_header() {
        let bytes = sample_user_data();
        let header = MpqUserDataHeader::parse(&bytes).unwrap();

        assert_eq!(header.user_data_size, 512);
        assert_eq!(header.header_offset, 1024);
        assert_eq!(header.user_data_header_size, 114);
    }

    #[test]
    fn rejects_invalid_signature() {
        let mut bytes = sample_user_data();
        bytes[3] = 0x00; // corrupt the signature

        let err = MpqUserDataHeader::parse(&bytes).unwrap_err();
        assert!(matches!(err, MpqParseError::InvalidSignature { .. }));
    }

    #[test]
    fn reports_eof_instead_of_panicking() {
        let bytes = [0x4d, 0x50, 0x51, 0x1b]; // signature only, nothing else
        let err = MpqUserDataHeader::parse(&bytes).unwrap_err();
        assert!(matches!(err, MpqParseError::UnexpectedEof { .. }));
    }
}
