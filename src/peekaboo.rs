use std::{cell::Cell, fmt::Debug, ops::{Deref, DerefMut}};

/// A stack that allows to peek at progressively more elements
#[derive(Default)]
pub struct Boo<T> {
    vec: Vec<T>,
    peeked: Cell<usize>
}

impl<T> Boo<T> {
    pub fn top(&self) -> &T {
        let peeked = self.peeked.get();
        &self.vec[self.vec.len() - peeked - 1]
    }

    pub fn peek(&self) -> &T {
        let peeked = self.peeked.get();
        self.peeked.set(peeked + 1);
        &self.vec[self.vec.len() - peeked - 1]
    }

    pub fn reset(&self) {
        self.peeked.set(0);
    }
}

impl<T> AsRef<Vec<T>> for Boo<T> {
    fn as_ref(&self) -> &Vec<T> {
        &self.vec
    }
}

impl<T> AsMut<Vec<T>> for Boo<T> {
    fn as_mut(&mut self) -> &mut Vec<T> {
        &mut self.vec
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