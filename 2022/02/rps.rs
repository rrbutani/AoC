#!/usr/bin/env rustr

use std::cmp::Ordering;

use aoc::*;

#[derive(Copy, Clone, PartialEq, Eq)]
enum Move {
    Rock = 1,
    Paper = 2,
    Scissors = 3,
}

impl Move {
    // can model as a rotate..
    fn gt(&self) -> Self {
        use Move::*;
        match self {
            Rock => Paper,
            Paper => Scissors,
            Scissors => Rock,
        }
    }

    fn le(&self) -> Self {
        use Move::*;
        match self {
            Rock => Scissors,
            Paper => Rock,
            Scissors => Paper,
        }
    }
}

impl Ord for Move {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for Move {
    fn partial_cmp(&self, other: &Move) -> Option<Ordering> {
        use Move::*;

        match (self, other) {
            (Paper, Rock) => Some(Ordering::Greater),
            (Rock, Scissors) => Some(Ordering::Greater),
            (Scissors, Paper) => Some(Ordering::Greater),
            (a, b) if a == b => Some(Ordering::Equal),
            (a, b) => b.partial_cmp(a).map(Ordering::reverse),
        }
    }
}

impl FromStr for Move {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        let res = match s {
            "A" | "X" => Move::Rock,
            "B" | "Y" => Move::Paper,
            "C" | "Z" => Move::Scissors,
            _ => return Err(()),
        };

        Ok(res)
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Outcome {
    Lose,
    Draw,
    Win,
}

impl FromStr for Outcome {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Outcome::*;
        Ok(match s {
            "X" => Lose,
            "Y" => Draw,
            "Z" => Win,
            _ => return Err(()),
        })
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2022, 2);
    let input = aoc.get_input();
    let games = input.lines().map(|l| l.split_once(' ').unwrap());

    let score = |(opp, you): (Move, Move)| {
        (you as u64)
            + match you.cmp(&opp) {
                Ordering::Greater => 6,
                Ordering::Equal => 3,
                _ => 0,
            }
    };
    let p1: u64 = games
        .clone()
        .map(|(opp, you)| (opp.parse().unwrap(), you.parse().unwrap()))
        .map(score)
        .sum();
    aoc.submit_p1(p1).unwrap();

    let p2: u64 = games
        .clone()
        .map(|(opp, you)| (opp.parse().unwrap(), you.parse().unwrap()))
        .map(|(opp, out): (Move, Outcome)| {
            let you = match out {
                Outcome::Lose => opp.le(),
                Outcome::Draw => opp,
                Outcome::Win => opp.gt(),
            };

            (opp, you)
        })
        .map(score)
        .sum();
    aoc.submit_p2(p2).unwrap();
}
