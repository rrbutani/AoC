#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::convert::TryInto;

fn main() {
    let mut aoc = AdventOfCode::new(2020, 05);
    let input: String = aoc.get_input();

    let bin_strs = input
        .replace("F", "0")
        .replace("B", "1")
        .replace("L", "0")
        .replace("R", "1");

    let passes = bin_strs
        .lines()
        .map(|pass| usize::from_str_radix(pass, 2).unwrap());

    let p1: usize = passes.clone().max().unwrap();
    let _ = aoc.submit_p1(p1);

    let mut passes: Vec<_> = passes.collect();
    passes.sort();

    let p2 = passes
        .chunks(2)
        .map(|i| TryInto::<[usize; 2]>::try_into(i).unwrap())
        .filter(|[lo, hi]| lo + 2 == *hi)
        .map(|[lo, _hi]| lo + 1)
        .next();
    let _ = aoc.submit_p2(p2.unwrap());
}
