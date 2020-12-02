#!/usr/bin/env rustr
extern crate aoc;
#[macro_use(scan_fmt)] extern crate scan_fmt;

#[allow(unused_imports)]
use std::fmt::Display;
use aoc::{AdventOfCode, friends::*};
use std::collections::VecDeque;

trait ConditionalPredicate<T> {
    fn if_some<U, F: FnOnce(&T) -> U>(self, f: F) -> Self;
}

impl<T> ConditionalPredicate<T> for Option<T> {

    #[inline]
    fn if_some<U, F: FnOnce(&T) -> U>(self, f: F) -> Self {
        match self {
            Some(ref v) => { f(v); self},
            None => None
        }
    }
}

/// Returns things in [0, len)
/// i.e. -1 for len 6 -> 5
///      -7 for len 6 -> 5
///       8 for len 4 -> 0
#[inline]
fn wheel(pos: usize, len: usize, move_by: isize) -> usize {
    match (pos as isize + move_by) % (len as isize) {
        r if r <  0 => (len as isize + r) as usize,
        r if r >= 0 => r as usize,
        _ => unreachable!(),
    }
}

// Todo: Deref on CircleNode<T>
#[derive(Debug)]
struct CircleNode<T: Clone> {
    previous_id: usize,
    next_id: usize,
    inner: T
}

struct Circle<T: Clone> {
    count: usize,
    /// The backing data structure
    pool: Vec<CircleNode<T>>,
    position: Option<usize>,
    cleanup: Option<Vec<usize>>,
}

impl<T: Clone> Circle<T> {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(size: usize) -> Self {
        Self {
            count: 0,
            pool: Vec::with_capacity(size),
            position: None,
            cleanup: None,
        }
    }

    pub fn get_node_by_id(&self, id: usize) -> Option<&CircleNode<T>> {
        self.pool.get(id)
    }

    pub fn get_value_by_id(&self, id: usize) -> Option<&T> {
        self.get_node_by_id(id).map(|n| &n.inner)
    }

    pub fn size(&self) -> usize {
        self.count
    }

    pub fn get(&self) -> Option<&CircleNode<T>> {
        self.position.and_then(|p| self.pool.get(p))
    }

    pub fn remove(&mut self) -> Option<T> {
        // If we're empty, return None:
        let pos = if let Some(pos) = self.position { pos }
            else { return None };

        // Mark the id for reuse if we're doing that:
        if let Some(ref mut to_reuse) = &mut self.cleanup {
            to_reuse.push(pos);
        }

        // Update the previous and next nodes:
        let prev = self.pool[pos].previous_id;
        let next = self.pool[pos].next_id;

        self.pool[prev].next_id = next;
        self.pool[next].previous_id = prev;

        // Update our position and counts:
        let curr = &mut self.pool[pos];
        self.position = Some(curr.next_id);
        self.count -= 1;

        // Make this it's own little circle:
        curr.previous_id = pos;
        curr.next_id = pos;

        // In case that was our last (self-pointing) node:
        if self.count == 0 { self.position = None }

        Some(curr.inner.clone())
    }

    pub fn insert_tap(&mut self, val: T) -> &mut Self {
        self.insert(val);
        self
    }

    pub fn insert(&mut self, val: T) -> &CircleNode<T> {
        let id = if let Some(ref mut to_reuse) = &mut self.cleanup {
            // If we have an id that we're reusing, remove it from the
            // vector:
            to_reuse.pop().if_some(|i| self.pool.remove(*i))
        } else {
            None
        };

        let id = id.unwrap_or_else(|| self.pool.len());

        // If we have no position, it means we have no elements:
        let node = if let Some(pos) = self.position {
            // Change current.next = {new}
            //        {current.next}.previous = {new}
            //        new.previous = {current}
            //        new.next = current.next
            let next = self.pool[pos].next_id;

            self.pool[pos].next_id = id;
            self.pool[next].previous_id = id;

            CircleNode {
                previous_id: pos,
                next_id: next,
                inner: val,
            }
        } else {
            // If we're empty, just make a Node that points to itself:
            CircleNode {
                previous_id: id,
                next_id: id,
                inner: val
            }
        };

        self.pool.insert(id, node);
        self.position = Some(id);
        self.count += 1;

        &self.pool[id]
    }

