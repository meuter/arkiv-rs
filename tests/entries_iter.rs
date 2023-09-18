use arkiv::Archive;
use std::path::Path;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[allow(unused)]
fn test(path: impl AsRef<Path>) -> Result<()> {
    let mut archive = Archive::open(path)?;
    let mut expected = vec!["sample/", "sample/sample.txt"];
    let mut actual: Vec<String> = vec![];

    for entry in archive.entries_iter()? {
        let entry = entry?;
        let path = entry.path().display().to_string();
        actual.push(path)
    }

    actual.sort();
    expected.sort();

    assert_eq!(actual, expected);
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
