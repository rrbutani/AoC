use std::{collections::HashMap, ops::RangeInclusive};

use aoc::{itertools::MinMaxResult, *};
use owo_colors::OwoColorize;

macro_rules! d {
    ($($tt:tt)*) => {
        if DBG {
            eprintln!($($tt)*);
        }
    };
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
enum Dir {
    N,
    Ne,
    E,
    Se,
    S,
    Sw,
    W,
    Nw,
}

impl Dir {
    const ALL: [Self; Dir::Nw as usize + 1] = {
        use Dir::*;
        [N, Ne, E, Se, S, Sw, W, Nw]
    };

    fn adj(self) -> [Self; 3] {
        (-1..=1)
            .map(|offs| {
                Self::ALL[((offs + (self as isize)) + (Self::ALL.len() as isize)) as usize
                    % Self::ALL.len()]
            })
            .arr()
    }

    fn delta(self) -> Coord {
        use Dir::*;
        // (x, y)
        match self {
            N => (0, -1),
            Ne => (1, -1),
            E => (1, 0),
            Se => (1, 1),
            S => (0, 1),
            Sw => (-1, 1),
            W => (-1, 0),
            Nw => (-1, -1),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default)]
enum Cell {
    #[default]
    Empty,
    Elf,
}

impl TryFrom<char> for Cell {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use Cell::*;
        Ok(match value {
            '.' => Empty,
            '#' => Elf,
            _ => return Err(()),
        })
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Cell::*;
        match self {
            Empty => write!(f, "{}", '.'.dimmed()),
            Elf => write!(f, "{}", '#'.green()),
        }
    }
}

type Coord = (isize, isize); // (x, y)

#[derive(Debug, PartialEq, Eq, Clone)]
struct Map {
    grid: HashMap<Coord, Cell>,
}

impl FromStr for Map {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rows = s.lines().count();
        let cols = s.lines().map(|l| l.chars().count()).max().unwrap();
        let mut grid = HashMap::with_capacity(rows * cols);

        for (r, row) in s.lines().enumerate() {
            for (c, cell) in row.chars().enumerate() {
                grid.insert((c as _, r as _), cell.try_into().unwrap());
            }
        }

        Ok(Self { grid })
    }
}

impl Map {
    fn smallest_box(&self) -> (RangeInclusive<isize>, RangeInclusive<isize>) {
        let non_empty = self.elves();
        let MinMaxResult::MinMax(min_y, max_y) = non_empty.clone().map(|(_, y)| y).minmax() else {
            panic!();
        };
        let MinMaxResult::MinMax(min_x, max_x) = non_empty.map(|(x, _)| x).minmax() else {
            panic!();
        };

        ((min_x..=max_x), (min_y..=max_y))
    }

    fn elves(&self) -> impl Iterator<Item = Coord> + Clone + '_ {
        self.grid
            .iter()
            .filter(|(_, v)| !matches!(v, Cell::Empty))
            .map(|(k, _)| k)
            .cloned()
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x, y) = self.smallest_box();
        for y in y {
            for x in x.clone() {
                let cell = self.grid.get(&(x, y)).cloned().unwrap_or_default();
                Display::fmt(&cell, f)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl Map {
    // returns whether there was movement
    fn step(&mut self, strategies: impl Clone + Iterator<Item = Dir>) -> bool {
        let (x, y) = self.smallest_box();
        let mut moves = HashMap::<Coord, Result<Coord, ()>>::new(); // (new pos, Result<old pos, ()>)
                                                                    // Ok if only one elf is moving to `new pos`, else error

        // come up with the moves to apply
        for pos in x.cartesian_product(y) {
            let (x, y) = pos;

            // if we have an elf
            if self.grid.get(&pos) != Some(&Cell::Elf) {
                continue;
            }
            d!("Elf at {pos:?}");

            // that has at least one neighbour
            if !Dir::ALL
                .iter()
                .map(|d| d.delta())
                .map(|(dx, dy)| (x + dx, y + dy))
                .any(|pos| self.grid.get(&pos) == Some(&Cell::Elf))
            {
                continue;
            }

            // then try the strategies to see if one fits:
            for dir in strategies.clone() {
                // if we find one that fits:
                if dir
                    .adj()
                    .iter()
                    .map(|d| d.delta())
                    .map(|(dx, dy)| (x + dx, y + dy))
                    .all(|pos| self.grid.get(&pos) != Some(&Cell::Elf))
                {
                    // mark this as where the elf is going to move next:
                    let new_pos = {
                        let (dx, dy) = dir.delta();
                        (x + dx, y + dy)
                    };

                    // if this move is "taken", invalidate it
                    if let Some(old_pos) = moves.get_mut(&new_pos) {
                        *old_pos = Err(());
                    } else {
                        // otherwise insert
                        let res = moves.insert(new_pos, Ok(pos));
                        debug_assert_eq!(res, None);

                        d!("  - moving {dir:?} to {new_pos:?}");
                    }

                    // and we're done with this elf; move on to the next
                    break;
                } else {
                    d!("  - cannot move {dir:?}; chk: {:?}", dir.adj());
                }
            }

            // if no strategy fit we're staying put.
        }

        let mut movement = false;

        // apply the moves:
        for (new, old) in moves {
            if let Ok(old) = old {
                movement = true;
                let o = self.grid.get_mut(&old).unwrap();
                debug_assert_eq!(*o, Cell::Elf);
                *o = Cell::Empty;

                let n = self.grid.entry(new).or_default();
                debug_assert_eq!(*n, Cell::Empty);
                *n = Cell::Elf;
            }
        }

        movement
    }

    fn run<E>(&mut self, mut until: impl FnMut(bool, &Self, usize) -> Result<(), E>) -> E {
        let strats = {
            use Dir::*;
            [N, S, W, E]
        };
        let strats_it = strats.iter().cycle().cloned();

        let mut i = 0;
        loop {
            let strats = strats_it.clone().skip(i % strats.len()).take(strats.len());
            d!("step {i}; strats: {:?}", strats.clone().collect_vec());
            let res = self.step(strats);
            d!("{self}");

            if let Err(exit) = until(res, self, i) {
                break exit;
            }

            i += 1;
        }
    }

    fn score(&self) -> usize {
        let (x, y) = self.smallest_box();
        (x.count() * y.count()) - self.elves().count()
    }
}

const DBG: bool = false;

fn main() {
    let mut aoc = AdventOfCode::new(2022, 23);
    let inp = aoc.get_input();
    // let inp = include_str!("ex");
    // let inp = include_str!("ex_small");

    let map: Map = inp.parse().unwrap();
    d!("{map}");

    let p1 = {
        let mut map = map.clone();
        map.run(|_, map, i| {
            (0..10)
                .contains(&i)
                .then_some(())
                .ok_or_else(|| map.score())
        })
    };

    dbg!(p1);
    aoc.submit_p1(p1).unwrap();

    let p2 = {
        let mut map = map;
        map.run(|res, _, i| if !res { Err(i + 1) } else { Ok(()) })
    };
    dbg!(p2);
    aoc.submit_p2(p2).unwrap();
}
