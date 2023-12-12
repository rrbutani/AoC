#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

fn reduce(polymer: &mut Vec<u8>) -> &mut Vec<u8> {
    let mut i = 0;

    // This is unabashedly imperative and more than a little gross but I'm
    // racing the clock here..

    while polymer.len() > 0 && i < polymer.len() - 1 {
        if (polymer[i] as i16 - polymer[i + 1] as i16).abs() == 32 {
            polymer.remove(i);
            polymer.remove(i);
            i = i.saturating_sub(1);
        } else {
            i += 1;
        }
    }

    polymer
}

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new(2018, 5);
    let input: String = aoc.get_input();
    let input = input.lines().next().unwrap().get(0..).unwrap().bytes();

    let mut polymer = input.clone().collect::<Vec<u8>>();

    let p1 = reduce(&mut polymer).len();
    aoc.submit_p1(p1);

    let polymer = input.clone().collect::<Vec<u8>>();
    let p2 = ((b'A')..=(b'Z'))
        .map(|c| {
            let mut polymer = polymer.clone();
            polymer.retain(|i| *i != (c as u8) && *i != (c as u8 + 32));

            reduce(&mut polymer).len()
        })
        .min()
        .unwrap();

    aoc.submit_p2(p2);
}
