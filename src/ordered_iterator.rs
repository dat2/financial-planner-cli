use std::iter::{Iterator, Peekable};

pub struct OrderedIterator<I: Ord + Clone, P: Iterator<Item = I>> {
    first: Peekable<P>,
    second: Peekable<P>,
    pop_from: usize,
    first_empty: bool,
    second_empty: bool,
    max: Option<I>
}

impl<I, P> OrderedIterator<I, P>
    where I: Ord + Clone,
          P: Iterator<Item = I>
{
    pub fn new(first: P, second: P) -> OrderedIterator<I, P> {
        let mut first_peekable = first.peekable();
        let mut second_peekable = second.peekable();
        
        let first_empty = match first_peekable.peek() {
            Some(_) => false,
            None => true
        };
        let second_empty = match second_peekable.peek() {
            Some(_) => false,
            None => true
        };
    
        OrderedIterator {
            first: first_peekable,
            second: second_peekable,
            first_empty: first_empty,
            second_empty: second_empty,
            pop_from: 0,
            max: None
        }
    }
}

impl<I, P> Iterator for OrderedIterator<I, P>
    where I: Ord + Clone,
          P: Iterator<Item = I>
{
    type Item = I;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.first_empty {
            self.second.next()
        } else if self.second_empty {
            self.first.next()
        } else if let Some(max_item) = self.max.clone() {
            // popped <- pop_from
            let pop = if self.pop_from == 0 {
                self.first.next()
            } else {
                self.second.next()
            };
            
            if let Some(pop_item) = pop {
                // if pop_item < max_item, then we just return
                if pop_item < max_item {
                    Some(pop_item)
                // otherwise, we need to switch pop from
                // need to set self.max to pop_item
                } else {
                    self.pop_from = if self.pop_from == 0 { 1 } else { 0 };
                    self.max = Some(pop_item);
                    Some(max_item)
                }
            } else {
                // if we popped, and we got none, then we set the appropriate iterator
                // to empty
                if self.pop_from == 0 {
                    self.first_empty = true;
                } else {
                    self.second_empty = true;
                }
                Some(max_item.clone())
            }
        } else {
            // we know that first is not empty, and that second is not empty, so
            // we compare the two, return the smallest, and set self.pop_from
            let first_item = self.first.next().unwrap();
            let second_item = self.second.next().unwrap();
            
            let (min, max, pop_from) = if first_item < second_item {
                (first_item, second_item, 0)
            } else {
                (second_item, first_item, 1)
            };
            self.max = Some(max);
            self.pop_from = pop_from;
            Some(min)
        }
    }
}
