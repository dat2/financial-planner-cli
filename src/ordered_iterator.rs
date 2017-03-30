use std::iter::{Iterator, Peekable};

pub struct OrderedIterator<I: Ord, P: Iterator<Item = I>> {
    first: Peekable<P>,
    second: Peekable<P>
}

impl<I, P> OrderedIterator<I, P>
    where I: Ord,
          P: Iterator<Item = I>
{
    pub fn new(first: P, second: P) -> OrderedIterator<I, P> {
        let first_peekable = first.peekable();
        let second_peekable = second.peekable();

        OrderedIterator {
            first: first_peekable,
            second: second_peekable
        }
    }
}

impl<I, P> Iterator for OrderedIterator<I, P>
    where I: Ord + Clone,
          P: Iterator<Item = I>
{
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {

        let first_is_none = self.first.peek().is_none();
        let second_is_none = self.second.peek().is_none();

        if first_is_none && second_is_none {
            return None;
        }

        if first_is_none {
            return self.second.next();
        } else if second_is_none {
            return self.first.next();
        }

        if self.first.peek().unwrap() < self.second.peek().unwrap() {
            return self.first.next();
        } else {
            return self.second.next();
        }

    }
}
