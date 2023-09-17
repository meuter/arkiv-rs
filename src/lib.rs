#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

mod format;

#[allow(deprecated)]
pub use format::{ArchiveKind, Format};

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

/// A collection of files, possibly compressed (e.g. `tgz`, `zip`, ...).
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
            Format::Zip => Ok(Archive(Box::new(zip::ZipArchive::new(_file)?))),

            #[cfg(feature = "tar")]
            Format::Tar => Ok(Archive(Box::new(tar::Archive::new(_file)))),

            #[cfg(all(feature = "gzip", feature = "tar"))]
            Format::TarGzip => Ok(Archive(Box::new(tar::Archive::new(
                flate2::read::GzDecoder::new(_file),
            )))),

            #[cfg(all(feature = "bzip2", feature = "tar"))]
            Format::TarBzip2 => Ok(Archive(Box::new(tar::Archive::new(
                bzip2::read::BzDecoder::new(_file),
            )))),

            #[cfg(all(feature = "xz", feature = "tar"))]
            Format::TarXz2 => Ok(Archive(Box::new(tar::Archive::new(
                xz2::read::XzDecoder::new(_file),
            )))),

            #[cfg(all(feature = "zstd", feature = "tar"))]
            Format::TarZstd => Ok(Archive(Box::new(tar::Archive::new(
                zstd::stream::Decoder::new(_file)?,
            )))),

            _ => Err(ErrorKind::Other.into()),
        }
    }

    /// Returns the list of entries stored within the archive.
    ///
    /// # Warning
    ///
    /// For convenience, these entries are returned as an already
    /// collected `Vec<String>`. If the archive contains a large
    /// number of files, the amount of memory required to store
    /// these entries might be large.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arkiv::{Archive, Result};
    ///
    /// fn main() -> Result<()> {
    ///    let mut archive = Archive::open("path/to/archive.tgz")?;
    ///    let entries: Vec<String> = archive.entries()?;
    ///
    ///    for entry in entries {
    ///        print!("{entry}");
    ///    }
    ///    Ok(())
    /// }
    /// ```
    ///
    ///
    pub fn entries(&mut self) -> Result<Vec<String>> {
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
