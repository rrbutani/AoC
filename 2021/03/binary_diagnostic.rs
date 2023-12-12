#!/usr/bin/env rustr

use aoc::*;

fn main() {
    let mut aoc = AdventOfCode::new(2021, 3);
    let inp = aoc.get_input();
    //     let inp = "00100
    // 11110
    // 10110
    // 10111
    // 10101
    // 01111
    // 00111
    // 11100
    // 10000
    // 11001
    // 00010
    // 01010";
    let report: Vec<_> = inp
        .lines()
        .map(|l| usize::from_str_radix(l, 2).unwrap())
        .collect();
    let bits = inp.lines().next().unwrap().len();

    let mut gamma = 0;
    let mut epsilon = 0;
    for b in 0..bits {
        if report.iter().filter(|i| (**i & (1 << b)) > 0).count() >= report.len() / 2 {
            gamma |= 1 << b;
        } else {
            epsilon |= 1 << b;
        }
    }
    aoc.submit_p1(gamma * epsilon).unwrap();

    /*     let mut oxygen = report.clone();
    let mut co2 = report.clone();
    for b in (0..bits).rev() {
        dbg!(oxygen
            .iter()
            .map(|i| format!("{:#07b}", i))
            .collect::<Vec<_>>());
        dbg!(co2
            .iter()
            .map(|i| format!("{:#07b}", i))
            .collect::<Vec<_>>());
        oxygen.filter_in_place(|v| *v & (1 << b) == gamma & (1 << b));
        co2.filter_in_place(|v| *v & (1 << b) == epsilon & (1 << b));
    }
    dbg!(oxygen
        .iter()
        .map(|i| format!("{:#07b}", i))
        .collect::<Vec<_>>());
    dbg!(co2
        .iter()
        .map(|i| format!("{:#07b}", i))
        .collect::<Vec<_>>());

    assert!(oxygen.len() == 1 && co2.len() == 1);
    dbg!(oxygen, co2); */
    // aoc.submit_p2(oxygen[0] * co2[0]).unwrap();
    let search = |pick_bit: fn(&mut dyn Iterator<Item = bool>) -> bool| {
        let mut report = report.clone();
        for b in (0..bits).rev() {
            if report.len() <= 1 {
                break;
            }
            let bit: bool = pick_bit(&mut report.iter().map(|i| (*i & (1 << b)) > 0));
            report.filter_in_place(|r| (*r & (1 << b)) >> b == (bit as usize));
        }

        assert_eq!(report.len(), 1);
        *report.last().unwrap()
    };
    let oxygen = search(|it| it.most_common().unwrap_or(true));
    let co2 = search(|it| it.least_common().unwrap_or(false));
    aoc.submit_p2(oxygen * co2).unwrap();
}
