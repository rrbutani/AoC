#!/usr/bin/env rustr
extern crate aoc;

#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};

fn letter_counts(s: &str) -> (bool, bool) {
    let mut chars = [0u8; 256]; // We are making an assumption here..
    s.chars().for_each(|c| chars[c as usize] += 1);

    let f = |n| chars.iter().any(|v| *v == n);
    (f(2), f(3))
}

fn compare_allowing_one(a: &[&str]) -> Option<String> {
    let v = a[0].chars().zip(a[1].chars());

    if v.clone().filter(|(a, b)| a != b).count() == 1 {
        Some(v.filter(|(a, b)| a == b).map(|(a, _)| a).collect())
    } else { None }
}

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new_with_year(2018, 02);
    let input: String = aoc.get_input();

    let counts = input.lines()
        .map(letter_counts)
        .fold((0, 0), |a, x| (a.0 + x.0 as u32, a.1 + x.1 as u32));

    let p1 = counts.0 * counts.1;
    aoc.submit_p1(p1);

    let mut p2 = input.lines().collect::<Vec<&str>>();
    p2.sort();
    let p2 = p2.as_slice()
        .windows(2)
        .find_map(compare_allowing_one)
        .unwrap();
    aoc.submit_p2(p2);
}
