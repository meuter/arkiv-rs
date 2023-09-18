use std::path::Path;

use crate::{archive::Archived, Error, Result};

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
