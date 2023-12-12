#!/usr/bin/env rustr

use std::collections::HashSet;

use aoc::*;

struct HeightMap {
    map: Vec<Vec<u8>>,
}

impl FromStr for HeightMap {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        Ok(Self {
            map: s
                .lines()
                .map(|l| {
                    l.chars()
                        .map(|c| c.to_digit(10).map(|u| u as u8))
                        .collect::<Option<_>>()
                })
                .collect::<Option<_>>()
                .ok_or(())?,
        })
    }
}

// NESW
type Adj = [Option<u8>; 4];

#[allow(non_snake_case)]
impl HeightMap {
    fn explore(&self, x: usize, y: usize, explored: &mut HashSet<(usize, usize)>) -> usize {
        if !explored.insert((x, y)) {
            return 0;
        }

        let height = self.map[x][y];
        return 1 + self
            .adj(x, y)
            .iter()
            .filter_map(|&c| c)
            .filter(|&(x, y)| {
                Some(self.map[x][y])
                    .filter(|&x| x > height)
                    .filter(|&x| x != 9)
                    .is_some()
            })
            .map(|(x, y)| self.explore(x, y, explored))
            .sum::<usize>();
    }

    fn adj(&self, x: usize, y: usize) -> [Option<(usize, usize)>; 4] {
        let (X, Y) = (self.map.len(), self.map[0].len());

        let n = y.checked_sub(1).map(|y| (x, y));
        let e = Some(x + 1).filter(|&x| x < X).map(|x| (x, y));
        let s = Some(y + 1).filter(|&y| y < Y).map(|y| (x, y));
        let w = x.checked_sub(1).map(|x| (x, y));

        [n, e, s, w]
    }

    fn adj_iter(&self) -> impl Iterator<Item = (Adj, u8, (usize, usize))> + '_ {
        let (X, Y) = (self.map.len(), self.map[0].len());
        (0..X).flat_map(move |x| {
            (0..Y).map(move |y| {
                let l = |c: Option<(usize, usize)>| c.map(|(x, y)| self.map[x][y]);

                (self.adj(x, y).map(l), self.map[x][y], (x, y))
            })
        })
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2021, 9);

    let map: HeightMap = aoc.get_input().parse().unwrap();

    let low_points = map
        .adj_iter()
        .filter(|&(adj, a, _)| adj.iter().all(|&n| a < n.unwrap_or(9)))
        .collect_vec();
    let p1 = low_points
        .iter()
        .map(|(_, a, _)| (a + 1) as usize)
        .sum::<usize>();
    aoc.submit_p1(p1).unwrap();

    let mut exp = HashSet::default();
    let p2 = low_points
        .iter()
        .map(|&(_, _, (x, y))| map.explore(x, y, &mut exp))
        .sorted()
        .rev()
        .take(3)
        .product::<usize>();
    aoc.submit_p2(p2).unwrap();
}
