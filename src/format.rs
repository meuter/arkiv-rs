use std::path::Path;

/// Available archive file formats.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Format {
    /// Compressed zip archive
    Zip,

    /// Uncompressed tar archive
    Tar,

    /// File comressed with Gzip
    Gzip,

    /// File compressed with Zstd
    Zstd,

    /// File compressed with Bzip2
    Bzip2,

    /// File compressed with Xz2
    Xz2,

    /// Tar archive compressed with Gzip
    TarGzip,

    /// Tar archive compressed with Bzip2
    TarBzip2,

    /// Tar archive compressed with Xz2
    TarXz2,

    /// Tar archive compressed with Zstd
    TarZstd,

    /// unknown archive format.
    Unknown,
}

#[deprecated(
    since = "0.4.0",
    note = "arkiv::ArchiveKind has been renamed to arkiv::Format"
)]
pub type ArchiveKind = Format;

macro_rules! match_ext {
    ($path: expr, $ext: expr) => {
        match $path.extension() {
            Some(ext) if ext.to_ascii_lowercase() == std::ffi::OsStr::new($ext) => true,
            _ => false,
        }
    };
    ($path: expr, $ext1: expr, $ext2: expr) => {
        match $path.extension() {
            Some(ext) if ext.to_ascii_lowercase() == std::ffi::OsStr::new($ext2) => {
                match $path.file_stem().map(std::path::Path::new) {
                    Some(path) => match_ext!(path, $ext1),
                    _ => false,
                }
            }
            _ => false,
        }
    };
}

impl Format {
    /// Infers the archive format from the file extension of a provided
    /// path.
    ///
    /// # Arguments
    ///
    /// - `path`: a path to a candidate archive file
    ///
    /// # Example
    ///
    /// ```
    /// use arkiv::Format;
    ///
    /// let format = Format::infer_from_file_extension("sample/sample.tgz");
    /// assert_eq!(format, Format::TarGzip);
    /// ```
    pub fn infer_from_file_extension(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();

        if match_ext!(path, "zip") {
            Format::Zip
        } else if match_ext!(path, "tar") {
            Format::Tar
        } else if match_ext!(path, "tgz") || match_ext!(path, "tar", "gz") {
            Format::TarGzip
        } else if match_ext!(path, "tar", "xz") {
            Format::TarXz2
        } else if match_ext!(path, "tar", "bz2") {
            Format::TarBzip2
        } else if match_ext!(path, "tar", "zstd") || match_ext!(path, "tar", "zst") {
            Format::TarZstd
        } else if match_ext!(path, "gz") {
            Format::Gzip
        } else if match_ext!(path, "xz") {
            Format::Xz2
        } else if match_ext!(path, "bz2") {
            Format::Bzip2
        } else if match_ext!(path, "zstd") || match_ext!(path, "zst") {
            Format::Zstd
        } else {
            Format::Unknown
        }
    }

    /// Returns `true` if a the format is compressed
    ///
    /// Example
    ///
    /// ```
    /// use arkiv::Format;
    ///
    /// assert_eq!(Format::Tar.is_compressed(), false);
    /// assert_eq!(Format::TarGzip.is_compressed(), true);
    /// assert_eq!(Format::Gzip.is_compressed(), true);
    /// assert_eq!(Format::Zip.is_compressed(), true);
    /// ```
    pub fn is_compressed(&self) -> bool {
        !matches!(self, Format::Tar)
    }

    /// Returns `true` if the format is an archive (as opposed
    /// to a single compressed file).
    ///
    /// #Example
    /// ```
    /// use arkiv::Format;
    ///
    /// assert_eq!(Format::Tar.is_archive(), true);
    /// assert_eq!(Format::TarGzip.is_archive(), true);
    /// assert_eq!(Format::Gzip.is_archive(), false);
    /// assert_eq!(Format::Zip.is_archive(), true);
    /// ```
    pub fn is_archive(&self) -> bool {
        match self {
            Format::Zip => true,
            Format::Tar => true,
            Format::Gzip => false,
            Format::Zstd => false,
            Format::Bzip2 => false,
            Format::Xz2 => false,
            Format::TarGzip => true,
            Format::TarBzip2 => true,
            Format::TarXz2 => true,
            Format::TarZstd => true,
            Format::Unknown => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn infer_from_file_extension() {
        macro_rules! assert_ext {
            ($path: expr, $expected: expr) => {
                assert_eq!(Format::infer_from_file_extension($path), $expected)
            };
        }
        assert_ext!("sample.zip", Format::Zip);
        assert_ext!("sample.Zip", Format::Zip);
        assert_ext!("sample.tar", Format::Tar);
        assert_ext!("sample.TAR", Format::Tar);
        assert_ext!("sample.tar.gz", Format::TarGzip);
        assert_ext!("sample.tAr.gz", Format::TarGzip);
        assert_ext!("sample.tgz", Format::TarGzip);
        assert_ext!("sample.tar.xz", Format::TarXz2);
        assert_ext!("sample.tar.bz2", Format::TarBzip2);
        assert_ext!("sample.tar.zstd", Format::TarZstd);
        assert_ext!("sample.tar.zst", Format::TarZstd);
        assert_ext!("sample.xz", Format::Xz2);
        assert_ext!("sample.bz2", Format::Bzip2);
        assert_ext!("sample.exe", Format::Unknown);
        assert_ext!("sample.txt.gz", Format::Gzip);
        assert_ext!("sample.txt.zstd", Format::Zstd);
        assert_ext!("sample.txt.zst", Format::Zstd);
    }

    #[test]
    fn is_compressed() {
        macro_rules! assert_ext {
            ($format: expr, $expected: expr) => {
                assert_eq!(($format).is_compressed(), $expected)
            };
        }
        assert_ext!(Format::Zip, true);
        assert_ext!(Format::Tar, false);
        assert_ext!(Format::TarGzip, true);
        assert_ext!(Format::TarXz2, true);
        assert_ext!(Format::TarBzip2, true);
        assert_ext!(Format::TarZstd, true);
        assert_ext!(Format::Xz2, true);
        assert_ext!(Format::Bzip2, true);
        assert_ext!(Format::Gzip, true);
        assert_ext!(Format::Zstd, true);
        assert_ext!(Format::Zstd, true);
    }

    #[test]
    fn is_archive() {
        macro_rules! assert_ext {
            ($format: expr, $expected: expr) => {
                assert_eq!(($format).is_archive(), $expected)
            };
        }
        assert_ext!(Format::Zip, true);
        assert_ext!(Format::Tar, true);
        assert_ext!(Format::TarGzip, true);
        assert_ext!(Format::TarXz2, true);
        assert_ext!(Format::TarBzip2, true);
        assert_ext!(Format::TarZstd, true);
        assert_ext!(Format::Xz2, false);
        assert_ext!(Format::Bzip2, false);
        assert_ext!(Format::Gzip, false);
        assert_ext!(Format::Zstd, false);
        assert_ext!(Format::Zstd, false);
    }
}
