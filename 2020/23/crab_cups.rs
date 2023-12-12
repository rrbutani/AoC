#!/usr/bin/env rustr

// 10:10AM

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};
use num_traits::{ops::checked::CheckedSub, Num};

use std::marker::PhantomData;
use std::ops::{RangeFrom, RangeInclusive};
use std::{
    collections::{LinkedList, VecDeque},
    iter::FromIterator,
    mem,
};

#[derive(Debug)]
struct Small;
#[derive(Debug)]
struct Big;

trait GameNum: Num + Copy + CheckedSub + Ord + From<u8> + TryFrom<usize> {
    type R: Iterator<Item = Self>;
    fn new_range(lower: Self, upper: Self) -> Self::R;

    type RFrom: Iterator<Item = Self>;
    fn new_range_from(lower: Self) -> Self::RFrom;
}
// impl<T: Real + CheckedSub + Ord + From<u8> + Step> GameNum for T {
//     type R = Range<T>;
// }

// All this because `Step` is not yet stable..
macro_rules! gn {
    ($($ty:ty)*) => {$(
        impl GameNum for $ty {
            type R = RangeInclusive<$ty>;
            fn new_range(lower: $ty, upper: $ty) -> Self::R {
                lower..=upper
            }

            type RFrom = RangeFrom<$ty>;
            fn new_range_from(lower: $ty) -> Self::RFrom {
                lower..
            }
        }
    )*};
}

gn! { u8 u16 u32 u64 u128 usize }

#[derive(Clone, Debug, PartialEq, Eq)]
struct Game<Ty: GameNum, B: BackingStore<Ty>, Size> {
    cups: B,
    min: Ty,
    max: Ty,
    len: usize,
    move_num: usize,
    _sz: PhantomData<Size>,
}

trait BackingStore<T: GameNum>: Debug + FromIterator<T> {
    fn rotate_left(&mut self);
    fn front(&self) -> T;
    fn remove_three(&mut self, idx: usize) -> [T; 3];
    fn find_elem_idx(&self, elem: &T) -> Option<usize>;
    fn insert_three(&mut self, three: [T; 3], idx: usize);

    // Really, we want GATs so that we don't need to use `dyn Iterator` here.
    fn inorder_iterator<'a>(&'a self) -> Box<dyn Iterator<Item = T> + 'a>;
}

impl<T: GameNum + Debug> BackingStore<T> for VecDeque<T> {
    fn rotate_left(&mut self) {
        self.rotate_left(1)
    }

    fn front(&self) -> T {
        *self.front().unwrap()
    }

    fn remove_three(&mut self, idx: usize) -> [T; 3] {
        self.drain(idx..(idx + 3)).collect::<Vec<_>>().to()
    }

    // We waste some time here since we need to scan the backing array.
    fn find_elem_idx(&self, elem: &T) -> Option<usize> {
        self.iter()
            .find_position(|i| *i == elem)
            .map(|(idx, _)| idx)
    }

    // We waste tons of time here since we have to shift everything.
    fn insert_three(&mut self, three: [T; 3], idx: usize) {
        for (idx_offset, val) in three.iter().enumerate() {
            self.insert(idx + idx_offset + 1, *val);
        }
    }

    fn inorder_iterator<'a>(&'a self) -> Box<dyn Iterator<Item = T> + 'a> {
        Box::new(self.iter().copied())
    }
}

impl<T: GameNum + Debug> BackingStore<T> for LinkedList<T> {
    fn rotate_left(&mut self) {
        let removed = self.pop_front().unwrap();
        self.push_back(removed);
    }

    fn front(&self) -> T {
        *self.front().unwrap()
    }

    // Dumb but we're not wasting time here; it's just that the LinkedList API
    // can be a little unwieldy.
    fn remove_three(&mut self, idx: usize) -> [T; 3] {
        let mut removed = self.split_off(idx);
        let mut readd = removed.split_off(3);
        self.append(&mut readd);

        removed.iter().copied().collect::<Vec<_>>().to()
    }

    // We waste lots of time here and since we're not even searching through
    // packed elements in a backing array, we waste so much time here that the
    // performance of this BackingStore implementation altogether is something
    // like an order of magnitude worse than VecDeque's.
    //
    // It's dumb because since each node in the list should stay in a fixed
    // place in memory (with only the links between the nodes changing _value_)
    // we should be able to hold references to the individual nodes and throw
    // them in a HashMap or something so that we can get constant time lookup
    // for a particular node.
    //
    // `LinkedList` does not allow us to hold references to individual nodes
    // without using the Cursor API which is unstable so we resort to doing a
    // dumb O(N) search here.
    //
    // Note that for this particular problem because we're dealing with "nodes"
    // that have an obvious sequential ordering we don't even need a HashMap:
    // just a big old array (which is exactly what the
    // `ArrayPretendingToBeALinkedList` type does).
    fn find_elem_idx(&self, elem: &T) -> Option<usize> {
        self.iter()
            .find_position(|i| *i == elem)
            .map(|(idx, _)| idx)
    }

