use std::{
    borrow::Cow,
    fs::File,
    path::{Path, PathBuf},
};

#[cfg(feature = "zip")]
use zip::ZipArchive as Zip;

#[cfg(feature = "tar")]
use tar::Archive as Tar;

#[cfg(all(feature = "tar", feature = "gzip"))]
use flate2::read::GzDecoder;

#[cfg(all(feature = "tar", feature = "bzip2"))]
use bzip2::read::BzDecoder;

#[cfg(all(feature = "tar", feature = "xz2"))]
use xz2::read::XzDecoder;

#[cfg(all(feature = "tar", feature = "zstd"))]
use zstd::stream::Decoder as ZstdDecoder;

use crate::{Entries, Entry, Error, FindEntries, Format, Result};

/// private interface for an archive backend (zip or archive)
pub(crate) trait Archived {
    fn unpack(&mut self, dest: &Path) -> Result<()>;
    fn entries(&mut self) -> Result<Entries>;
    fn unpack_entry(&mut self, entry: &Entry, dest: &Path) -> Result<()>;
}

#[derive(Debug)]
pub(crate) enum Storage {
    FileOnDisk {
        path: PathBuf,
    },
    #[cfg(feature = "download")]
    FileInTempDirectory {
        temp: tempfile::TempDir,
        file_name: std::ffi::OsString,
    },
}

impl Storage {
    pub(crate) fn as_path(&self) -> Cow<Path> {
        match self {
            Storage::FileOnDisk { path } => Cow::Borrowed(path),
            #[cfg(feature = "download")]
            Storage::FileInTempDirectory { temp, file_name } => {
                Cow::Owned(temp.path().join(file_name))
            }
        }
    }

    #[cfg(feature = "download")]
    pub(crate) fn create(&self) -> Result<File> {
        if let Storage::FileOnDisk { path } = self {
            std::fs::create_dir_all(path)?;
        }
        Ok(File::create(self.as_path())?)
    }
}

/// A collection of files, possibly compressed (e.g. `tar`, `tar.gz`, `zip`, ...).
///
/// # Supported Formats
///
/// - `sample.zip` (requires the zip feature).
/// - `sample.tar` (requires the `tar` feature).
/// - `sample.tgz` or `sample.tar.gz` (requires `tar` and `gzip` features).
/// - `sample.tar.xz` (requires `tar` and `xz` features).
/// - `sample.tar.bz2` (requires `tar` and `bzip` features).
/// - `sample.tar.zstd` or `sample.tar.zst` (requires `tar` and `zstd` features).
pub struct Archive {
    format: Format,
    storage: Storage,
    archived: Option<Box<dyn Archived>>,
}

impl Archive {
    pub(crate) fn new(storage: Storage) -> Result<Self> {
        let archived = None;
        let format = Format::infer_from_file_extension(storage.as_path());
        if !format.is_archive() {
            Err(Error::UnsupportedArchive(
                "unsupported format, did you enable the proper feature?",
            ))?;
        }

        Ok(Archive {
            format,
            storage,
            archived,
        })
    }

