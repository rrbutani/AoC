#!/usr/bin/env rustr

use aoc::*;

fn main() {
    let mut aoc = AdventOfCode::new(2021, 1);
    let inp = aoc.get_input();
    let measurements = inp.lines().map(|l| l.parse::<usize>().unwrap());

    let p1 = measurements
        .clone()
        .tuple_windows()
        .filter(|(a, b)| b > a)
        .count();
    aoc.submit_p1(p1).unwrap();

    let p2 = measurements
        .tuple_windows()
        .filter(|(a, _, _, d)| d > a)
        .count();
    aoc.submit_p2(p2).unwrap();
}