    fn insert_three(&mut self, three: [T; 3], idx: usize) {
        let mut after = self.split_off(idx + 1);
        let mut three = three.iter().copied().collect();

        self.append(&mut three);
        self.append(&mut after);
    }

    fn inorder_iterator<'a>(&'a self) -> Box<dyn Iterator<Item = T> + 'a> {
        Box::new(self.iter().copied())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ArrayPretendingToBeALinkedList<T: GameNum + TryInto<usize>>
where
    <T as TryInto<usize>>::Error: Debug,
    <usize as TryInto<T>>::Error: Debug,
{
    inner: Vec<Option<(usize, usize)>>, // [num] ⇒ Option<(prev, next)>
    front: usize,
    _t: PhantomData<T>,
}

impl<T: GameNum + TryInto<usize>> ArrayPretendingToBeALinkedList<T>
where
    <T as TryInto<usize>>::Error: Debug,
    <usize as TryInto<T>>::Error: Debug,
{
    fn front_cursor(&self) -> Cursor<'_, T> {
        Cursor(self.front, self)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Cursor<'a, T: GameNum + TryInto<usize>>(usize, &'a ArrayPretendingToBeALinkedList<T>)
where
    <T as TryInto<usize>>::Error: Debug,
    <usize as TryInto<T>>::Error: Debug;

impl<'a, T: GameNum + TryInto<usize>> Cursor<'a, T>
where
    <T as TryInto<usize>>::Error: Debug,
    <usize as TryInto<T>>::Error: Debug,
{
    fn next(&self) -> Self {
        let next_idx = self.1.inner[self.0].unwrap().1;

        Cursor(next_idx, self.1)
    }

    fn prev(&self) -> Self {
        let prev_idx = self.1.inner[self.0].unwrap().0;

        Cursor(prev_idx, self.1)
    }

    fn iter<D: IterDir>(&self) -> IteratorForArrayPretendingToBeALinkedList<'a, T, D> {
        IteratorForArrayPretendingToBeALinkedList(*self, PhantomData)
    }

    fn iter_forwards(&self) -> IteratorForArrayPretendingToBeALinkedList<'a, T, Forward> {
        self.iter()
    }

    fn iter_backwards(&self) -> IteratorForArrayPretendingToBeALinkedList<'a, T, Backward> {
        self.iter()
    }

    fn hop_forward(&mut self) {
        *self = self.next();
    }

    fn hop_backward(&mut self) {
        *self = self.next();
    }

    fn idx(self) -> usize {
        self.0
    }

    fn val(self) -> T {
        self.0.to()
    }
}

trait IterDir {
    fn step<'a, T: GameNum + TryInto<usize>>(cursor: &mut Cursor<'a, T>)
    where
        <T as TryInto<usize>>::Error: Debug,
        <usize as TryInto<T>>::Error: Debug;
}

#[derive(Debug)]
struct Forward;

#[derive(Debug)]
struct Backward;

impl IterDir for Forward {
    fn step<'a, T: GameNum + TryInto<usize>>(cursor: &mut Cursor<'a, T>)
    where
        <T as TryInto<usize>>::Error: Debug,
        <usize as TryInto<T>>::Error: Debug,
    {
        cursor.hop_forward()
    }
}

impl IterDir for Backward {
    fn step<'a, T: GameNum + TryInto<usize>>(cursor: &mut Cursor<'a, T>)
    where
        <T as TryInto<usize>>::Error: Debug,
        <usize as TryInto<T>>::Error: Debug,
    {
        cursor.hop_backward()
    }
}

struct IteratorForArrayPretendingToBeALinkedList<'a, T: GameNum + TryInto<usize>, Dir: IterDir>(
    Cursor<'a, T>,
    PhantomData<Dir>,
)
where
    <T as TryInto<usize>>::Error: Debug,
    <usize as TryInto<T>>::Error: Debug;

// This will never end!
impl<'a, T: GameNum + TryInto<usize>, D: IterDir> Iterator
    for IteratorForArrayPretendingToBeALinkedList<'a, T, D>
where
    <T as TryInto<usize>>::Error: Debug,
    <usize as TryInto<T>>::Error: Debug,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let elem = self.0.val();
        D::step(&mut self.0);

        Some(elem)
    }
}

