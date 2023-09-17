use arkiv::Archive;
use std::{fs::read_to_string, path::Path};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[allow(unused)]
fn test(path: impl AsRef<Path>) -> Result<()> {
    let sandbox = tempfile::tempdir()?;
    let mut archive = Archive::open(path)?;
    archive.unpack(&sandbox)?;

    assert_eq!(
        read_to_string(sandbox.path().join("sample/sample.txt"))?,
        "sample\n"
    );

    Ok(())
}

#[test]
#[cfg(feature = "zip")]
fn zip_archive() -> Result<()> {
    test("tests/sample/sample.zip")
}

#[test]
#[cfg(all(feature = "gzip", feature = "tar"))]
fn tar_gz_archive() -> Result<()> {
    test("tests/sample/sample.tar.gz")?;
    test("tests/sample/sample.tgz")
}

#[test]
#[cfg(all(feature = "bzip", feature = "tar"))]
fn tar_bz2_archive() -> Result<()> {
    test("tests/sample/sample.tar.bz2")
}

#[test]
#[cfg(all(feature = "xz", feature = "tar"))]
fn tar_xz_archive() -> Result<()> {
    test("tests/sample/sample.tar.xz")
}

#[test]
#[cfg(all(feature = "zstd", feature = "tar"))]
fn tar_zst_archive() -> Result<()> {
    test("tests/sample/sample.tar.zstd")?;
    test("tests/sample/sample.tar.zst")
}
