#!/usr/bin/env rustr

// 8:09AM
// 8:16AM
// 8:18AM

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

fn main() {
    let mut aoc = AdventOfCode::new(2020, 06);
    let input: String = aoc.get_input();
    let groups = input.split("\n\n");

    let p1: u32 = groups
        .clone()
        .map(|g| {
            g.lines()
                .map(answers_to_bits)
                .fold(0, |acc, ans| acc | ans)
                .count_ones()
        })
        .sum();
    let _ = aoc.submit_p1(p1);

    let p2: u32 = groups
        .map(|g| {
            g.lines()
                .map(answers_to_bits)
                .fold(usize::MAX, |acc, ans| acc & ans)
                .count_ones()
        })
        .sum();
    let _ = aoc.submit_p2(p2);
}

fn answers_to_bits(ans: &str) -> usize {
    ans.chars()
        .map(|a| 1 << ((a as u8) - b'a'))
        .fold(0, |acc, b| acc | b)
}
