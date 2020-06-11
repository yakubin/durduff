use std::cmp::Ordering;

use std::iter::Iterator;
use std::iter::Peekable;

/// Indicates which source iterator an item yielded from `SumIter` was yielded from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SumIterSelector {
    Left,
    Right,
    Both,
}

impl SumIterSelector {
    fn wrap<T>(self, v: Option<T>) -> Option<(Self, T)> {
        v.map(|v| (self, v))
    }
}

/// Result of peeking with two iterators
enum TwoItersPeek {
    /// Indicates that only the left iterator yielded an element.
    Left,

    /// Indicates that only the right iterator yielded an element.
    Right,

    /// Indicates that both iterators yielded elements.
    /// `Ordering` is the result of comparing these elements (`lhs.cmp(rhs)`).
    Both(Ordering),
}

/// Iterator yielding the sum (in the set-theoretic sense) of two iterators along with a selector
/// indicating which iterator yielded a given item
///
///     +---------+         +---------+                       +------------+
///     | LhsIter |         | RhsIter |                       |  SumIter   |
///     +=========+         +=========+                       +============+
///     |    2    |         |    1    |     SumIter::new      | (Right, 1) |
///     +---------+    +    +---------+   ---------------->   +------------+
///     |    5    |         |    7    |                       | (Left, 2)  |
///     +---------+         +---------+                       +------------+
///     |    9    |         |    8    |                       | (Left, 5)  |
///     +---------+         +---------+                       +------------+
///                         |    9    |                       | (Right, 7) |
///                         +---------+                       +------------+
///                                                           | (Right, 8) |
///                                                           +------------+
///                                                           | (Both, 9)  |
///                                                           +------------+
pub struct SumIter<I: Iterator> {
    lhs_iter: Peekable<I>,
    rhs_iter: Peekable<I>,

    cmp: fn(&I::Item, &I::Item) -> Ordering,
}

impl<I: Iterator> SumIter<I> {
    /// Creates a new `SumIter`. Assumes that the contents of `lhs` and `rhs` are unique, sorted
    /// according to `cmp` and that `cmp` gives a total order.
    pub fn new(lhs: I, rhs: I, cmp: fn(&I::Item, &I::Item) -> Ordering) -> Self {
        Self {
            lhs_iter: lhs.peekable(),
            rhs_iter: rhs.peekable(),
            cmp,
        }
    }

    fn skip_rhs_take_lhs(&mut self) -> Option<I::Item> {
        self.rhs_iter.next();
        self.lhs_iter.next()
    }
}

impl<I: Iterator> Iterator for SumIter<I> {
    type Item = (SumIterSelector, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let peek = match (self.lhs_iter.peek(), self.rhs_iter.peek()) {
            (Some(lhs), Some(rhs)) => TwoItersPeek::Both((self.cmp)(lhs, rhs)),
            (Some(_), None) => TwoItersPeek::Left,
            (None, _) => TwoItersPeek::Right,
        };

        match peek {
            TwoItersPeek::Left | TwoItersPeek::Both(Ordering::Less) => {
                SumIterSelector::Left.wrap(self.lhs_iter.next())
            }
            TwoItersPeek::Right | TwoItersPeek::Both(Ordering::Greater) => {
                SumIterSelector::Right.wrap(self.rhs_iter.next())
            }
            TwoItersPeek::Both(Ordering::Equal) => {
                SumIterSelector::Both.wrap(self.skip_rhs_take_lhs())
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lhs_min, lhs_max) = self.lhs_iter.size_hint();
        let (rhs_min, rhs_max) = self.rhs_iter.size_hint();

        let min = std::cmp::max(lhs_min, rhs_min);

        let max = match (lhs_max, rhs_max) {
            (Some(l), Some(r)) => Some(l + r),
            _ => None,
        };

        (min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rudimentary() {
        let mut lhs: Vec<u32> = vec![2, 5, 9];
        let mut rhs: Vec<u32> = vec![1, 7, 8, 9];

        let sum: Vec<_> = SumIter::new(lhs.drain(..), rhs.drain(..), u32::cmp).collect();

        let expected = vec![
            (SumIterSelector::Right, 1),
            (SumIterSelector::Left, 2),
            (SumIterSelector::Left, 5),
            (SumIterSelector::Right, 7),
            (SumIterSelector::Right, 8),
            (SumIterSelector::Both, 9),
        ];

        assert_eq!(sum, expected);
    }
}
