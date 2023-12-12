#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Display};
use std::mem;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Spot {
    Occupied,
    Empty,
    Floor,
}

impl Display for Spot {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Spot::*;
        let c = match *self {
            Occupied => '#',
            Empty => 'L',
            Floor => '.',
        };

        write!(fmt, "{}", c)
    }
}

impl TryFrom<char> for Spot {
    type Error = ();

    fn try_from(c: char) -> Result<Spot, ()> {
        use Spot::*;
        Ok(match c {
            '#' => Occupied,
            'L' => Empty,
            '.' => Floor,
            _ => return Err(()),
        })
    }
}

impl FromStr for Spot {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        if s.len() != 1 {
            Err(())
        } else {
            s.chars().next().unwrap().try_into()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct WaitingArea {
    dimensions: (usize, usize), // (rows, cols)
    grid: Vec<Vec<Spot>>,
    staging: Vec<Vec<Spot>>,
}

impl Display for WaitingArea {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.grid {
            for spot in row {
                write!(fmt, "{}", spot)?;
            }
            writeln!(fmt, "")?;
        }

        Ok(())
    }
}

impl FromStr for WaitingArea {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        let grid: Vec<Vec<Spot>> = s
            .lines()
            .map(|r| r.chars().map(|c| c.try_into()).collect::<Result<_, _>>())
            .collect::<Result<_, _>>()?;

        let cols = if grid.len() > 1 {
            let len = grid[0].len();
            assert!(grid.iter().all(|r| r.len() == len));
            len
        } else {
            0
        };

        let staging = grid.clone();
        Ok(Self {
            dimensions: (grid.len(), cols),
            grid,
            staging,
        })
    }
}

impl WaitingArea {
    fn immediately_adjacent(
        grid: &Vec<Vec<Spot>>,
        (rows, cols): &(usize, usize),
        r: usize,
        c: usize,
    ) -> usize {
        let mut count = 0;
        for rr in r.checked_sub(1).unwrap_or(r)..=((r + 1).min(rows - 1)) {
            for cc in c.checked_sub(1).unwrap_or(c)..=((c + 1).min(cols - 1)) {
                if rr == r && cc == c {
                    continue;
                }

                if let Spot::Occupied = grid[rr][cc] {
                    count += 1
                };
            }
        }

        count
    }

    // fn repro<
    //     'a,
    //     F: for<'b> Fn(&'b Vec<Vec<Spot>>, &'b (usize, usize), usize, usize) -> usize + 'a,
    // >(
    //     &'a mut self,
    //     adj_func: F,
    //     adj_threshold: usize,
    // ) -> usize {
    //     0
    // }

    // fn step<F: for<'a> Fn(&'a Vec<Vec<Spot>>, &(usize, usize), usize, usize) -> usize>(
    fn step<'a, F: Fn(&Vec<Vec<Spot>>, &(usize, usize), usize, usize) -> usize>(
        &'a mut self,
        adj_func: &'a F,
        adj_threshold: usize,
    ) -> usize {
        let mut changed = 0;
        for (r, row) in self.grid.iter().enumerate() {
            for (c, spot) in row.iter().enumerate() {
                // match match is friend
                self.staging[r][c] =
                    match match (spot, adj_func(&self.grid, &self.dimensions, r, c)) {
                        (Spot::Empty, count) if count == 0 => Err(Spot::Occupied),
                        (Spot::Occupied, count) if count >= adj_threshold => Err(Spot::Empty),
                        (state, _) => Ok(*state),
                    } {
                        Ok(state) => state,
                        Err(state) => {
                            changed += 1;
                            state
                        }
                    };
            }
        }

        mem::swap(&mut self.staging, &mut self.grid);

        changed
    }

    fn occupied(&self) -> usize {
        self.grid
            .iter()
            .flat_map(|r| r)
            .filter(|c| matches!(c, Spot::Occupied))
            .count()
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2020, 11);
    let input: String = aoc.get_input();

    let mut w: WaitingArea = input.parse().unwrap();
    while w.step(&WaitingArea::immediately_adjacent, 4) != 0 {}

    let p1 = w.occupied();
    let _ = aoc.submit_p1(p1);

    // If we'd realize that the place to look for each spot is a function of the
    // floor spots; as in, it's fixed. Which means we can compute this once and
    // then just always look in that place.
    //
    // Actually, yeah. Let's do that.
    // let mut spots_to_look_at: Vec<Vec<[Option<(usize, usize)>; 8]>> =
    //     vec![vec![[None; 8]; w.dimensions.1]; w.dimensions.0];

    let mut w: WaitingArea = input.parse().unwrap();

    let spots: Vec<Vec<Vec<(usize, usize)>>> = w
        .grid
        .iter()
        .enumerate()
        .map(|(r, row)| {
            row.iter()
                .enumerate()
                .map(|(c, spot)| {
                    let mut spots = vec![];

                    if let Spot::Floor = spot {
                        return spots;
                    }

                    for r_offset in -1..=1isize {
                        for c_offset in -1..=1isize {
                            if r_offset == 0 && c_offset == 0 {
                                continue;
                            }

                            let (mut rr, mut cc) = (r, c);
                            while let (Some(arr), Some(see)) = (
                                TryInto::<isize>::try_into(rr)
                                    .unwrap()
                                    .checked_add(r_offset)
                                    .filter(|r| {
                                        (0..w.dimensions.0.try_into().unwrap()).contains(r)
                                    }),
                                TryInto::<isize>::try_into(cc)
                                    .unwrap()
                                    .checked_add(c_offset)
                                    .filter(|c| {
                                        (0..w.dimensions.1.try_into().unwrap()).contains(c)
                                    }),
                            ) {
                                rr = arr as usize;
                                cc = see as usize;

                                if w.grid[rr][cc] != Spot::Floor {
                                    spots.push((rr, cc));
                                    break;
                                }
                            }
                        }
                    }

                    spots
                })
                .collect()
        })
        .collect();

    let line_of_sight_count = move |grid: &Vec<Vec<Spot>>, _: &_, r: usize, c: usize| {
        spots[r][c]
            .iter()
            .filter(|(r, c)| matches!(grid[*r][*c], Spot::Occupied))
            .count()
    };

    while w.step(&line_of_sight_count, 5) != 0 {}
    // while w.repro(&line_of_sight_count, 5) != 0 {}

    let p2 = w.occupied();
    // println!("{}", p2);
    let _ = aoc.submit_p2(p2);
}

// impl WaitingArea {
//     // fn repro<F: Fn(Vec<Vec<Spot>>, bool, usize, usize)>(
//     fn repro<'a, F: Fn(&Vec<Vec<Spot>>, &(usize, usize), usize, usize)>(
//         &mut self,
//         adj_func: &'a F,
//         adj_threshold: usize,
//     ) -> usize {
//         0
//     }
// }
