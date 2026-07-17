# Agent context — mpq-parser

Blizzard MPQ (Mo'PaQ) container parser in pure Rust. **Public repo, published on crates.io** — currently 0.2.0. The bottom layer of a three-repo project (`mpq-parser` → `sc2reader-rs` → `sc2trainer`); full project context lives in the private `sc2trainer-workspace` repo (`CLAUDE.md` + `docs/HISTORY.md`). Owner: `aldezex` (Spanish speaker; code/docs in English).

## What matters most here

- **Hand-written by the owner as a Rust learning exercise** (deliberately no third-party MPQ crates). Significant rewrites are the owner's call; keep changes surgical.
- Scope is the *container only*: user-data header, MPQ header, encrypted hash/block tables, file lookup, zlib/bzip2 decompression. No knowledge of SC2 file contents — that's `sc2reader-rs`.
- SC2 replay internal files are bzip2 (`0x10`) except tiny stored-as-is ones; other MPQ compression methods (PKWare, LZMA) are intentionally unimplemented and error out.
- `decompress()` takes an `uncompressed_size_hint` (MPQ block entries always know the exact size — callers via `extract_file` get one exact allocation). `0` degrades gracefully; a wrong hint is a perf issue, never a correctness one.
- Perf reality: bzip2 decompression dominates everything this crate does (~90%+); don't chase micro-optimizations here — batch-level parallelism lives downstream.
- This crate appears in `sc2reader-rs`'s public API (`ReplayError::Mpq` wraps `MpqParseError`), so version bumps here cascade as breaking changes there.
- Publishing a new version requires the owner's explicit go-ahead (irreversible).
