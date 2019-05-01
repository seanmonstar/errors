use std::fmt;
use super::{Error, ErrorRef, StdError};

/// Wrap a value as an opaque `Error`.
///
/// The value is used for formatting, but not exposed as the `source`.
///
/// # Example
///
/// ```
/// let err = errors::opaque("sound the alarm");
///
/// assert_eq!(err.to_string(), "sound the alarm");
/// assert!(err.source().is_none());
/// ```
pub fn opaque<D>(err: D) -> Error
where
    D: fmt::Debug + fmt::Display + Send + Sync + 'static,
{
    Opaque(err).into()
}

/// Wrap an error with some additional message.
///
/// Includes the error as the source of this wrapped error.
///
/// ```
/// let err = errors::wrap("exploded", "cat hair in generator");
///
/// assert_eq!(err.to_string(), "exploded");
/// assert_eq!(err.source().unwrap().to_string(), "cat hair in generator");
/// ```
pub fn wrap<D, E>(msg: D, cause: E) -> Error
where
    D: fmt::Debug + fmt::Display + Send + Sync + 'static,
    E: Into<Error>,
{
    Wrapped(msg, cause.into()).into()
}

struct Opaque<D>(D);

impl<D: fmt::Debug> fmt::Debug for Opaque<D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl<D: fmt::Display> fmt::Display for Opaque<D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl<D: fmt::Debug + fmt::Display> StdError for Opaque<D> {
    // no source
}

struct Wrapped<D>(D, Error);

impl<D: fmt::Debug> fmt::Debug for Wrapped<D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("")
            .field(&self.0)
            .field(&self.1)
            .finish()
    }
}

impl<D: fmt::Display> fmt::Display for Wrapped<D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl<D: fmt::Debug + fmt::Display> StdError for Wrapped<D> {
    fn source(&self) -> Option<&ErrorRef> {
        Some(&*self.1)
    }
}
