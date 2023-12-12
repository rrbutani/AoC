#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::collections::VecDeque;

fn xmas_verify(numbers: &Vec<u64>, window_size: usize) -> impl Iterator<Item = (u64, bool)> + '_ {
    numbers
        .windows(window_size)
        .zip(numbers.iter().skip(window_size))
        .map(|(window, num)| {
            // Not gonna try binary search because of the duplicates problem...
            // let mut v = vec![];
            // v.extend_from_slice(window);
            // v.sort();

            for (idx, n) in window.iter().enumerate() {
                if let Some(complement) = num.checked_sub(*n) {
                    if window
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != idx)
                        .find(|(_, n)| **n == complement)
                        .is_some()
                    {
                        return (*num, true);
                    }
                }
            }

            (*num, false)
        })
}

fn main() {
    let mut aoc = AdventOfCode::new(2020, 09);
    let input: String = aoc.get_input();

    let numbers: Vec<u64> = input.lines().map(|l| l.parse().unwrap()).collect();

    let p1 = xmas_verify(&numbers, 25)
        .filter(|(_, valid)| !valid)
        .next()
        .unwrap()
        .0;
    let _ = aoc.submit_p1(p1);

    let mut range = VecDeque::new();
    let mut sum = 0;
    let mut p2 = None;
    for n in &numbers {
        while sum > p1 {
            sum -= range.pop_front().unwrap();
        }

        if sum == p1 {
            p2 = Some(*range.iter().min().unwrap() + *range.iter().max().unwrap());
            break;
        }

        sum += n;
        range.push_back(n);
    }
    let _ = aoc.submit_p2(p2.unwrap());
}
