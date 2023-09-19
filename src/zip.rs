use std::{
    io::{Read, Seek},
    path::Path,
};

use zip::{
    read::ZipFile,
    result::{ZipError, ZipResult},
    ZipArchive,
};

use crate::{
    archive::{Archived, Entries},
    Entry, Error, Result,
};

impl From<::zip::result::ZipError> for Error {
    fn from(value: ZipError) -> Self {
        match value {
            ZipError::Io(err) => Error::Io(err),
            ZipError::InvalidArchive(err) => Error::InvalidArchive(err),
            ZipError::UnsupportedArchive(err) => Error::UnsupportedArchive(err),
            ZipError::FileNotFound => Error::FileNotFound,
        }
    }
}

struct ZipEntries<'a, R: 'a> {
    index: usize,
    archive: &'a mut ZipArchive<R>,
}

impl<'a, R> Iterator for ZipEntries<'a, R>
where
    R: 'a + Seek + Read,
    Self: 'a,
{
    type Item = Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        fn convert(zip_file: ZipResult<ZipFile>) -> Result<Entry> {
            let zip_file = zip_file?;
            let path = zip_file
                .enclosed_name()
                .ok_or(Error::InvalidArchive("invalid filename"))?
                .to_path_buf();
            let entry = Entry { path };
            Ok(entry)
        }

        if self.index < self.archive.len() {
            let index = self.index;
            self.index += 1;
            Some(convert(self.archive.by_index(index)))
        } else {
            None
        }
    }
}

impl<R: Read + Seek> Archived for ZipArchive<R> {
    fn unpack(&mut self, dest: &Path) -> Result<()> {
        Ok(self.extract(dest)?)
    }

    fn entries(&mut self) -> Result<Entries> {
        // Ok(Box::new(ZipEntries(self.file_names())))
        let archive = self;
        let index = 0;
        let zip_entries = ZipEntries { archive, index };
        Ok(Box::new(zip_entries))
    }
}
