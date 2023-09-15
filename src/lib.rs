use std::{
    fs::File,
    io::{ErrorKind, Read, Seek, SeekFrom},
    path::Path,
};

pub type Error = std::io::Error;
pub type Result<T> = std::io::Result<T>;

trait Archived {
    fn unpack(&mut self, dest: &Path) -> Result<()>;
    fn entries(&mut self) -> Result<Vec<String>>;
}

#[cfg(feature = "zip")]
impl<R: Read + Seek> Archived for zip::ZipArchive<R> {
    fn unpack(&mut self, dest: &Path) -> Result<()> {
        Ok(self.extract(dest)?)
    }

    fn entries(&mut self) -> Result<Vec<String>> {
        let files = self.file_names().map(|e| e.into()).collect();
        Ok(files)
    }
}

#[cfg(feature = "tar")]
impl<R: Read> Archived for tar::Archive<R> {
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

pub struct Archive(Box<dyn Archived>);

impl Archive {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
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

    pub fn entries(&mut self) -> Result<Vec<String>> {
        self.0.entries()
    }

    pub fn unpack(&mut self, dest: impl AsRef<Path>) -> Result<()> {
        self.0.unpack(dest.as_ref())
    }
}
