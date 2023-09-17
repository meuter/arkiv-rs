# Arkiv

[![Crates.io](https://img.shields.io/crates/v/arkiv)](https://crates.io/crates/arkiv)
[![Docs.rs](https://docs.rs/arkiv/badge.svg)](https://docs.rs/arkiv)
[![Crates.io](https://img.shields.io/crates/d/arkiv)](https://crates.io/crates/arkiv)
[![Crates.io](https://img.shields.io/crates/l/arkiv)](https://github.com/meuter/arkiv-rs/blob/main/LICENSE)

[![Build](https://github.com/meuter/arkiv-rs/actions/workflows/build.yml/badge.svg)](https://github.com/meuter/arkiv-rs/actions/workflows/build.yml)
[![Build](https://github.com/meuter/arkiv-rs/actions/workflows/test.yml/badge.svg)](https://github.com/meuter/arkiv-rs/actions/workflows/test.yml)
[![Clippy](https://github.com/meuter/arkiv-rs/actions/workflows/clippy.yml/badge.svg)](https://github.com/meuter/arkiv-rs/actions/workflows/clippy.yml)

Arkiv is a convenience library to open, consult and extract archives of various format
through a single interface.

## Usage

```rust , no_run
use arkiv::{Result, Archive};

fn main() -> Result<()> {
    // open the archive
    let mut archive = arkiv::Archive::open("path/to/archive.tar.xz")?;

    // iterate over entries
    for entry in archive.entries()? {
        println!("{entry}");
    }

    // extract the archive (perserves permission on unix targets)
    archive.unpack("/tmp/")?;

    Ok(())
}
```

