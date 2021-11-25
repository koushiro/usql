// Based on https://github.com/rust-itertools/itertools/blob/master/src/multipeek_impl.rs
//
// Licensed under the Apache License, Version 2.0 https://www.apache.org/licenses/LICENSE-2.0
// or the MIT license https://opensource.org/licenses/MIT, at your option.

#[cfg(not(feature = "std"))]
use alloc::collections::VecDeque;
use core::iter::Fuse;
#[cfg(feature = "std")]
use std::collections::VecDeque;

/// An [`Iterator`] blanket implementation that provides extra adaptors and methods.
pub trait PeekIteratorExt: Iterator {
    /// An iterator adaptor that allows the user to peek at multiple `.next()`
    /// values without advancing the base iterator.
    fn multipeek(self) -> MultiPeek<Self>
    where
        Self: Sized,
    {
        multipeek(self)
    }
}

impl<T: ?Sized> PeekIteratorExt for T where T: Iterator {}

/// See [`multipeek()`] for more information.
#[derive(Clone, Debug)]
pub struct MultiPeek<I>
where
    I: Iterator,
{
    iter: Fuse<I>,
    buf: VecDeque<I::Item>,
    index: usize,
}

/// An iterator adaptor that allows the user to peek at multiple `.next()`
/// values without advancing the base iterator.
pub fn multipeek<I>(iterable: I) -> MultiPeek<I::IntoIter>
where
    I: IntoIterator,
{
    MultiPeek {
        iter: iterable.into_iter().fuse(),
        buf: VecDeque::new(),
        index: 0,
    }
}

impl<I: Iterator> MultiPeek<I> {
    /// Returns a reference to the next() value without advancing the iterator.
    ///
    /// Like [`next`], if there is a value, it is wrapped in a `Some(T)`.
    /// But if the iteration is over, `None` is returned.
    pub fn peek(&mut self) -> Option<&I::Item> {
        if self.index < self.buf.len() {
            Some(&self.buf[self.index])
        } else {
            match self.iter.next() {
                Some(x) => {
                    self.buf.push_back(x);
                    Some(&self.buf[self.index])
                }
                None => None,
            }
        }
    }

    /// Works exactly like `.next()` with the only difference that it doesn't
    /// advance itself. `.peek_next()` can be called multiple times, to peek
    /// further ahead.
    /// When `.next()` is called, reset the peeking "cursor".
    #[inline]
    pub fn peek_next(&mut self) -> Option<&I::Item> {
        let ret = if self.index < self.buf.len() {
            Some(&self.buf[self.index])
        } else {
            match self.iter.next() {
                Some(x) => {
                    self.buf.push_back(x);
                    Some(&self.buf[self.index])
                }
                None => return None,
            }
        };
        if ret.is_some() {
            self.index += 1;
        }
        ret
    }

    /// Reset the peeking "cursor".
    #[inline]
    pub fn reset_cursor(&mut self) {
        self.index = 0;
    }

    /// Returns the peeking "cursor".
    #[inline]
    pub fn peek_cursor(&self) -> usize {
        self.index
    }

    /// Consume and return the next value of this iterator if a condition is true.
    ///
    /// If `func` returns `true` for the next value of this iterator, consume and return it.
    /// Otherwise, return `None`.
    pub fn next_if(&mut self, func: impl FnOnce(&I::Item) -> bool) -> Option<I::Item> {
        match self.peek() {
            Some(matched) if func(matched) => self.next(),
            _ => None,
        }
    }

    /// Consume and return the next item if it is equal to `expected`.
    pub fn next_if_eq<T>(&mut self, expected: &T) -> Option<I::Item>
    where
        T: ?Sized,
        I::Item: PartialEq<T>,
    {
        self.next_if(|next| next == expected)
    }
}

impl<I> Iterator for MultiPeek<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.index = 0;
        self.buf.pop_front().or_else(|| self.iter.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        size_hint::add_scalar(self.iter.size_hint(), self.buf.len())
    }
}

impl<I> ExactSizeIterator for MultiPeek<I> where I: ExactSizeIterator {}

mod size_hint {
    /// **SizeHint** is the return type of **Iterator::size_hint()**.
    pub type SizeHint = (usize, Option<usize>);

    /// Add **x** correctly to a **SizeHint**.
    #[inline]
    pub fn add_scalar(sh: SizeHint, x: usize) -> SizeHint {
        let (mut low, mut hi) = sh;
        low = low.saturating_add(x);
        hi = hi.and_then(|elt| elt.checked_add(x));
        (low, hi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multipeek() {
        let mut iter = (0..3).multipeek();
        assert_eq!(iter.peek(), Some(&0));
        assert_eq!(iter.peek_cursor(), 0);
        assert_eq!(iter.peek(), Some(&0));
        assert_eq!(iter.peek_cursor(), 0);

        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.peek_cursor(), 0);

        assert_eq!(iter.peek_next(), Some(&1));
        assert_eq!(iter.peek_cursor(), 1);
        assert_eq!(iter.peek_next(), Some(&2));
        assert_eq!(iter.peek_cursor(), 2);
        assert_eq!(iter.peek_next(), None);
        assert_eq!(iter.peek_cursor(), 2);

        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.peek_cursor(), 0);
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.peek_cursor(), 0);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.peek_cursor(), 0);
    }

    #[test]
    fn test_multipeek_next_if() {
        let mut iter = (0..5).multipeek();
        // The first item of the iterator is 0; consume it.
        assert_eq!(iter.next_if(|&x| x == 0), Some(0));
        // The next item returned is now 1, so `consume` will return `false`.
        assert_eq!(iter.next_if(|&x| x == 0), None);
        // `next_if` saves the value of the next item if it was not equal to `expected`.
        assert_eq!(iter.next(), Some(1));

        let mut iter = (1..20).multipeek();
        // Consume all numbers less than 10
        while iter.next_if(|&x| x < 10).is_some() {}
        assert_eq!(iter.peek_cursor(), 0);
        // The next value returned will be 10
        assert_eq!(iter.next(), Some(10));
        assert_eq!(iter.peek_cursor(), 0);

        let mut iter = (0..5).multipeek();
        // The first item of the iterator is 0; consume it.
        assert_eq!(iter.next_if_eq(&0), Some(0));
        assert_eq!(iter.peek_cursor(), 0);
        // The next item returned is now 1, so `consume` will return `false`.
        assert_eq!(iter.next_if_eq(&0), None);
        assert_eq!(iter.peek_cursor(), 0);
        // `next_if_eq` saves the value of the next item if it was not equal to `expected`.
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.peek_cursor(), 0);
    }
}
