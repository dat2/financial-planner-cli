use std::iter::{Iterator, Peekable};
use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::cell::RefCell;
use std::iter::FromIterator;

#[derive(Clone)]
pub struct OrdIterator<T: Ord, P: Iterator<Item = T> + Clone>(RefCell<Peekable<P>>);

impl<T: Ord, P: Iterator<Item = T> + Clone> PartialEq for OrdIterator<T, P> {
    fn eq(&self, other: &Self) -> bool {

        let mut this = self.0.borrow_mut();
        let mut other = other.0.borrow_mut();

        let this = this.peek();
        let other = other.peek();

        if this.is_none() && other.is_none() {
            return true;
        }

        if this.is_none() || other.is_none() {
            return false;
        }
        let this = this.unwrap();
        let other = other.unwrap();

        *this == *other
    }
}

impl<T: Ord, P: Iterator<Item = T> + Clone> Eq for OrdIterator<T, P> {}

impl<T: Ord, P: Iterator<Item = T> + Clone> PartialOrd for OrdIterator<T, P> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {

        let mut this = self.0.borrow_mut();
        let mut other = other.0.borrow_mut();

        let this = this.peek();
        let other = other.peek();

        if this.is_none() || other.is_none() {
            return None;
        }

        let this = this.unwrap();
        let other = other.unwrap();

        match this.partial_cmp(other) {
            None => None,
            Some(ord) => {
                let ord = match ord {
                    Ordering::Greater => Ordering::Less,
                    Ordering::Less => Ordering::Greater,
                    Ordering::Equal => ord,
                };
                Some(ord)
            }
        }
    }
}

// http://stackoverflow.com/questions/39949939/how-can-i-implement-a-min-heap-of-f64-with-rusts-binaryheap
impl<T: Ord, P: Iterator<Item = T> + Clone> Ord for OrdIterator<T, P> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Clone)]
pub struct SortedIterator<T: Ord, P: Iterator<Item = T> + Clone> {
    heap: BinaryHeap<OrdIterator<T, P>>,
}

impl<T: Ord, P: Iterator<Item = T> + Clone> FromIterator<P> for SortedIterator<T, P> {
    fn from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item = P>
    {
        let iter = iter.into_iter().map(|x| OrdIterator(RefCell::new(x.peekable())));
        SortedIterator { heap: BinaryHeap::from_iter(iter) }
    }
}

impl<T, P> Iterator for SortedIterator<T, P>
    where T: Ord + Clone,
          P: Iterator<Item = T> + Clone
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {

        match self.heap.pop() {
            None => None,
            Some(iter) => {

                let next = {
                    iter.0.borrow_mut().next()
                };

                let has_next = iter.0.borrow_mut().peek().is_some();
                if has_next {
                    self.heap.push(iter);
                }

                next
            }
        }

    }
}
