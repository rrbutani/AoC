#!/usr/bin/env rustr

use std::collections::VecDeque;
use arrayvec::ArrayVec;
#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};

fn count(pots: VecDeque<bool>, num_negatives: usize) -> isize {
    pots.iter().enumerate().filter(|(_, v)| **v).map(|(i, _)| i as isize - num_negatives as isize).sum::<isize>()
}

fn rounds(initial: VecDeque<bool>, notes: &Vec<([bool; 5], bool)>, num_rounds: usize, print: bool) -> isize {
    if print { print!("{:2}: ", 0); for c in initial.iter() { print!("{}", if *c == true { '#' } else { '.' }); } print!("\n"); }
    let _prin = |k: &[bool]| k.iter().map(|b| if *b == true { '#' } else { '.' }).collect::<String>();

    let mut staging = initial.clone();
    let mut added_below_zero: usize = 0;

    let mut diff_count = 0;
    let mut last_diff = 0;
    let mut last_count = 0;

    for i in 1..=num_rounds {
        let mut state = Vec::with_capacity(staging.len() + 8);

        // Assuming no "....." patterns, we've got to pad by 4 on both sides:
        (0..4).for_each(|_| state.push(false));
        state.extend(staging.iter());
        (0..4).for_each(|_| state.push(false));
        // for _ in 0..4 { state.push_front(false); state.push_back(false); }
        for i in 0..staging.len() { staging[i] = false; }

        let mut round_adjustment = 0;

        // And now, go!
        for (idx, v) in state.windows(5).enumerate() {
            // We start with [-4, 0] which maps to idx -2.
            let idx: isize = idx as isize - 2;
            // println!("Window {} at {}", idx, prin(v));
            for (p, s) in notes.iter() {
                let idx_adj = idx + round_adjustment;

                if v == p {
                    if idx_adj >= staging.len() as isize {
                        while idx_adj >= staging.len() as isize {
                            staging.push_back(false);
                        }
                    }
                    if idx_adj < 0 {
                        added_below_zero += idx_adj.abs() as usize;
                        // If, for example, we add two elements at the start
                        // representing 2 negative numbers, all indexes from
                        // now will have to be adjusted accordingly. i.e. to
                        // access 0, we need to use index 2!
                        round_adjustment += idx_adj.abs();

                        for _ in 0..idx_adj.abs() {
                            staging.push_front(false);
                        }
                    }
                    // Use round_adjustment in case it changed (negative idx)
                    staging[(idx + round_adjustment) as usize] = *s;
                }
            }
        }
        if print { print!("{:2}: ", i + 1); for c in staging.iter() { print!("{}", if *c == true { '#' } else { '.' }); } print!("\n"); }

        // We keep growing, so this didn't work:

        // if let Some(idx) = seen.insert(staging.clone(), i) {
        //     // If seen this state before, calculate the cycle time:
        //     println!("Yo, we found something! {} == {}", idx, i);
        //     let cycle = i - idx; // This many cycles from now, we'll end up back here.

        //     let additional_rounds = (num_rounds - i) % cycle;
        //     let (v, a) = rounds(staging, notes, additional_rounds, print);

        //     return (v, a + added_below_zero);
        // }

        // So, let's track counts instead:
        let curr = count(staging.clone(), added_below_zero);
        let diff = curr - last_count;

        if diff == last_diff {
            diff_count += 1;
        } else {
            last_diff = diff;
            diff_count = 0;
        }

        last_count = curr;

        // Arbitary threshold for stabilization:
        if diff_count > 100 {
            return (num_rounds - i) as isize * last_diff + last_count;
        }
    }

    count(staging, added_below_zero)
}

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new(2018, 12);
    let input: String = aoc.get_input();
    let mut input = input.lines();

    let initial = scan_fmt!(input.next().unwrap(), "initial state: {}", String).unwrap();
    let initial = initial.chars().map(|c| c == '#').collect::<VecDeque<bool>>();

    let notes = input.filter_map(|l| {
        let (s, r) = scan_fmt!(l, "{[#.]} => {[#.]}", String, char);

        Some((s?, r?))
    }).filter(|(s, _)| s.len() == 5)
    .map(|(s, r)|
        (s.chars().map(|c| c == '#').collect::<ArrayVec<[bool; 5]>>(), r == '#')
    ).filter_map(|(s, r)| {
        Some((s.into_inner().ok()?, r))
    }).collect::<Vec<([bool; 5], bool)>>();


    aoc.submit_p1(rounds(initial.clone(), &notes, 20, false));

    aoc.submit_p2(rounds(initial.clone(), &notes, 50_000_000_000, false));
}
