#!/usr/bin/env rustr

// Incidentally: https://twitter.com/Dev14e/status/1337634285944143872

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::convert::{TryFrom, TryInto};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Degrees {
    Ninety = 1,
    OneEighty = 2,
    TwoSeventy = 3,
}

impl FromStr for Degrees {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        use Degrees::*;
        Ok(match s {
            "90" => Ninety,
            "180" => OneEighty,
            "270" => TwoSeventy,
            _ => return Err(()),
        })
    }
}

impl TryFrom<usize> for Degrees {
    type Error = ();

    fn try_from(u: usize) -> Result<Self, ()> {
        use Degrees::*;
        Ok(match u {
            90 => Ninety,
            180 => OneEighty,
            270 => TwoSeventy,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum RotateDir {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NavInsn {
    Forward { steps: usize },
    Rotate { dir: RotateDir, degrees: Degrees },
    Move { dir: Direction, steps: usize },
}

impl FromStr for NavInsn {
    type Err = ();

    #[rustfmt::skip]
    fn from_str(s: &str) -> Result<Self, ()> {
        let n: usize = s[1..].parse().map_err(|_| ())?;

        use Direction::*;
        use NavInsn::*;
        use RotateDir::*;
        Ok(match s.trim().chars().next().unwrap() {
            'N' => Move { dir: North, steps: n },
            'S' => Move { dir: South, steps: n },
            'E' => Move { dir: East, steps: n },
            'W' => Move { dir: West, steps: n },
            'L' => Rotate { dir: Left, degrees: n.try_into()? },
            'R' => Rotate { dir: Right, degrees: n.try_into()? },
            'F' => Forward { steps: n },
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    fn pair(&self, mul: isize) -> (isize, isize) {
        use Direction::*;
        match self {
            North => (0, mul),
            East => (mul, 0),
            South => (0, -mul),
            West => (-mul, 0),
        }
    }

    fn rotate(&mut self, dir: RotateDir, degrees: Degrees) {
        use Direction::*;
        use RotateDir::*;
        const DIRS: [Direction; 4] = [North, East, South, West];

        let dir = match dir {
            Right => 1,
            Left => -1,
        };

        *self = DIRS[(*self as isize + 4 + dir * (degrees as isize)) as usize % DIRS.len()];
    }

    fn rotate_pos((x, y): (isize, isize), dir: RotateDir, degrees: Degrees) -> (isize, isize) {
        use Degrees::*;
        use RotateDir::*;
        match (dir, degrees) {
            (Right, Ninety) | (Left, TwoSeventy) => (y, -x),
            (Right, OneEighty) | (Left, OneEighty) => (-x, -y),
            (Right, TwoSeventy) | (Left, Ninety) => (-y, x),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Ship {
    pos: (isize, isize),
    dir: Direction,
}

impl Ship {
    fn step(&mut self, insn: NavInsn) {
        use NavInsn::*;

        let pos = &mut self.pos;
        let mut m = move |dir: Direction, steps: usize| {
            let (dx, dy) = dir.pair(steps.try_into().unwrap());
            pos.0 += dx;
            pos.1 += dy;
        };

        match insn {
            Forward { steps } => m(self.dir, steps),
            Rotate { dir, degrees } => self.dir.rotate(dir, degrees),
            Move { dir, steps } => m(dir, steps),
        }
    }

    fn abs_dist(&self) -> usize {
        self.pos.0.abs() as usize + self.pos.1.abs() as usize
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShipWithWaypoint {
    waypoint: (isize, isize),
    ship: (isize, isize),
    dir: Direction,
}

impl ShipWithWaypoint {
    fn step(&mut self, insn: NavInsn) {
        use NavInsn::*;

        let m = |pos: &mut (isize, isize),
                 dir: Option<Direction>,
                 steps: usize,
                 (x, y): (isize, isize)| {
            let steps = steps.try_into().unwrap();
            let (dx, dy) = dir.map(|d| d.pair(steps)).unwrap_or((steps, steps));
            pos.0 += dx * x;
            pos.1 += dy * y;
        };

        match insn {
            Forward { steps } => m(&mut self.ship, None, steps, self.waypoint),
            Rotate { dir, degrees } => {
                self.waypoint = Direction::rotate_pos(self.waypoint, dir, degrees)
            }
            Move { dir, steps } => m(&mut self.waypoint, Some(dir), steps, (1, 1)),
        }
    }

    fn abs_dist(&self) -> usize {
        self.ship.0.abs() as usize + self.ship.1.abs() as usize
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2020, 12);
    let input: String = aoc.get_input();
    let instructions = input.lines().map(|l| l.parse().unwrap());

    let mut s = Ship {
        pos: (0, 0),
        dir: Direction::East,
    };
    instructions.clone().for_each(|i| s.step(i));

    let p1 = s.abs_dist();
    let _ = aoc.submit_p1(p1);

    let mut s = ShipWithWaypoint {
        waypoint: (10, 1),
        ship: (0, 0),
        dir: Direction::East,
    };
    instructions.for_each(|i| s.step(i));

    let p2 = s.abs_dist();
    let _ = aoc.submit_p2(p2);
}
