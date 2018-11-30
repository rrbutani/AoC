extern crate aoc;

use aoc::{AdventOfCode};

fn main() {
    let mut aoc = AdventOfCode::new_with_year(2015, 1);
    let input: String = aoc.get_input();
 
    let level: i64 = input.chars().map(|c| match c { '(' => 1, ')' => -1, _ => 0 }).sum();

    aoc.submit_p1(level);
}