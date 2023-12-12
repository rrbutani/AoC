use std::{fmt::Debug, iter::Map, marker::PhantomData, str::FromStr};

#[derive(Debug, Clone, Copy, Hash)]
pub struct MapParseIter<I, T>(PhantomData<T>, I);

impl<'a, I: Iterator<Item = &'a str>, T: FromStr> Iterator for MapParseIter<I, T>
where
    T::Err: Debug,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.1.next().map(|i| i.trim().parse().unwrap())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.1.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.1.count()
    }
}

impl<I: ExactSizeIterator, T> ExactSizeIterator for MapParseIter<I, T>
where
    Self: Iterator,
{
    fn len(&self) -> usize {
        self.1.len()
    }
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct MapTryIntoIter<I, T>(PhantomData<T>, I);

impl<I: Iterator, T: TryFrom<I::Item>> Iterator for MapTryIntoIter<I, T>
where
    T::Error: Debug,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.1.next().map(|i| i.try_into().unwrap())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.1.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.1.count()
    }
}

impl<I: ExactSizeIterator, T: TryFrom<I::Item>> ExactSizeIterator for MapTryIntoIter<I, T>
where
    T::Error: Debug,
{
    fn len(&self) -> usize {
        self.1.len()
    }
}

pub trait IterMapExt: Sized + Iterator {
    fn map_parse<'a, T>(self) -> MapParseIter<Self, T>
    where
        Self: Iterator<Item = &'a str>,
        T: FromStr,
        T::Err: Debug,
    {
        MapParseIter(PhantomData, self)
    }

    fn map_into<T>(self) -> MapTryIntoIter<Self, T>
    where
        T: TryFrom<Self::Item>,
        T::Error: Debug,
    {
        MapTryIntoIter(PhantomData, self)
    }

    fn map_try<T, U, F: FnMut(T) -> U>(self, func: F) -> Map<MapTryIntoIter<Self, T>, F>
    where
        T: TryFrom<Self::Item>,
        T::Error: Debug,
    {
        self.map_into().map(func)
    }
}

impl<I: Iterator + Sized> IterMapExt for I {}
