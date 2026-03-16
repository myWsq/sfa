use std::fmt::{Display, Formatter};
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum UnixFsError {
    Io(io::Error),
    Core(sfa_core::Error),
    PathValidation(PathValidationError),
    UnsupportedEntryKind(PathBuf),
    MissingParent(PathBuf),
    InvalidState(&'static str),
}

impl Display for UnixFsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Core(err) => write!(f, "core archive error: {err}"),
            Self::PathValidation(err) => write!(f, "path validation error: {err}"),
            Self::UnsupportedEntryKind(path) => {
                write!(f, "unsupported filesystem entry kind: {}", path.display())
            }
            Self::MissingParent(path) => write!(f, "missing parent path: {}", path.display()),
            Self::InvalidState(msg) => write!(f, "invalid state: {msg}"),
        }
    }
}

impl std::error::Error for UnixFsError {}

impl From<io::Error> for UnixFsError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<sfa_core::Error> for UnixFsError {
    fn from(value: sfa_core::Error) -> Self {
        Self::Core(value)
    }
}

impl From<PathValidationError> for UnixFsError {
    fn from(value: PathValidationError) -> Self {
        Self::PathValidation(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathValidationError {
    AbsolutePath,
    ParentTraversal,
    EmptySegment,
    DotSegment,
    NulByte,
    NonUtf8OrNonUnixNormal,
    SymlinkTraversal(PathBuf),
    NotADirectory(PathBuf),
    OutsideRoot(PathBuf),
}

impl Display for PathValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AbsolutePath => write!(f, "absolute paths are not allowed"),
            Self::ParentTraversal => write!(f, "parent traversal is not allowed"),
            Self::EmptySegment => write!(f, "empty path segment is not allowed"),
            Self::DotSegment => write!(f, "dot segment is not allowed"),
            Self::NulByte => write!(f, "nul byte is not allowed"),
            Self::NonUtf8OrNonUnixNormal => write!(f, "only normal unix path segments are allowed"),
            Self::SymlinkTraversal(path) => {
                write!(f, "symlink traversal is not allowed: {}", path.display())
            }
            Self::NotADirectory(path) => write!(f, "not a directory: {}", path.display()),
            Self::OutsideRoot(path) => write!(f, "path escapes output root: {}", path.display()),
        }
    }
}

impl std::error::Error for PathValidationError {}
