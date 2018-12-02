#!/usr/bin/env rustr
extern crate aoc;

#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};
use std::collections::{HashMap};

fn letter_counts(s: String) -> (bool, bool) {
    let mut hm = HashMap::<char, u8>::new();
    s.chars().for_each(|c| { let j = { *hm.get(&c).unwrap_or(&0) }; hm.insert(c, 1 + j); });

    (hm.values().into_iter().find(|v| **v == 2).is_some(),
        hm.values().into_iter().find(|v| **v == 3).is_some())
}

fn compare_allowing_one(a: &[&str]) -> Option<String> {
    let (a, b) = (a[0], a[1]);

    if a.chars().zip(b.chars()).map(|(a, b)| if a != b {1} else {0}).sum::<i32>() == 1 {
        Some(a.chars().zip(b.chars()).filter_map(|(a, b)| if a == b {Some(a)} else {None}).collect::<String>())
    } else { None }
}

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new_with_year(2018, 02);
    let input: String = aoc.get_input();

    let counts = input.lines().map(|s| letter_counts(s.to_string())).fold((0, 0) ,|acc, x| (acc.0 + if x.0 {1} else {0}, acc.1 + if x.1 {1} else {0}));

    let p1 = counts.0 * counts.1;
    aoc.submit_p1(p1);

    let mut p2 = input.lines().collect::<Vec<&str>>();
    p2.sort();
    let p2 = p2.as_slice().windows(2).find_map(compare_allowing_one).unwrap();
    println!("{:?}", p2);
    aoc.submit_p2(p2);
}
