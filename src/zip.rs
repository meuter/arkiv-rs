use std::{
    fs::{create_dir_all, set_permissions, File, Permissions},
    io::{self, Read, Seek},
    path::Path,
};

use zip::{
    read::ZipFile,
    result::{ZipError, ZipResult},
    ZipArchive,
};

use crate::{archive::Archived, entry::EntryType, Entries, Entry, Error, Result};

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
        fn convert(index: usize, zip_file: ZipResult<ZipFile>) -> Result<Entry> {
            let zip_file = zip_file?;
            let path = zip_file
                .enclosed_name()
                .ok_or(Error::InvalidArchive("invalid filename"))?
                .to_path_buf();
            let size = zip_file.size();
            let entry_type = if zip_file.is_dir() {
                EntryType::Directory
            } else {
                EntryType::File
            };
            let entry = Entry {
                index,
                path,
                size,
                entry_type,
            };
            Ok(entry)
        }

        if self.index < self.archive.len() {
            let index = self.index;
            self.index += 1;
            Some(convert(index, self.archive.by_index(index)))
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
        let archive = self;
        let index = 0;
        let zip_entries = ZipEntries { archive, index };
        Ok(Box::new(zip_entries))
    }

    fn unpack_entry(&mut self, entry: &Entry, dest: &Path) -> Result<()> {
        let outpath = dest.join(entry.path());
        if entry.is_dir() {
            create_dir_all(&outpath)?;
        } else if entry.is_file() {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    create_dir_all(p)?;
                }
            }
            let mut file_in_zip = self.by_index(entry.index())?;
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file_in_zip, &mut outfile)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file_in_zip.unix_mode() {
                    set_permissions(&outpath, Permissions::from_mode(mode))?;
                }
            }
        }

        Ok(())
    }
}
