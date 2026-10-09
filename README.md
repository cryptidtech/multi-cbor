[![](https://img.shields.io/badge/made%20by-Cryptid%20Technologies-gold.svg?style=flat-square)](https://cryptid.tech/)
[![](https://img.shields.io/badge/project-provenance-purple.svg?style=flat-square)](https://github.com/cryptidtech/provenance-specifications/)

[![Build Status](https://github.com/cryptidtech/multi-cbor/actions/workflows/rust.yml/badge.svg)](https://github.com/cryptidtech/multi-cbor/actions)
[![License](https://img.shields.io/crates/l/multi-cbor?style=flat-square)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/multi-cbor?style=flat-square)](https://crates.io/crates/multi-cbor)
[![Documentation](https://docs.rs/multi-cbor/badge.svg?style=flat-square)](https://docs.rs/multi-cbor)

# multi-cbor

CBOR serialization and deserialization for [`serde`], forked from the archived
upstream [`serde_cbor`] crate and extended with a `tags` feature for
[DAG-CBOR] tag round-tripping.

`multi-cbor` is a maintained drop-in replacement for `serde_cbor`. Upstream
[`serde_cbor`] has been unmaintained and archived since 2021. The crate name on
crates.io is still owned by the archived upstream, so this fork publishes under
the `multi-cbor` name. The license stays `MIT OR Apache-2.0`, the same as
upstream.

The `tags` feature is the reason this fork exists. Upstream ignored CBOR tags
during deserialization and refused to emit them during serialization. The
`tags` feature adds the thread-local tag plumbing that DAG-CBOR needs.

## Table of Contents

- [Features](#features)
- [Install](#install)
- [Usage](#usage)
- [The `tags` Feature](#the-tags-feature)
- [Feature Flags](#feature-flags)
- [`no_std` Support](#no_std-support)
- [Fork Notes](#fork-notes)
- [Testing](#testing)
- [Maintainers](#maintainers)
- [Contribute](#contribute)
- [License](#license)

## Features

- CBOR serialization and deserialization for any `serde`-compatible type.
- Packed encoding. Struct keys and unit enum variants encode as integers.
- Self-describing documents. The CBOR magic number can be prepended.
- The `Value` enum for untyped CBOR data, like `serde_json::Value`.
- DAG-CBOR tag round-tripping under the `tags` feature.
- `no_std` support with an `alloc` mode.
- Zero unsafe code.

## Install

Add this to your `Cargo.toml`:

```toml
[dependencies]
multi-cbor = "0.1"
```

For `no_std` builds:

```toml
[dependencies]
multi-cbor = { version = "0.1", default-features = false }
# Enable alloc to get from_slice and to_vec without std:
# multi-cbor = { version = "0.1", default-features = false, features = ["alloc"] }
```

MSRV: Rust 1.99 (Edition 2021).

## Usage

```rust
use serde_derive::{Deserialize, Serialize};
use multi_cbor::{from_slice, to_vec};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Mascot {
    name: String,
    species: String,
    year_of_birth: u32,
}

let ferris = Mascot {
    name: "Ferris".to_owned(),
    species: "crab".to_owned(),
    year_of_birth: 2015,
};

let bytes = to_vec(&ferris).unwrap();
let back: Mascot = from_slice(&bytes).unwrap();
assert_eq!(ferris, back);
```

For streaming I/O, use `to_writer` and `from_reader` (these require the `std`
feature, which is on by default):

```rust
use std::fs::File;
use multi_cbor::{to_writer, from_reader};

# use serde_derive::{Deserialize, Serialize};
# #[derive(Debug, Serialize, Deserialize)]
# struct Mascot { name: String, species: String, year_of_birth: u32 }
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let mascot = Mascot {
    name: "Tux".to_owned(),
    species: "penguin".to_owned(),
    year_of_birth: 1996,
};
to_writer(File::create("tux.cbor")?, &mascot)?;

let file = File::open("tux.cbor")?;
let back: Mascot = from_reader(file)?;
assert_eq!(mascot.name, back.name);
# Ok(())
# }
```

## The `tags` Feature

CBOR [tags] mark a data item with a tag number that tells the consumer how to
interpret the enclosed value. The [DAG-CBOR] subset of CBOR uses tags for the
`Cid` type (tag `42`) and the `Link` wrapper (tag `42` on a byte string).

Upstream `serde_cbor` ignored tags on read and refused to write them. This fork
adds the `tags` cargo feature. When the feature is on:

- `Tagged<T>` serializes a `(Option<u64>, T)` pair, writing the tag before the
  value.
- `current_cbor_tag()` returns the tag that is in scope during a
  `visit_newtype_struct` call. This lets a `Deserialize` implementation read
  the tag and dispatch on it.
- A thread-local `TagGuard` records and restores the active tag during nested
  serialization.

When the feature is off, the same API compiles, but `current_cbor_tag()` always
returns `None` and tags are not written. This keeps the no-tags build
byte-compatible with upstream `serde_cbor`.

```toml
[dependencies]
multi-cbor = { version = "0.1", features = ["tags"] }
```

```rust
use multi_cbor::tags::Tagged;
use multi_cbor::{from_slice, to_vec};

let value = Tagged::new(Some(42), b"hello".to_vec());
let bytes = to_vec(&value).unwrap();
let back: Tagged<Vec<u8>> = from_slice(&bytes).unwrap();
assert_eq!(value.tag, back.tag);
assert_eq!(value.value, back.value);
```

[tags]: https://www.rfc-editor.org/rfc/rfc8949.html#section-3.4
[DAG-CBOR]: https://github.com/ipld/carbites/blob/main/dag-cbor.md

## Feature Flags

| Feature | Default | Effect |
|---|---|---|
| `std` | yes | Enables `from_reader`, `to_writer`, and the `Value` module. Requires `serde/std`. |
| `alloc` | no | Enables `from_slice` and `to_vec` for `no_std` + `alloc` builds. Requires `serde/alloc`. |
| `tags` | no | Enables CBOR tag round-tripping for DAG-CBOR. Requires the `std` feature (uses `thread_local!`). See [The `tags` Feature](#the-tags-feature). |
| `unsealed_read_write` | no | Exposes the `read::Read` and `write::Write` traits so external callers can build custom sources and sinks. |

## `no_std` Support

With `default-features = false` the crate builds without `std`. Only the
`Serializer` and `Deserializer` types and the `SliceWrite` and `SliceRead`
helpers are available. Enable `alloc` to get `from_slice` and `to_vec`. The
`Value` module and `from_reader` / `to_writer` require `std`.

```toml
[dependencies]
multi-cbor = { version = "0.1", default-features = false, features = ["alloc"] }
serde = { version = "1.0", default-features = false, features = ["alloc", "derive"] }
```

## Fork Notes

This crate is a fork of the archived upstream [`serde_cbor`] 0.7.0. The
differences from upstream are:

- The crate is renamed from `serde_cbor` to `multi-cbor`.
- The `tags` cargo feature is added for DAG-CBOR tag round-tripping.
- The `repository` field points at the `cryptidtech/multi-cbor` GitHub repo.
- The MSRV is declared as 1.99.
- The dev-dependencies that referenced the `bs-*` BetterSign workspace crates
  are removed. The `cid_linked_list` example is removed for the same reason.
  A later release will re-point it at the standalone `multi-cid` crate.

The license stays `MIT OR Apache-2.0`, the same as upstream.

[`serde`]: https://serde.rs
[`serde_cbor`]: https://github.com/pyfisch/cbor

## Testing

```bash
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --all-features
```

## Maintainers

- Dave Grantham <dwg@linuxprogrammer.org>

## Contribute

Pull requests go to the [`cryptidtech/multi-cbor`](https://github.com/cryptidtech/multi-cbor)
repository. Sign commits with GPG. Use Conventional Commits messages.

## License

Dual-licensed as `MIT OR Apache-2.0`, the same as upstream `serde_cbor`.

See [`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE) for the
full text.