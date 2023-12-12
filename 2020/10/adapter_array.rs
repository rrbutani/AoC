#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::collections::HashMap;

fn main() {
    let mut aoc = AdventOfCode::new(2020, 10);
    let input: String = aoc.get_input();

    //     let input = "16
    // 10
    // 15
    // 5
    // 1
    // 11
    // 7
    // 19
    // 6
    // 12
    // 4";

    // jolts! :-D

    let joltages = input.lines().map(|j| j.parse::<u16>().unwrap());
    let device_rating = joltages.clone().max().unwrap() + 3;

    let mut joltages: Vec<_> = joltages.collect();
    joltages.push(device_rating);
    joltages.sort();

    let (ones, threes) = joltages
        .iter()
        .scan(0, |j, r| {
            let diff = r - *j;
            *j = *r;

            // dbg!((j, diff));

            Some(diff)
        })
        // .inspect(|d| println!("diff: {}", d))
        .fold((0, 0), |(one, three), diff| match diff {
            1 => (one + 1, three),
            2 | 0 => (one, three),
            3 => (one, three + 1),
            x => panic!("{}", x),
        });

    let p1 = ones * threes;
    println!("{}", p1);
    let _ = aoc.submit_p1(p1);

    let mut computed: HashMap<u16, usize> = HashMap::new();

    computed.insert(0, 1);
    for joltage in joltages.iter() {
        let paths = (1..=3)
            .filter_map(|diff| joltage.checked_sub(diff).and_then(|j| computed.get(&j)))
            .sum();

        assert!(computed.insert(*joltage, paths).is_none());
    }

    // fn count_distinct_paths(to: u16, computed: &mut HashMap<u16, usize>) -> usize {}

    let p2 = *computed.get(&device_rating).unwrap();
    println!("{}", p2);
    let _ = aoc.submit_p2(p2);
}
