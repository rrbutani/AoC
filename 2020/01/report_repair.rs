#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};
use std::collections::HashSet;

fn main() {
    let mut aoc = AdventOfCode::new(2020, 1);
    let set = aoc
        .get_input()
        .lines()
        .filter_map(|l| l.parse().ok())
        .collect();

    let p1 = pair(&set).map(|(a, b)| a * b).unwrap();
    let _ = aoc.submit_p1(p1);

    let p2 = triple(&set).map(|(a, b, c)| a * b * c).unwrap();
    let _ = aoc.submit_p2(p2);
}

fn pair(entries: &HashSet<u32>) -> Option<(u32, u32)> {
    // We could do this as we're collecting the values (i.e. with a fold, as we
    // go) but this is okay too.
    for val in entries.iter() {
        if let Some(complement) = 2020u32.checked_sub(*val) {
            if entries.contains(&complement) {
                return Some((*val, complement));
            }
        }
    }

    None
}

fn triple(entries: &HashSet<u32>) -> Option<(u32, u32, u32)> {
    // There's probably a better way to do this...
    for uno in entries.iter() {
        for dos in entries.iter().filter(|v| *v != uno) {
            if let Some(complement) = 2020u32.checked_sub(uno + dos) {
                if entries.contains(&complement) {
                    return Some((*uno, *dos, complement))
                }
            }
        }
    }

    None
}
