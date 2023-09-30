use ureq::Response;

use crate::{archive::Storage, Archive};

use super::{Error, Result};
use std::path::{Path, PathBuf};

/// URL is missing in [Downloader].
pub struct UrlMissing;

/// URL is provided in [Downloader].
pub struct UrlProvided(String);

/// Download destination is missing in [Downloader].
pub struct DestMissing;

/// Download destination is provided in [Downloader].
pub enum DestProvided {
    /// Archive file will be downloaded in a temporary directory.
    TempDir,

    /// Archive file will be downloaded in a specific directory.
    Dir(PathBuf),
}

/// Allows to download an archive file and open it. This struct
/// provides a bit more flexibility compared to [Archive::download]
///
/// This type is only available if the `download` feature is enabled.
///
/// # Example
///
/// ```no_run
/// use arkiv::{Downloader, Result};
///
/// pub fn example() -> Result<()> {
///     let url = "https://github.com/meuter/arkiv-rs/raw/main/tests/sample/sample.zip";
///     let mut archive = Downloader::new()
///         .url(url)
///         .to_temp()
///         .download()?;
///     archive.unpack("/path/to/unpacked")?;
///     Ok(())
/// }
///
/// ```
pub struct Downloader<U, D> {
    url: U,
    dest: D,
}

impl Downloader<UrlMissing, DestMissing> {
    /// Returns a new `Downloader`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Downloader<UrlMissing, DestMissing> {
    /// Returns a default `Downloader`.
    fn default() -> Self {
        Self {
            url: UrlMissing,
            dest: DestMissing,
        }
    }
}

impl<D> Downloader<UrlMissing, D> {
    /// Allows to specify the URL to the archive that should be downloaded.
    ///
    /// # Arguments
    /// - `url`: the URL to the archive to downaload
    ///
    pub fn url(self, url: impl AsRef<str>) -> Downloader<UrlProvided, D> {
        let url = UrlProvided(url.as_ref().to_string());
        let dest = self.dest;
        Downloader { url, dest }
    }
}

impl<U> Downloader<U, DestMissing> {
    /// Allows to specify that the downloaded archive file should be stored
    /// in the temporary directory.
    pub fn to_temp(self) -> Downloader<U, DestProvided> {
        let url = self.url;
        let dest = DestProvided::TempDir;
        Downloader { url, dest }
    }

    /// Allows to specify that the downloaded archive file should be stored
    /// in the specified destination directory. If this directory does not
    /// exists, it will be created when the archive is downloaded.
    ///
    /// See [Downloader::download]
    ///
    /// # Arguments
    /// - `dest`: the destination directory.
    ///
    pub fn to_directory(self, dest: impl AsRef<Path>) -> Downloader<U, DestProvided> {
        let url = self.url;
        let dest = DestProvided::Dir(dest.as_ref().to_path_buf());
        Downloader { url, dest }
    }
}

impl Downloader<UrlProvided, DestProvided> {
    fn storage(&self) -> Result<Storage> {
        let file_name = Path::new(&self.url.0)
            .file_name()
            .ok_or(Error::InvalidUrl(self.url.0.clone()))?;

        let storage = match &self.dest {
            DestProvided::TempDir => Storage::FileInTempDirectory {
                temp: tempfile::tempdir()?,
                file_name: file_name.to_os_string(),
            },
            DestProvided::Dir(dir) => Storage::FileOnDisk {
                path: dir.join(file_name),
            },
        };
        Ok(storage)
    }
}

impl<D> Downloader<UrlProvided, D> {
    fn get(&self) -> Result<Response> {
        let response = ureq::get(&self.url.0)
            .call()
            .map_err(|err| Error::InvalidRequest(err.to_string()))?;
        Ok(response)
    }
}

impl Downloader<UrlProvided, DestProvided> {
    /// Downloads the archive and opens it. Return an [Archive]. If the
    /// the archive file was downloaded to a temporary directory, the file will
    /// be deleted once the [Archive] is dropped.
    pub fn download(self) -> Result<Archive> {
        let response = self.get()?;
        let storage = self.storage()?;

        let mut source = response.into_reader();
        let mut dest = storage.create()?;

        std::io::copy(&mut source, &mut dest)?;

        Archive::new(storage)
    }
}
