use std::iter;

use aoc::*;

const INP: &str = "\
two1nine
eightwothree
abcone2threexyz
xtwone3four
4nineeightseven2
zoneight234
7pqrstsixteen
";

fn digits_in_string(mut remaining: &str) -> impl Iterator<Item = usize> + Clone + '_ {
    iter::from_fn(move || {
        const DIGITS: &[&str] = &[
            "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
        ];

        // note: always only jump forward one character; letter digits are
        // allowed to overlap!
        let mut chars = remaining.chars();
        loop {
            if let Some(first_char) = chars.next() {
                if first_char.is_numeric() {
                    remaining = chars.as_str();
                    return Some((first_char as u8 - b'0') as _);
                }

                for (i, digit) in DIGITS.iter().enumerate().map(|(i, v)| (i + 1, v)) {
                    if let Some(_) = remaining.strip_prefix(digit) {
                        remaining = chars.as_str();
                        return Some(i);
                    }
                }

                remaining = chars.as_str();
                continue;
            } else {
                // string is empty; we're done
                return None;
            }
        }
    })
}

fn main() {
    let mut aoc = AdventOfCode::new(2023, 1);
    let inp = aoc.get_input();
    let p1 = calibration_value_sum(&inp, |l| {
        l.chars()
            .filter(|c| c.is_numeric())
            .map(|c| (c as u8 - b'0') as _)
    });
    aoc.submit_p1(p1).unwrap();

    let p2 = calibration_value_sum(&inp, digits_in_string);
    // let p2 = calibration_value_sum(INP, digits_in_string);
    // dbg!(p2);
    aoc.submit_p2(p2).unwrap();
}

fn calibration_value_sum<'l, It: Iterator<Item = usize> + Clone + 'l>(
    lines: &'l str,
    nums_for_line: impl Fn(&'l str) -> It,
) -> usize {
    lines
        .lines()
        .map(|l| {
            let digits = nums_for_line(l);
            let first = digits.clone().next().unwrap();
            let last = digits.last().unwrap();

            assert!(first < 10);
            assert!(last < 10);

            let out = first * 10 + last;
            // eprintln!("{l}\n  - {first}, {last} ({out})");

            out
        })
        .sum()
}

#[test]
fn parse_digits() {
    let x = digits_in_string("eightwo").collect::<Vec<_>>();
    assert!(matches!(x[..], [8, 2]));
}
