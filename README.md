# Arkiv

[![Crates.io](https://img.shields.io/crates/v/arkiv)](https://crates.io/crates/arkiv)
[![Docs.rs](https://docs.rs/arkiv/badge.svg)](https://docs.rs/arkiv)
[![Crates.io](https://img.shields.io/crates/d/arkiv)](https://crates.io/crates/arkiv)
[![Crates.io](https://img.shields.io/crates/l/arkiv)](https://github.com/meuter/arkiv-rs/blob/main/LICENSE)

[![Build](https://github.com/meuter/arkiv-rs/actions/workflows/build.yml/badge.svg)](https://github.com/meuter/arkiv-rs/actions/workflows/build.yml)
[![Build](https://github.com/meuter/arkiv-rs/actions/workflows/test.yml/badge.svg)](https://github.com/meuter/arkiv-rs/actions/workflows/test.yml)
[![Clippy](https://github.com/meuter/arkiv-rs/actions/workflows/clippy.yml/badge.svg)](https://github.com/meuter/arkiv-rs/actions/workflows/clippy.yml)

Thin convenience library to manupulate compressed archive of vairous types
through a single interface.

## Usage

```rust
fn main() -> Result<()> {
    let archive = arkiv::Archive::open("/path/to/archive.tar.xz")?;
    archive.unpack("/tmp/")?;
    Ok(())
}
```

