use std::{
    io::{self, Read},
    path::Path,
};

use crate::{
    archive::{Archived, Entries},
    Entry, Result,
};

struct TarEntries<'a, R: 'a + Read>(::tar::Entries<'a, R>);

impl<'a, R> Iterator for TarEntries<'a, R>
where
    R: Read,
{
    type Item = Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        fn convert<'a, R: 'a + Read>(
            orig_tar_entry: io::Result<tar::Entry<'a, R>>,
        ) -> Result<Entry> {
            let path = orig_tar_entry?.path()?.to_path_buf();
            let entry = Entry { path };
            Ok(entry)
        }

        Some(convert(self.0.next()?))
    }
}

impl<R: Read> Archived for tar::Archive<R> {
    fn unpack(&mut self, dest: &Path) -> Result<()> {
        Ok(self.unpack(dest)?)
    }

    fn entries(&mut self) -> Result<Vec<String>> {
        let files = tar::Archive::entries(self)?
            .map(|e| e.unwrap().path().unwrap().to_str().unwrap().into())
            .collect();
        Ok(files)
    }

    fn entries_iter(&mut self) -> Result<Entries> {
        let inner_entries = tar::Archive::entries(self)?;
        Ok(Box::new(TarEntries(inner_entries)))
    }
}
