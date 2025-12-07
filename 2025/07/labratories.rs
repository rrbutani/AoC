use std::{collections::HashSet, mem};

use aoc::{AdventOfCode, Grid, SmallVec};
use smallvec::smallvec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(strum::EnumString, strum::Display)]
enum Cell {
    #[strum(serialize = ".")]
    Empty,
    #[strum(serialize = "S")]
    Start,
    #[strum(serialize = "^")]
    Splitter,
}

// given a list of incoming columns and a row, what outgoing columns will the beam be on?
fn step_simultaneous<'i, 'r, F: FnMut(usize)>(
    incoming_cols: &'i HashSet<usize>,
    row: &'r [Cell],
    mut split_callback: F,
) -> impl Iterator<Item = usize> + use<'i, 'r, F> {
    incoming_cols
        .iter()
        .flat_map(move |&c| -> SmallVec<[usize; 2]> {
            use Cell::*;
            match row[c] {
                Start => unreachable!(),
                Empty => smallvec![c], // pass through
                Splitter => {
                    split_callback(c);
                    let mut out = smallvec![];
                    if let Some(left) = c.checked_sub(1) {
                        out.push(left);
                    }
                    if let Some(right) = c.checked_add(1).filter(|&c| c < row.len()) {
                        out.push(right);
                    }
                    out
                }
            }
        })
}

fn main() {
    let mut aoc = AdventOfCode::new(2025, 7);

    let grid: Grid<Cell> = aoc.get_input().parse().unwrap();
    let start = grid.find(|&c| c == Cell::Start).next().unwrap();

    let splits = {
        let (mut curr_cols, mut next_cols) = (HashSet::from([start.col]), HashSet::new());
        let mut split_count = 0;
        for row in &(*grid)[start.row + 1..] {
            next_cols.clear();
            let new = step_simultaneous(&curr_cols, row, |_| {
                split_count += 1;
            });
            next_cols.extend(new);
            mem::swap(&mut curr_cols, &mut next_cols);
        }

        split_count
    };
    _ = aoc.submit_p1(splits);
}
