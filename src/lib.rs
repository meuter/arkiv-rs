use std::{fs::File, io::ErrorKind, path::Path};

pub type Error = std::io::Error;
pub type Result<T> = std::io::Result<T>;

trait Archived {
    fn unpack(&mut self, dest: &Path) -> Result<()>;
    fn entries(&mut self) -> Result<Vec<String>>;
}

#[cfg(feature = "zip")]
impl<R: ::std::io::Read + ::std::io::Seek> Archived for zip::ZipArchive<R> {
    fn unpack(&mut self, dest: &Path) -> Result<()> {
        Ok(self.extract(dest)?)
    }

    fn entries(&mut self) -> Result<Vec<String>> {
        let files = self.file_names().map(|e| e.into()).collect();
        Ok(files)
    }
}

#[cfg(feature = "tar")]
impl<R: ::std::io::Read> Archived for tar::Archive<R> {
    fn unpack(&mut self, dest: &Path) -> Result<()> {
        self.unpack(dest)
    }

    fn entries(&mut self) -> Result<Vec<String>> {
        let files = tar::Archive::entries(self)?
            .map(|e| e.unwrap().path().unwrap().to_str().unwrap().into())
            .collect();
        Ok(files)
    }
}

pub struct Archive(Box<dyn Archived>);

impl Archive {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let _file = File::open(&path)?;
        match ArchiveKind::infer_from_file_extension(path) {
            #[cfg(feature = "zip")]
            ArchiveKind::Zip => Ok(Archive(Box::new(zip::ZipArchive::new(_file)?))),

            #[cfg(feature = "tar")]
            ArchiveKind::Tar => Ok(Archive(Box::new(tar::Archive::new(_file)))),

            #[cfg(all(feature = "gzip", feature = "tar"))]
            ArchiveKind::TarGzip => Ok(Archive(Box::new(tar::Archive::new(
                flate2::read::GzDecoder::new(_file),
            )))),

            #[cfg(all(feature = "bzip2", feature = "tar"))]
            ArchiveKind::TarBzip2 => Ok(Archive(Box::new(tar::Archive::new(
                bzip2::read::BzDecoder::new(_file),
            )))),

            #[cfg(all(feature = "xz", feature = "tar"))]
            ArchiveKind::TarXz2 => Ok(Archive(Box::new(tar::Archive::new(
                xz2::read::XzDecoder::new(_file),
            )))),

            _ => Err(ErrorKind::Other.into()),
        }
    }

    pub fn entries(&mut self) -> Result<Vec<String>> {
        self.0.entries()
    }

    pub fn unpack(&mut self, dest: impl AsRef<Path>) -> Result<()> {
        self.0.unpack(dest.as_ref())
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ArchiveKind {
    Zip,
    Tar,
    Gzip,
    Bzip2,
    Xz2,
    TarGzip,
    TarBzip2,
    TarXz2,
    Unknown,
}

macro_rules! match_ext {
    ($path: expr, $ext: expr) => {
        match $path.extension() {
            Some(ext) if ext.to_ascii_lowercase() == ::std::ffi::OsStr::new($ext) => true,
            _ => false,
        }
    };
    ($path: expr, $ext1: expr, $ext2: expr) => {
        match $path.extension() {
            Some(ext) if ext.to_ascii_lowercase() == ::std::ffi::OsStr::new($ext2) => {
                match $path.file_stem().map(::std::path::Path::new) {
                    Some(path) => match_ext!(path, $ext1),
                    _ => false,
                }
            }
            _ => false,
        }
    };
}

impl ArchiveKind {
    pub fn infer_from_file_extension(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();

        if match_ext!(path, "zip") {
            ArchiveKind::Zip
        } else if match_ext!(path, "tar") {
            ArchiveKind::Tar
        } else if match_ext!(path, "tgz") || match_ext!(path, "tar", "gz") {
            ArchiveKind::TarGzip
        } else if match_ext!(path, "tar", "xz") {
            ArchiveKind::TarXz2
        } else if match_ext!(path, "tar", "bz2") {
            ArchiveKind::TarBzip2
        } else if match_ext!(path, "gz") {
            ArchiveKind::Gzip
        } else if match_ext!(path, "xz") {
            ArchiveKind::Xz2
        } else if match_ext!(path, "bz2") {
            ArchiveKind::Bzip2
        } else {
            ArchiveKind::Unknown
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! assert_ext {
        ($path: expr, $expected: expr) => {
            assert_eq!(ArchiveKind::infer_from_file_extension($path), $expected)
        };
    }

    #[test]
    fn test_archive_type() -> Result<()> {
        assert_ext!("sample.zip", ArchiveKind::Zip);
        assert_ext!("sample.Zip", ArchiveKind::Zip);
        assert_ext!("sample.tar", ArchiveKind::Tar);
        assert_ext!("sample.TAR", ArchiveKind::Tar);
        assert_ext!("sample.tar.gz", ArchiveKind::TarGzip);
        assert_ext!("sample.tAr.gz", ArchiveKind::TarGzip);
        assert_ext!("sample.tgz", ArchiveKind::TarGzip);
        assert_ext!("sample.tar.xz", ArchiveKind::TarXz2);
        assert_ext!("sample.tar.bz2", ArchiveKind::TarBzip2);
        assert_ext!("sample.xz", ArchiveKind::Xz2);
        assert_ext!("sample.bz2", ArchiveKind::Bzip2);
        assert_ext!("sample.exe", ArchiveKind::Unknown);
        assert_ext!("sample.txt.gz", ArchiveKind::Gzip);
        Ok(())
    }
}
