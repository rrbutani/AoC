#!/usr/bin/env rustr

use aoc::*;
use owo_colors::OwoColorize;

use std::fmt::{self, Debug};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Board {
    grid: [[(u8, bool); 5]; 5],
    done: bool,
}

impl FromStr for Board {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            grid: s
                .lines()
                .map(|l| {
                    l.trim()
                        .split_whitespace()
                        .map_parse()
                        .map(|n| (n, false))
                        .arr()
                })
                .arr(),
            done: false,
        })
    }
}

impl Display for Board {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in self.grid.iter() {
            let mut first = true;
            for (cell, called) in line.iter() {
                if !first {
                    write!(fmt, " ")?;
                }

                if *called && fmt.alternate() {
                    write!(fmt, "{:2}", cell.bold())?;
                } else {
                    write!(fmt, "{:2}", cell)?;
                }
                first = false;
            }
            writeln!(fmt)?;
        }

        Ok(())
    }
}

impl Board {
    pub fn iter(&self) -> impl Iterator<Item = &(u8, bool)> + '_ {
        self.grid.iter().flat_map(|l| l.iter())
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut (u8, bool)> + '_ {
        self.grid.iter_mut().flat_map(|l| l.iter_mut())
    }

    fn set(&mut self, n: u8) {
        self.iter_mut().for_each(|(cell, state)| {
            if *cell == n {
                *state = true;
            }
        })
    }

    pub fn unmarked(&self) -> impl Iterator<Item = u8> + '_ {
        self.iter().filter(|(_, s)| !*s).map(|(c, _)| *c)
    }

    fn check(&self) -> bool {
        self.grid.iter().any(|l| l.iter().all(|(_, s)| *s))
            || (0..5).any(|col| (0..5).all(|row| self.grid[row][col].1))
    }

    pub fn set_and_check(&mut self, n: u8) -> bool {
        if self.done {
            return false;
        }
        self.set(n);
        if self.check() {
            self.done = true;
            true
        } else {
            false
        }
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2021, 4);
    let inp = aoc.get_input();
    let (numbers, boards) = inp.split_once("\n\n").unwrap();
    let numbers = numbers.split(',').map_parse::<u8>().collect_vec();
    let mut boards = boards.split("\n\n").map_parse::<Board>().collect_vec();

    let mut p1 = None;
    let mut p2 = None;
    let mut set_once = |score| {
        if p1.is_none() {
            p1 = Some(score);
        }
    };
    let mut set_last = |score| p2 = Some(score);

    for n in numbers {
        for b in boards.iter_mut() {
            if b.set_and_check(n) {
                let score = b.unmarked().map(|n| n as usize).sum::<usize>() * n as usize;
                set_once(score);
                set_last(score);
            }
        }
    }

    aoc.submit_p1(p1.unwrap()).unwrap();
    aoc.submit_p2(p2.unwrap()).unwrap();
}
