#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new(2018, 08);
    let input: String = aoc.get_input();
    let input = input.lines().next().unwrap().split(" ").map(|s| s.parse::<u32>().unwrap());

    fn metadata_sum(i: &mut impl Iterator<Item=u32>) -> u32 {
        let (children, metadata) = (i.next().unwrap(), i.next().unwrap());

        (0..children).map(|_| metadata_sum(i)).sum::<u32>()
            + i.take(metadata as usize).sum::<u32>()
    }

    aoc.submit_p1(metadata_sum(&mut input.clone()));

    fn sum(i: &mut impl Iterator<Item=u32>) -> u32 {
        let (kid_count, metadata) = (i.next().unwrap(), i.next().unwrap());

        let kids: Vec<u32> = (0..kid_count).map(|_| sum(i)).collect();
        let metadata = i.take(metadata as usize);

        if kids.len() == 0 { metadata.sum() }
        else {
            metadata.filter(|i| 1 <= *i && *i <= kid_count).map(|i| kids[i as usize - 1]).sum()
        }
    }

    aoc.submit_p2(sum(&mut input.clone()));
}
