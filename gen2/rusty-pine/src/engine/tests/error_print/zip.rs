pub struct Zip<A, B> {
    left: A,
    right: B,
}

impl<A, B> Zip<A, B> {
    pub fn new(left: A, right: B) -> Self {
        Zip { left, right }
    }
}

impl<A, B> Iterator for Zip<A, B>
where
    A: Iterator,
    B: Iterator,
{
    type Item = (Option<A::Item>, Option<B::Item>);

    fn next(&mut self) -> Option<Self::Item> {
        let left = self.left.next();
        let right = self.right.next();

        if left.is_none() && right.is_none() {
            None
        } else {
            Some((left, right))
        }
    }
}
