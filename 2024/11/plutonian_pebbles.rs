use aoc::*;
use std::{mem, num::NonZeroUsize};

use fxhash::FxHashMap as HashMap;

/*
*/

const fn next_pow_ten(n: usize) -> usize {
    match n {
        0 => 1,
        _ => 10 * next_pow_ten(n / 10),
    }
}
const _: () = assert!(next_pow_ten(10) == 100);

fn split_digits_even(n: usize) -> Option<(usize, usize)> {
    let t = next_pow_ten(n);
    let pow = t.trailing_zeros();
    if pow % 2 == 0 {
        let half = 10usize.pow(pow / 2);
        Some((n / half, n % half))
    } else {
        None
    }
}

fn blink(stone: usize) -> SmallVec<[usize; 2]> {
    let one = |n| SmallVec::from_buf_and_len([n, 0], 1);

    match stone {
        0 => one(1),
        n => {
            if let Some((l, r)) = split_digits_even(n) {
                SmallVec::from_buf([l, r])
            } else {
                one(n * 2024)
            }
        }
    }
}

fn step_with_counts(stones: &HashMap<usize, usize>, out: &mut HashMap<usize, usize>) {
    for (&stone, count) in stones {
        for next in blink(stone) {
            *out.entry(next).or_default() += count;
        }
    }
}

/*
fn split_digits_even(n: NonZeroUsize) -> Option<(NonZeroUsize, usize)> {
    let (mut c, mut r) = (0, n.get());
    while r != 0 {
        r /= 10;
        c += 1;
    }

    if c % 2 != 0 {
        return None;
    }

    // `log10(usize::MAX) // 2` -> 19
    const POW_10: &[usize] = &[
        0,
        10,
        100,
        1000,
        10000,
        100000,
        1000000,
        10000000,
        100000000,
        1000000000,
        10000000000,
    ];

    let pow = unsafe { POW_10.get_unchecked(c / 2) };
    Some((unsafe { NonZeroUsize::new_unchecked(n.get() / pow) }, n.get() % pow))
}

// or: `(Option<NonZeroUsize>, usize)`
fn blink(stone: usize) -> (Option<NonZeroUsize>, usize) {
    match NonZeroUsize::new(stone) {
        Some(s) => {
            if let Some((l, r)) = split_digits_even(s) {
                (Some(l), r)
            } else {
                (None, stone * 2024)
            }
        }
        None => (None, 1),
    }
}

fn step_with_counts(stones: &HashMap<usize, usize>, out: &mut HashMap<usize, usize>) {
    for (&stone, count) in stones {
        let (l, r) = blink(stone);
        if let Some(l) = l {
            *out.entry(l.get()).or_default() += count;
        }
        *out.entry(r).or_default() += count;
    }
}
*/

fn blink_n(stones: &[usize], n: usize) -> usize {
    let mut counts: HashMap<_, _> = stones.iter().map(|&s| (s, 1)).collect();
    let mut next = HashMap::default();

    for _ in 0..n {
        next.clear();
        step_with_counts(&counts, &mut next);
        mem::swap(&mut counts, &mut next);
    }

    counts.into_iter().map(|(_, c)| c).sum()
}

fn main() {
    let mut aoc = AdventOfCode::new(2024, 11);
    let inp = aoc.get_input().split_whitespace().map_parse().collect_vec();

    _ = aoc.submit_p1(blink_n(&inp, 25));
    _ = aoc.submit_p2(blink_n(&inp, 75));
}
