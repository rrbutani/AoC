use std::{fmt::Debug, iter::Map, str::Lines};

use crate::iterator_map_ext::MapTryIntoIter;

use super::iterator_map_ext::IterMapExt;

pub trait LineTryMap: AsRef<str> {
    fn line_try_map<'s, T, U, F: FnMut(T) -> U>(
        &'s self,
        func: F,
    ) -> Map<MapTryIntoIter<Lines<'s>, T>, F>
    where
        T: TryFrom<&'s str>,
        <T as TryFrom<&'s str>>::Error: Debug,
    {
        self.as_ref().lines().map_try(func)
    }

    fn line_bytes_try_map<'s, T, U, F: FnMut(T) -> U>(
        &'s self,
        func: F,
    ) -> Map<MapTryIntoIter<Map<Lines<'s>, fn(&'s str) -> &'s [u8]>, T>, F>
    where
        T: TryFrom<&'s [u8]>,
        <T as TryFrom<&'s [u8]>>::Error: Debug,
    {
        self.as_ref()
            .lines()
            .map((|l: &'s str| -> &'s [u8] { l.as_bytes() }) as _)
            .map_try(func)
    }
}

impl<S: AsRef<str>> LineTryMap for S {}
