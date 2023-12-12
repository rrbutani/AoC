#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

#[derive(Debug, PartialEq, Eq)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl From<char> for Direction {
    fn from(c: char) -> Self {
        use Direction::*;
        match c {
            'U' => Up,
            'R' => Right,
            'D' => Down,
            'L' => Left,
            _ => panic!(),
        }
    }
}

impl Direction {
    fn diff(&self) -> (i8, i8) {
        use Direction::*;
        match *self {
            Up => (0, -1),
            Right => (1, 0),
            Down => (0, 1),
            Left => (-1, 0),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Keypad {
    x: u8,
    y: u8,
}

impl Default for Keypad {
    fn default() -> Self {
        Keypad { x: 2, y: 2 }
    }
}

impl Keypad {
    fn update(&mut self, dir: Direction) {
        let (dx, dy) = dir.diff();

        self.x = (self.x as i8 + dx).min(3).max(1) as u8;
        self.y = (self.y as i8 + dy).min(3).max(1) as u8;
    }

    fn val(&self) -> u8 {
        self.y * 3 + self.x - 3
    }
}

#[derive(Debug, PartialEq, Eq)]
struct EvilKeypad {
    x: i8,
    y: i8,
}

impl Default for EvilKeypad {
    fn default() -> Self {
        EvilKeypad { x: 0, y: 0 }
    }
}

impl EvilKeypad {
    fn update(&mut self, dir: Direction) {
        let (dx, dy) = dir.diff();

        let x = self.x + dx;
        let y = self.y + dy;

        if (x.abs() + y.abs()) > 2 {
            return;
        }

        self.x = x;
        self.y = y;
    }

    #[rustfmt::skip]
    fn val(&self) -> char {
        match (self.x, self.y) {
            (-2,  0) => '5',
            (-1, -1) => '2',
            (-1,  0) => '6',
            (-1,  1) => 'A',
            ( 0, -2) => '1',
            ( 0, -1) => '3',
            ( 0,  0) => '7',
            ( 0,  1) => 'B',
            ( 0,  2) => 'D',
            ( 1, -1) => '4',
            ( 1,  0) => '8',
            ( 1,  1) => 'C',
            ( 2,  0) => '9',
            _ => panic!(),
        }
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2016, 02);
    let input: String = aoc.get_input();

    let mut k = Keypad::default();
    let p1 = input
        .lines()
        .map(|l| {
            for insn in l.chars() {
                k.update(insn.into());
            }

            k.val()
        })
        .map(|d| format!("{}", d))
        .collect::<String>();

    let _ = aoc.submit_p1(p1);

    let mut k = EvilKeypad::default();
    let p2 = input
        .lines()
        .map(|l| {
            for insn in l.chars() {
                k.update(insn.into());
            }

            k.val()
        })
        .collect::<String>();
    let _ = aoc.submit_p2(p2);
}
