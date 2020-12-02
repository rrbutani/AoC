#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};
use std::u32;
use md5::compute as md5;

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new(2015, 4);
    let input: String = aoc.get_input();
    let input = input.lines().next().unwrap();

    let search = |string, input| (0..=u32::MAX)
        .filter(|i| format!("{:x}", md5(format!("{}{}", input, i)))
            .starts_with(string))
        .next()
        .unwrap();

    aoc.submit_p1(search("00000", input));
    aoc.submit_p2(search("000000", input));
}
