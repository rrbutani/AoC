#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::collections::HashMap;
use std::iter;

fn main() {
    let mut aoc = AdventOfCode::new(2020, 15);
    let input: String = aoc.get_input();
    // let input = "0,3,6";

    let numbers = input
        .trim()
        .split(',')
        .map(|n| n.parse::<usize>().unwrap())
        .enumerate();

    let len = numbers.clone().count();
    let last = numbers.clone().last();

    let get_nth = |nth: usize| {
        let mut history: HashMap<usize, usize> = numbers
            .clone()
            .take(len - 1)
            .map(|(idx, n)| (n, idx))
            .collect();

        let mut i = numbers
            .clone()
            .take(len - 1)
            .chain(iter::successors(last, |(idx, prev)| {
                let num = if let Some(old_idx) = history.get(prev) {
                    idx - old_idx
                } else {
                    0
                };

                history.insert(*prev, *idx);
                Some((idx + 1, num))
            }));

        i.nth(nth - 1).unwrap().1
    };

    // .inspect(|(idx, num)| {
    //     println!("{}: {}", idx, num);
    // });

    // let p1 = i.nth(2019).unwrap().1;
    // println!("{}", p1);
    aoc.submit_p1(get_nth(2020)).unwrap();
    aoc.submit_p2(get_nth(30_000_000)).unwrap();
}
