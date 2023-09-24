# Arkiv

[![Crates.io](https://img.shields.io/crates/v/arkiv)](https://crates.io/crates/arkiv)
[![Docs.rs](https://docs.rs/arkiv/badge.svg)](https://docs.rs/arkiv)
[![Crates.io](https://img.shields.io/crates/d/arkiv)](https://crates.io/crates/arkiv)
[![Crates.io](https://img.shields.io/crates/l/arkiv)](https://github.com/meuter/arkiv-rs/blob/main/LICENSE)

[![Build](https://github.com/meuter/arkiv-rs/actions/workflows/build.yml/badge.svg)](https://github.com/meuter/arkiv-rs/actions/workflows/build.yml)
[![Build](https://github.com/meuter/arkiv-rs/actions/workflows/test.yml/badge.svg)](https://github.com/meuter/arkiv-rs/actions/workflows/test.yml)
[![Clippy](https://github.com/meuter/arkiv-rs/actions/workflows/clippy.yml/badge.svg)](https://github.com/meuter/arkiv-rs/actions/workflows/clippy.yml)

Arkiv is a convenience library to download, open, consult and extract archives of various 
format through a single consistent interface.

## Supported Formats

- `sample.zip` (requires the zip feature).
- `sample.tar` (requires the `tar` feature).
- `sample.tgz` or `sample.tar.gz` (requires `tar` and `gzip` features).
- `sample.tar.xz` (requires `tar` and `xz` features).
- `sample.tar.bz2` (requires `tar` and `bzip` features).
- `sample.tar.zstd` or `sample.tar.zst` (requires `tar` and `zstd` features).

## Usage

```rust , no_run
use arkiv::{Result, Archive};

fn main() -> Result<()> {
    // open the archive from a local file
    let mut archive = arkiv::Archive::open("path/to/archive.tar.xz")?;

    // or download it over HTTP(S) - requires the `download` feature.
    #[cfg(feature="download")]
    let mut archive = {
        let url = "https://github.com/meuter/arkiv-rs/raw/main/tests/sample/sample.tar.zstd";
        arkiv::Archive::download(url)?
    };

    // iterate over entries
    for entry in archive.entries_iter()? {
        let entry = entry?;
        println!("{} {}", entry.size(), entry.path().display());
    }

    // extract the archive (perserves permission on unix targets)
    archive.unpack("/tmp/")?;

    Ok(())
}
```

