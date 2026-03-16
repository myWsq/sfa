use std::io;
use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unsupported data codec value: {0}")]
    UnsupportedDataCodec(u16),
    #[error("unsupported manifest codec value: {0}")]
    UnsupportedManifestCodec(u16),
    #[error("unsupported integrity mode value: {0}")]
    UnsupportedIntegrityMode(u8),
    #[error("unsupported frame hash algorithm value: {0}")]
    UnsupportedFrameHashAlgo(u8),
    #[error("unsupported manifest hash algorithm value: {0}")]
    UnsupportedManifestHashAlgo(u8),
    #[error("unsupported entry kind value: {0}")]
    UnsupportedEntryKind(u8),
    #[error("invalid header: {0}")]
    InvalidHeader(&'static str),
    #[error("invalid frame: {0}")]
    InvalidFrame(&'static str),
    #[error("invalid manifest: {0}")]
    InvalidManifest(&'static str),
    #[error("manifest hash mismatch")]
    ManifestHashMismatch,
    #[error("frame hash mismatch for bundle {bundle_id}")]
    FrameHashMismatch { bundle_id: u64 },
    #[error("trailer hash mismatch")]
    TrailerHashMismatch,
    #[error("archive truncated")]
    UnexpectedEof,
    #[error("invalid archive state: {0}")]
    InvalidState(&'static str),
    #[error("path is required for regular entry {0}")]
    MissingSourcePath(u32),
    #[error("invalid path: {0}")]
    InvalidPath(String),
    #[error("io error at {path:?}: {source}")]
    Io {
        path: Option<PathBuf>,
        #[source]
        source: io::Error,
    },
    #[error("{0}")]
    Message(String),
}

impl Error {
    pub fn io(source: io::Error) -> Self {
        Self::Io { path: None, source }
    }

    pub fn io_at(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: Some(path.into()),
            source,
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::io(value)
    }
}
