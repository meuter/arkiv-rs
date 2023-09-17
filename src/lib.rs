#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

mod archive;
mod format;
mod result;

pub use archive::Archive;
pub use format::Format;
pub use result::{Error, Result};

#[allow(deprecated)]
pub use format::ArchiveKind;

use std::path::Path;

pub(crate) trait Archived {
    fn unpack(&mut self, dest: &Path) -> Result<()>;
    fn entries(&mut self) -> Result<Vec<String>>;
}

#[cfg(feature = "zip")]
mod zip_feature {
    use super::*;

    impl From<::zip::result::ZipError> for Error {
        fn from(value: ::zip::result::ZipError) -> Self {
            match value {
                ::zip::result::ZipError::Io(err) => Error::Io(err),
                ::zip::result::ZipError::InvalidArchive(err) => Error::InvalidArchive(err),
                ::zip::result::ZipError::UnsupportedArchive(err) => Error::UnsupportedArchive(err),
                ::zip::result::ZipError::FileNotFound => Error::FileNotFound,
            }
        }
    }

    impl<R: ::std::io::Read + ::std::io::Seek> Archived for zip::ZipArchive<R> {
        fn unpack(&mut self, dest: &Path) -> Result<()> {
            Ok(self.extract(dest)?)
        }

        fn entries(&mut self) -> Result<Vec<String>> {
            let files = self.file_names().map(|e| e.into()).collect();
            Ok(files)
        }
    }
}

#[cfg(feature = "tar")]
impl<R: ::std::io::Read> Archived for tar::Archive<R> {
    fn unpack(&mut self, dest: &Path) -> Result<()> {
        Ok(self.unpack(dest)?)
    }

    fn entries(&mut self) -> Result<Vec<String>> {
        let files = tar::Archive::entries(self)?
            .map(|e| e.unwrap().path().unwrap().to_str().unwrap().into())
            .collect();
        Ok(files)
    }
}