impl<T: GameNum + TryInto<usize>> FromIterator<T> for ArrayPretendingToBeALinkedList<T>
where
    <T as TryInto<usize>>::Error: Debug,
    <usize as TryInto<T>>::Error: Debug,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let nums: Vec<_> = iter.into_iter().collect();

        let (min, max) = nums.iter().minmax().into_option().unwrap();
        assert!(*min == T::one());
        assert_eq!(((*max).to() - (*min).to() + 1), nums.len());

        // We have no 0 element so 1 more than nums.len().
        let mut inner: Vec<Option<(usize, usize)>> = (0..(nums.len() + 1)).map(|_| None).collect();

        for (idx, num) in nums.iter().map(|n| (*n).to::<usize>()).enumerate() {
            let prev_idx = idx.checked_sub(1).unwrap_or(nums.len() - 1);
            let next_idx = (idx + 1) % nums.len();

            inner[num] = Some((nums[prev_idx].to(), nums[next_idx].to()));
        }

        Self {
            inner,
            front: nums[0].to(),
            _t: PhantomData,
        }
    }
}

impl<T: GameNum + TryInto<usize> + Debug> BackingStore<T> for ArrayPretendingToBeALinkedList<T>
where
    <T as TryInto<usize>>::Error: Debug,
    <usize as TryInto<T>>::Error: Debug,
{
    fn rotate_left(&mut self) {
        self.front = self.inner[self.front].unwrap().1;
    }

    fn front(&self) -> T {
        self.front.to()
    }

    fn remove_three(&mut self, idx: usize) -> [T; 3] {
        // We're taking advantage of the fact that we only ever remove the
        // three elements right after `front`; if we were asked to remove
        // elements starting at an arbitary index we'd have to walk the list or
        // maintain some kind of "index to value" map (also just an array).
        assert_eq!(idx, 1);

        // println!("Before: {:?}", self.inorder_iterator().collect::<Vec<_>>());

        // let (_, first_next) = self.inner[self.front].as_mut().unwrap();

        // self.front_cursor()

        let mut next = self.front_cursor().next();
        let mut removed = Vec::with_capacity(3);
        for _ in 0..3 {
            let idx = next.idx();
            let val = next.val();
            removed.push(val);

            // Instead of doing this we just fix the links at the end, after
            // we've removed all the elements.
            // let (p, n) = mem::replace(&mut self.inner[idx], None).unwrap();
            // self.inner[p].as_mut().unwrap().1 = n;
            // self.inner[n].as_mut().unwrap().0 = p;

            // // self.inner[idx] = None;
            let (_, n) = mem::replace(&mut self.inner[idx], None).unwrap();

            next = Cursor(n, self);
        }

        let next = next.idx();
        self.inner[self.front].as_mut().unwrap().1 = next;
        self.inner[next].as_mut().unwrap().0 = self.front;

        // println!("After:  {:?}", self.inorder_iterator().collect::<Vec<_>>());

        removed.to()
    }

    // Under our scheme, indexes _are_ elements!
    fn find_elem_idx(&self, elem: &T) -> Option<usize> {
        let elem = (*elem).to();
        self.inner[elem].map(|_| elem)
    }

    // Since indexes are elements for us...
    fn insert_three(&mut self, three: [T; 3], idx: usize) {
        // println!("Before: {:?}", self.inorder_iterator().collect::<Vec<_>>());

        // We do some extra writes here but it's fine.
        let mut prev = Cursor(idx, self);
        let mut next = prev.next();
        for elem in three.iter().map(|i| (*i).to()) {
            assert_eq!(self.inner[elem], None);

            let (p, n) = (prev.idx(), next.idx());
            self.inner[elem] = Some((p, n));
            self.inner[p].as_mut().unwrap().1 = elem;
            self.inner[n].as_mut().unwrap().0 = elem;

            prev = Cursor(elem, self);
            next = prev.next();
        }

        // println!("After:  {:?}", self.inorder_iterator().collect::<Vec<_>>());
    }

    fn inorder_iterator<'a>(&'a self) -> Box<dyn Iterator<Item = T> + 'a> {
        Box::new(
            self.front_cursor()
                .iter_forwards()
                .take(self.inner.len() - 1),
        )
    }
}

impl<T: GameNum, B: BackingStore<T>> FromStr for Game<T, B, Small> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        Ok(Game::new(s.chars().map(|l| (l as u8 - b'0').to::<T>())))
    }
}

