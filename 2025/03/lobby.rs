use std::cmp::Reverse;

use aoc::{AdventOfCode, Grid};

fn main() {
    let mut aoc = AdventOfCode::new(2025, 03);
    let banks: Grid<u8> = aoc.get_input().parse().unwrap();

    let joltage = banks
        .iter()
        .map(|bank| {
            // find the highest joltage in [..(-1)] and then the highest in [(highest_idx)..]
            //
            // for the first battery, prefer elements at lower indexes: this allows for considering
            // more batteries for the 2nd choice
            let (left_idx, &l) = bank[..bank.len() - 1]
                .iter()
                .enumerate()
                .max_by_key(|&(idx, &j)| (j, Reverse(idx)))
                .unwrap();
            let r = bank[(left_idx + 1)..].iter().max().unwrap();
            (l * 10 + r) as usize
        })
        .sum::<usize>();
    _ = aoc.submit_p1(joltage);
}
