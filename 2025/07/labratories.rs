use std::{collections::HashMap, mem};

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

// given a list of incoming columns (w/counts) and a row, what outgoing columns will the beam be on?
fn step<'r, I: Iterator<Item = (usize, usize)>, F: FnMut(usize, usize)>(
    incoming_cols: I, // col, count
    row: &'r [Cell],
    mut split_callback: F,
) -> impl Iterator<Item = (usize, usize)> + use<'r, I, F> {
    incoming_cols.flat_map(move |(c, count)| -> SmallVec<[(usize, usize); 2]> {
        use Cell::*;
        match row[c] {
            Start => unreachable!(),
            Empty => smallvec![(c, count)], // pass through
            Splitter => {
                split_callback(c, count);
                let mut out = smallvec![];
                if let Some(left) = c.checked_sub(1) {
                    out.push((left, count));
                }
                if let Some(right) = c.checked_add(1).filter(|&c| c < row.len()) {
                    out.push((right, count));
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

    let (mut curr_cols, mut next_cols) = (HashMap::from([(start.col, 1)]), HashMap::new());
    let mut split_count = 0;
    for row in &(*grid)[start.row + 1..] {
        for (col, count) in step(curr_cols.drain(), row, |_, _| {
            split_count += 1;
        }) {
            *next_cols.entry(col).or_default() += count;
        }
        mem::swap(&mut curr_cols, &mut next_cols);
    }

    _ = aoc.submit_p1(split_count);
    _ = aoc.submit_p2(curr_cols.values().sum::<usize>());
}

/*

```
.......S.......
.......1.......
......1^1......
......1.1......
.....1^2^1.....
.....1.2.1.....
....1^3^3^1....
....1.3.3.1....
...1^4^331^1...
...1.4.331.1...
..1^5^434^2^1..
..1.5.434.2.1..
.1^154^74.21^1.
.1.154.74.21.1.
1^2^A^B^B^211^1
1.2.A.B.B.211.1
```

`1 + 2 + 10 + 11 + 11 + 2 + 1 + 1 + 1 -> 40`

*/
