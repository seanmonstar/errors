use std::fmt;
use super::{BoxError, Error, ErrorRef};

/// Simple way to create an error value.
///
/// # Example
///
/// ```
/// let err = errors::new("sound the alarm");
///
/// assert_eq!(err.to_string(), "sound the alarm");
/// ```
pub fn new<D>(err: D) -> impl Error
where
    D: fmt::Debug + fmt::Display + Send + Sync + 'static,
{
    Wrapper {
        message: err,
        cause: None,
    }
}

/// Wrap an error with some additional message.
///
/// Includes the error as the source of this wrapped error.
///
/// ```
/// use std::error::Error;
///
/// let err = errors::wrap("exploded", "cat hair in generator");
///
/// assert_eq!(err.to_string(), "exploded");
/// assert_eq!(err.source().unwrap().to_string(), "cat hair in generator");
/// ```
pub fn wrap<D, E>(message: D, cause: E) -> impl Error
where
    D: fmt::Debug + fmt::Display + Send + Sync + 'static,
    E: Into<BoxError>,
{
    Wrapper {
        message,
        cause: Some(cause.into()),
    }
}

/// Wrap a value as a new `Error`, while hiding its source chain.
///
/// The value is used for formatting, but not exposed as the `source`.
///
/// # Usage
///
/// This allows continuing to provide all relevant debug information of an
/// error source chain, without exposing the exact types contained within.
///
/// Imagine wrapping an error that resulted from some timeout. A user may
/// consider retrying an operation if the error chain contains said timeout.
/// But perhaps the timed out operation has already been retried a few times,
/// and you wish to return an error that no longer matches some "is_timeout"
/// check. You can make the error "opaque", so that it still includes the
/// timeout information in logs, but no longer is programmatically a "timeout".
///
/// # Example
///
/// ```
/// use std::error::Error;
///
/// let orig = errors::wrap("request failed", "timeout");
///
/// let err = errors::opaque(orig);
///
/// // Still prints all the information...
/// assert_eq!(format!("{:+}", err), "request failed: timeout");
/// // But is no longer programatically available.
/// assert!(err.source().is_none());
/// ```
pub fn opaque<E>(err: E) -> impl Error
where
    E: Into<BoxError>,
{
    Opaque(err.into())
}

pub(crate) fn wrap_ref<'a>(err: &'a dyn Error) -> impl Error + 'a {
    WrapperRef {
        message: err,
        cause: err.source(),
    }
}

struct Wrapper<D> {
    message: D,
    cause: Option<BoxError>,
}


struct WrapperRef<'a, D> {
    message: D,
    cause: Option<&'a ErrorRef>,
}

struct Opaque(BoxError);

// ===== impl Wrapper =====

impl<D> Wrapper<D>
where
    D: fmt::Debug + fmt::Display + 'static,
{
    fn wrap_ref(&self) -> WrapperRef<&D> {
        WrapperRef {
            message: &self.message,
            cause: self.source(),
        }
    }
}

impl<D> fmt::Debug for Wrapper<D>
where
    D: fmt::Debug + fmt::Display + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.wrap_ref(), f)
    }
}

impl<D> fmt::Display for Wrapper<D>
where
    D: fmt::Debug + fmt::Display + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.wrap_ref(), f)
    }
}

impl<D> Error for Wrapper<D>
where
    D: fmt::Debug + fmt::Display + 'static,
{
    fn source(&self) -> Option<&ErrorRef> {
        self.cause.as_ref().map(|e| &**e as _)
    }
}

// ===== impl WrapperRef =====


impl<'a, D> WrapperRef<'a, D>
where
    D: fmt::Debug + fmt::Display,
{
    fn joiner(&self, f: &fmt::Formatter) -> &'static str {
        if f.alternate() {
            "\nCaused by: "
        } else {
            ": "
        }
    }

    fn fmt_all_sources(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let joiner = self.joiner(f);
        for err in ::iter::sources(self) {
            f.write_str(joiner)?;

            // Propagate if chain ends in `Opaque`
            if err.is::<Opaque>() {
                return if f.alternate() {
                    write!(f, "{:+#}", err)
                } else {
                    write!(f, "{:+}", err)
                };
            }

            // else
            write!(f, "{:-}", err)?;
        }

        Ok(())
    }

    fn fmt_max_sources(&self, f: &mut fmt::Formatter, mut max: usize) -> fmt::Result {
        let joiner = self.joiner(f);
        let mut sources = ::iter::sources(self);
        loop {
            if max == 0 {
                return Ok(());
            }
            max -= 1;

            let err = match sources.next() {
                Some(err) => err,
                None => break,
            };

            f.write_str(joiner)?;

            // Propagate if chain ends in `Opaque`
            if err.is::<Opaque>() {
                return if f.alternate() {
                    write!(f, "{:+#.*}", max, err)
                } else {
                    write!(f, "{:+.*}", max, err)
                };
            }

            //else
            write!(f, "{:-}", err)?;

        }

        Ok(())
    }
}

