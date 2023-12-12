use aoc::{itertools::MinMaxResult, *};

use owo_colors::OwoColorize;
use std::{
    fmt::{self, Display},
    iter::once,
};
// started 4 hours, 45 minutes late..

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
struct Pos {
    x: usize,
    y: usize,
}

impl FromStr for Pos {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, y) = s.split_once(',').ok_or(())?;
        Ok(Self {
            x: x.parse().unwrap(),
            y: y.parse().unwrap(),
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct Path {
    segments: Vec<Pos>,
}

impl FromStr for Path {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Path {
            segments: s.split(" -> ").map(|p| p.parse().unwrap()).collect_vec(),
        })
    }
}

impl Path {
    fn iter(&self) -> impl Iterator<Item = Pos> + '_ {
        let start = self.segments[0];
        let rest = self.segments.windows(2).flat_map(|segs| {
            let [s @ Pos { x: sx, y: sy }, e @ Pos { x: ex, y: ey }] = *segs else {
                unreachable!();
            };
            // we're okay with the positions being out of order..
            if sx == ex {
                // vertical
                let (a, b) = if sy < ey { (sy, ey) } else { (ey, sy) };
                Box::new((a..=b).map(move |y| (sx, y))) as Box<dyn Iterator<Item = (usize, usize)>>
            } else if sy == ey {
                // horizontal
                let (a, b) = if sx < ex { (sx, ex) } else { (ex, sx) };
                Box::new((a..=b).map(move |x| (x, sy))) as Box<dyn Iterator<Item = (usize, usize)>>
            } else {
                panic!("can't go from {s:?} to {e:?}")
            }
        });
        once(start).chain(rest.skip(1).map(|(x, y)| Pos { x, y }))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
enum Cell {
    Rock,
    Floor,
    Sand,
}

impl Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Cell::*;
        match self {
            Rock => write!(f, "{}", "#".bold()),
            Floor => write!(f, "{}", "=".red()),
            Sand => write!(f, "{}", "o".yellow()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct Cave {
    grid: Vec<Vec<Option<Cell>>>, // [y][x]
    lowest_y: usize,
    min_x: usize,
    max_x: usize,
}

impl Cave {
    fn new<'p>(paths: impl Iterator<Item = &'p Path> + Clone) -> Self {
        let it = || paths.clone().flat_map(|p| p.iter());
        let MinMaxResult::MinMax(min_x, max_x) = it().map(|Pos { x, .. }| x).minmax() else {
            panic!()
        };
        let MinMaxResult::MinMax(_, max_y) = it().map(|Pos { y, .. }| y).minmax() else {
            panic!()
        };
        let max_x = max_x.max(500);

        let (x, y) = (max_x + 500, max_y + 5); // fudge factor

        let mut grid = vec![vec![None; x + 1]; y + 1];

        for Pos { x, y } in it() {
            grid[y][x] = Some(Cell::Rock)
        }

        Self {
            grid,
            lowest_y: max_y,
            min_x,
            max_x,
        }
    }
}

impl Display for Cave {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (skip_x, lim) = if f.alternate() {
            let ext = self.lowest_y;
            dbg!((self.min_x - ext, self.max_x + ext))
        } else {
            (self.min_x.saturating_sub(1), self.max_x.saturating_add(1))
        };
        writeln!(f, "{}", skip_x)?;
        writeln!(f, "v")?;

        for row in &self.grid {
            write!(f, "  ")?;
            for (_, cell) in row
                .iter()
                .enumerate()
                .skip(skip_x)
                .filter(|(x, _)| *x < lim)
            {
                if let Some(c) = cell {
                    write!(f, "{c}")?;
                } else {
                    write!(f, "{}", '.'.dimmed())?;
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl Cave {
    // `Ok` if came to rest, `Err` otherwise (or if `starting_at` is already
    // occupied)..
    fn place(&mut self, starting_at: Pos) -> Result<Pos, ()> {
        let Pos { mut x, mut y } = starting_at;
        if self.grid[y][x].is_some() {
            return Err(());
        }

        'outer: loop {
            if y > self.lowest_y {
                return Err(());
            }

            const MOVES: &[(isize, isize)] = &[
                /* (∆y, ∆x) */
                (1, 0),  // one step down, if we can:
                (1, -1), // down and to the left
                (1, 1),  // down and to the right
            ];

            for (dy, dx) in MOVES {
                let (ny, nx) = ((y as isize + dy) as usize, (x as isize + dx) as usize);

                if self.grid[ny][nx].is_none() {
                    y = ny;
                    x = nx;
                    continue 'outer;
                }
            }

            // if we couldn't move in any of the directions, we've settled:
            self.grid[y][x] = Some(Cell::Sand);
            break Ok(Pos { x, y });
        }
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2022, 14);
    let inp = aoc.get_input();
    // let inp = include_str!("ex");

    let paths = inp
        .lines()
        .map(|l| l.parse::<Path>().unwrap())
        .collect_vec();
    let mut cave = Cave::new(paths.iter());

    let p1 = (1..)
        .take_while(|_| {
            // eprintln!("{cave}");
            cave.place(Pos { x: 500, y: 0 }).is_ok()
        })
        .last()
        .unwrap();
    eprintln!("{cave}");
    aoc.submit_p1(dbg!(p1)).unwrap();

    // Now, place the floor and continue..
    for cell in &mut cave.grid[cave.lowest_y + 2] {
        *cell = Some(Cell::Floor);
    }
    cave.lowest_y += 2;
    let p2 = (p1 + 1..)
        .take_while(|_| {
            // eprintln!("{cave}");
            cave.place(Pos { x: 500, y: 0 }).is_ok()
        })
        .last()
        .unwrap();
    eprintln!("{cave:#}");
    aoc.submit_p2(dbg!(p2)).unwrap();

    // TODO: can do dfs instead

    // let pairs: Pairs = inp.parse().unwrap();
    // println!("{pairs}");

    // let p1 = pairs.score();
    // aoc.submit_p1(dbg!(p1)).unwrap();

    // let p2 = pairs.decoder_key();
    // aoc.submit_p2(dbg!(p2)).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs() -> Pairs {
        include_str!("ex").parse().unwrap()
    }

    #[test]
    fn p1() {
        assert_eq!(pairs().score(), 13);
    }

    #[test]
    fn p2() {
        assert_eq!(pairs().decoder_key(), 140);
    }

    // #[test]
    // fn p2() {
    //     assert_eq!(super::p2(&ex(), false), 2713310158);
    // }
}
