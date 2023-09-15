use arkiv::Archive;
use std::{fs::read_to_string, path::Path};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

mod test_unpack {
    use super::*;

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
    fn tar_zipped_archive() -> Result<()> {
        test("tests/sample/sample.zip")
    }

    #[test]
    fn tar_gzipped_archive() -> Result<()> {
        test("tests/sample/sample.tar.gz")
    }

    #[test]
    fn tar_bzipped_archive() -> Result<()> {
        test("tests/sample/sample.tar.bz2")
    }

    #[test]
    fn tar_xzipped_archive() -> Result<()> {
        test("tests/sample/sample.tar.xz")
    }
}
