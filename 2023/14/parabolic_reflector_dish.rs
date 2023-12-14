use std::collections::{hash_map::Entry, HashMap};

use aoc::{iterator_map_ext::IterMapExt, AdventOfCode, Display, FromStr, Itertools};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumString, strum::Display,
)]
enum Cell {
    #[strum(serialize = ".")]
    Empty = 0,
    #[strum(serialize = "O")]
    Round = 1,
    #[strum(serialize = "#")]
    Cube,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Platform {
    grid: Vec<Vec<Cell>>,
    height: usize,
    width: usize,
}

impl FromStr for Platform {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let grid = s
            .lines()
            .map(|l| l.split_inclusive(|_| true).map_parse().collect_vec())
            .collect_vec();

        let height = grid.len();
        let width = grid[0].len();
        for r in &grid {
            assert_eq!(width, r.len());
        }

        Ok(Self {
            grid,
            height,
            width,
        })
    }
}

impl Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in &self.grid {
            for c in r {
                c.fmt(f)?;
            }
            "\n".fmt(f)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Direction {
    North = 0,
    East,
    South,
    West,
}

impl Direction {
    fn as_offs(self) -> (isize, isize) {
        use Direction::*;
        match self {
            North => (-1, 0),
            East => (0, 1),
            South => (1, 0),
            West => (0, -1),
        }
    }
}

type Coord = (usize, usize);

impl Platform {
    // instead of this we should: count the number of rounds in the row/col and
    // then rewrite accordingly?
    //
    // taking care not to move cubes..
    //
    // i guess something like: partition into cube separated segments; within
    // each, hoist all the rocks to the top?
    //
    // but would that even be faster...
    //
    // the "obvious" source of parallelism is columns/rows (depending on the
    // movement direction) but that seems too fine-grained for a thread-pool to
    // provide any speedup

    fn move_cell_in_dir(&mut self, dir: Direction, (mut row, mut col): Coord) {
        let (offs_r, offs_c) = dir.as_offs();
        let (orig_row, orig_col) = (row, col);
        // eprintln!("moving ({row}, {col}) {dir:?}");
        loop {
            let new_row = row.checked_add_signed(offs_r).filter(|&r| r < self.height);
            let new_col = col.checked_add_signed(offs_c).filter(|&c| c < self.width);
            let Some((new_row, new_col)) = new_row.zip(new_col) else {
                break;
            };

            if let Cell::Empty = self.grid[new_row][new_col] {
                row = new_row;
                col = new_col;
                continue;
            } else {
                break;
            }
        }

        let at_dest = self.grid[row][col];
        self.grid[row][col] = self.grid[orig_row][orig_col];
        self.grid[orig_row][orig_col] = at_dest;
    }

    #[inline(always)]
    fn tilt<const D: u8>(&mut self /* , dir: Direction */) {
        use Direction::*;
        #[rustfmt::skip]
        let dir = match D {
            0 => North, 1 => East, 2 => South, 3 => West, _ => panic!(),
        };

        // iteration order:
        //   - if tilting north/south; go by rows; else cols
        //   - backwards of direction of movement: for north/west (i.e. moving
        //     to lower indexes), iterate from 0..; for south/east (i.e. higher
        //     indexes), iterate from <lim> down to 0
        let (outer, inner, flip) = match dir {
            North | South => (self.height, self.width, false),
            East | West => (self.width, self.height, true),
        };
        let invert_outer = matches!(dir, South | East);

        // 100 * 4 * 100 * 100 * (avg 1..100 -> 50)
        // 20'000'000 -> ~70ms; ...
        for a in 0..outer {
            for b in 0..inner {
                let a = if invert_outer { outer - 1 - a } else { a };
                let (r, c) = if flip { (b, a) } else { (a, b) };

                // dbg!((r, c));
                if let Cell::Round = self.grid[r][c] {
                    // eprintln!("({r},{c})");
                    self.move_cell_in_dir(dir, (r, c));
                }
            }
        }
    }

    fn spin(&mut self) {
        use Direction::*;
        self.tilt::<{ North as _ }>();
        self.tilt::<{ West as _ }>();
        self.tilt::<{ South as _ }>();
        self.tilt::<{ East as _ }>();
    }

    fn find_load(&self) -> usize {
        self.grid
            .iter()
            .rev()
            .enumerate()
            .map(|(r, row)| row.iter().filter(|&&c| c == Cell::Round).count() * (r + 1))
            .sum()
    }
}

// state is uniquely defined by: log2(3 ^ (height * width)) bits
// the input is 100 * 100, so: 15850 bits; about 2KB
//
// really we can do better: the location of cubes is constant so we can just
// skip those coordinates in the output; this gets us down to around 8500 cells
// with two states (empty, occupied) for each meaning: 8500 bits; just over 1KB
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct State {
    bytes: Vec<u8>,
}

// note: since we only do around ~115 spins before we arrive at a cycle,
// swapping this in for the hashmap's key doesn't confer much (any?) speedup:
// we do have fewer allocations but we need to run the below for every state
// and it's hard to compete against memcpy (even it's split up into 100
// invocations)
//
// the real bottleneck, of course, it doing the spins
//
// with LTO this takes ~40ms in total
//
// going to call that good enough for now...
impl State {
    fn make(plat: &Platform) -> Self {
        use Cell::*;
        let mut b = Vec::with_capacity(plat.height * plat.width / 8);

        let byte_chunks = plat
            .grid
            .iter()
            .flat_map(|r| r.iter())
            .filter(|&&c| c != Cell::Cube)
            .chunks(8);
        let bytes = byte_chunks.into_iter().map(|byte_chunk| {
            byte_chunk
                .enumerate()
                .map(|(i, c)| {
                    (match c {
                        Empty => 0,
                        Round => 1,
                        Cube => unreachable!(),
                    }) << i
                })
                .sum::<u8>()
        });

        b.extend(bytes);

        Self { bytes: b }
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2023, 14);
    let inp = aoc.get_input();
    let platform: Platform = inp.parse().unwrap();

    let p1 = {
        let mut plat = platform.clone();
        plat.tilt::<{ Direction::North as _ }>();
        plat.find_load()
    };
    _ = aoc.submit_p1(p1);

    const TARGET: usize = 1_000_000_000;
    let p2 = {
        let mut plat = platform.clone();

        let mut states = HashMap::new();
        let mut i = 0;
        let (first, next) = loop {
            // if let Some(last_seen_at) = states.get(&plat) {
            //     break (last_seen_at, i);
            // } else {
            //     // TODO: borrow repr that's thinner than `Platform` so we don't
            //     // need to do so much allocation!!
            //     states.insert(plat.clone(), i);
            // }

            match states.entry(State::make(&plat)) {
                Entry::Occupied(last_seen_at) => break (*last_seen_at.get(), i),
                Entry::Vacant(v) => {
                    v.insert(i);
                }
            }

            plat.spin();
            i += 1;
        };

        let cycle_len = next - first;
        let remaining = TARGET - first;
        let extra_spins = remaining % cycle_len;
        eprintln!("{first}, {next}: cycle len = {cycle_len}; remaining: {remaining}; extra: {extra_spins}");

        for _ in 0..extra_spins {
            plat.spin();
        }
        plat.find_load()
    };
    _ = aoc.submit_p2(p2);
}
