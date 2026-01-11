use std::error::Error;
use std::fmt::Display;

/// Error kind of the [`ImportError`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ImportErrorKind {
    /// Failed to read source file.
    SourceFileReadFailed,
    /// Failed to read config file.
    ConfigFileReadFailed,
    /// Source file format is unknown.
    UnknownSourceFileFormat,
    /// Source is invalid.
    InvalidSource,
    /// Config is invalid.
    InvalidConfig,
    /// Config entry isn't found.
    ConfigNotFound,
    /// Failed to emit output.
    OutputFailed,
    /// Unimplemented feature.
    Unimplemented,
    /// Internal failure, most likely a bug.
    Internal,
}

impl ImportErrorKind {
    /// Returns `&str` for [`ImportErrorKind`].
    pub fn as_str(&self) -> &'static str {
        match self {
            ImportErrorKind::SourceFileReadFailed => "failed to read source file",
            ImportErrorKind::ConfigFileReadFailed => "failed to read config file",
            ImportErrorKind::UnknownSourceFileFormat => "unknown source file format",
            ImportErrorKind::InvalidSource => "invalid source",
            ImportErrorKind::InvalidConfig => "invalid config",
            ImportErrorKind::ConfigNotFound => "corresponding config not found",
            ImportErrorKind::OutputFailed => "output failed",
            ImportErrorKind::Unimplemented => "unimplemented",
            ImportErrorKind::Internal => "internal error",
        }
    }
}

impl Display for ImportErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Error to represent a failure in import.
#[derive(Debug)]
pub struct ImportError {
    kind: ImportErrorKind,
    message: String,
    source: Option<Box<dyn Error>>,
}

impl ImportError {
    /// Creates a new [`ImportError`].
    pub fn new(kind: ImportErrorKind, message: String) -> Self {
        Self {
            kind,
            message,
            source: None,
        }
    }

    /// Creates a new [`ImportError`] with source.
    pub fn with_source<E: Error + 'static>(
        kind: ImportErrorKind,
        message: String,
        source: E,
    ) -> Self {
        Self {
            kind,
            message,
            source: Some(source.into()),
        }
    }

    #[cfg(test)]
    pub fn error_kind(&self) -> ImportErrorKind {
        self.kind
    }

    #[cfg(test)]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl Error for ImportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(Box::as_ref)
    }
}

pub trait LazyMessage {
    fn into(self) -> String;
}

impl<'a> LazyMessage for &'a str {
    fn into(self) -> String {
        self.to_string()
    }
}

impl<F: FnOnce() -> String> LazyMessage for F {
    fn into(self) -> String {
        self()
    }
}

mod private {
    pub trait IntoImportErrSeal {}
    pub trait AnnotateImportErrorSeal {}
}

/// Adds method [`into_import_err`] as a convenient method.
pub trait IntoImportError<T>: private::IntoImportErrSeal {
    fn into_import_err<M: LazyMessage>(
        self,
        kind: ImportErrorKind,
        message: M,
    ) -> Result<T, ImportError>;
}

impl<T, E> private::IntoImportErrSeal for Result<T, E> {}

impl<T, E> IntoImportError<T> for Result<T, E>
where
    E: Error + 'static,
{
    fn into_import_err<M: LazyMessage>(
        self,
        kind: ImportErrorKind,
        message: M,
    ) -> Result<T, ImportError> {
        self.map_err(|e| ImportError::with_source(kind, message.into(), e))
    }
}

impl<T> private::IntoImportErrSeal for Option<T> {}

impl<T> IntoImportError<T> for Option<T> {
    fn into_import_err<M: LazyMessage>(
        self,
        kind: ImportErrorKind,
        message: M,
    ) -> Result<T, ImportError> {
        self.ok_or_else(|| ImportError::new(kind, message.into()))
    }
}

/// Allows annotating the given [`ImportError`].
pub trait AnnotateImportError<T>: private::AnnotateImportErrorSeal {
    fn annotate<F: FnOnce() -> Msg, Msg: Display>(self, prefix: F) -> Self;
}

impl<T> private::AnnotateImportErrorSeal for Result<T, ImportError> {}

impl<T> AnnotateImportError<T> for Result<T, ImportError> {
    fn annotate<F: FnOnce() -> Msg, Msg: Display>(self, prefix: F) -> Self {
        self.map_err(|e| ImportError {
            kind: e.kind,
            message: format!("{}: {}", prefix(), e.message),
            source: e.source,
        })
    }
}
