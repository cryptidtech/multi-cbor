# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-08-17

### Added

- `src/cid.rs` with the `Cid` struct, `Builder`, `EncodedCid` and `LegacyEncodedCid` type aliases, and the `SIGIL` multicodec constant. The type implements `EncodeIntoBuffer`, `EncodeInto`, `TryDecodeFrom`, `Null`, `CodecInfo`, and `EncodingInfo` and supports v0, v1, v2, and v3 CIDs with binary and base-encoding round-tripping. Staged behind not-yet-wired module plumbing; not yet re-exported from the crate root.

## [0.1.0] - 2026-08-12

### Added

- Initial standalone release of `multi-cbor` on crates.io.
- `tags` cargo feature for [DAG-CBOR] tag round-tripping. When the feature is on, the `Tagged<T>` helper, the `current_cbor_tag()` accessor, and the `TagGuard` RAII thread-local state record CBOR tags during serialization and restore them during deserialization. When the feature is off, the same API compiles but tags are always `None`. This mirrors the behavior the BetterSign workspace needed for IPLD DAG-CBOR encoding. The `tags` feature implies `std` because it uses `thread_local!` and `std::cell::RefCell` for the tag-state plumbing.
- `unsealed_read_write` cargo feature, kept from upstream. It exposes the `read::Read` and `write::Write` traits so external callers can build custom `Deserializer` and `Serializer` input and output sources.
- `alloc` cargo feature for `no_std` + `alloc` builds. It enables `from_slice` and `to_vec` without `std`.
- `no_std` support. With `default-features = false` the crate builds without `std`. The `std` feature enables `from_reader`, `to_writer`, and the `Value` module.
- `[lints.clippy]` config in `Cargo.toml` with `pedantic`, `nursery`, and `cargo` groups. `multiple_crate_versions` is allowed (the `syn 2.x`/`3.x` split clears when the ecosystem upgrades). Crate-level `#![allow(...)]` suppresses stylistic lints where the idiomatic CBOR encoding pattern conflicts with the lint; each allow has a comment explaining why.

### Fixed

- Fixed `no_std` build. Upstream `serde_cbor` did not build with `--no-default-features`. Two `std::mem::take` calls in `src/read.rs` (`MutSliceRead::clear_buffer` and `MutSliceRead::take_buffer`) used `std::mem` unconditionally. Changed to `core::mem::take`, which is available in `no_std`.
- Fixed `no_std` + `tags` build. The `tags` feature used `std::cell::RefCell` and `thread_local!`, which are not in `core` or `alloc`. The `tags` feature now implies `std` in `Cargo.toml`, so `--no-default-features --features tags` pulls in `std` automatically.
- Replaced `std::convert::Into::into` with `core::convert::Into::into` in `src/ser.rs` (15 sites) so the serializer's `map_err` calls compile under `no_std`.
- Replaced `s.len().is_multiple_of(2)` (stable since Rust 1.87) with `s.len() % 2 == 0` in `tests/std_types.rs` to honor the declared MSRV of 1.85.
- Renamed `_msg` parameters to `msg` in `Error::message` (both the `unsealed_read_write` and `not(unsealed_read_write)` variants) and added `let _ = msg;` in the `no_std` branch. The underscore prefix triggered `clippy::used_underscore_binding` when the `std` branch used the value.
- Removed unnecessary `Result` wrapping from `StructSerializer::skip_field_inner` and `StructSerializer::end_inner`. The call sites now wrap with `Ok(())`. `end_inner` keeps `self` by value (annotated `#[allow(clippy::unused_self)]`) to consume the serializer and prevent further field writes after `end`.
- Replaced `for tag in get_tag()` with `if let Some(tag) = get_tag()` in `Serializer::serialize_newtype_struct` to clear `for_loops_over_fallibles`.

### Changed

- Forked from the archived upstream `serde_cbor` 0.7.0 crate. The crate is renamed from `serde_cbor` to `multi-cbor`. All `use serde_cbor::...` references now use `use multi_cbor::...`.
- Updated the crate `repository` field to `https://github.com/cryptidtech/multi-cbor`.
- Updated the `description` field to mention DAG-CBOR tags support.
- Set the MSRV to 1.85.
- Replaced the `serde = { workspace = true }` workspace dependency with a direct `serde = { version = "1.0", default-features = false, features = ["alloc"] }` runtime dependency.
- Dropped the `bs-multibase`, `bs-multicid`, `bs-multicodec`, `bs-multihash`, `bs-multitrait`, and `bs-multiutil` dev-dependencies. These were only used by the `cid_linked_list` example, which is removed. The remaining tests, benches, and examples do not need them.
- Dropped the `cid_linked_list` example. It depended on `bs-*` workspace crates. It will be re-pointed at the standalone `multi-cid` crate in a later release.
- Repointed the bench and example paths from the BetterSign workspace `../../benches/serde_cbor/` and `../../examples/serde_cbor/` locations to the local `benches/` and `examples/` directories.

### Notes

- This is a fork of the archived upstream [`serde_cbor`](https://github.com/pyfisch/cbor) crate. Upstream has been unmaintained since 2021. The fork keeps the MIT OR Apache-2.0 license. The rename to `multi-cbor` avoids the `serde_cbor` crate name on crates.io, which the archived upstream still owns.
- The `tags` feature is the reason this fork exists. Upstream ignored CBOR tags during deserialization and refused to emit them during serialization. The `tags` feature adds the thread-local tag plumbing that DAG-CBOR needs.

[DAG-CBOR]: https://github.com/ipld/carbites/blob/main/dag-cbor.md

[0.1.0]: https://github.com/cryptidtech/multi-cbor/releases/tag/v0.1.0

[0.1.1]: https://github.com/cryptidtech/multi-cbor/releases/tag/v0.1.1