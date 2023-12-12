#![allow(clippy::absurd_extreme_comparisons)]

use std::{collections::HashSet, mem};

use aoc::{itertools::MinMaxResult, *};
use derive_more as d;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, d::Add, d::Sub, d::AddAssign,
)]
struct Coord {
    x: isize,
    y: isize,
    z: isize,
}

impl FromStr for Coord {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, y, z) = s.split(',').tuple::<3>();
        Ok(Self {
            x: x.parse().map_err(|_| ())?,
            y: y.parse().map_err(|_| ())?,
            z: z.parse().map_err(|_| ())?,
        })
    }
}

impl Coord {
    fn adjacent(&self) -> impl Iterator<Item = Coord> + '_ {
        [
            Coord { x: 1, y: 0, z: 0 },
            Coord { x: 0, y: 1, z: 0 },
            Coord { x: 0, y: 0, z: 1 },
        ]
        .into_iter()
        .flat_map(|translate| [*self + translate, *self - translate].into_iter())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Droplet {
    cubes: HashSet<Coord>,
}

impl FromStr for Droplet {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Droplet {
            cubes: s.lines().map(|l| l.parse().unwrap()).collect(),
        })
    }
}

impl Droplet {
    // dumb approach: scan the 6 adjacent coords..
    pub fn surface_area(&self) -> usize {
        self.cubes
            .iter()
            .map(|c| c.adjacent().filter(|adj| !self.cubes.contains(adj)).count())
            .sum()
    }
}

fn p1(inp: &str) -> usize {
    let droplet: Droplet = inp.parse().unwrap();
    droplet.surface_area()
}

impl Droplet {
    // hmm
    //
    // let's do a floodfill.
    pub fn exterior_surface_area(&self) -> usize {
        let outside = {
            // naïve overestimate of the space the droplet occupies but it'll do
            let range = |func: fn(&Coord) -> isize| {
                let MinMaxResult::MinMax(min, max) = self.cubes
                    .iter()
                    .map(func)
                    .minmax()
                else {
                    unreachable!()
                };

                (min - 1)..=(max + 1)
            };

            let x = range(|c| c.x);
            let y = range(|c| c.y);
            let z = range(|c| c.z);

            let mut outside = HashSet::with_capacity(self.cubes.len());
            let mut queued = vec![Coord {
                x: *x.start(),
                y: *y.start(),
                z: *z.start(),
            }];
            outside.insert(queued[0]);
            let mut new = vec![];

            while !queued.is_empty() {
                for c in queued.drain(..) {
                    for also_outside in c
                        .adjacent()
                        .filter(|c| x.contains(&c.x) && y.contains(&c.y) && z.contains(&c.z))
                        .filter(|c| !self.cubes.contains(c))
                        .filter(|c| outside.insert(*c))
                    {
                        new.push(also_outside);
                    }
                }

                mem::swap(&mut queued, &mut new);
            }

            outside
        };

        // now, count all the cube sides that are _adjacent_ to 1 or more
        // outside coords:
        self.cubes
            .iter()
            .map(|c| c.adjacent().filter(|adj| outside.contains(adj)).count())
            .sum()
    }
}

fn p2(inp: &str) -> usize {
    let droplet: Droplet = inp.parse().unwrap();
    droplet.exterior_surface_area()
}

fn main() {
    let mut aoc = AdventOfCode::new(2022, 18);
    let inp = aoc.get_input();

    let p1 = p1(&inp);
    aoc.submit_p1(dbg!(p1)).unwrap();

    let p2 = p2(&inp);
    aoc.submit_p2(dbg!(p2)).unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn p1_small() {
        const EX_SMALL: &str = "1,1,1\n2,1,1\n";
        assert_eq!(super::p1(EX_SMALL), 10);
    }

    #[test]
    fn p1_ex() {
        assert_eq!(super::p1(include_str!("ex")), 64);
    }

    #[test]
    fn p2_ex() {
        assert_eq!(super::p2(include_str!("ex")), 58);
    }
}
