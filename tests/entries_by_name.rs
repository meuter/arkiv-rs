use arkiv::Archive;
use std::path::Path;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[allow(unused)]
fn test(path: impl AsRef<Path>) -> Result<()> {
    let mut archive = Archive::open(&path)?;

    let entry = archive.entry_by_name("sample/")?;
    assert!(entry.is_dir());
    assert!(!entry.is_file());
    assert_eq!(entry.size(), 0);
    assert_eq!(entry.path(), Path::new("sample/"));

    let entry = archive.entry_by_name("sample/sample.txt")?;
    assert!(!entry.is_dir());
    assert!(entry.is_file());
    assert_eq!(entry.size(), 7);
    assert_eq!(entry.path(), Path::new("sample/sample.txt"));

    let not_found = archive.entry_by_name("not_found.txt");
    assert!(matches!(not_found, Err(arkiv::Error::FileNotFound)));

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
fn tar_zstd_archive() -> Result<()> {
    test("tests/sample/sample.tar.zstd")?;
    test("tests/sample/sample.tar.zst")
}
