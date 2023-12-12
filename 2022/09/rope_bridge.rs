use std::{collections::HashSet, ops::Range};

use aoc::*;
use owo_colors::{OwoColorize, Rgb};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[rustfmt::skip]
enum Dir {
    Up, Right, Down, Left,
}

impl Dir {
    fn delta(self) -> (isize, isize) {
        use Dir::*;
        match self {
            // (x, y)
            Up => (0, 1),
            Right => (1, 0),
            Down => (0, -1),
            Left => (-1, 0),
        }
    }
}

impl FromStr for Dir {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Dir::*;
        match s {
            "U" | "u" => Ok(Up),
            "R" | "r" => Ok(Right),
            "D" | "d" => Ok(Down),
            "L" | "l" => Ok(Left),
            _ => Err(()),
        }
    }
}

type Coord = (isize, isize);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct State<const KNOTS: usize = 2> {
    knots: [Coord; KNOTS],
}

impl<const K: usize> Default for State<K>
where
    [Coord; K]: Default,
{
    fn default() -> Self {
        Self {
            knots: Default::default(),
        }
    }
}

impl<const K: usize> State<K> {
    fn step_knot(prev: Coord, curr: &mut Coord) {
        let (px, py) = prev;
        let (cx, cy) = curr;
        let (dx, dy) = (px - *cx, py - *cy);

        if !(dx.abs() >= 2 || dy.abs() >= 2) {
            return;
        }

        // can only move 1 cell in each dir in a step:
        let bound_to_one = |x: isize| x.abs().min(1) * x.signum();

        *cx += bound_to_one(dx);
        *cy += bound_to_one(dy);
    }

    fn step_head(&mut self, dir: Dir) {
        let (x, y) = &mut self.knots[0];
        let (dx, dy) = dir.delta();

        *x += dx;
        *y += dy;

        for (prev, curr) in (0..K).tuple_windows() {
            Self::step_knot(self.knots[prev], &mut self.knots[curr])
        }
    }
}

struct StateViewer<'s, const K: usize> {
    s: &'s State<K>,
    x: Range<isize>,
    y: Range<isize>,
}

impl<const K: usize> State<K> {
    fn display(&self, x: Range<isize>, y: Range<isize>) -> impl Display + '_ {
        StateViewer { s: self, x, y }
    }
}

impl<const K: usize> Display for StateViewer<'_, K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in self.y.clone().rev() {
            for x in self.x.clone() {
                const RAINBOW: &[Rgb] = &[
                    Rgb(100, 000, 200),
                    Rgb(200, 000, 100),
                    Rgb(200, 100, 000),
                    Rgb(100, 200, 000),
                    Rgb(000, 200, 000),
                    Rgb(000, 100, 100),
                    Rgb(000, 000, 200),
                ];

                let count = self.s.knots.iter().filter(|c| **c == (x, y)).count();
                let knot = self
                    .s
                    .knots
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| **c == (x, y))
                    .map(|(idx, _)| {
                        let char = match idx {
                            0 => 'H',
                            x @ 1..=9 => (b'0' + (x as u8)) as char,
                            x => (b'a' + (x as u8) - 9) as char,
                        };

                        let color = RAINBOW[idx % RAINBOW.len()];

                        (char, color)
                    })
                    .next();

                if let Some((c, color)) = knot {
                    if count > 1 {
                        write!(f, "{}", c.color(color).bold())?
                    } else {
                        write!(f, "{}", c.color(color))?
                    }
                } else {
                    write!(f, "{}", '.'.dimmed())?
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl<const K: usize> State<K> {
    fn drive(
        &mut self,
        moves: impl Iterator<Item = (Dir, usize)>,
        mut func: impl FnMut(&State<K>),
    ) {
        func(self);

        for (d, n) in moves {
            for _ in 0..n {
                self.step_head(d);

                func(self)
            }
        }
    }
}

const EX: &str = "R 4
U 4
L 3
D 1
R 4
D 1
L 5
R 2";

fn main() {
    let mut aoc = AdventOfCode::new(2022, 9);
    let inp = aoc.get_input();
    // let inp = EX;

    let moves = inp.lines().map(|l| {
        let (dir, num) = l.split_once(' ').unwrap();
        (dir.parse().unwrap(), num.parse().unwrap())
    });

    let p1 = {
        let mut state = State::<2>::default();
        let mut visited = HashSet::new();
        state.drive(moves.clone(), |s| {
            // eprintln!("\n{}", s.display(0..6, 0..5));
            visited.insert(*s.knots.last().unwrap());
        });

        visited.len()
    };
    aoc.submit_p1(dbg!(p1)).unwrap();

    let p2 = {
        let mut state = State::<10>::default();
        let mut visited = HashSet::new();
        state.drive(moves, |s| {
            // eprintln!("\n{}", s.display(-12..14, -6..15));
            visited.insert(*s.knots.last().unwrap());
        });

        visited.len()
    };
    aoc.submit_p2(dbg!(p2)).unwrap();
}
