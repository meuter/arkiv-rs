use ureq::Response;

use crate::{archive::Storage, Archive};

use super::{Error, Result};
use std::{
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

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

/// Progress callback is not provided in [Downloader]
pub struct OnProgressNotProvided;

/// Progress callback is provided in [Downloader]
pub struct OnProgressProvided<F: FnMut(u64, u64)>(F);

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
pub struct Downloader<U, D, O> {
    url: U,
    dest: D,
    on_progress: O,
}

impl Downloader<UrlMissing, DestMissing, OnProgressNotProvided> {
    /// Returns a new `Downloader`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Downloader<UrlMissing, DestMissing, OnProgressNotProvided> {
    /// Returns a default `Downloader`.
    fn default() -> Self {
        Self {
            url: UrlMissing,
            dest: DestMissing,
            on_progress: OnProgressNotProvided,
        }
    }
}

impl<D, O> Downloader<UrlMissing, D, O> {
    /// Allows to specify the URL to the archive that should be downloaded.
    ///
    /// # Arguments
    /// - `url`: the URL to the archive to downaload
    ///
    pub fn url(self, url: impl AsRef<str>) -> Downloader<UrlProvided, D, O> {
        let url = UrlProvided(url.as_ref().to_string());
        let dest = self.dest;
        let on_progress = self.on_progress;
        Downloader {
            url,
            dest,
            on_progress,
        }
    }
}

impl<U, O> Downloader<U, DestMissing, O> {
    /// Allows to specify that the downloaded archive file should be stored
    /// in the temporary directory.
    pub fn to_temp(self) -> Downloader<U, DestProvided, O> {
        let url = self.url;
        let dest = DestProvided::TempDir;
        let on_progress = self.on_progress;
        Downloader {
            url,
            dest,
            on_progress,
        }
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
    pub fn to_directory(self, dest: impl AsRef<Path>) -> Downloader<U, DestProvided, O> {
        let url = self.url;
        let dest = DestProvided::Dir(dest.as_ref().to_path_buf());
        let on_progress = self.on_progress;
        Downloader {
            url,
            dest,
            on_progress,
        }
    }
}

impl<U, D> Downloader<U, D, OnProgressNotProvided> {
    /// Sets a callback that will be regularily called during the download to
    /// nonitor the progress.
    ///
    /// # Arguments
    ///
    /// - `callback`: closure that will be called with two values:
    ///     - the current number of bytes already downloaded
    ///     - the total number of bytes that needs to be downloaded
    ///
    /// # Example
    ///
    ///
    pub fn on_progress<F>(self, callback: F) -> Downloader<U, D, OnProgressProvided<F>>
    where
        F: FnMut(u64, u64),
    {
        let url = self.url;
        let dest = self.dest;
        let on_progress = OnProgressProvided(callback);
        Downloader {
            url,
            dest,
            on_progress,
        }
    }
}

impl<O> Downloader<UrlProvided, DestProvided, O> {
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

impl<D, O> Downloader<UrlProvided, D, O> {
    fn get(&self) -> Result<Response> {
        let response = ureq::get(&self.url.0)
            .call()
            .map_err(|err| Error::InvalidRequest(err.to_string()))?;
        Ok(response)
    }
}

impl Downloader<UrlProvided, DestProvided, OnProgressNotProvided> {
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

impl<F: FnMut(u64, u64)> Downloader<UrlProvided, DestProvided, OnProgressProvided<F>> {
    /// Downloads the archive and opens it. Return an [Archive]. If the
    /// the archive file was downloaded to a temporary directory, the file will
    /// be deleted once the [Archive] is dropped.
    ///
    /// During the download the provided progress callback will be called.
    pub fn download(mut self) -> Result<Archive> {
        let response = self.get()?;
        let content_length = response
            .header("content-length")
            .ok_or(Error::InvalidRequest(
                "response does not contain 'content-length' header".to_string(),
            ))?
            .parse::<u64>()
            .map_err(|err| {
                Error::InvalidRequest(format!(
                    "'content-length' in the response header could not be parsed '{err}'"
                ))
            })?;
        let storage = self.storage()?;
        let mut source = response.into_reader();
        let mut dest = storage.create()?;

        let mut buf = [0; 16384];
        let mut written = 0;
        loop {
            self.on_progress.0(written as u64, content_length);
            let bytes_read = match source.read(&mut buf) {
                Ok(0) => return Archive::new(storage),
                Ok(len) => len,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e.into()),
            };
            dest.write_all(&buf[..bytes_read])?;
            written += bytes_read;
        }
    }
}
