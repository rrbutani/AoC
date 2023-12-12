use std::{
    collections::{hash_map::RandomState, HashMap},
    hash::{BuildHasher, Hash},
};

use itertools::Itertools;

pub trait IterFreqExt: Iterator + Sized
where
    Self::Item: Eq + Hash,
{
    fn most_common(self) -> Option<Self::Item> {
        self.most_common_with_hash_state::<RandomState>()
    }

    fn most_common_with_hash_state<HS: BuildHasher + Default>(self) -> Option<Self::Item> {
        let mut v = iter_to_sorted_counts::<Self, HS>(self);
        if let Some((e, count)) = v.pop() {
            match v.pop() {
                Some((_, next_most_count)) if next_most_count == count => None,
                _ => Some(e),
            }
        } else {
            None
        }
    }

    fn least_common(self) -> Option<Self::Item> {
        self.least_common_with_hash_state::<RandomState>()
    }

    fn least_common_with_hash_state<HS: BuildHasher + Default>(self) -> Option<Self::Item> {
        let mut v = iter_to_sorted_counts::<Self, HS>(self).into_iter();
        if let Some((e, count)) = v.next() {
            match v.next() {
                Some((_, next_least_count)) if next_least_count == count => None,
                _ => Some(e),
            }
        } else {
            None
        }
    }
}

impl<It> IterFreqExt for It
where
    It: Iterator + Sized,
    It::Item: Eq + Hash,
{
}

fn iter_to_sorted_counts<I, HashState>(it: I) -> Vec<(I::Item, usize)>
where
    I: Iterator,
    I::Item: Eq + Hash,
    HashState: BuildHasher + Default,
{
    let mut hm: HashMap<I::Item, usize, HashState> = HashMap::default();
    // TODO: trusted len reserve optimization

    for e in it {
        *hm.entry(e).or_default() += 1;
    }

    hm.into_iter().sorted_by_key(|(_, count)| *count).collect()
}
