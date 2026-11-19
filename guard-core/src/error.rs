use crate::error_code::ErrorCode;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum SteamError {
    Io { context: String, source: io::Error },
    Config(ErrorCode),
}

impl SteamError {
    pub fn io(context: impl Into<String>, source: io::Error) -> Self {
        SteamError::Io {
            context: context.into(),
            source,
        }
    }

    pub fn config(message: ErrorCode) -> Self {
        SteamError::Config(message)
    }

    pub fn code(&self) -> ErrorCode {
        match self {
            SteamError::Io { source, .. } if source.kind() == io::ErrorKind::PermissionDenied => {
                ErrorCode::FilePermission
            }
            SteamError::Io { source, .. } => {
                ErrorCode::from_code(&source.to_string()).unwrap_or(ErrorCode::IoError)
            }
            SteamError::Config(code) => *code,
        }
    }

    pub fn context(&self) -> Option<&str> {
        match self {
            SteamError::Io { context, .. } => Some(context),
            SteamError::Config(_) => None,
        }
    }
}

impl fmt::Display for SteamError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code().as_str())
    }
}

impl std::error::Error for SteamError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SteamError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, SteamError>;