    /// `rotate_clockwise` and `rotate_counterclockwise` are perhaps named
    /// badly; they're not rotating the circle so much as a cursor around the
    /// circle. If you think about it like that, CW and CCW make sense.
    pub fn rotate(&mut self, steps: isize) -> &mut Self {
        match steps {
            s if s > 0 => self.rotate_clockwise(s as usize),
            s if s < 0 => self.rotate_counterclockwise(s.abs() as usize),
            _ => self,
        }
    }

    pub fn rotate_clockwise(&mut self, steps: usize) -> &mut Self {
        let mut pos = self.position.expect("Can't rotate an empty Circle!");

        for _ in 0..steps { pos = self.pool[pos].next_id; }

        self.position = Some(pos);
        self
    }

    pub fn rotate_counterclockwise(&mut self, steps: usize) -> &mut Self {
        let mut pos = self.position.expect("Can't rotate an empty Circle!");

        for _ in 0..steps { pos = self.pool[pos].previous_id; }

        self.position = Some(pos);
        self
    }

    // pub fn insert_node()

    pub fn enable_cleanup(&mut self) -> &mut Self {
        self.cleanup = Some(Vec::new());
        self
    }

    /// Warning: This drops all the id's we've marked for reuse! This won't
    /// cause problems, but we currently have no mechanism to recoup the
    /// space used by the elements we've marked.
    pub fn disable_cleanup(&mut self) -> &mut Self {
        if let Some(ref mut v) = &mut self.cleanup {
            drop(v)
        }

        self.cleanup = None;
        self
    }

    pub fn iter(&self, direction: Direction) -> CircleIterator<T> {
        CircleIterator {
            circle: self,
            current_pos: self.position,
            direction,
        }
    }

    pub fn clockwise_iter(&self) -> CircleIterator<T> {
        self.iter(Direction::Clockwise)
    }

    pub fn counterclockwise_iter(&self) -> CircleIterator<T> {
        self.iter(Direction::Counterclockwise)
    }
}

impl<'a, T: Clone> IntoIterator for &'a Circle<T> {
    type Item = T;
    type IntoIter = CircleIterator<'a, T>;

    fn into_iter(self) -> CircleIterator<'a, T> {
        CircleIterator {
            circle: self,
            current_pos: self.position,
            direction: Direction::Clockwise,
        }
    }
}

enum Direction {
    Clockwise,
    Counterclockwise,
}

struct CircleIterator<'a, T: Clone> {
    circle: &'a Circle<T>,
    current_pos: Option<usize>,
    direction: Direction,
}

impl<'a, T: Clone> Iterator for CircleIterator<'a, T> {
    type Item = T;

    /// Will only return None if the Circle we're iterating over is empty.
    /// Otherwise, we'll go around in forever.
    /// Unless something is broken.
    #[inline]
    fn next(&mut self) -> Option<T> {
        match self.current_pos {
            Some(pos) => {
                let node = self.circle.get_node_by_id(pos)?;
                self.current_pos = Some(match self.direction {
                    Direction::Clockwise => node.next_id,
                    Direction::Counterclockwise => node.previous_id,
                });

                Some(node.inner.clone())
            },
            None => None,
        }
    }
}

use std::fmt;
impl<T: Display + Clone> Display for Circle<T> {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.iter(Direction::Clockwise).take(self.size()).fold(Ok(()), |acc, x|
            acc.and_then(|_| {
                write!(f, "{} <> ", x)
            })
        ).and_then(|_| write!(f, "[loop]"))
    }
}

