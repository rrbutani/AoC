use core::iter::Iterator;
use core::iter::Sum;
use core::mem::take;
use core::ops::{Add, Deref, DerefMut, Div};

use crate::iterator_collect_ext::IterCollectExt;
#[doc(inline)]
pub use crate::sequence;

pub use ::scan_fmt as sf;
pub use scan_fmt::scan_fmt_some as scan_fmt;

pub use itertools::{self, Itertools};

pub use std::convert::{TryFrom, TryInto};
pub use std::fmt::{self, Debug, Display};
pub use std::str::FromStr;

pub trait TryConvert {
    fn try_to<T>(self) -> Result<T, <Self as TryInto<T>>::Error>
    where
        Self: TryInto<T>,
    {
        self.try_into()
    }

    fn to<T>(self) -> T
    where
        Self: TryInto<T>,
        <Self as TryInto<T>>::Error: Debug,
    {
        self.try_to().unwrap()
    }
}

impl<T> TryConvert for T {}

pub trait PushAndGet<T> {
    fn put(&mut self, val: T) -> &mut T;
}

impl<T> PushAndGet<T> for Vec<T> {
    fn put(&mut self, val: T) -> &mut T {
        Vec::push(self, val);
        self.last_mut().unwrap()
    }
}

pub trait FilterInPlace<T> {
    fn filter_in_place<F: Fn(&T) -> bool>(&mut self, func: F);
}

impl<T> FilterInPlace<T> for Vec<T> {
    fn filter_in_place<F: Fn(&T) -> bool>(&mut self, func: F) {
        let v = take(self);
        *self = v.into_iter().filter(func).collect();
    }
}

pub trait AverageIter<T>: Iterator<Item = T> + Sized {
    fn average<S>(self) -> S
    where
        T: Sum<T>,
        S: TryFrom<T>,
        <S as TryFrom<T>>::Error: Debug,
        S: Div<Output = S>,
        S: TryFrom<usize>,
        <S as TryFrom<usize>>::Error: Debug,
    {
        self.average_with_len_map(|u| u.try_into().unwrap())
    }

    fn average_with_len_map<S, F: Fn(usize) -> S>(self, f: F) -> S
    where
        T: Sum<T>,
        S: TryFrom<T>,
        <S as TryFrom<T>>::Error: Debug,
        S: Div<Output = S>;
}

impl<It> AverageIter<It::Item> for It
where
    It: Iterator,
    It: Clone,
{
    fn average_with_len_map<S, F: Fn(usize) -> S>(self, f: F) -> S
    where
        It::Item: Sum<It::Item>,
        S: TryFrom<Self::Item>,
        <S as TryFrom<Self::Item>>::Error: Debug,
        S: Div<Output = S>,
    {
        self.clone().sum::<It::Item>().try_to::<S>().unwrap() / f(self.count())
    }
}

pub trait Average<T>: Sized {
    fn average<S>(self) -> S
    where
        T: Sum<T>,
        S: TryFrom<T>,
        <S as TryFrom<T>>::Error: Debug,
        S: Div<Output = S>,
        S: TryFrom<usize>,
        <S as TryFrom<usize>>::Error: Debug,
    {
        self.average_with_len_map(|u| u.try_into().unwrap())
    }

    fn average_with_len_map<S, F: Fn(usize) -> S>(self, f: F) -> S
    where
        T: Sum<T>,
        S: TryFrom<T>,
        <S as TryFrom<T>>::Error: Debug,
        S: Div<Output = S>;
}

impl<T: Clone> Average<T> for &'_ [T] {
    fn average_with_len_map<S, F: Fn(usize) -> S>(self, f: F) -> S
    where
        T: Sum<T>,
        S: TryFrom<T>,
        <S as TryFrom<T>>::Error: Debug,
        S: Div<Output = S>,
    {
        self.iter().cloned().sum::<T>().try_to::<S>().unwrap() / f(self.len())
    }
}

/// Wrapper type indicating we're not certain it's the median.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]

pub struct Middle<T>(Median<T>);

impl<T> Deref for Middle<T> {
    type Target = Median<T>;

    fn deref(&self) -> &Median<T> {
        &self.0
    }
}

impl<T> DerefMut for Middle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Median<T> {
    Single(T),
    Two { lower: T, upper: T },
}

impl<T> Median<T> {
    pub fn new(n: T) -> Median<T> {
        Median::Single(n)
    }
}

impl<T: PartialEq> Median<T> {
    pub fn new_from_pair(l: T, u: T) -> Self {
        if l == u {
            Median::Single(l)
        } else {
            Median::Two { lower: l, upper: u }
        }
    }
}

impl<T> Median<T> {
    pub fn single(self) -> Option<T> {
        match self {
            Median::Single(x) => Some(x),
            _ => None,
        }
    }
}

impl<T> Median<T>
where
    T: num_traits::One + Div<Output = T> + Add<Output = T>,
{
    pub fn get(self) -> T {
        match self {
            Median::Single(x) => x,
            Median::Two { lower, upper } => (lower + upper) / (T::one() + T::one()),
        }
    }
}

