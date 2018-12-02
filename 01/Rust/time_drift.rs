#!/usr/bin/env rustr
extern crate aoc;

use aoc::{AdventOfCode};
use std::collections::HashSet;

fn main() {
    let mut aoc = AdventOfCode::new_with_year(2018, 01);
    let input: String = aoc.get_input();

    let p1: i32 = input.lines().map(|f| f.parse::<i32>().unwrap()).sum();

    aoc.submit_p1(p1);

    let mut hm: HashSet<i64> = HashSet::new();
    let mut current_freq: i64 = 0;
    let p2: i64 = input
        .lines()
        .cycle()
        .map(|f| f.parse::<i64>().unwrap())
        .map(|f| { current_freq += f; current_freq })
        .filter(|f| ! hm.insert(*f))
        .next()
        .unwrap();

    aoc.submit_p2(p2);
}
