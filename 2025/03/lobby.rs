use std::cmp::Reverse;

use aoc::{AdventOfCode, Grid};

// O(B ^ N)
//   - where `B` is the length of the bank
fn highest_n_joltages<const N: usize>(mut bank: &[u8]) -> [u8; N] {
    debug_assert!(bank.len() >= N, "{N}; {bank:?} ({})", bank.len());

    let mut out = [None; N];
    for (slot_idx, slot) in out.iter_mut().enumerate() {
        // picking a joltage in the bank to fill this slot limits what joltages we can fill the
        // remaining slots with: we can only use joltages further down in the bank
        //
        // so, in order to ensure that we don't "run out" of joltages to fill our slots, limit the
        // part of the bank that we search:
        let remaining = N - slot_idx - 1;
        let search = &bank[..bank.len() - remaining];

        // prefer joltages further left (at lower indexes): this allows considering more joltages
        // to satisfy future slots:
        let (joltage_idx, &joltage) = search
            .iter()
            .enumerate()
            .max_by_key(|&(idx, &j)| (j, Reverse(idx)))
            .unwrap();

        *slot = Some(joltage);

        // constrain future searches:
        bank = &bank[joltage_idx + 1..];
    }

    out.map(Option::unwrap)
}

fn collective_joltage(bank_joltages: impl AsRef<[u8]>) -> usize {
    let mut out = 0;
    for &j in bank_joltages.as_ref() {
        out *= 10;
        out += j as usize;
    }
    out
}

fn main() {
    let mut aoc = AdventOfCode::new(2025, 03);
    let banks: Grid<u8> = aoc.get_input().parse().unwrap();

    let joltage = banks
        .iter()
        .map(|bank| highest_n_joltages::<2>(bank))
        .map(collective_joltage)
        .sum::<usize>();
    _ = aoc.submit_p1(joltage);

    let joltage = banks
        .iter()
        .map(|bank| highest_n_joltages::<12>(bank))
        .map(collective_joltage)
        .sum::<usize>();
    _ = aoc.submit_p2(joltage);
}