    /// Opens an archive stored on the filesystem.
    ///
    /// The format of the archive will be inferred from the file
    /// extension. See [`infer_from_file_extension`](Format::infer_from_file_extension).
    ///
    /// # Arguments:
    ///
    /// - `path`: the path to the archive file to open
    ///
    /// # Examples:
    ///
    /// ```
    /// use arkiv::Archive;
    ///
    /// let archive = Archive::open("path/to/archive.zip");
    /// ```
    ///
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let storage = Storage::FileOnDisk { path };
        Archive::new(storage)
    }

    /// Downloads an archive to a temporary directory and opens the archive.
    ///
    /// This function is only available if the `download` feature is enabled.
    ///
    /// This function is a simple convenience wrapper around the [`Downloader`](crate::Downloader),
    /// which provides more features.
    ///
    /// # Arguments:
    ///
    /// - `url`: the url to the archive file to open
    ///
    /// # Examples:
    ///
    /// ```no_run
    /// use arkiv::Archive;
    ///
    /// let url = "https://github.com/meuter/arkiv-rs/raw/main/tests/sample/sample.zip";
    /// let archive = Archive::download(url);
    /// ```
    ///
    #[cfg(feature = "download")]
    pub fn download(url: impl AsRef<str>) -> Result<Self> {
        crate::Downloader::new().url(url).to_temp().download()
    }

    fn archived(&mut self) -> Result<&mut Box<dyn Archived>> {
        #[allow(unused)]
        let file = File::open(self.path())?;

        let result: Result<Box<dyn Archived>> = match self.format {
            #[cfg(feature = "zip")]
            Format::Zip => Ok(Box::new(Zip::new(file)?)),

            #[cfg(feature = "tar")]
            Format::Tar => Ok(Box::new(Tar::new(file))),

            #[cfg(all(feature = "tar", feature = "gzip"))]
            Format::TarGzip => Ok(Box::new(Tar::new(GzDecoder::new(file)))),

            #[cfg(all(feature = "tar", feature = "bzip2"))]
            Format::TarBzip2 => Ok(Box::new(Tar::new(BzDecoder::new(file)))),

            #[cfg(all(feature = "tar", feature = "xz2"))]
            Format::TarXz2 => Ok(Box::new(Tar::new(XzDecoder::new(file)))),

            #[cfg(all(feature = "tar", feature = "zstd"))]
            Format::TarZstd => Ok(Box::new(Tar::new(ZstdDecoder::new(file)?))),

            _ => Err(Error::UnsupportedArchive(
                "unsupported format, did you enable the proper feature?",
            )),
        };

        self.archived.replace(result?);
        Ok(self
            .archived
            .as_mut()
            .expect("inner was freshly replaced, this should never happen"))
    }

    /// Returns the format of the archive.
    pub fn format(&self) -> &Format {
        &self.format
    }

    /// Returns the path of the archive.
    pub fn path(&self) -> Cow<Path> {
        self.storage.as_path()
    }

    /// Returns the list of entries stored within the archive.
    ///
    /// # Warning
    ///
    /// For convenience, these entries are returned as an already
    /// collected `Vec<String>`. If the archive contains a large
    /// number of files, the amount of memory required to store
    /// these entries might be large. See [`entries_iter`](Self::entries_iter)
    /// for an iterator version.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arkiv::{Archive, Result};
    ///
    /// fn main() -> Result<()> {
    ///     let mut archive = Archive::open("path/to/archive.tgz")?;
    ///     let entries: Vec<String> = archive.entries()?;
    ///
    ///     for entry in entries {
    ///         print!("{entry}");
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn entries(&mut self) -> Result<Vec<String>> {
        let mut result: Vec<String> = vec![];
        for entry in self.entries_iter()? {
            let entry = entry?;
            result.push(entry.path().display().to_string());
        }
        Ok(result)
    }

    /// Constructs an iterator over the entries in this archive
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arkiv::{Archive, Result};
    ///
    /// fn main() -> Result<()> {
    ///     let mut archive = Archive::open("path/to/archive.tgz")?;
    ///
    ///     for entry in archive.entries_iter()? {
    ///         let entry = entry?;
    ///         print!("{}", entry.path().display());
    ///     }
    ///     Ok(())
    /// }
    /// ```
    ///
    ///
    pub fn entries_iter(&mut self) -> Result<Entries> {
        self.archived()?.entries()
    }

    /// Unpacks the contents of the archive. On unix systems all permissions
    /// will be preserved.
    ///
    /// # Arguments
    ///
    /// - `dest`: the destination folder (will be created if necessary)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arkiv::{Archive, Result};
    //
    /// fn main() -> Result<()> {
    ///    let mut archive = Archive::open("path/to/archive.tgz")?;
    ///    archive.unpack("/tmp/extracted/")?;
    ///    Ok(())
    /// }
    /// ```
    pub fn unpack(&mut self, dest: impl AsRef<Path>) -> Result<()> {
        self.archived()?.unpack(dest.as_ref())
    }

    /// Returns an entry corresponding to a given path within the archive
    ///
    /// # Arguments
    ///
    /// - `entry_path`: the path of the enty to look up
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arkiv::{Archive, Result};
    //
    /// fn main() -> Result<()> {
    ///    let mut archive = Archive::open("path/to/archive.tgz")?;
    ///    let entry = archive.entry_by_name("some/file_in_the_archive.txt")?;
    ///    println!("{}", entry.path().display());
    ///    Ok(())
    /// }
    /// ```
    pub fn entry_by_name(&mut self, entry_path: impl AsRef<Path>) -> Result<Entry> {
        self.find(|entry| entry.path() == entry_path.as_ref())?
            .next()
            .unwrap_or(Err(Error::FileNotFound))
    }

    /// Returns an iterator over the entries in the archive
    /// that match a given boolean predicate.
    ///
    /// # Arguments
    ///
    /// - `predicate`: a boolean predicate on `Entry`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use arkiv::{Archive, Result};
    ///
    /// fn main() -> Result<()> {
    ///     let mut archive = Archive::open("path/to/archive.tgz")?;
    ///
    ///     // iterate over all the files in the archive
    ///     for entry in archive.find(|entry| entry.is_file())? {
    ///         let entry = entry?;
    ///         print!("{}", entry.path().display());
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn find<P: FnMut(&Entry) -> bool>(&mut self, predicate: P) -> Result<FindEntries<P>> {
        Ok(FindEntries {
            predicate,
            inner: self.entries_iter()?,
        })
    }

    /// Extracts an entry to the provided destination directory.
    ///
    /// If the entry is a directory, the corresponding directory
    /// will be created in the destination folder.
    ///
    /// If the entry is a file, the contents of the file will be extracted
    /// and stored in at the same path as the enty, but relative to the
    /// destination directory.
    ///
    /// The destination directory and ann intermediate directory to it
    /// will be created as necessary.
    ///
    /// # Arguments
    ///
    /// - `entry`: the entry to extract
    /// - `dest`: path to a directory where the entry will be extracted.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arkiv::{Archive, Result};
    /// use std::fs::read_to_string;
    //
    /// fn main() -> Result<()> {
    ///    let mut archive = Archive::open("path/to/archive.tgz")?;
    ///    let some_file = archive.entry_by_name("some/file/in/archive.txt")?;
    ///    archive.unpack_entry(&some_file, "/tmp/extracted")?;
    ///
    ///    let some_file_contents = read_to_string("/tmp/extracted/file/in/archive.txt")?;
    ///    Ok(())
    /// }
    /// ```
    pub fn unpack_entry(&mut self, entry: &Entry, dest: impl AsRef<Path>) -> Result<()> {
        self.archived()?.unpack_entry(entry, dest.as_ref())
    }
}
