use std::fmt;
use super::{BoxError, Error, ErrorRef};

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
pub fn opaque<D>(err: D) -> BoxError
where
    D: fmt::Debug + fmt::Display + Send + Sync + 'static,
{
    Wrapper {
        message: err,
        cause: None,
    }.into()
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
pub fn wrap<D, E>(message: D, cause: E) -> BoxError
where
    D: fmt::Debug + fmt::Display + Send + Sync + 'static,
    E: Into<BoxError>,
{
    Wrapper {
        message,
        cause: Some(cause.into()),
    }.into()
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
            write!(f, "{}", self.message)?;
            let joiner = if f.alternate() {
                "\nCaused by: "
            } else {
                ": "
            };
            // precision flag signals max source chain iteration...
            if let Some(max) = f.precision() {
                for err in ::iter::sources(self).take(max) {
                    f.write_str(joiner)?;
                    write!(f, "{}", err)?;
                }
            } else {
                for err in ::iter::sources(self) {
                    f.write_str(joiner)?;
                    write!(f, "{}", err)?;
                }
            }

            Ok(())
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

#[cfg(test)]
mod tests {
    #[test]
    fn display_default() {
        let cause = "cat hair in generator";
        let top = "ship exploded";

        let op = super::opaque(cause);
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

        let op = super::opaque(cause);
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

        let op = super::opaque(cause);
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
        let op = super::opaque(a);
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
}
