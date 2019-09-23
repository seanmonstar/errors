//! Utilities for formatting `Error`s.

use std::fmt;
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

impl fmt::Debug for Main {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = crate::new::wrap_ref(&*self.0);
        write!(f, "{:+#}", err)
    }
}

impl<E: Into<BoxError>> From<E> for Main {
    fn from(err: E) -> Main {
        Main(err.into())
    }
}

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
pub fn chain<'a>(err: &'a dyn Error) -> impl fmt::Display + 'a {
    Chain(::new::wrap_ref(err))
}

struct Chain<T>(T);

impl<T: fmt::Display> fmt::Display for Chain<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Force `sign_plus` flag on....
        // But we otherwise want to pass the rest of the flags... ;_;
        if let Some(max) = f.precision() {
            write!(f, "{:+.max$}", self.0, max = max)
        } else {
            write!(f, "{:+}", self.0)
        }
    }
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
        assert_eq!(format!("{}", super::chain(&err)), a);
        assert_eq!(format!("{:.0}", super::chain(&err)), a);
        assert_eq!(format!("{:+}", super::chain(&err)), a);
        assert_eq!(format!("{:+.0}", super::chain(&err)), a);

        // nest 1
        let err = Naive(Some(err.into()));
        let naive = "naive";
        let naive_a = "naive: a";
        assert_eq!(format!("{}", super::chain(&err)), naive_a);
        assert_eq!(format!("{:.0}", super::chain(&err)), naive);
        assert_eq!(format!("{:+}", super::chain(&err)), naive_a);
        assert_eq!(format!("{:+.0}", super::chain(&err)), naive);
        assert_eq!(format!("{:+.1}", super::chain(&err)), naive_a);
    }

    #[test]
    fn chain_wraps_our_errors() {
        let err = ::wrap("b", "a");
        let b = "b";
        let b_a = "b: a";
        assert_eq!(format!("{}", super::chain(&*err)), b_a);
        assert_eq!(format!("{:.0}", super::chain(&*err)), b);
        assert_eq!(format!("{:+}", super::chain(&*err)), b_a);
        assert_eq!(format!("{:+.0}", super::chain(&*err)), b);
        assert_eq!(format!("{:+.1}", super::chain(&*err)), b_a);
    }
}
