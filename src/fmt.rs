//! Utilities for formatting `Error`s.

use std::fmt as std_fmt;
use super::{BoxError, Error};

/// An adapter to pretty-print an error source chain.
///
/// # Example
///
/// ```no_run
/// fn main() -> Result<(), errors::Main> {
///     // Any program that returns a normal `impl Error`
///     Err("ruh roh")?;
///
///     Ok(())
/// }
/// ```
pub struct Main(BoxError);

impl std_fmt::Debug for Main {
    fn fmt(&self, f: &mut std_fmt::Formatter) -> std_fmt::Result {
        let err = crate::new::wrap_ref(&*self.0);
        write!(f, "{:+#}", err)
    }
}

impl<E: Into<BoxError>> From<E> for Main {
    fn from(err: E) -> Main {
        Main(err.into())
    }
}

/// Create a `Display` adapter that applies the formatting rules to any error.
///
/// # Example
///
/// ```
/// use std::io;
///
/// let orig = errors::wrap("exploded", "cat hair in generator");
/// let err = io::Error::new(io::ErrorKind::Other, orig);
///
/// // Foreign type might not know how to format sources...
/// // But now it does!
/// assert_eq!(
///     format!("{:+}", errors::fmt(&err)),
///     "exploded: cat hair in generator"
/// );
/// ```
pub fn fmt<'a>(err: &'a dyn Error) -> impl std_fmt::Display + 'a {
    ::new::wrap_ref(err)
}

#[cfg(test)]
mod tests {
    use std::fmt;
    use std::io;

    use {BoxError, Error};

    #[derive(Debug)]
    struct Naive(Option<BoxError>);

    impl fmt::Display for Naive {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            "naive".fmt(f)
        }
    }

    impl Error for Naive {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            self.0.as_ref().map(|e| &**e as _)
        }
    }

    #[test]
    fn chain_wraps_outside_errors() {
        let a = "a";

        // root
        let err = io::Error::new(io::ErrorKind::Other, a);
        assert_eq!(format!("{}", super::fmt(&err)), a);
        assert_eq!(format!("{:.0}", super::fmt(&err)), a);
        assert_eq!(format!("{:+}", super::fmt(&err)), a);
        assert_eq!(format!("{:+.0}", super::fmt(&err)), a);

        // nest 1
        let err = Naive(Some(err.into()));
        let naive = "naive";
        let naive_a = "naive: a";
        assert_eq!(format!("{}", super::fmt(&err)), naive);
        assert_eq!(format!("{:.0}", super::fmt(&err)), naive);
        assert_eq!(format!("{:+}", super::fmt(&err)), naive_a);
        assert_eq!(format!("{:+.0}", super::fmt(&err)), naive);
        assert_eq!(format!("{:+.1}", super::fmt(&err)), naive_a);
    }

    #[test]
    fn chain_wraps_our_errors() {
        let err = ::wrap("b", "a");
        let b = "b";
        let b_a = "b: a";
        assert_eq!(format!("{}", super::fmt(&err)), b);
        assert_eq!(format!("{:.0}", super::fmt(&err)), b);
        assert_eq!(format!("{:+}", super::fmt(&err)), b_a);
        assert_eq!(format!("{:+.0}", super::fmt(&err)), b);
        assert_eq!(format!("{:+.1}", super::fmt(&err)), b_a);
    }

    /// Simulate an error type that by default prefers to show one level
    /// deep in its source chain, but wants to opt-in to behaving correctly
    /// with `errors::fmt`.
    #[derive(Debug)]
    struct OneDeep(BoxError);

    impl fmt::Display for OneDeep {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if f.sign_minus() {
                write!(f, "one deep")
            } else {
                write!(f, "one deep: {}", self.0)
            }
        }
    }

    impl Error for OneDeep {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&*self.0)
        }
    }

    #[test]
    fn one_deep_is_passed_minus() {
        let orig = ::new("a");
        let one_deep = OneDeep(orig.into());

        assert_eq!(format!("{}", one_deep), "one deep: a");

        let err = ::wrap("b", one_deep);
        let b = "b";
        let b_1 = "b: one deep";
        let b_1_a = "b: one deep: a";
        assert_eq!(format!("{}", err), b);
        assert_eq!(format!("{:.0}", err), b);
        assert_eq!(format!("{:+}", err), b_1_a);
        assert_eq!(format!("{:+.0}", err), b);
        assert_eq!(format!("{:+.1}", err), b_1);
    }

    #[test]
    fn one_deep_opaque_is_passed_minus() {
        let orig = ::new("a");
        let one_deep = ::opaque(OneDeep(orig.into()));

        assert_eq!(format!("{}", one_deep), "one deep: a");

        let err = ::wrap("b", one_deep);
        let b = "b";
        let b_1 = "b: one deep";
        let b_1_a = "b: one deep: a";
        assert_eq!(format!("{}", err), b);
        assert_eq!(format!("{:.0}", err), b);
        assert_eq!(format!("{:+}", err), b_1_a);
        assert_eq!(format!("{:+.0}", err), b);
        assert_eq!(format!("{:+.1}", err), b_1);
    }
}
