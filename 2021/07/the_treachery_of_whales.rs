#!/usr/bin/env rustr

use aoc::*;

fn main() {
    let mut aoc = AdventOfCode::new(2021, 7);
    let inp = aoc.get_input();
    let positions = inp.split(',').map_parse::<i32>();

    let median = positions.clone().median().unwrap().get();
    let p1: i32 = positions.clone().map(|p| (p - median).abs()).sum();
    aoc.submit_p1(p1).unwrap();

    let avg = positions.clone().average::<usize>() as i32;
    let seq_sum = |n| n * (n + 1) / 2;
    let p2: i32 = positions.map(|p| (p - avg).abs()).map(seq_sum).sum();
    aoc.submit_p2(p2).unwrap();
}