impl<T, B: BackingStore<T>> FromStr for Game<T, B, Big>
where
    T: GameNum + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    B: BackingStore<T>,
{
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        Ok(Game::new(
            s.chars()
                .map(|l| (l as u8 - b'0').to::<T>())
                .chain(T::new_range((s.len() + 1).to::<T>(), 1_000_000.to())),
        ))
    }
}

impl<T: GameNum, B: BackingStore<T>, S> Game<T, B, S> {
    fn new(nums: impl Iterator<Item = T>) -> Self {
        let cups: B = nums.collect();

        Game {
            min: cups.inorder_iterator().min().unwrap(),
            max: cups.inorder_iterator().max().unwrap(),
            len: cups.inorder_iterator().count(),
            cups,
            move_num: 0,
            _sz: PhantomData,
        }
    }

    fn step(&mut self) {
        self.move_num += 1;
        // println!("-- move {} --", self.move_num);
        // println!("cups: {:?}", self.cups);

        // if self.move_num % 10_000 == 0 {
        // println!("Move #{}", self.move_num);
        // }

        // let current = *self.cups.front().unwrap();
        // let removed: [u64; 3] = self.cups.drain(1..4).collect::<Vec<_>>().to();

        let current = self.cups.front();
        let removed = self.cups.remove_three(1);

        // let mut removed = self.cups.split_off(1);
        // let mut readd = removed.split_off(3);
        // self.cups.append(&mut readd);

        // println!("pick up: {:?}", removed);

        let dest_idx = T::new_range_from(T::one())
            .map(|i| {
                let x = current - self.min;
                let r = self.max - self.min + T::one();
                let o = (x + r).checked_sub(&i).unwrap() % r;
                o + self.min
            })
            .filter_map(|i| self.cups.find_elem_idx(&i))
            .next()
            .unwrap();

        // println!("destination: {} ({})\n", dest, dest_idx);

        // for (idx_offset, val) in removed.iter().enumerate() {
        //     self.cups.insert(dest_idx + idx_offset + 1, *val);
        // }
        // let mut after = self.cups.split_off(dest_idx + 1);
        // self.cups.append(&mut removed);
        // self.cups.append(&mut after);
        self.cups.insert_three(removed, dest_idx);

        // let front = self.cups.pop_front().unwrap();
        // self.cups.push_back(front);

        self.cups.rotate_left();
    }

    fn state(&self) -> impl Iterator<Item = T> + '_ {
        self.cups
            .inorder_iterator()
            .chain(self.cups.inorder_iterator())
            .skip_while(|c| *c != T::one())
            .skip(1)
            .take(self.len - 1)
    }
}

// type B<T> = VecDeque<T>;
// type B<T> = LinkedList<T>;
type B<T> = ArrayPretendingToBeALinkedList<T>;

fn main() {
    let mut aoc = AdventOfCode::new(2020, 23);
    let input: String = aoc.get_input();
    // let input = "389125467";

    let mut game: Game<u8, B<_>, Small> = input.trim().parse().unwrap();
    (0..100).for_each(|_| game.step());

    let p1: String = game.state().map(|c| format!("{}", c)).collect();
    aoc.submit_p1(p1).unwrap();

    let mut game: Game<u64, B<_>, Big> = input.trim().parse().unwrap();
    (0..10_000_000).for_each(|_| game.step());

    let p2: u64 = game.state().take(2).product();
    let _ = aoc.submit_p2(p2);
}

#[cfg(test)]
mod tests {
    use super::*;

    const EX: &'static str = "389125467";
    const P1_ANS: &'static str = "67384529";
    const P2_ANS: u64 = 149245887792;

    macro_rules! backing_store_tests {
        (($nom_p1:ident, $nom_p2:ident) -> $($bs:tt)*) => {
            #[test]
            fn $nom_p1() {
                let mut game: Game<u8, $($bs)*, Small> = EX.trim().parse().unwrap();
                (0..100).for_each(|_| game.step());

                let p1: String = game.state().map(|c| format!("{}", c)).collect();
                assert_eq!(p1, P1_ANS);
            }

            #[test]
            fn $nom_p2() {
                let mut game: Game<u64, $($bs)*, Big> = EX.trim().parse().unwrap();
                (0..10_000_000).for_each(|_| game.step());

                let p2: u64 = game.state().take(2).product();
                assert_eq!(p2, P2_ANS);
            }
        };
    }

    backing_store_tests!((vec_deque_p1, vec_deque_p2) -> VecDeque<_>);
    backing_store_tests!((linked_list_p1, linked_list_p2) -> LinkedList<_>);
    backing_store_tests!((other_linked_list_p1, other_linked_list_p2) -> ArrayPretendingToBeALinkedList<_>);
}
