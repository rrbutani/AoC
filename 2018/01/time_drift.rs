#!/usr/bin/env rustr
extern crate aoc;

#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};
use std::collections::HashSet;

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new(2018, 01);
    let input: String = aoc.get_input();
    let input = input.lines().map(|f| f.parse::<i32>().unwrap());


    let p1: i32 = input.clone().sum();
    aoc.submit_p1(p1);

    let mut hs: HashSet<i32> = HashSet::new();
    let p2: i32 = input
        .cycle()
        .accumulate_sum()
        .filter(|f| ! hs.insert(*f))
        .next()
        .unwrap();

    aoc.submit_p2(p2);
}
