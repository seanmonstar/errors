use super::{Error, ErrorRef};

/// Get an `Iterator` of the whole chain of errors.
///
/// Includes the `err` in the iterator as the first item.
///
/// # Example
///
/// ```
/// let err = errors::wrap("c", errors::wrap("b", "a"));
///
/// let expected = ["c", "b", "a"];
///
/// for (err, &s) in errors::iter::chain(&*err).zip(expected.iter()) {
///     assert_eq!(err.to_string(), s);
/// }
/// ```
pub fn chain<'a>(err: &'a ErrorRef) -> impl Iterator<Item = &'a ErrorRef> + 'a {
    Iter { err: Some(err) }
}

/// Get the root source of an `Error`.
///
/// If the provided `Error` has a source chain, this will find the last one
/// in the chain. If there is no chain, returns the same `Error`.
///
/// # Example
///
/// ```
/// // Error chain: c -> b -> a (root)
/// let err = errors::wrap("c", errors::wrap("b", "a"));
///
/// assert_eq!(errors::iter::root(&*err).to_string(), "a");
///
/// // No chain:
/// let root = errors::new("ninja cat");
///
/// assert_eq!(errors::iter::root(&*root).to_string(), "ninja cat");
/// ```
pub fn root(err: &ErrorRef) -> &ErrorRef {
    chain(err)
        .last()
        .expect("errors::iter::chain always yields at least 1 item")
}

/// Get an `Iterator` of the source chain of this error.
///
/// Skips `err`, starting as `err.source()`. Equivalent to `chain(err).skip(1)`.
///
/// # Example
///
/// ```
/// let err = errors::wrap("c", errors::wrap("b", "a"));
///
///
/// let expected = ["b", "a"];
///
/// for (err, &s) in errors::iter::sources(&*err).zip(expected.iter()) {
///     assert_eq!(err.to_string(), s);
/// }
/// ```
pub fn sources(err: &dyn Error) -> impl Iterator<Item = &ErrorRef> {
    Iter { err: err.source() }
}

struct Iter<'a> {
    err: Option<&'a ErrorRef>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a ErrorRef;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.err?;
        self.err = next.source();
        Some(next)
    }
}
