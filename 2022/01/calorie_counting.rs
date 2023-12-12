#!/usr/bin/env rustr

use aoc::*;

fn main() {
    let mut aoc = AdventOfCode::new(2022, 1);
    let inp = aoc.get_input();

    let elves = inp
        .split("\n\n")
        .map(|m| m.lines().map(|c| c.parse::<usize>().unwrap()).sum());

    let p1: usize = elves.clone().max().unwrap();
    aoc.submit_p1(p1).unwrap();

    let p2: usize = elves.sorted().rev().take(3).sum();
    aoc.submit_p2(p2).unwrap();
}
