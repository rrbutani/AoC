#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new(2015, 1);
    let input: String = aoc.get_input();

    let input = input.chars().map(|c| match c { '(' => 1, ')' => -1, _ => 0 });

    let level: i64 = input.clone().sum();
    aoc.submit_p1(level);

    let neg: usize = input.accumulate_sum::<i64>()
        .position(|e| e < 0)
        .unwrap() + 1;

    aoc.submit_p2(neg);
}
