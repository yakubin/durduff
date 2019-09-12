use std::iter::Iterator;

/// Adaptor for an `Iterator` of `Result`s.
///
/// As long as the inner `Iterator` yields `Ok(v)`, `OkIter` yields `v`.
/// When the inner `Iterator` yeilds `Err(e)`, `OkIter` assigns `Some(e)` to the variable
/// referenced in call to `OkIter::new` and ceases to yield any new items.
pub struct OkIter<'a, T, E, I>
where
    I: Iterator<Item = Result<T, E>>,
{
    inner: I,

    err: &'a mut Option<E>,
}

impl<'a, T, E, I> Iterator for OkIter<'a, T, E, I>
where
    I: Iterator<Item = Result<T, E>>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.err.is_some() {
            return None;
        }

        let item = self.inner.next();

        match item {
            Some(Ok(i)) => Some(i),
            Some(Err(e)) => {
                *self.err = Some(e);
                None
            }
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.err.is_some() {
            (0, Some(0))
        } else {
            (0, None)
        }
    }
}

impl<'a, T, E, I> OkIter<'a, T, E, I>
where
    I: Iterator<Item = Result<T, E>>,
{
    /// Creates a new `OkIter`. `err` is assumed to be `None`.
    pub fn new(inner: I, err: &'a mut Option<E>) -> OkIter<'a, T, E, I> {
        OkIter {
            inner,
            err,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rudimentary() {
        let mut results = vec![Ok(4), Ok(27), Ok(1), Err(5), Ok(9), Err(500)];
        let mut err = None;

        let oked: Vec<u32> = OkIter::new(results.drain(..), &mut err).collect();

        assert_eq!(oked, vec![4, 27, 1]);
        assert_eq!(err, Some(5));
    }
}
