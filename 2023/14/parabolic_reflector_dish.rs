use std::collections::HashMap;

use aoc::{iterator_map_ext::IterMapExt, AdventOfCode, Display, FromStr, Itertools};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumString, strum::Display,
)]
enum Cell {
    #[strum(serialize = ".")]
    Empty,
    #[strum(serialize = "#")]
    Cube,
    #[strum(serialize = "O")]
    Round,
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

    // fn tilt_north(&mut self) {
    //     let (height, width) = (self.grid.len(), self.grid[0].len());

    //     for r in 0..height {
    //         for c in 0..width {
    //             if let Cell::Round = self.grid[r][c] {
    //                 // eprintln!("\nmoving ({r}, {c})");
    //                 // Move upwards until we hit a cube or reach the top:
    //                 let mut dest_r = r;
    //                 loop {
    //                     let Some(new_r) = dest_r.checked_sub(1) else {
    //                         break;
    //                     };
    //                     if self.grid[new_r][c] == Cell::Empty {
    //                         dest_r = new_r;
    //                         continue;
    //                     }

    //                     break;
    //                 }

    //                 // swap
    //                 let at_dest = self.grid[dest_r][c];
    //                 self.grid[dest_r][c] = self.grid[r][c];
    //                 self.grid[r][c] = at_dest;
    //             }
    //         }
    //     }
    // }

    fn find_load(&self) -> usize {
        self.grid
            .iter()
            .rev()
            .enumerate()
            .map(|(r, row)| row.iter().filter(|&&c| c == Cell::Round).count() * (r + 1))
            .sum()
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
            if let Some(last_seen_at) = states.get(&plat) {
                break (last_seen_at, i);
            } else {
                // TODO: borrow repr that's thinner than `Platform` so we don't
                // need to do so much allocation!!
                states.insert(plat.clone(), i);
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
