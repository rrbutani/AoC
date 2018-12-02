pub use itertools::Itertools;

use std::iter::Iterator;
use core::ops::{Add, DerefMut};

pub trait Accumulator: Iterator {

    /// Self::Item is the input type
    /// B is the output type
    /// A is the accumulating function: (A, B) -> B
    #[inline]
    fn accumulate<B, A>(self, f: A) -> Accumulate<Self, A, B>
    where
        Self: Sized,
        A: FnMut(&B, Self::Item) -> B,
        B: Default + Clone
    {
        Accumulate { iter: self, f, acc: B::default() }
    }

    #[inline]
    fn accumulate_with<B, A>(self, f: A, initial: B) -> Accumulate<Self, A, B>
    where
        Self: Sized,
        A: FnMut(&B, Self::Item) -> B,
        B: Default + Clone
    {
        Accumulate { iter: self, f, acc: initial }
    }

    #[inline]
    fn accumulate_sum<B>(self) -> AccumulateBoxed<Self, Box<FnMut(&B, Self::Item) -> B>, B>
    where
        Self: Sized,
        B: Default + Clone,
        B: Add<Self::Item, Output = B>,
    {
        let addf: Box<FnMut(&B, Self::Item) -> B> = Box::new(|acc: &B, i: Self::Item| acc.clone() + i);
        AccumulateBoxed { iter: self, f: addf, acc: B::default() }
    }
}

impl<T: ?Sized> Accumulator for T where T: Iterator { }

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Debug, Clone)]
pub struct Accumulate<I, A, B> {
    iter: I,
    f: A,
    acc: B,
}


#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Debug, Clone)]
pub struct AccumulateBoxed<I, A, B> {
    iter: I,
    f: A,
    acc: B,
}

impl<I: Iterator, A, B> Iterator for AccumulateBoxed<I, A, B>
where
    A: DerefMut<Target = FnMut(&B, I::Item) -> B>,
    B: Clone
{
    type Item = B;

    #[inline]
    fn next(&mut self) -> Option<B> {
        self.iter.next().map(|i| { self.acc = (self.f.deref_mut())(&self.acc, i); self.acc.clone() })
    }    
}

impl<I: Iterator, A, B> Iterator for Accumulate<I, A, B>
where
    A: FnMut(&B, I::Item) -> B,
    B: Clone
{
    type Item = B;

    #[inline]
    fn next(&mut self) -> Option<B> {
        self.iter.next().map(|i| { self.acc = (self.f)(&self.acc, i); self.acc.clone() })
    }    
}

#[cfg(test)]
mod tests {
    use crate::friends::Accumulator;

    #[test]
    fn addition() {
        let v = vec![1, 2, 3, 4, 5];
        let res = vec![1, 3, 6, 10, 15];

        assert_eq!(res, v.iter().accumulate(|a, i| a + i).collect::<Vec<i32>>());
    }

    #[test]
    fn multiplication() {
        let v = vec![1, 2, 3, 4, 5];
        let factorials = vec![1, 2, 6, 24, 120];

        assert_eq!(factorials, v.iter().accumulate_with(|a, i| a * i, 1).collect::<Vec<i32>>());//.collect());
    }

    #[test]
    fn addition_fancy() {
        let v = vec![1, 2, 3, 4, 5];
        let res = vec![1, 3, 6, 10, 15];

        assert_eq!(res, v.iter().accumulate_sum().collect::<Vec<i32>>());
    }
}
