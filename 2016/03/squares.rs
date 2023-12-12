#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::convert::TryInto;

fn main() {
    let mut aoc = AdventOfCode::new(2016, 03);
    let input: String = aoc.get_input();

    let p1 = input
        .lines()
        .filter_map(|l| sf::scan_fmt!(l, "{}{}{}", usize, usize, usize).ok())
        .filter(|(a, b, c)| (a + b) > *c && (a + c) > *b && (b + c) > *a)
        .count();
    let _ = aoc.submit_p1(p1);

    let cols: Vec<Vec<usize>> = (0..3)
        .map(|f| {
            input
                .lines()
                .filter_map(|l| {
                    l.split_whitespace()
                        .skip(f)
                        .take(1)
                        .next()
                        .and_then(|l| l.parse().ok())
                })
                .collect::<Vec<usize>>()
        })
        .collect();

    let p2: usize = cols
        .iter()
        .map(|col| {
            col.chunks(3)
                .map(|c| TryInto::<[usize; 3]>::try_into(c).unwrap())
                .filter(|[a, b, c]| (a + b) > *c && (a + c > *b) && (b + c) > *a)
                .count()
        })
        .sum();
    let _ = aoc.submit_p2(p2);
}
