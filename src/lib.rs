use std::{
    fs::File,
    io::{ErrorKind, Read, Seek, SeekFrom},
    path::Path,
};

trait Archived {
    fn unpack(&mut self, dest: &Path) -> std::io::Result<()>;
}

#[cfg(feature = "zip")]
impl<R: Read + Seek> Archived for zip::ZipArchive<R> {
    fn unpack(&mut self, dest: &Path) -> std::io::Result<()> {
        Ok(self.extract(dest)?)
    }
}

#[cfg(feature = "tar")]
impl<R: Read> Archived for tar::Archive<R> {
    fn unpack(&mut self, dest: &Path) -> std::io::Result<()> {
        self.unpack(dest)
    }
}

pub struct Archive(Box<dyn Archived>);

impl Archive {
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let mut magic = [0u8; 2];
        let mut file = File::open(&path)?;
        file.read_exact(&mut magic)?;
        file.seek(SeekFrom::Start(0))?;

        match magic {
            #[cfg(feature = "zip")]
            [0x50, 0x4B] => Ok(Archive(Box::new(zip::ZipArchive::new(file)?))),
            #[cfg(all(feature = "xz", feature = "tar"))]
            [0xFD, 0x37] => Ok(Archive(Box::new(tar::Archive::new(
                xz2::read::XzDecoder::new(file),
            )))),
            #[cfg(all(feature = "gzip", feature = "tar"))]
            [0x1F, 0x8B] => Ok(Archive(Box::new(tar::Archive::new(
                flate2::read::GzDecoder::new(file),
            )))),
            #[cfg(all(feature = "bzip2", feature = "tar"))]
            [0x42, 0x5A] => Ok(Archive(Box::new(tar::Archive::new(
                bzip2::read::BzDecoder::new(file),
            )))),
            _ => Err(ErrorKind::InvalidData.into()),
        }
    }

    pub fn unpack(&mut self, dest: impl AsRef<Path>) -> std::io::Result<()> {
        self.0.unpack(dest.as_ref())
    }
}
