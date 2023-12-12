use std::{collections::BTreeSet, mem};

use aoc::{iterator_map_ext::IterMapExt, AdventOfCode, FromStr, Itertools};

use rayon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumString)]
enum Cell {
    #[strum(serialize = "#")]
    Galaxy,
    #[strum(serialize = ".", serialize = "+")]
    Empty,
}

type Coord = (usize, usize);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Snapshot {
    grid: Vec<Vec<Cell>>,
    empty_cols: BTreeSet<usize>,
    empty_rows: BTreeSet<usize>,
    galaxies: Vec<Coord>,
}

impl FromStr for Snapshot {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let grid = s
            .lines()
            .map(|l| l.split_inclusive(|_| true).map_parse().collect_vec())
            .collect_vec();

        let mut galaxies = Vec::new();
        let mut empty_rows = Vec::new();
        for (r, row) in grid.iter().enumerate() {
            let mut empty = true;
            for (c, cell) in row.iter().enumerate() {
                if let Cell::Galaxy = cell {
                    galaxies.push((r, c));
                    empty = false;
                }
            }

            if empty {
                empty_rows.push(r);
            }
        }

        let mut empty_cols = Vec::new();
        for c in 0..grid[0].len() {
            let mut empty = true;
            for r in 0..grid.len() {
                if let Cell::Galaxy = grid[r][c] {
                    empty = false;
                }
            }
            if empty {
                empty_cols.push(c);
            }
        }

        Ok(Self {
            grid,
            empty_cols: empty_cols.into_iter().collect(),
            empty_rows: empty_rows.into_iter().collect(),
            galaxies,
        })
    }
}

impl Snapshot {
    #[rustfmt::skip]
    fn distance_with_expansion(
        &self,
        (mut r1, mut c1): Coord,
        (mut r2, mut c2): Coord,
        multiplier: usize,
    ) -> (usize, usize) {
        if r2 < r1 { mem::swap(&mut r1, &mut r2); }
        if c2 < c1 { mem::swap(&mut c1, &mut c2); }

        let rows = self.empty_rows.range(r1..=r2).count() * multiplier + (r2 - r1);
        let cols = self.empty_cols.range(c1..=c2).count() * multiplier + (c2 - c1);

        (rows, cols)
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2023, 11);
    let inp = aoc.get_input();
    let snapshot: Snapshot = inp.parse().unwrap();

    let find_distances = |multiplier: usize| {
        // perfect use case for `Itertools`' `combinations` but... it allocates
        // and is slower. and we can't use it with `rayon`.
        (0..snapshot.galaxies.len())
            .into_par_iter()
            .map(|a| {
                (a..snapshot.galaxies.len())
                    .map(|b| {
                        let (r, c) = snapshot.distance_with_expansion(
                            snapshot.galaxies[a],
                            snapshot.galaxies[b],
                            multiplier - 1,
                        );
                        r + c
                    })
                    .sum::<usize>()
            })
            .sum()
    };

    let p1: usize = find_distances(2);
    _ = aoc.submit_p1(p1);

    let p2: usize = find_distances(1_000_000);
    _ = aoc.submit_p2(p2);
}
