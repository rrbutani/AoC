#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Deck {
    inner: VecDeque<u8>,
}

// impl Display for Deck {
//     fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
//         self.inner.map(|c| format_args!())
//     }
// }

impl FromStr for Deck {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        Ok(Self {
            inner: s.lines().skip(1).map(|l| l.parse().unwrap()).collect(),
        })
    }
}

impl Deck {
    fn score(&self) -> usize {
        self.inner
            .iter()
            .rev()
            .enumerate()
            .map(|(idx, val)| ((idx + 1) * (*val as usize)))
            .sum()
    }

    fn top(&self) -> Option<u8> {
        self.inner.front().copied()
    }

    fn pop(&mut self) -> u8 {
        self.inner.pop_front().unwrap()
    }

    fn push(&mut self, card: u8) {
        let top = self.inner.pop_front().unwrap();
        // assert!(top > card);
        self.inner.push_back(top);
        self.inner.push_back(card);
    }

    fn sub_deck(&self, len: u8) -> Deck {
        Deck {
            inner: self
                .inner
                .iter()
                .copied()
                .skip(1)
                .take(len as usize)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Combat {
    p1: Deck,
    p2: Deck,
}

impl Combat {
    fn new(p1: Deck, p2: Deck) -> Self {
        Self { p1, p2 }
    }

    // fn play(Self { mut p1, mut p2 }: Self) -> Deck {
    fn play(self) -> Deck {
        let Self { mut p1, mut p2 } = self;
        loop {
            let c1 = if let Some(c1) = p1.top() {
                c1
            } else {
                break p2;
            };

            let c2 = if let Some(c2) = p2.top() {
                c2
            } else {
                break p1;
            };

            // The cards should never be equal!
            if c1 > c2 {
                p1.push(p2.pop())
            } else {
                p2.push(p1.pop())
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Player {
    One,
    Two,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RecursiveCombat {
    previous_games: HashMap<(Deck, Deck), (Player, Deck)>,
    p1: Deck,
    p2: Deck,
}

impl RecursiveCombat {
    fn new(p1: Deck, p2: Deck) -> Self {
        Self {
            previous_games: HashMap::new(),
            p1,
            p2,
        }
    }

    fn play(mut self) -> (Player, Deck) {
        self.play_a_game(self.p1.clone(), self.p2.clone())
    }

    fn play_a_game(&mut self, mut p1: Deck, mut p2: Deck) -> (Player, Deck) {
        // println!("\nplaying: {:?}, {:?}", p1, p2);

        let mut previous_rounds = HashSet::new();
        let (orig_p1, orig_p2) = (p1.clone(), p2.clone());

        let ret = loop {
            // println!("   + step: {:?}, {:?}", p1, p2);

            // If we've played this game already, don't play it again:
            if let Some(res) = self.previous_games.get(&(p1.clone(), p2.clone())) {
                break res.clone();
            }

            // No loops!
            if !previous_rounds.insert((p1.clone(), p2.clone())) {
                break (Player::One, p1);
            }

            // If a player is out of cards, the other player wins.
            let c1 = if let Some(c1) = p1.top() {
                c1
            } else {
                break (Player::Two, p2);
            };

            let c2 = if let Some(c2) = p2.top() {
                c2
            } else {
                break (Player::One, p1);
            };

            #[allow(clippy::clippy::collapsible_if)]
            let winner = if p1.inner.len() > c1 as usize && p2.inner.len() > c2 as usize {
                // recurse, if we can:
                self.play_a_game(p1.sub_deck(c1), p2.sub_deck(c2)).0
            } else {
                // otherwise, highest card wins:
                if c1 > c2 {
                    Player::One
                } else {
                    Player::Two
                }
            };

            match winner {
                Player::One => p1.push(p2.pop()),
                Player::Two => p2.push(p1.pop()),
            }

            // if self.previous_games.contains(&(p1.clone(), p2.clone())) {
            //     break (Player::One, p1);
            // }
        };

        // Remember this game, in case it was new.
        self.previous_games.insert((orig_p1, orig_p2), ret.clone());

        // println!("ret: {:?}\n\n", ret);
        ret
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2020, 22);
    let input: String = aoc.get_input();
    let mut input = input.split("\n\n");

    let d1: Deck = input.next().unwrap().parse().unwrap();
    let d2: Deck = input.next().unwrap().parse().unwrap();

    let p1 = Combat::new(d1.clone(), d2.clone()).play().score();
    println!("{}", p1);
    let _ = aoc.submit_p1(p1);

    let p2 = RecursiveCombat::new(d1, d2).play().1.score();
    println!("{}", p2);
    let _ = aoc.submit_p2(p2);
}