impl<'a, D: fmt::Debug> fmt::Debug for WrapperRef<'a, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref cause) = self.cause {
            f.debug_tuple("")
                .field(&self.message)
                .field(cause)
                .finish()
        } else {
            fmt::Debug::fmt(&self.message, f)
        }
    }
}

impl<'a, D> fmt::Display for WrapperRef<'a, D>
where
    D: fmt::Debug + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // {:+} means print the chain
        if f.sign_plus() {
            // first message with no flags...
            write!(f, "{:-}", self.message)?;
            // precision flag signals max source chain iteration...
            if let Some(max) = f.precision() {
                self.fmt_max_sources(f, max)
            } else {
                self.fmt_all_sources(f)
            }
        } else {
            // reset all formatter flags
            write!(f, "{}", self.message)
        }
    }
}

impl<'a, D> Error for WrapperRef<'a, D>
where
    D: fmt::Debug + fmt::Display,
{
    fn source(&self) -> Option<&ErrorRef> {
        self.cause.as_ref().map(|e| *e)
    }
}

// ===== impl Opaque =====

impl Opaque {
    fn wrap_ref(&self) -> WrapperRef<&ErrorRef> {
        WrapperRef {
            message: &*self.0,
            cause: self.0.source(),
        }
    }
}

impl fmt::Debug for Opaque {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for Opaque {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.wrap_ref(), f)
    }
}

// No source chains for opaque errors!
impl Error for Opaque {}

#[cfg(test)]
mod tests {
    #[test]
    fn display_default() {
        let cause = "cat hair in generator";
        let top = "ship exploded";

        let op = super::new(cause);
        assert_eq!(format!("{}", op), cause);

        let wp = super::wrap(top, cause);
        assert_eq!(format!("{}", wp), top);

        let wp_op = super::wrap(top, op);
        assert_eq!(format!("{}", wp_op), top);
    }

    #[test]
    fn display_chain() {
        let cause = "cat hair in generator";
        let top = "ship exploded";

        let op = super::new(cause);
        assert_eq!(format!("{:+}", op), cause);

        let wp = super::wrap(top, cause);
        assert_eq!(format!("{:+}", wp), format!("{}: {}", top, cause));

        let wp_op = super::wrap(top, op);
        assert_eq!(format!("{:+}", wp_op), format!("{}: {}", top, cause));
    }

    #[test]
    fn display_chain_when_message_is_wrapped() {

        let msg = super::wrap("b", "a");
        let err = super::wrap(msg, "z");

        assert_eq!(format!("{:+}", err), "b: z");
    }

    #[test]
    fn display_alternative() {
        let cause = "cat hair in generator";
        let top = "ship exploded";

        let op = super::new(cause);
        assert_eq!(format!("{:#}", op), cause);
        assert_eq!(format!("{:+#}", op), cause);

        let alt = format!("{}\nCaused by: {}", top, cause);

        let wp = super::wrap(top, cause);
        assert_eq!(format!("{:+#}", wp), alt);

        let wp_op = super::wrap(top, op);
        assert_eq!(format!("{:+#}", wp_op), alt);
    }

    #[test]
    fn display_chain_max() {
        let a = "a";
        let op = super::new(a);
        assert_eq!(format!("{:.0}", op), a);
        assert_eq!(format!("{:.1}", op), a);
        assert_eq!(format!("{:+.0}", op), a);
        assert_eq!(format!("{:+.1}", op), a);

        let wp = super::wrap("b", "a");
        assert_eq!(format!("{:.0}", wp), "b");
        assert_eq!(format!("{:.1}", wp), "b");
        assert_eq!(format!("{:+.0}", wp), "b");
        assert_eq!(format!("{:+.1}", wp), "b: a");

        let wp2 = super::wrap("c", wp);
        assert_eq!(format!("{:.0}", wp2), "c");
        assert_eq!(format!("{:.1}", wp2), "c");
        assert_eq!(format!("{:+.0}", wp2), "c");
        assert_eq!(format!("{:+.1}", wp2), "c: b");
        assert_eq!(format!("{:+.2}", wp2), "c: b: a");
    }

    // opaque()

    #[test]
    fn opaque_has_no_sources() {
        use crate::Error;

        let w = super::wrap("b", "a");
        assert!(w.source().is_some());

        let op = super::opaque(w);
        assert!(op.source().is_none());
    }

    #[test]
    fn opaque_displays_chain() {
        let w = super::wrap("b", "a");
        let op = super::opaque(w);

        assert_eq!(format!("{:+}", op), "b: a");
    }

    #[test]
    fn opaque_wrapped_forwards_display_chain_flags() {
        let w = super::wrap("b", "a");
        let w = super::wrap("c", w);
        let op = super::opaque(w);
        let e = super::wrap("d", op);

        assert_eq!(format!("{:+}", e), "d: c: b: a");
        assert_eq!(format!("{:+.1}", e), "d: c");
    }
}
