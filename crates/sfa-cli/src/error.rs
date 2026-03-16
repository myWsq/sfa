use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Usage,
    Config,
    Io,
    Parse,
    Integrity,
    Safety,
    BackendUnavailable,
    Internal,
}

impl ErrorKind {
    pub fn exit_code(self) -> u8 {
        match self {
            Self::Usage => 2,
            Self::Config => 10,
            Self::Io => 20,
            Self::Parse => 30,
            Self::Integrity => 40,
            Self::Safety => 50,
            Self::BackendUnavailable => 70,
            Self::Internal => 1,
        }
    }
}

#[derive(Debug)]
pub struct CliError {
    pub kind: ErrorKind,
    message: String,
}

impl CliError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn usage(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Usage, message)
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Io, message)
    }

    pub fn parse(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Parse, message)
    }

    pub fn integrity(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Integrity, message)
    }

    pub fn safety(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Safety, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Internal, message)
    }

    pub fn backend(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::BackendUnavailable, message)
    }

    pub fn exit_code(&self) -> u8 {
        self.kind.exit_code()
    }
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "error({:?}): {}", self.kind, self.message)
    }
}

impl std::error::Error for CliError {}
