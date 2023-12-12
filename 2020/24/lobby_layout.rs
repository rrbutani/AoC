#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::collections::HashMap;
use std::mem;
use std::ops::RangeInclusive;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Dir {
    East,
    Southeast,
    Southwest,
    West,
    Northwest,
    Northeast,
}

impl Dir {
    ///      / \
    ///    /     \
    ///  /         \
    /// |           |
    /// |           |
    /// |           |
    ///  \         /
    ///    \     /
    ///      \ /
    ///
    /// (x, y)
    ///
    /// i.e. Northeast + Southeast + West = (0, 0)
    fn offset(&self) -> (isize, isize) {
        use Dir::*;
        match self {
            East => (2, 0),
            Southeast => (1, 1),
            Southwest => (-1, 1),
            West => (-2, 0),
            Northwest => (-1, -1),
            Northeast => (1, -1),
        }
    }

    const ALL: [Self; 6] = [
        Dir::East,
        Dir::Southeast,
        Dir::Southwest,
        Dir::West,
        Dir::Northwest,
        Dir::Northeast,
    ];
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Side {
    Black,
    White,
}

impl Side {
    fn flip(&mut self) {
        *self = match self {
            Side::Black => Side::White,
            Side::White => Side::Black,
        };
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Exhibit {
    floor: HashMap<(isize, isize), Side>,
    staging: HashMap<(isize, isize), Side>,
    range: (RangeInclusive<isize>, RangeInclusive<isize>),
}

impl Exhibit {
    fn new(floor: HashMap<(isize, isize), Side>) -> Self {
        let (min_x, max_x) = floor.keys().map(|(x, _)| x).minmax().into_option().unwrap();
        let (min_y, max_y) = floor.keys().map(|(_, y)| y).minmax().into_option().unwrap();

        Exhibit {
            staging: floor.clone(),
            range: (min_x - 2..=max_x + 2, min_y - 2..=max_y + 2),
            floor,
        }
    }

    fn step(&mut self) {
        for (x, y) in self.range.0.clone().cartesian_product(self.range.1.clone()) {
            let side = self.floor.get(&(x, y)).copied().unwrap_or(Side::White);

            let adj_black_count = Dir::ALL
                .iter()
                .map(|d| d.offset())
                .filter_map(|(dx, dy)| {
                    self.floor
                        .get(&(x + dx, y + dy))
                        .filter(|t| matches!(t, Side::Black))
                })
                .count();

            let new_side = match (side, adj_black_count) {
                (Side::Black, c) if c == 0 || c > 2 => Side::White,
                (Side::White, c) if c == 2 => Side::Black,
                _ => side,
            };

            self.staging.insert((x, y), new_side);
        }

        mem::swap(&mut self.floor, &mut self.staging);

        fn expand_range(r: &mut RangeInclusive<isize>) {
            let start = *r.start() - 2;
            let end = *r.end() + 2;

            *r = start..=end;
        }

        expand_range(&mut self.range.0);
        expand_range(&mut self.range.1);
    }

    fn black_tile_count(&self) -> usize {
        self.floor.values().filter(|s| **s == Side::Black).count()
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2020, 24);
    let input: String = aoc.get_input();

    let dirs: Vec<Vec<Dir>> = input
        .lines()
        .map(|l| {
            let mut dirs = vec![];

            let mut consumed = 0;
            let mut iter = AsRef::<[u8]>::as_ref(l).windows(2).fuse();
            while let Some(chunk) = iter.next() {
                let (dir, consumed_both) = match chunk {
                    b"se" => (Dir::Southeast, true),
                    b"sw" => (Dir::Southwest, true),
                    b"nw" => (Dir::Northwest, true),
                    b"ne" => (Dir::Northeast, true),
                    &[b'e', _] => (Dir::East, false),
                    &[b'w', _] => (Dir::West, false),
                    _ => panic!("Got: {:?}", chunk),
                };

                dirs.push(dir);
                if consumed_both {
                    iter.next();
                    consumed += 2;
                } else {
                    consumed += 1;
                }
            }

            if consumed < l.len() {
                dirs.push(match AsRef::<[u8]>::as_ref(l).last().unwrap() {
                    b'e' => Dir::East,
                    b'w' => Dir::West,
                    _ => panic!(),
                })
            }

            dirs
        })
        .collect();

    let idxes = dirs.iter().map(|l| {
        l.iter().fold((0, 0), |(x, y), dir| {
            let (dx, dy) = dir.offset();
            (dx + x, dy + y)
        })
    });

    let mut sides = HashMap::new();
    for idx in idxes.clone() {
        sides.entry(idx).or_insert(Side::White).flip();
    }

    let mut exhibit = Exhibit::new(sides);

    let p1 = exhibit.black_tile_count();
    let _ = aoc.submit_p1(p1);

    (1..=100).for_each(|_| exhibit.step());
    let p2 = exhibit.black_tile_count();
    let _ = aoc.submit_p2(p2);
}