fn points_naive(marbles: u32) -> Vec<u32> {
    let mut circle: VecDeque<u32> = VecDeque::with_capacity(marbles as usize);
    let mut points: Vec<u32> = Vec::with_capacity((marbles / 23) as usize);

    circle.push_front(0);
    let mut pos = 1;
    for i in 1..=marbles {
        if i % 23 == 0 {
            pos = wheel(pos, circle.len(), - 9);
            points.push(circle.remove(pos).unwrap() + i as u32);
        } else {
            circle.insert(pos, i);
        }

        pos = wheel(pos, circle.len(), 1) + 1;
    }

    points
}

fn points_circle(marbles: u32) -> Vec<u32> {
    type Marble = u32;

    let mut circle: Circle<Marble> = Circle::with_capacity(marbles as usize);
    let mut points: Vec<u32> = Vec::with_capacity((marbles / 23) as usize);

    circle.insert(0);
    for i in 1..=marbles {

        if i % 23 == 0 {
            circle.rotate_counterclockwise(8);
            points.push(circle.remove().unwrap() + i as u32);
        } else {
            circle.insert(i);
        }

        circle.rotate_clockwise(1);
    }

    points
}

enum DS {
    VecDeque,
    LinkedList,
}

fn winning_score(players: u32, marbles: u32, approach: DS) -> u32 {
    let points = match approach {
        DS::VecDeque => points_naive,
        DS::LinkedList => points_circle
    }(marbles);

    let mut scores = (0..players).map(|_| 0u32).collect::<Vec<u32>>();

    points.iter().enumerate()
        .for_each(|(idx, p)| scores[(23 * idx) % players as usize] += p);

    *scores.iter().max().unwrap()
}

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new(2018, 09);
    let input: String = aoc.get_input();

    let (players, marbles) = scan_fmt!(input.lines().next().unwrap(),
            "{} players; last marble is worth {} points",
            u32, u32);
    let (players, marbles) = (players.unwrap(), marbles.unwrap());

    aoc.submit_p1(winning_score(players, marbles, DS::LinkedList));
    aoc.submit_p2(winning_score(players, 100 * marbles, DS::LinkedList));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wheel_tests() {
        assert_eq!(wheel(1, 3, 0), 1);
        assert_eq!(wheel(1, 3, 1), 2);
        assert_eq!(wheel(1, 3,-1), 0);

        assert_eq!(wheel(1, 3,-2), 2);
        assert_eq!(wheel(5, 86, -9), 82);

        for (i, v) in (-20..20).enumerate() {
            assert_eq!(wheel(0, 5, v), i % 5);
        }
    }

    fn p1_tests(winning_score: impl Fn(u32, u32) -> u32) {
        assert_eq!(winning_score(9, 25), 32);
        assert_eq!(winning_score(10, 1618), 8317);
        assert_eq!(winning_score(13, 7999), 146373);
        assert_eq!(winning_score(17, 1104), 2764);
        assert_eq!(winning_score(21, 6111), 54718);
        assert_eq!(winning_score(30, 5807), 37305);
    }

    fn big_tests(winning_score: impl Fn(u32, u32) -> u32) {
        assert_eq!(winning_score(430, 71588), 422748);
        assert_eq!(winning_score(430, 71588 * 100), 3412522480);
    }

    #[test]
    fn circle_tests_unit() {
        let mut circle = Circle::<u32>::new();

        circle.enable_cleanup();

        // An empty circle should return None and have no elements:
        assert!(circle.get().is_none());
        assert_eq!(circle.size(), 0);

        // Removing from an empty circle should return None:
        assert!(circle.remove().is_none());

        // Trying to rotate an empty circle should panic:
        // let res = std::panic::catch_unwind(|| circle.rotate_clockwise(1));
        // assert!(res.is_err());

        // let res = std::panic::catch_unwind(|| circle.rotate_counterclockwise(1));
        // assert!(res.is_err());

        // Try to insert a Node and check that it comes back right:
        let node = circle.insert(32);

        assert_eq!(node.inner, 32);
        assert_eq!(node.previous_id, node.next_id);

        // Check that rotates work fine with one node:
        circle.rotate_clockwise(100);
        let node = circle.get();
        assert!(node.is_some());
        assert_eq!(node.unwrap().inner, 32);

        circle.rotate_counterclockwise(100);
        let node = circle.get();
        assert!(node.is_some());
        assert_eq!(node.unwrap().inner, 32);

        assert_eq!(circle.size(), 1);

        // Check that removing the last node works:
        let node = circle.remove();
        assert!(node.is_some());
        assert_eq!(node.unwrap(), 32);
        assert_eq!(circle.size(), 0);

        // Try multiple nodes:
        circle.insert_tap(1)
            .insert_tap(20)
            .insert_tap(30)
            .insert_tap(40);

        assert_eq!(circle.size(), 4);

        // Get current node:
        let node = circle.get();
        assert!(node.is_some());
        let node = node.unwrap();
        assert_eq!(node.inner, 40);

        // Now let's check the whole circle by walking backwards:
        // We're expecting this:
        //  / 1 <-> 20 <-> 30 <-> 40 \
        //  \ <--------------------> /

        let assert_id = |id, v: u32| {
            let val = circle.get_value_by_id(id);
            assert!(val.is_some());
            assert_eq!(*val.unwrap(), v);
        };

        let assert_node = |id, v: u32| -> (usize, usize) {
            let node = circle.get_node_by_id(id);
            assert!(node.is_some());
            let node = node.unwrap();
            assert_eq!(node.inner, v);

            (node.previous_id, node.next_id)
        };

        // On 40:
        let prev_id = node.previous_id;
        let next_id = node.next_id;

        assert_id(prev_id, 30);
        assert_id(next_id, 1);

        // On 30:
        let (prev_id, next_id) = assert_node(prev_id, 30);
        let starting_id = next_id;

        assert_id(prev_id, 20);
        assert_id(next_id, 40);

        // On 20:
        let (prev_id, next_id) = assert_node(prev_id, 20);

        assert_id(prev_id, 1);
        assert_id(next_id, 30);

        // On 1:
        let (prev_id, next_id) = assert_node(prev_id, 1);

        assert_id(prev_id, 40);
        assert_id(next_id, 20);

        // Back to 40, hopefully:
        assert_eq!(prev_id, starting_id);

        // And now let's walk the circle forward using rotate:
        let assert_current = |circle: &Circle<u32>, expected_val| {
            let node = circle.get();
            assert!(node.is_some());
            assert_eq!(node.unwrap().inner, expected_val);
        };

        assert_current(&circle, 40); circle.rotate_clockwise(1);
        assert_current(&circle, 1);  circle.rotate_clockwise(1);
        assert_current(&circle, 20); circle.rotate_clockwise(1);
        assert_current(&circle, 30); circle.rotate_clockwise(1);
        assert_current(&circle, 40);

        // And now counterclockwise:
        assert_current(&circle, 40); circle.rotate_counterclockwise(1);
        assert_current(&circle, 30); circle.rotate_counterclockwise(1);
        assert_current(&circle, 20); circle.rotate_counterclockwise(1);
        assert_current(&circle, 1);  circle.rotate_counterclockwise(1);
        assert_current(&circle, 40);

        // And now multiple rotates:
        assert_current(&circle, 40); circle.rotate_counterclockwise(4);
        assert_current(&circle, 40); circle.rotate_clockwise(40);
        assert_current(&circle, 40); circle.rotate_clockwise(42);
        assert_current(&circle, 20); circle.rotate(-2);
        assert_current(&circle, 40);

        // TODO: Regression test for when a node is removed but other
        // nodes still remain.
    }

    #[test]
    fn naive_tests_full() {
        p1_tests(|a, b| winning_score(a, b, DS::VecDeque))

    }

    #[test]
    fn circle_tests_full() {
        p1_tests(|a, b| winning_score(a, b, DS::LinkedList))
    }

    #[test]
    fn circle_tests_big() {
        big_tests(|a, b| winning_score(a, b, DS::LinkedList))
    }
}
