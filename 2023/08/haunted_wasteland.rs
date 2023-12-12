use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
    mem,
    ops::Index,
    time::Instant,
};

use aoc::{AdventOfCode, Itertools};
use rayon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[rustfmt::skip]
enum Step { Right, Left }

impl TryFrom<char> for Step {
    type Error = char;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        Ok(match c {
            'R' => Step::Right,
            'L' => Step::Left,
            other => return Err(other),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// struct Node<'s>(&'s str);
struct Node<'s>(u16, PhantomData<&'s str>);

#[derive(Debug, Clone, PartialEq, Eq)]
struct Map<'s> {
    instructions: Vec<Step>,
    next: HashMap<Node<'s>, (Node<'s>, Node<'s>)>,
    interned_names: HashMap<Node<'s>, &'s str>,
    name_to_node: HashMap<&'s str, Node<'s>>,
}

impl<'s> TryFrom<&'s str> for Map<'s> {
    type Error = ();

    fn try_from(s: &'s str) -> Result<Self, Self::Error> {
        let (instructions, nodes) = s.split_once("\n\n").unwrap();
        let mut name_to_node = HashMap::new();
        let mut count = 0;
        let mut put = |n: &'s str| {
            *name_to_node.entry(n).or_insert_with(|| {
                let ret = Node(count, PhantomData);
                count += 1;
                ret
            })
        };
        let next = nodes
            .lines()
            .map(|l| {
                let (from, tos) = l.split_once(" = (").unwrap();
                let (left, right) = tos.strip_suffix(')').unwrap().split_once(", ").unwrap();

                (put(from), (put(left), put(right)))
            })
            .collect();

        Ok(Map {
            instructions: instructions
                .chars()
                .map(Step::try_from)
                .collect::<Result<_, _>>()
                .unwrap(),
            next,
            interned_names: name_to_node.iter().map(|(&k, &v)| (v, k)).collect(),
            name_to_node,
        })
    }
}

impl<'s> Index<&'s str> for Map<'s> {
    type Output = Node<'s>;

    fn index(&self, index: &'s str) -> &Self::Output {
        &self.name_to_node[index]
    }
}

impl<'s> Map<'s> {
    fn step(&self, node: Node<'s>, step: Step) -> Node<'s> {
        let (left, right) = self.next[&node];
        match step {
            Step::Left => left,
            Step::Right => right,
        }
    }
    fn instructions(&self) -> impl Iterator<Item = Step> + '_ {
        self.instructions.iter().copied().cycle()
    }

    fn steps_to_end_single(&self, mut curr: Node<'s>, end: Node<'s>) -> usize {
        let mut count = 0;
        let mut instructions = self.instructions();

        while curr != end {
            count += 1;
            curr = self.step(curr, instructions.next().unwrap());
        }

        count
    }

