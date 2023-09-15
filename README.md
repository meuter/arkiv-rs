# Arkiv

[![Build](../../actions/workflows/build.yml/badge.svg)](../../actions/workflows/build.yml)
[![Build](../../actions/workflows/test.yml/badge.svg)](../../actions/workflows/test.yml)
[![Clippy](../../actions/workflows/clippy.yml/badge.svg)](../../actions/workflows/clippy.yml)

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

