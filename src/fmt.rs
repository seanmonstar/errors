//! Utilities for formatting `Error`s.

use std::fmt;
use super::ErrorRef;

/// Create a `Display` adapter that outputs the error chain.
///
/// # Example
///
/// ```
/// let err = errors::wrap("exploded", "cat hair in generator");
///
/// // Only prints top error...
/// assert_eq!(err.to_string(), "exploded");
///
/// // What if we want the whole chain...
/// assert_eq!(
///     errors::fmt::chain(&*err).to_string(),
///     "exploded: cat hair in generator"
/// );
/// ```
pub fn chain<'a>(err: &'a ErrorRef) -> impl fmt::Display + 'a {
    Chain(err)
}

struct Chain<'a>(&'a ErrorRef);

impl<'a> fmt::Display for Chain<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.0, f)?;
        for err in ::iter::sources(self.0) {
            f.write_str(": ")?;
            fmt::Display::fmt(err, f)?
        }
        Ok(())
    }
}