pub trait IteratorMedianExt: Iterator
where
    Self::Item: Ord,
    Self: Sized,
{
    fn median(self) -> Option<Median<Self::Item>> {
        // TODO: we _could_ specialize on ExactSizeIterator here
        // doesn't really help much though
        let mut v = self.collect_vec();
        v.sort();

        // because we just sorted the iterator we can be confident it's the
        // median and not just the middle

        // because we don't want to limit ourself to `T: Clone` here we have
        // to duplicate the logic; else we could just do:
        // `v.middle().map(|m| m.0)`
        match (v.len(), v.len() % 2) {
            (0, _) => None,
            (len, 0) => {
                let [lower, upper] = v.drain((len / 2 - 1)..=(len / 2)).arr();
                Some(Median::Two { lower, upper })
            }
            (len, _) => Some(Median::Single(v.drain(..).nth(len / 2).unwrap())),
        }
    }
}

impl<I: Iterator> IteratorMedianExt for I
where
    I: Sized,
    I::Item: Ord,
{
}

pub trait GetMiddle<T> {
    fn middle(&self) -> Option<Middle<T>>;

    // when `is_sorted` is stable
    /*
    /// returns Err if not sorted
    fn median(&self) -> Result<Median<T>, ()> where T: PartialOrd<T>;
    */
}

impl<T: Clone> GetMiddle<T> for [T] {
    fn middle(&self) -> Option<Middle<T>> {
        match (self.len(), self.len() % 2) {
            (0, _) => None,
            (len, 0) => Some(Middle(Median::Two {
                lower: self[len / 2 - 1].clone(),
                upper: self[len / 2].clone(),
            })),
            (len, _) => Some(Middle(Median::Single(self[len / 2].clone()))),
        }
    }
}

/// Types that implement this can cycle through some set number of states.
/// It is up to the implementer whether to repeat states once they have all
/// been covered or to simply repeatedly return the last state.
pub trait StateSequence {
    fn next(&self) -> Self;
}

/// Like StateSequence, but also mutates the item it is called from. Like with
/// StateSequence, it is up the implementer whether states are cycled through
/// again or whether the last state is simply repeated.
///
/// Note that while it is permissible to implement both StateSequence and
/// StateSequenceMutate on a type, the `#[sequence()]` macro will never do this.
pub trait StateSequenceMutate {
    fn next(&mut self) -> Self;
}

pub trait Accumulator: Iterator {
    /// Self::Item is the input type
    /// B is the output type
    /// A is the accumulating function: (A, B) -> B
    #[inline]
    fn accumulate<B, A>(self, f: A) -> Accumulate<Self, A, B>
    where
        Self: Sized,
        A: FnMut(&B, Self::Item) -> B,
        B: Default + Clone,
    {
        Accumulate {
            iter: self,
            f,
            acc: B::default(),
        }
    }

    #[inline]
    fn accumulate_with<B, A>(self, f: A, initial: B) -> Accumulate<Self, A, B>
    where
        Self: Sized,
        A: FnMut(&B, Self::Item) -> B,
        B: Default + Clone,
    {
        Accumulate {
            iter: self,
            f,
            acc: initial,
        }
    }

    #[inline]
    fn accumulate_sum<B>(self) -> AccumulateBoxed<Self, Box<dyn FnMut(&B, Self::Item) -> B>, B>
    where
        Self: Sized,
        B: Default + Clone,
        B: Add<Self::Item, Output = B>,
    {
        let addf: Box<dyn FnMut(&B, Self::Item) -> B> =
            Box::new(|acc: &B, i: Self::Item| acc.clone() + i);
        AccumulateBoxed {
            iter: self,
            f: addf,
            acc: B::default(),
        }
    }
}

impl<T: ?Sized> Accumulator for T where T: Iterator {}

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
    A: DerefMut<Target = dyn FnMut(&B, I::Item) -> B>,
    B: Clone,
{
    type Item = B;

    #[inline]
    fn next(&mut self) -> Option<B> {
        self.iter.next().map(|i| {
            self.acc = (self.f.deref_mut())(&self.acc, i);
            self.acc.clone()
        })
    }
}

impl<I: Iterator, A, B> Iterator for Accumulate<I, A, B>
where
    A: FnMut(&B, I::Item) -> B,
    B: Clone,
{
    type Item = B;

    #[inline]
    fn next(&mut self) -> Option<B> {
        self.iter.next().map(|i| {
            self.acc = (self.f)(&self.acc, i);
            self.acc.clone()
        })
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

        assert_eq!(
            factorials,
            v.iter()
                .accumulate_with(|a, i| a * i, 1)
                .collect::<Vec<i32>>()
        ); //.collect());
    }

    #[test]
    fn addition_fancy() {
        let v = vec![1, 2, 3, 4, 5];
        let res = vec![1, 3, 6, 10, 15];

        assert_eq!(res, v.iter().accumulate_sum().collect::<Vec<i32>>());
    }
}