    // number of initial nodes seems small; no use using a `HashSet` for `curr`
    // to try to collapse states
    fn steps_to_end_multi_naive(
        &self,
        mut curr: Vec<Node<'s>>,
        end_cond: fn(Node<'_>) -> bool,
    ) -> usize {
        let mut count = 0;
        let mut instructions = self.instructions();
        let mut other = Vec::with_capacity(curr.len());

        eprintln!("{:?}", curr);
        while !curr.iter().cloned().all(end_cond) {
            let step = instructions.next().unwrap();
            count += 1;
            other.extend(curr.drain(..).map(|c| self.step(c, step)));
            mem::swap(&mut other, &mut curr);
            eprintln!("{:?}", curr);
        }

        count
    }

    fn steps_to_end_multi(
        &self,
        starting: Vec<Node<'s>>,
        end_cond: fn(&Self, Node<'_>) -> bool,
    ) -> usize {
        let before = Instant::now();
        let cycle_counts: Vec<usize> = starting
            .par_iter()
            .copied()
            .map(|mut curr| {
                // let orig = curr;
                let mut seen =
                    HashSet::with_capacity(self.instructions.len() * self.name_to_node.len() / 10);
                let mut instructions = self
                    .instructions
                    .iter()
                    .copied()
                    .enumerate()
                    .cycle()
                    .peekable();
                // eprintln!("{curr:?}");
                while !seen.contains(&(curr, instructions.peek().unwrap().0)) {
                    let (idx, step) = instructions.next().unwrap();
                    seen.insert((curr, idx));
                    curr = self.step(curr, step);
                    // eprintln!("  - {curr:?} -> {}", instructions.peek().unwrap().0);

                    // note: could do more sophisticated state equivalence
                    // checking (i.e. patterns in instructions, L == R for a
                    // state, etc.) but not going to bother
                }

                let cycle_last = (curr, instructions.peek().unwrap().0);
                // println!("{:?}", cycle_last);
                // let cycle_len = 1 + self.steps_to_end_single(self.step(curr), end);
                let mut cycle_len = 0;
                let mut offsets_at_end_state = vec![];
                loop {
                    cycle_len += 1;
                    curr = self.step(curr, instructions.next().unwrap().1);

                    if end_cond(self, curr) {
                        offsets_at_end_state.push(cycle_len);
                    }

                    if (curr, instructions.peek().unwrap().0) == cycle_last {
                        break;
                    }
                }

                // for i in 1..=cycle_len {
                //     curr = self.step(curr, instructions.next().unwrap().1);
                //     if end_cond(self, curr) {
                //         offsets_at_end_state.push(i % cycle_len);
                //     }
                // }
                // dbg!(cycle_len, offsets_at_end_state);
                // eprintln!("[{orig:?}] states seen: {}; cycle len: {}; offsets at end state: {offsets_at_end_state:?}", seen.len(), cycle_len);

                // we're looking for an equation of the form:
                // `starting_offset + cycle_len * N + within_cycle_offset` for
                // which the given starting node + the instructions arrive at a
                // final node
                //
                // this simplifies to: `offset + cycle_len * N`
                //
                // if there are multiple offsets within a cycle, this is a
                // little tricky... we end up with _multiple_ equations of which
                // only 1 needs to be satisfied
                //
                // fortunately, the actual input appears to be generated such
                // that this is not the case (not true for the example input
                // though)
                if offsets_at_end_state.len() != 1 {
                    unimplemented!("uh-oh")
                }
                let starting_offset = seen.len() - cycle_len;

                // even more fortuitously, for the actual input, `offset`
                // appears to always be equal to `cycle_len`
                //
                // i.e. there are, in the cycle, an equal number of nodes after
                // the point where we hit the ending state as there are special
                // not-in-the-cycle leading nodes
                //
                // this means that this problem boils down to LCM:

                let offset = starting_offset + offsets_at_end_state[0];
                if offset != cycle_len {
                    unimplemented!("uh-oh")
                }

                cycle_len
            })
            .collect();

        // dbg!(&cycle_counts);

        fn euclid(a: usize, b: usize) -> usize {
            if a == 0 {
                b
            } else {
                euclid(b % a, a)
            }
        }
        fn lcm(a: usize, b: usize) -> usize {
            (a * b) / euclid(a, b)
        }

        cycle_counts.into_iter().fold(1, lcm)
    }
}

const INP: &str = "RL

AAA = (BBB, CCC)
BBB = (DDD, EEE)
CCC = (ZZZ, GGG)
DDD = (DDD, DDD)
EEE = (EEE, EEE)
GGG = (GGG, GGG)
ZZZ = (ZZZ, ZZZ)";

const INP2: &str = "LR

11A = (11B, XXX)
11B = (XXX, 11Z)
11Z = (11B, XXX)
22A = (22B, XXX)
22B = (22C, 22C)
22C = (22Z, 22Z)
22Z = (22B, 22B)
XXX = (XXX, XXX)";

fn main() {
    let mut aoc = AdventOfCode::new(2023, 8);
    let inp = aoc.get_input();
    let map: Map = (&*inp).try_into().unwrap();

    // let p1 = map.steps_to_end_single(Node(&"AAA"), Node(&"ZZZ"));
    let p1 = map.steps_to_end_single(map["AAA"], map["ZZZ"]);
    _ = aoc.submit_p1(p1);

    let p2 = map.steps_to_end_multi(
        map.next
            .keys()
            .copied()
            // .filter(|n| n.0.ends_with("A"))
            .filter(|n| map.interned_names[n].ends_with('A'))
            .collect_vec(),
        // |Node(name)| name.ends_with("Z"),
        |map, node| map.interned_names[&node].ends_with("Z"),
    );
    _ = aoc.submit_p2(p2);
}
