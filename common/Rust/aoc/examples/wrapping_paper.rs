extern crate aoc;
extern crate itertools;

#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};
use std::cmp;
use std::u16;

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new_with_year(2015, 2);
    let input: String = aoc.get_input();
 
    let input = input.lines()
        .map(|s| s.split('x').collect::<Vec<&str>>())
        .filter(|v| v.len() >= 3)
        .map(|v|
            v.iter()
                .take(3)
                .map(|i| i.parse::<u16>().unwrap())
                .collect::<Vec<u16>>()
    );

    let wrapping_paper: u32 = input.clone().map(|v| {
        let t = v.iter()
            .combinations(2)
            .map(|v| v[0] * v[1])
            .fold((u16::MAX, 0u32), |acc, x| (cmp::min(acc.0, x), x as u32 + acc.1));

        (t.0 as u32) + 2 * t.1
    }).sum();

    aoc.submit_p1(wrapping_paper);

    let ribbon: u32 = input.clone().map(|mut v| {
        v.sort();
        (2 * (v[0] + v[1]) + v.iter().fold(1, |acc, x| acc * x)) as u32
    }).sum();

    aoc.submit_p2(ribbon);
}