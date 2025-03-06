use std::{cell::Cell, fmt::Debug, ops::{Deref, DerefMut}};

/// A stack that allows to peek at progressively more elements
pub struct Boo<T> {
    vec: Vec<T>,
    peeked: Cell<usize>
}

impl<T> Boo<T> {
    pub fn top(&self) -> Option<&T> {
        let peeked = self.peeked.get();
        if peeked >= self.len() {
            None
        } else {
            self.vec.get(self.vec.len() - peeked - 1)
        }
    }

    pub fn peek(&self) -> Option<&T> {
        let peeked = self.peeked.get();
        self.peeked.set(peeked + 1);
        if peeked >= self.len() {
            None
        } else {
            self.vec.get(self.vec.len() - peeked - 1)
        }
    }

    pub fn reset(&self) {
        self.peeked.set(0);
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
    #[test]
    fn default_is_empty() {
        let boo: super::Boo<usize> = super::Boo::default();
        assert_eq!(boo.vec.len(), 0);
        assert_eq!(boo.peeked.get(), 0)
    }

    #[test]
    fn deref_works() {
        let mut boo = super::Boo::default();
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
        let boo = super::Boo::from(vec.clone());

        for i in 0..2 {
            assert_eq!(vec[i], boo[i]);
        }
    }

    #[test]
    fn debug_is_equivalent() {
        let vec = vec![1, 2, 3];
        let boo = super::Boo::from(vec.clone());

        assert_eq!(format!("{vec:?}"), format!("{boo:?}"));
    }

    #[test]
    fn peek_persists() {
        let boo = super::Boo::from(vec![0, 1, 2, 3, 4]);

        assert_eq!(boo.peek(), Some(&4));
        assert_eq!(boo.peeked.get(), 1);

        assert_eq!(boo.peek(), Some(&3));
        assert_eq!(boo.peeked.get(), 2);

        assert_eq!(boo.peek(), Some(&2));
        assert_eq!(boo.peeked.get(), 3);
    }

    #[test]
    fn top_relies_on_peeked() {
        let boo = super::Boo::from(vec![0, 1, 2, 3, 4]);

        assert_eq!(boo.peek(), Some(&4));
        assert_eq!(boo.peeked.get(), 1);

        assert_eq!(boo.top(), Some(&3));
        assert_eq!(boo.peeked.get(), 1);

        assert_eq!(boo.peek(), Some(&3));
        assert_eq!(boo.peeked.get(), 2);

        assert_eq!(boo.top(), Some(&2));
        assert_eq!(boo.peeked.get(), 2);
    }

    #[test]
    fn peek_exhausts_boo() {
        let boo = super::Boo::from(vec![0, 1, 2]);

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
        let boo = super::Boo::from(vec![0]);

        assert_eq!(boo.top(), Some(&0));
        assert_eq!(boo.peeked.get(), 0);

        assert_eq!(boo.peek(), Some(&0));
        assert_eq!(boo.peeked.get(), 1);

        assert_eq!(boo.top(), None);
        assert_eq!(boo.peeked.get(), 1);

        boo.reset();

        assert_eq!(boo.top(), Some(&0));
        assert_eq!(boo.peeked.get(), 0);

        assert_eq!(boo.peek(), Some(&0));
        assert_eq!(boo.peeked.get(), 1);
    }
}