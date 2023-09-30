use std::path::{Path, PathBuf};

use crate::Result;

/// The type of an entry within an archive.
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum EntryType {
    /// The entry is a directory
    Directory,

    /// The entry is a file
    File,

    /// The entry is neither a directory nor a regular file
    #[cfg(feature = "tar")]
    Other,
}

/// A descriptor of one entry in an archive.
#[derive(Debug, Clone)]
pub struct Entry {
    pub(crate) path: PathBuf,
    pub(crate) size: u64,
    pub(crate) entry_type: EntryType,
}

impl Entry {
    /// Returns the path of the entry
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the size of the entry
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Returns `true` if the entry corresponds to a directory
    pub fn is_dir(&self) -> bool {
        self.entry_type == EntryType::Directory
    }

    /// Returns `true` if the entry corresponds to a file
    pub fn is_file(&self) -> bool {
        self.entry_type == EntryType::File
    }
}

/// An iterator over the entries of the archive
pub type Entries<'a> = Box<dyn 'a + Iterator<Item = Result<Entry>>>;

/// An iterator over entries matching a given boolean predicate
pub struct FindEntries<'a, P: FnMut(&Entry) -> bool> {
    pub(crate) predicate: P,
    pub(crate) inner: Entries<'a>,
}

impl<'a, P: FnMut(&Entry) -> bool> Iterator for FindEntries<'a, P> {
    type Item = Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next()? {
                Ok(entry) => {
                    if (self.predicate)(&entry) {
                        return Some(Ok(entry));
                    }
                }
                Err(err) => return Some(Err(err)),
            }
        }
    }
}
