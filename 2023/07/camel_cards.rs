use std::{
    cmp::{Ordering, Reverse},
    collections::HashMap,
    iter,
};

use aoc::{iterator_map_ext::IterMapExt, AdventOfCode, FromStr, Itertools};

const INP: &str = "32T3K 765
T55J5 684
KK677 28
KTJJT 220
QQQJA 483";

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
enum CamelCard<const JOKER: bool = false> {
    _2, _3, _4, _5, _6, _7, _8, _9, T, J, Q, K, A,
}

impl<const J: bool> PartialOrd for CamelCard<J> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut this = *self as u8 + 1;
        let mut that = *other as u8 + 1;

        if J {
            if *self == Self::J {
                this = 0;
            }
            if *other == Self::J {
                that = 0;
            }
        }

        this.partial_cmp(&that)
    }
}

impl<const J: bool> Ord for CamelCard<J> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<const J: bool> TryFrom<char> for CamelCard<J> {
    type Error = char;

    #[rustfmt::skip]
    fn try_from(c: char) -> Result<Self, Self::Error> {
        use CamelCard::*;
        Ok(match c {
            '2' => _2, '3' => _3, '4' => _4, '5' => _5, '6' => _6, '7' => _7,
            '8' => _8, '9' => _9, 'T' => T, 'J' => J, 'Q' => Q, 'K' => K,
            'A' => A, other => return Err(other),
        })
    }
}

impl<const J: bool> FromStr for CamelCard<J> {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 1 {
            if let Ok(c) = s.chars().next().unwrap().try_into() {
                return Ok(c);
            }
        }

        Err(s.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Hand<const JOKER: bool> {
    cards: [CamelCard<JOKER>; 5],
    freq: [usize; 5],
    bid: usize,
}

impl<const J: bool> Hand<J> {
    fn make_frequency_list<const N: usize>(cards: &[CamelCard<J>; N]) -> [usize; N] {
        let mut hm = HashMap::with_capacity(cards.len());

        let mut joker_count = 0;
        for c in cards {
            if J && c == &CamelCard::J {
                joker_count += 1;
            } else {
                *hm.entry(c).or_default() += 1;
            }
        }

        let mut frequencies = hm
            .values()
            .copied()
            .chain(iter::repeat(0))
            .take(5)
            .collect_vec();
        frequencies.sort_by_key(|&x| Reverse(x));

        if J {
            frequencies[0] += joker_count;
        }

        frequencies.try_into().unwrap()
    }
}

impl<const J: bool> FromStr for Hand<J> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (cards, bid) = s.split_once(" ").unwrap();
        let cards = cards
            .chars()
            .map(|c| c.try_into().unwrap())
            .collect_vec()
            .try_into()
            .unwrap();

        Ok(Hand {
            freq: Self::make_frequency_list(&cards),
            cards,
            bid: bid.parse().unwrap(),
        })
    }
}

impl<const J: bool> PartialOrd for Hand<J> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // type first:
        match self.freq.partial_cmp(&other.freq) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        }

        // individual cards as the tiebreaker:
        self.cards.partial_cmp(&other.cards)
    }
}

impl<const J: bool> Ord for Hand<J> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn winnings<const JOKER: bool>(hands: impl Iterator<Item = Hand<JOKER>>) -> usize {
    let mut hands = hands.collect_vec();
    hands.sort();

    hands
        .iter()
        .enumerate()
        // .inspect(|(rank, hand)| eprintln!("{}: {:?}", rank + 1, hand.cards))
        .map(|(rank, hand)| (rank + 1) * hand.bid)
        .sum()
}

fn main() {
    let mut aoc = AdventOfCode::new(2023, 7);
    let inp = aoc.get_input();
    // let inp = INP;

    let hands = inp.lines().map_parse();
    let p1 = winnings::<false>(hands);
    aoc.submit_p1(p1).unwrap();

    let hands = inp.lines().map_parse();
    let p2 = winnings::<true>(hands);
    aoc.submit_p2(p2).unwrap();
}
