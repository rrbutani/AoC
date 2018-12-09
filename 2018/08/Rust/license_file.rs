#!/usr/bin/env rustr
extern crate aoc;

#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new_with_year(2018, 08);
    let input: String = aoc.get_input();
    let input = input.lines().next().unwrap();
    let input = input.split(" ").map(|s| s.parse::<u32>().unwrap());

    fn metadata_sum(i: &mut impl Iterator<Item=u32>) -> u32 {
        let (children, metadata) = (i.next().unwrap(), i.next().unwrap());

        let sum: u32 = (0..children).map(|_| metadata_sum(i)).sum();

        sum + (0..metadata).map(|_| i.next().unwrap()).sum::<u32>()
    }

    aoc.submit_p1(metadata_sum(&mut input.clone()));

    fn sum(i: &mut impl Iterator<Item=u32>) -> u32 {
        let (children, metadata) = (i.next().unwrap(), i.next().unwrap());

        let children: Vec<u32> = (0..children).map(|_| sum(i)).collect();
        let metadata: Vec<u32> = (0..metadata).map(|_| i.next().unwrap()).collect();

        if children.len() == 0 {
            metadata.iter().sum()
        } else {
            metadata.iter().filter(|i| 1 <= **i && **i as usize <= children.len()).map(|i| children[*i as usize - 1]).sum()
        }
    }

    aoc.submit_p2(sum(&mut input.clone()));
}
