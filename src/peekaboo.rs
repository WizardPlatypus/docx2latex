use std::{cell::Cell, fmt::Debug, ops::{Deref, DerefMut}};

/// A stack that allows to peek at progressively more elements
pub struct Boo<T> {
    vec: Vec<T>,
    peeked: Cell<usize>
}

pub trait Peek {
    type Item;

    // required
    fn peeked(&self) -> usize;
    fn incr(&self);
    fn reset(&self);
    fn len(&self) -> usize;
    fn get(&self, index: usize) -> Option<&Self::Item>;
    // provided
    fn peek(&self) -> Option<&Self::Item> {
        let peeked = self.peeked();
        self.incr();
        if peeked >= self.len() {
            None
        } else {
            self.get(self.len() - peeked - 1)
        }
    }
}

impl<T> Peek for Boo<T> {
    type Item = T;

    fn peeked(&self) -> usize {
        self.peeked.get()
    }

    fn incr(&self) {
        self.peeked.set(self.peeked() + 1);
    }

    fn reset(&self) {
        self.peeked.set(0);
    }

    fn len(&self) -> usize {
        self.vec.len()
    }

    fn get(&self, index: usize) -> Option<&Self::Item> {
        self.vec.get(index)
    }
}

impl<T> Default for Boo<T> {
    fn default() -> Self {
        Self { vec: vec![], peeked: Default::default() }
    }
}

impl<T> Deref for Boo<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl<T> DerefMut for Boo<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}

impl<T> From<Vec<T>> for Boo<T> {
    fn from(value: Vec<T>) -> Self {
        Boo { vec: value, peeked: Cell::new(0) }
    }
}

impl<T: Debug> Debug for Boo<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.vec.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn default_is_empty() {
        let boo: Boo<usize> = Boo::default();
        assert_eq!(boo.vec.len(), 0);
        assert_eq!(boo.peeked.get(), 0)
    }

    #[test]
    fn deref_works() {
        let mut boo = Boo::default();
        assert!(boo.is_empty());
        boo.push(1);
        boo.push(2);
        boo.push(3);
        assert_eq!(boo.len(), 3);
    }

    #[test]
    fn from_builds_equivalent() {
        let box_1 = Box::new(1);
        let box_2 = Box::new(2);
        let box_3 = Box::new(3);

        let vec = vec![box_1, box_2, box_3];
        let boo = Boo::from(vec.clone());

        for i in 0..2 {
            assert_eq!(vec[i], boo[i]);
        }
    }

    #[test]
    fn debug_is_equivalent() {
        let vec = vec![1, 2, 3];
        let boo = Boo::from(vec.clone());

        assert_eq!(format!("{vec:?}"), format!("{boo:?}"));
    }

    #[test]
    fn peek_persists() {
        let boo = Boo::from(vec![0, 1, 2, 3, 4]);

        assert_eq!(boo.peek(), Some(&4));
        assert_eq!(boo.peeked.get(), 1);

        assert_eq!(boo.peek(), Some(&3));
        assert_eq!(boo.peeked.get(), 2);

        assert_eq!(boo.peek(), Some(&2));
        assert_eq!(boo.peeked.get(), 3);
    }

    #[test]
    fn peek_exhausts_boo() {
        let boo = Boo::from(vec![0, 1, 2]);

        assert_eq!(boo.peek(), Some(&2));
        assert_eq!(boo.peeked.get(), 1);

        assert_eq!(boo.peek(), Some(&1));
        assert_eq!(boo.peeked.get(), 2);

        assert_eq!(boo.peek(), Some(&0));
        assert_eq!(boo.peeked.get(), 3);

        assert_eq!(boo.peek(), None);
        assert_eq!(boo.peeked.get(), 4);

        assert_eq!(boo.peek(), None);
        assert_eq!(boo.peeked.get(), 5);

        assert_eq!(boo.peek(), None);
        assert_eq!(boo.peeked.get(), 6);
    }

    #[test]
    fn reset_works() {
        let boo = Boo::from(vec![0]);

        assert_eq!(boo.peek(), Some(&0));
        assert_eq!(boo.peeked.get(), 1);

        boo.reset();

        assert_eq!(boo.peek(), Some(&0));
        assert_eq!(boo.peeked.get(), 1);
    }
}