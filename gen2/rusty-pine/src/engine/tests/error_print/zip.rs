/// Iterator that fully consumes both inner iterators.
///
/// It's greedy because it will consume the absolute most it can.
///
/// The regular Zip implementation stops when either inner iterator finishes. This is unfortunate
/// when we want to print a side by side diff of files. Consider the following:
/// ```txt
/// left line 1     |     right line 2
/// left line 2     |     right line 2
/// left line 3     <
/// ```
///
/// Left has 3 lines, while right has 2. The std Zip would stop after line 2, since the right
/// iterator is finished. We would never print line 3.
pub struct GreedyZip<Left, Right> {
    left: Left,
    right: Right,
}

impl<Left, Right> GreedyZip<Left, Right> {
    pub fn new(left: Left, right: Right) -> Self {
        GreedyZip { left, right }
    }
}

impl<Left, Right> Iterator for GreedyZip<Left, Right>
where
    Left: Iterator,
    Right: Iterator,
{
    type Item = ZipItem<Left::Item, Right::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let left = self.left.next();
        let right = self.right.next();

        use ZipItem::*;

        match (left, right) {
            (Some(left), Some(right)) => Some(Both(left, right)),
            (Some(left), None) => Some(LeftOnly(left)),
            (None, Some(right)) => Some(RightOnly(right)),
            (None, None) => None,
        }
    }
}

/// A single item in a GreedyZip.
///
/// If both the left and right iterators of our GreedyZip return None, we also return None,
/// signaling that the iterator is finished.
///
/// A previous version of the GreedyZip used (Option<Left>, Option<Right>) as the iterator Item.
/// This had the downside of technically allowing the return of (None, None). While this would
/// never actually happen, the compiler complained if you tried to omit it in match {} blocks.
///
/// Using ZipItem instead of (Option, Option) allows us to properly represent the actual shape of
/// the data in the type system, with the downside of writing more code and comments like this one.
pub enum ZipItem<Left, Right> {
    LeftOnly(Left),
    RightOnly(Right),
    Both(Left, Right),
}
