use std::{
    iter,
    ops::{Add, Mul},
    str::FromStr,
};

use aoc::{AdventOfCode, Grid, Itertools, iterator_map_ext::IterMapExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(strum::EnumString, strum::Display)]
enum Operation {
    #[strum(serialize = "+")]
    Add,
    #[strum(serialize = "*")]
    Mul,
}

impl Operation {
    fn as_func(self) -> fn(u64, u64) -> u64 {
        use Operation as O;
        match self {
            O::Add => Add::add,
            O::Mul => Mul::mul,
        }
    }

    fn identity(self) -> u64 {
        use Operation::*;
        match self {
            Add => 0,
            Mul => 1,
        }
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2025, 6);
    let inp = aoc.get_input();
    let (inps, ops) = {
        let mut l = inp.lines().peekable();
        let mut ops = None;

        let inp_rows = iter::from_fn(|| {
            let this = l.next()?;

            // the last row is operations, not inputs (numbers):
            if l.peek().is_none() {
                ops = Some(this);
                return None;
            }

            return Some(this.split_whitespace().map_parse::<u64>());
        });
        let inps = Grid::new_from_iter(inp_rows);
        let ops = ops
            .unwrap()
            .split_whitespace()
            .map_parse::<Operation>()
            .collect_vec();

        (inps, ops)
    };

    let grand_total = inps
        .column_iter()
        .zip(&ops)
        .map(|(col, op)| col.map(|(_, &i)| i).reduce(op.as_func()).unwrap())
        .sum::<u64>();
    _ = aoc.submit_p1(grand_total);

    // eric helpfully pads out the inputs on the right side so that we get a grid; this means we can
    // use our grid type:
    let char_grid = Grid::<char>::from_str(&inp).unwrap();
    let (ops, inps) = char_grid.split_last().unwrap();
    let mut ops = ops.iter().copied().enumerate();

    let mut grand_total = 0;
    while let Some((mut col_idx, op_char)) = ops.next() {
        // this loop should run once per group (set of vertical numbers associated with an op)
        //
        // the start of a group should have an operation; i.e. `op_char` should be `*`/`+`
        let op = op_char.to_string().parse::<Operation>().unwrap();
        let func = op.as_func();

        // groups are delimited by a column of spaces; continue until we see this
        //
        // note that we're processing the numbers left to right; since `*`/`+` are commutative this
        // is fine
        let mut out = op.identity();
        loop {
            let col = inps
                .iter()
                .map(|row| row[col_idx])
                .filter(|&c| c != ' ')
                .map(|c| (c as u8 - b'0') as u64);

            let Some(num) = col.reduce(|acc, digit| acc * 10 + digit) else {
                // if we got all spaces, the group is over
                break;
            };
            out = func(out, num);

            // move to the next column
            let Some((new_col_idx, op_char)) = ops.next() else {
                break; // if we're out of columns, we're done
            };
            assert_eq!(op_char, ' ', "should not hit start of new group; shouldn't see operator");
            col_idx = new_col_idx;
        }

        grand_total += out;
    }
    _ = aoc.submit_p2(grand_total);
}

// bleh. so verbose
