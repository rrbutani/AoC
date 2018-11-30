extern crate aoc;

use aoc::{AdventOfCode};

fn main() {
    let mut aoc = AdventOfCode::new_with_year(2015, 1);
    let input: String = aoc.get_input();
 
    let level: i64 = input.chars().map(|c| match c { '(' => 1, ')' => -1, _ => 0 }).sum();

    aoc.submit_p1(level);

    let mut sum: i64 = 0;
    let neg: usize = input.chars()
        .map(|c| { sum += match c { '(' => 1, ')' => -1, _ => 0 }; sum })
        .position(|e| e < 0)
        .unwrap() + 1;


    aoc.submit_p2(neg);
}