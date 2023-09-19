use std::{
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

use crate::{Error, Format, Result};

/// A descriptor of one entry in an archive.
#[derive(Debug)]
pub struct Entry {
    pub(crate) path: PathBuf,
}

impl Entry {
    /// Returns the path of the entry
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// An iterator over the entries of the archive
pub type Entries<'a> = Box<dyn 'a + Iterator<Item = Result<Entry>>>;

/// private interface for an archive backend (zip or archive)
pub(crate) trait Archived {
    fn unpack(&mut self, dest: &Path) -> Result<()>;
    fn entries(&mut self) -> Result<Entries>;
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
pub struct Archive(Box<dyn Archived>);

impl Archive {
    /// Opens an archive stored on the filesystem.
    ///
    /// The format of the archive will be inferred from the file
    /// extension. See [infer_from_file_extension](Format::infer_from_file_extension).
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
        let _file = File::open(&path)?;
        match Format::infer_from_file_extension(path) {
            #[cfg(feature = "zip")]
            Format::Zip => Ok(Archive(Box::new(Zip::new(_file)?))),

            #[cfg(feature = "tar")]
            Format::Tar => Ok(Archive(Box::new(Tar::new(_file)))),

            #[cfg(all(feature = "tar", feature = "gzip"))]
            Format::TarGzip => Ok(Archive(Box::new(Tar::new(GzDecoder::new(_file))))),

            #[cfg(all(feature = "tar", feature = "bzip2"))]
            Format::TarBzip2 => Ok(Archive(Box::new(Tar::new(BzDecoder::new(_file))))),

            #[cfg(all(feature = "tar", feature = "xz2"))]
            Format::TarXz2 => Ok(Archive(Box::new(Tar::new(XzDecoder::new(_file))))),

            #[cfg(all(feature = "tar", feature = "zstd"))]
            Format::TarZstd => Ok(Archive(Box::new(Tar::new(ZstdDecoder::new(_file)?)))),

            _ => Err(Error::UnsupportedArchive(
                "unsupported format, did you enable the proper feature?",
            )),
        }
    }

    /// Returns the list of entries stored within the archive.
    ///
    /// # Warning
    ///
    /// For convenience, these entries are returned as an already
    /// collected `Vec<String>`. If the archive contains a large
    /// number of files, the amount of memory required to store
    /// these entries might be large. See [Archive::entries_iter]
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
        self.0.entries()
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
        self.0.unpack(dest.as_ref())
    }
}
