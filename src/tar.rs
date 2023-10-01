use std::{
    fs::create_dir_all,
    io::{self, Read},
    iter::Enumerate,
    path::Path,
};

use crate::{archive::Archived, entry::EntryType, Entries, Entry, Error, Result};

struct TarEntries<'a, R: 'a + Read>(Enumerate<::tar::Entries<'a, R>>);

impl<'a, R> Iterator for TarEntries<'a, R>
where
    R: Read,
{
    type Item = Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        fn convert<'a, R: 'a + Read>(
            index: usize,
            orig_tar_entry: io::Result<tar::Entry<'a, R>>,
        ) -> Result<Entry> {
            let orig_tar_entry = orig_tar_entry?;
            let path = orig_tar_entry.path()?.to_path_buf();
            let size = orig_tar_entry.size();
            let entry_type = match orig_tar_entry.header().entry_type() {
                tar::EntryType::Regular => EntryType::File,
                tar::EntryType::Directory => EntryType::Directory,
                _ => EntryType::Other,
            };
            let entry = Entry {
                index,
                path,
                size,
                entry_type,
            };
            Ok(entry)
        }

        let (index, orig_tar_entry) = self.0.next()?;
        Some(convert(index, orig_tar_entry))
    }
}

impl<R: Read> Archived for tar::Archive<R> {
    fn unpack(&mut self, dest: &Path) -> Result<()> {
        Ok(self.unpack(dest)?)
    }

    fn entries(&mut self) -> Result<Entries> {
        let inner_entries = tar::Archive::entries(self)?.enumerate();
        Ok(Box::new(TarEntries(inner_entries)))
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
            // NOTE: this is terrible, we have to re-lookup the entry
            //       in the archive...
            for file_in_tar in tar::Archive::entries(self)? {
                let mut file_in_tar = file_in_tar?;
                if file_in_tar.path()? == entry.path() {
                    file_in_tar.unpack(outpath)?;
                    return Ok(());
                }
            }
        }
        Err(Error::FileNotFound)
    }
}
