# Arkiv

Thin convenience library to manupulate compressed archive of vairous types
through a single interface.

```rust
fn main() -> Result<()> {
    let archive = arkiv::Archive::open("/path/to/archive.tar.xz")?;
    archive.unpack("/tmp/")?;
    Ok(())
}
```
