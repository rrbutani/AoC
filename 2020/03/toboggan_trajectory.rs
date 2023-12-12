#!/usr/bin/env rustr

// 8:23AM
// 8:38AM
// 8:41AM

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::convert::{TryFrom, TryInto};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Coord {
    Empty,
    Tree,
}

impl TryFrom<char> for Coord {
    type Error = ();

    fn try_from(c: char) -> Result<Self, ()> {
        Ok(match c {
            '.' => Coord::Empty,
            '#' => Coord::Tree,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Grid {
    inner: Vec<Vec<Coord>>,
}

impl FromStr for Grid {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        let inner = s
            .lines()
            .map(|l| l.chars().map(|c| c.try_into().unwrap()).collect::<Vec<_>>())
            .collect();

        Ok(Grid { inner })
    }
}

impl Grid {
    fn height(&self) -> usize {
        self.inner.len()
    }

    // actual width is infinite
    fn inner_width(&self) -> usize {
        self.inner[0].len()
    }

    fn traverse(&self, (mut x, mut y): (usize, usize), (dx, dy): (usize, usize)) -> usize {
        let mut trees_hit = 0;

        while y < self.height() {
            if let Coord::Tree = self.inner[y][x] {
                trees_hit += 1;
            }

            x = (x + dx) % self.inner_width();
            y += dy;
        }

        trees_hit
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2020, 03);
    let input: String = aoc.get_input();

    let grid: Grid = input.parse().unwrap();

    let p1 = grid.traverse((0, 0), (3, 1));
    let _ = aoc.submit_p1(p1);

    let p2: usize = [(1, 1), (3, 1), (5, 1), (7, 1), (1, 2)]
        .iter()
        .map(|slope| grid.traverse((0, 0), *slope))
        .fold(1, |a, b| a * b);
    let _ = aoc.submit_p2(p2);
}
