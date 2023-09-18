#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

mod archive;
mod format;
mod result;

#[cfg(feature = "zip")]
mod zip;

#[cfg(feature = "tar")]
mod tar;

pub use archive::{Archive, Entries, Entry};
pub use format::Format;
pub use result::{Error, Result};

/// Available archive file formats.
#[allow(deprecated)]
pub use format::ArchiveKind;
