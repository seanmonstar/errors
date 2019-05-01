use super::ErrorRef;

/// Get an `Iterator` of the whole chain of errors.
///
/// Includes the `err` in the iterator as the first item.
///
/// # Example
///
/// ```
/// let err = errors::wrap("c", errors::wrap("b", "a"));
///
/// let mut iter = errors::iter::chain(&*err);
/// assert_eq!(iter.next().unwrap().to_string(), "c");
/// assert_eq!(iter.next().unwrap().to_string(), "b");
/// assert_eq!(iter.next().unwrap().to_string(), "a");
/// assert!(iter.next().is_none());
/// ```
pub fn chain<'a>(err: &'a ErrorRef) -> impl Iterator<Item = &ErrorRef> + 'a {
    Iter { err: Some(err) }
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
/// let mut iter = errors::iter::sources(&*err);
/// assert_eq!(iter.next().unwrap().to_string(), "b");
/// assert_eq!(iter.next().unwrap().to_string(), "a");
/// assert!(iter.next().is_none());
/// ```
pub fn sources<'a>(err: &'a ErrorRef) -> impl Iterator<Item = &ErrorRef> + 'a {
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
