#!/usr/bin/env rustr
#![feature(nll)] // I think I'm bad at making things not gross

extern crate aoc;
#[macro_use(scan_fmt)] extern crate scan_fmt;

#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};
use std::slice::Iter;

#[derive(Clone, Copy)]
struct Steps {
    name: char,
    prereqs: u32,
}

#[derive(Debug)]
struct Node {
    name: char,
    allows_for: Vec<Node>,
}

#[derive(Debug, Copy, Clone)]
enum IteratorEvent<'a> {
    Added(&'a Node), // Useful for BFS
    Entered(&'a Node), // Preorder DFS
    Exited(&'a Node), // Postorder DFS
}

// Recursive Impl:
#[derive(Clone)]
struct EventedNodeIterator<'a> {
    current: Option<&'a Node>,
    inner: Option<Vec<EventedNodeIterator<'a>>>,
    iter: Option<Iter<'a, Node>>,
}

impl Node {
    fn iter(&self) -> EventedNodeIterator {
        EventedNodeIterator { current: Some(&self), inner: None, iter: None }
    }
}

impl<'a> Iterator for EventedNodeIterator<'a> {
    type Item = IteratorEvent<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match (self.current, &mut self.inner, &mut self.iter) {
            (Some(c), None, None) => {
                // Entering a node:
                self.iter = Some(c.allows_for.iter());
                self.inner = Some(Vec::with_capacity(c.allows_for.len()));
                Some(IteratorEvent::Entered(c))
            },
            (Some(_), Some(ref mut v), Some(ref mut i)) => {
                // Sweeping it's children:
                if let Some(n) = i.next() {
                    // For each child, add it to our list and also yield it:
                    v.insert(0, n.iter());
                    Some(IteratorEvent::Added(n))
                } else {
                    // Once we're out of children, move on:
                    self.iter = None;
                    self.next()
                }
            }
            (Some(c), Some(ref mut v), None) => {
                match v.last_mut() {
                    None => {
                        self.current = None;
                        Some(IteratorEvent::Exited(c))
                    },
                    Some(ref mut n) => {
                        match n.next() {
                            None => {
                                // This iterator is finished; onto the next!
                                v.pop();
                                self.next()
                            },
                            Some(e) => Some(e),
                        }
                    }
                }
            }
            (None, _, None) => None,
            _ => unreachable!()

        }
    }
}

// Probably better non recursive impl that isn't finished:

// struct EventedNodeIterator<'a> {
//     stack: Vec<(&'a Node, usize)>,
// }

// impl Node {
//     fn iter(&self) -> EventedNodeIterator {
//         let mut v = Vec::new();
//         v.push((&Self, 0));
//         EventedNodeIterator { stack: v }
//     }
// }

// impl<'a> Iterator for EventedNodeIterator<'a> {
//     type Item = IteratorEvent;
    
//     fn next(&mut self) -> Option<Self::Item> {
//         while let Some((node, idx)) = self.stack.pop() {
//             if node.allows_for.len() >= idx { continue }
//         }

//         None
//     }
// }

const A: usize = b'A' as usize;

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new_with_year(2018, 07);
    let input: String = aoc.get_input();

    let mut steps: [Option<Steps>; 26] = [None; 26];
    let (mut curr, mut total) = (0u32, 0u32);

    fn find_or_create<'a>(steps: &'a mut [Option<Steps>], c: char, t: &mut u32) -> &'a mut Steps {
        if let None = steps[c as usize - A] {
            steps[c as usize - A] = Some(Steps { name: c, prereqs: 0 });
            *t |= 1 << (c as usize - A);
        }

        steps[c as usize - A].as_mut().unwrap()
    }

    input.lines().map(|s|{
        let (start, finish) = scan_fmt!(s, "Step {[A-Z]} must be finished before step {[A-Z]} can begin.",
            char, char);

        (start.unwrap(), finish.unwrap())
    }).for_each(|(s, e)| {
        find_or_create(&mut steps, e, &mut total).prereqs |= 1 << (s as usize - A);
        find_or_create(&mut steps, s, &mut total);
    });

    fn prop(steps: &[Option<Steps>], mut visited: u32, parent: &mut Node) -> u32 {
        for s in steps.iter().filter_map(|s| *s) {
            // Given the steps we've completed, check if we can complete this step:
            if s.prereqs | visited != visited { continue }

            // Check that we haven't already done this step:
            if visited | (1 << (s.name as usize - A)) == visited { continue }

            // Now check if this step actually depends on us:
            if parent.name != '@' && s.prereqs & (1 << (parent.name as usize - A)) == 0 { continue } 

            // If we haven't, make a Node for this step:
            let mut n = Node { name: s.name, allows_for: Vec::new() };

            // Check what steps finishing our current step allows us to do:
            visited |= prop(steps, visited | (1 << (s.name as usize - A)), &mut n);
            
            // And finally, add this node to it's parent:
            parent.allows_for.push(n);
        }

        visited
    }


    let mut root: Node = Node { name: '@', allows_for: Vec::new() };
    while curr != total { curr = prop(&steps, curr, &mut root) }

    let mut iter = root.iter().filter_map(|e|
        match e {
            IteratorEvent::Entered(n) => Some(n),
            _ => None
        });
    iter.next();

    let order: String = iter.map(|n| n.name).collect();


    // P1: DFS except you have to check that all incoming edges are traversed
    aoc.submit_p1(order);


    #[derive(Debug)]
    enum Next<'a> {
        Node(&'a Node),
        Stall(&'a Node),
    }

    let mut tranches: Vec<Next> = Vec::new();

    use self::IteratorEvent::*;

    let mut iter = root.iter();
    iter.next(); // Skip root
    let mut previous = iter.next().unwrap();
    for e in iter {
        match (previous, e) {
            (Entered(n), Added(_)) => tranches.push(Next::Stall(n)),
            (Added(n), _) => tranches.push(Next::Node(n)),
            _ => {}
        };

        previous = e;
    }

    let mut time_step: u32 = 0;
    let mut workers: [(Option<char>, u8); 5] = [(None, 0u8); 5];
    let mut curr: u32 = 0;

    while curr != total {
        for (n, t) in workers.iter_mut() {
            if *t == 0 {
                // For all the workers that are done with a step, mark curr to match:
                if let Some(name) = *n {
                    // println!("Worker is done with {} at {}", name, time_step);
                    curr |= 1 << (name as usize - A);

                    *n = None;
                }
            }
        }

        for (n, t) in workers.iter_mut() {
            // If this worker is done:
            if *t == 0 {
                // Try to find some new work for this node, if we still
                // have work:
                if tranches.len() == 0 { continue }

                // If we've got a stall condition:
                if let Next::Stall(n) = tranches[0] {
                    // See if we can resolve it:
                    if curr | 1 << (n.name as usize - A) == curr {
                        tranches.remove(0);
                        if tranches.len() == 0 { continue }
                    }
                }

                // If we (now) have new work, get it:
                if let Next::Node(next) = tranches[0] {
                    *n = Some(next.name);
                    *t = next.name as u8 - 5; // minus 1 because we include this step
                    tranches.remove(0);
                }
            } else {
                // If this worker isn't done, just business as usual:
                *t -= 1;
            }
        }

        time_step += 1;
        // println!("{}: {:?}", time_step, workers);
    }

    // P2: BFS. The timings and the fact that we're sorting by cost eliminates
    // all the edge cases, I think.
    aoc.submit_p2(time_step);
}
