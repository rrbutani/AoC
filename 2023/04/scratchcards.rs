use std::collections::{HashMap, HashSet};

use aoc::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScratchCard {
    winning_nums: HashSet<usize>,
    nums: HashSet<usize>,
}

const INP: &str = "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83
Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11";

impl FromStr for ScratchCard {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, rest) = s.split_once(": ").unwrap();
        let (winning, have) = rest.split_once(" | ").unwrap();

        Ok(ScratchCard {
            winning_nums: winning
                .trim()
                .split_whitespace()
                .map(|i| i.parse().unwrap())
                .collect(),
            nums: have
                .trim()
                .split_whitespace()
                .map(|i| i.parse().unwrap())
                .collect(),
        })
    }
}

impl ScratchCard {
    fn num_matching(&self) -> usize {
        self.winning_nums.intersection(&self.nums).count()
    }

    fn points(&self) -> usize {
        (1 << self.num_matching()) / 2
    }
}

fn p2_old(cards: &[ScratchCard]) -> usize {
    let mut processed = HashMap::with_capacity(cards.len());
    let mut to_process = HashMap::with_capacity(cards.len());

    for (idx, card) in cards.iter().enumerate() {
        // let idx = idx + 1;
        let multiplier = to_process.remove(&idx).unwrap_or(1);
        // eprintln!("card {}: {} times", idx + 1, multiplier);
        for matching_child_idx in (idx + 1)..(idx + 1 + card.num_matching()) {
            // eprintln!("  - +card {}", matching_child_idx + 1);
            *to_process.entry(matching_child_idx).or_insert(1) += multiplier;
        }

        processed.insert(idx, multiplier);
    }

    processed.values().sum()
}

fn p2(cards: &[ScratchCard]) -> usize {
    let mut card_counts = cards.iter().map(|_| 1).collect_vec();

    for (idx, card) in cards.iter().enumerate() {
        let multiplier = card_counts[idx];
        for matching_child_idx in (idx + 1)..(idx + 1 + card.num_matching()) {
            card_counts[matching_child_idx] += multiplier;
        }
    }

    // if we go backwards, the "child" counts are already "frozen"...
    //
    // but it's really just the same thing? propagating the multiplier upwards
    // instead of downwards

    card_counts.iter().sum()
}

fn main() {
    let mut aoc = AdventOfCode::new(2023, 4);
    let inp = aoc.get_input();
    let scorecards = inp.lines().map_parse::<ScratchCard>().collect_vec();

    let p1: usize = scorecards.iter().map(|s| s.points()).sum();
    aoc.submit_p1(p1).unwrap();

    let p2 = p2(&scorecards);
    aoc.submit_p2(p2).unwrap();
}
