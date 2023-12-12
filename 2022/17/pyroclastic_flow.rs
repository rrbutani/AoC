#![allow(clippy::absurd_extreme_comparisons)]

use aoc::*;

use owo_colors::OwoColorize;
use std::{
    collections::HashMap,
    iter::{from_fn, repeat},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum JetDir {
    Left,
    Right,
}

impl TryFrom<char> for JetDir {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use JetDir::*;
        Ok(match value {
            '>' => Right,
            '<' => Left,
            _ => return Err(()),
        })
    }
}

impl JetDir {
    fn delta(self) -> (isize, isize) {
        use JetDir::*;
        match self {
            Left => (-1, 0),
            Right => (1, 0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct JetPatterns {
    pattern: Vec<JetDir>,
}

impl FromStr for JetPatterns {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(JetPatterns {
            pattern: s.chars().map(|c| c.try_into().unwrap()).collect_vec(),
        })
    }
}

impl JetPatterns {
    fn iter(&self) -> impl Iterator<Item = JetDir> + '_ {
        self.pattern.iter().cycle().copied()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Rock {
    grid: Vec<Vec<bool>>, // r, c
}

impl Rock {
    #[rustfmt::skip]
    fn sequence() -> Vec<Rock> {
        vec![
            /* #### */
            Rock { grid: vec![vec![true, true, true, true]] },
            /*
             *  .#.
             *  ###
             *  .#.
            */
            Rock {
                grid: vec![
                    vec![false, true, false],
                    vec![true, true, true],
                    vec![false, true, false],
                ],
            },
            /*
             * ..#
             * ..#
             * ###
             */
            Rock {
                grid: vec![
                    vec![false, false, true],
                    vec![false, false, true],
                    vec![true, true, true],
                ],
            },
            /*
             * #
             * #
             * #
             * #
             */
            Rock {
                grid: vec![
                    vec![true],
                    vec![true],
                    vec![true],
                    vec![true],
                ],
            },
            /*
             * ##
             * ##
             */
            Rock {
                grid: vec![
                    vec![true, true],
                    vec![true, true],
                ]
            }
        ]
    }

    fn iter(&self, translate: (usize, usize)) -> impl Iterator<Item = (usize, usize)> + '_ {
        let (dx, dy) = translate;
        self.grid
            .iter()
            .rev() // bleh
            .enumerate()
            .flat_map(move |(r, row)| {
                row.iter()
                    .enumerate()
                    .filter(|&(_, &cell)| cell)
                    .map(move |(c, _)| (c + dx, r + dy))
            })
    }

    fn height(&self) -> usize {
        self.grid.len()
    }
    fn width(&self) -> usize {
        // we _should_ be getting the max of the lens of all the rows but...
        // it's okay we don't need this to be robust
        self.grid[0].len()
    }
}

impl Display for Rock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.grid {
            for &c in row {
                if c {
                    write!(f, "{}", "@".bold())?
                } else {
                    write!(f, "{}", '.'.dimmed())?
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
enum Col {
    Red = 1,
    Green = 2,
    Blue = 3,
    Yellow = 4,
    Pink = 5,
}

impl Col {
    fn cols() -> &'static [Self] {
        use Col::*;
        &[Red, Green, Blue, Yellow, Pink]
    }
}

impl Display for Col {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Col::*;
        match self {
            Red => write!(f, "{}", '#'.red()),
            Green => write!(f, "{}", '#'.green()),
            Blue => write!(f, "{}", '#'.blue()),
            Yellow => write!(f, "{}", '#'.yellow()),
            Pink => write!(f, "{}", '#'.purple()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Grid {
    top: usize, // next empty row
    inner: Vec<[Option<Col>; 7]>,
}

impl Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.inner[0..self.top].iter().rev() {
            if let Some(fmt::Alignment::Right) = f.align() {
                write!(f, "    ")?;
            }

            write!(f, "{}", '|'.bold())?;

            for cell in row {
                match cell {
                    Some(col) => write!(f, "{col}"),
                    None => write!(f, "{}", '.'.dimmed()),
                }?
            }

            writeln!(f, "{}", "|".bold())?;
        }

        if let Some(fmt::Alignment::Right) = f.align() {
            write!(f, "    ")?;
        }
        writeln!(f, "{}", "+-------+".bold())
    }
}

impl Grid {
    fn new() -> Self {
        Grid {
            top: 0,
            inner: Vec::with_capacity(1_000_000),
        }
    }

    fn width(&self) -> usize {
        7
    }

    fn height(&self) -> usize {
        self.top
    }

    fn collides(&self, mut it: impl Iterator<Item = (usize, usize)>) -> bool {
        it.any(|(x, y)| {
            let res = self
                .inner
                .get(y)
                .map(|row| row[x].is_some())
                .unwrap_or(false);
            if DEBUG >= 4 {
                eprintln!("  checkin ({x}, {y}): {res}");
            }
            res
        })
    }

    // coords is the bottom left corner of `rock`
    fn insert(&mut self, coords @ (_x, _y): (usize, usize), rock: &Rock, col: Col) {
        for (x, y) in rock.iter(coords) {
            let row = if let Some(r) = self.inner.get_mut(y) {
                r
            } else {
                // Append new rows as needed.
                for _ in self.inner.len()..=y {
                    self.inner.push(Default::default());
                }
                self.top = y + 1;
                &mut self.inner[y]
            };

            row[x] = Some(col);
        }
    }

    fn squish(&mut self) {}

    fn place(&mut self, jets: &mut impl Iterator<Item = JetDir>, rock: &Rock, col: Col) {
        // start in the 3rd column, 3 above the last rock
        let starting @ (_x, _y) = (2usize, self.top + 3);

        let jet_stream = from_fn(|| jets.next().map(JetDir::delta));
        let gravity = repeat((0, -1));
        let mut moves = jet_stream.interleave(gravity);

        let ret = moves.try_fold(starting, |(x, y), (dx, dy)| {
            let bail = (x, y);
            if DEBUG >= 2 {
                eprintln!(
                    "at: {bail:?}; move: {}",
                    (match (dx, dy) {
                        (0, -1) => "down",
                        (-1, 0) => "left",
                        (1, 0) => "right",
                        _ => unreachable!(),
                    })
                    .underline()
                );
            }

            let coor @ (_nx, _ny) = (
                /* left/right should saturate in [0, self.width() - rock.width()] */
                x.checked_add_signed(dx)
                    .unwrap_or(0)
                    .min(self.width() - rock.width()),
                /* up/down should be infallible; only error if we try to go
                 * below zero */
                y.checked_add_signed(dy).ok_or(bail)?,
            );

            // Now check if the rock, when placed at these coordinates, collides
            // with anything.
            if self.collides(rock.iter(coor)) {
                // if so, bail
                //
                // update! we only bail if it was a _downward_ movement.
                //
                // otherwise we just ignore it.
                if dy == 0 {
                    // invalid lateral movements don't cause us stop
                    Ok(bail)
                } else {
                    Err(bail)
                }
            } else {
                // if not, we can keep going
                if DEBUG >= 3 {
                    let mut grid = self.clone();
                    grid.insert(coor, rock, col);
                    eprintln!("{:>}", grid);
                }

                Ok(coor)
            }
        });

        let Err(final_coords @ (_x, _y)) = ret else {
            // `moves` is an infinite iterator so we should only arrive here
            // via an `Err`
            unreachable!()
        };

        // Now, we place this rock at the last position it got to before it ran
        // into something.
        self.insert(final_coords, rock, col);
    }
}

// level; higher is more verbose
const DEBUG: usize = 0;

fn inputs(inp: &str) -> (JetPatterns, impl Clone + Iterator<Item = (Rock, Col)>, Grid) {
    let inp = inp.lines().next().unwrap().parse().unwrap();

    let rocks = Rock::sequence();
    let rocks = rocks.into_iter().zip(Col::cols().iter().copied()).cycle();

    (inp, rocks, Grid::new())
}

fn run(inp: &str, n: usize) -> Grid {
    let (pat, mut rocks, mut grid) = inputs(inp);
    let mut pat = pat.iter();

    for (r, c) in rocks.take(n) {
        grid.place(&mut pat, &r, c);
        // eprintln!("{r}\n");
        if DEBUG >= 1 {
            eprintln!("{grid}\n");
        }
    }

    grid
}

fn main() {
    let mut aoc = AdventOfCode::new(2022, 17);
    let inp = aoc.get_input();
    // let inp = String::from(include_str!("ex"));

    let p1 = run(&inp, 2022).height();
    // aoc.submit_p1(dbg!(p1)).unwrap();

    // Two ideas for part 2:
    //   - the storage costs of doing 1 trillion+ rock placements are
    //     prohibitive so... we can drop the history once we have a "well" (i.e.
    //     a closed off divider; for simplicity we'll just do solid lines?
    //     hopefully those actually show up..)
    //  - even if we're okay on storage doing 1 trillion placements still takes
    //    lots of time! (1 trillion * ?100 cycles per / 4billion per sec) -> 7
    //    hours!
    //    + we should try to find _cycles_. we're only going to repeat at a
    //      cadence that:
    //        - has a length that's a multiple of the jet stream pattern length
    //          (assuming it has no internal patterns): 10091
    //        - has a length that's a multiple of the rock sequence length: 5
    //        - produces a "well" that's identical to something we've seen
    //          before..
    //
    // not going to bother with the first optimization.

    let (pat, rocks, mut grid) = inputs(&inp);
    let cycle_len_factor = pat.pattern.len() * Rock::sequence().len();
    let mut pat = pat.iter();

    let target = 1_000_000_000_000;

    // using a bad heuristic for now: just take the last 40 rows
    const WELL_HEIGHT: usize = 20;
    fn extract_well(grid: &Grid) -> [u8; WELL_HEIGHT] {
        // TODO: Really this needs to find the bottom of the well and then
        // include everything from it up to the top
        let mut out = [0; WELL_HEIGHT];

        let range = grid.inner.len().saturating_sub(WELL_HEIGHT)..;
        for (idx, row) in grid.inner[range].iter().enumerate() {
            let mut bits = 0;
            for (i, c) in row.iter().enumerate() {
                if c.is_some() {
                    bits |= 1 << i;
                }
            }

            out[idx] = bits;
        }

        out
    }
    fn well_to_grid(well: &[u8; WELL_HEIGHT]) -> Grid {
        let mut out = vec![[None; 7]; WELL_HEIGHT];

        let mut top = 0;
        for (idx, row) in well.iter().enumerate() {
            for c in 0..7 {
                out[idx][c] = if ((row >> c) & 1) != 0 {
                    top = idx + 1;
                    Some(Col::Red)
                } else {
                    None
                }
            }
        }

        Grid { inner: out, top }
    }

    // well -> rock num
    let mut wells = HashMap::new();
    let mut it = rocks.enumerate();
    let (offset, cadence) = 'outer: loop {
        let (idx, (r, c)) = it.next().unwrap();
        grid.place(&mut pat, &r, c);

        if idx < 10 {
            continue;
        }
        if idx % 1_000_000 == 0 {
            eprintln!("on rock: {idx} (min cadence {cycle_len_factor})");
        }

        let well = extract_well(&grid);
        let idxes: &mut Vec<_> = wells.entry(well).or_default();
        for i in idxes.iter() {
            // dbg!(idx, i);
            let delta = idx - i;
            if delta % cycle_len_factor == 0 {
                let grid = well_to_grid(&well);
                println!("grid:\n{grid}");
                println!("yoooo; cycle at {i} + X * {delta}");
                break 'outer (i, delta);
            } else if DEBUG >= 4 {
                eprintln!(
                    "match but misaligned: ({idx} - {i}) -> {delta} % {cycle_len_factor} => {}",
                    delta % cycle_len_factor
                );
            }
        }
        idxes.push(idx);
    };

    let offset_height = run(&inp, *offset).height();
    let cadence_added_height = run(&inp, offset + cadence).height() - offset_height;

    let cycles = (target - offset) / cadence;
    let extra = (target - offset) % cadence;

    let height = (cadence_added_height * cycles) + run(&inp, extra + offset).height();
    dbg!(height);

    // assert_eq!(height, 1_514_285_714_288);
    aoc.submit_p2(height).unwrap();
}

// >>><<><>><<
// >>><<><>><<<>><>>><<<>>><<<><<<>><>><<>>

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p1() {
        assert_eq!(super::run(include_str!("ex"), 2022), 3068);
    }

    #[test]
    fn p2() {
        assert_eq!(
            super::run(include_str!("ex"), 1_000_000_000_000),
            1_514_285_714_288 // -> 1.5TB even if used 1bit per cell
        );
    }
}
