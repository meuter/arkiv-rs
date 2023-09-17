use std::path::Path;

/// available archive file formats.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Format {
    /// compressed zip archive
    Zip,

    /// uncompressed tar archive
    Tar,

    /// file comressed with Gzip
    Gzip,

    /// file compressed with Zstd
    Zstd,

    /// file compressed with Bzip2
    Bzip2,

    /// file compressed with Xz2
    Xz2,

    /// tar archive compressed with Gzip
    TarGzip,

    /// tar archive compressed with Bzip2
    TarBzip2,

    /// tar archive compressed with Xz2
    TarXz2,

    /// tar archive compressed with Zstd
    TarZstd,

    /// Unknown archive format.
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
            Some(ext) if ext.to_ascii_lowercase() == ::std::ffi::OsStr::new($ext) => true,
            _ => false,
        }
    };
    ($path: expr, $ext1: expr, $ext2: expr) => {
        match $path.extension() {
            Some(ext) if ext.to_ascii_lowercase() == ::std::ffi::OsStr::new($ext2) => {
                match $path.file_stem().map(::std::path::Path::new) {
                    Some(path) => match_ext!(path, $ext1),
                    _ => false,
                }
            }
            _ => false,
        }
    };
}

impl Format {
    /// infers the archive format from the file extension of a provided
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
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Result;

    macro_rules! assert_ext {
        ($path: expr, $expected: expr) => {
            assert_eq!(Format::infer_from_file_extension($path), $expected)
        };
    }

    #[test]
    fn test_archive_type() -> Result<()> {
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
        Ok(())
    }
}
