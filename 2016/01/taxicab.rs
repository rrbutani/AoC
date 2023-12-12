#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::collections::HashSet;
use std::iter::repeat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rotate {
    Left,
    Right,
}

impl Direction {
    fn update_dir(&mut self, rot: Rotate) {
        use Direction::*;

        *self = match rot {
            Rotate::Right => match *self {
                North => East,
                East => South,
                South => West,
                West => North,
            },
            Rotate::Left => match *self {
                North => West,
                East => North,
                South => East,
                West => South,
            },
        }
    }

    #[inline]
    fn update_pos(
        &self,
        mag: isize,
        (x, y): &mut (isize, isize),
    ) -> impl Iterator<Item = (isize, isize)> {
        use Direction::*;

        let (dx, dy) = match *self {
            North => (0, mag),
            East => (mag, 0),
            South => (0, -mag),
            West => (-mag, 0),
        };

        let (old_x, old_y) = (*x, *y);

        *x += dx;
        *y += dy;

        let x = (old_x.min(*x)..=old_x.max(*x))
            .filter(move |x| *x != old_x)
            .chain(repeat(old_x));
        let y = (old_y.min(*y)..=old_y.max(*y))
            .filter(move |y| *y != old_y)
            .chain(repeat(old_y));

        x.zip(y).take(mag as usize)
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2016, 01);
    let input: String = aoc.get_input();

    let directions = input
        .split(",")
        .filter_map(|i| sf::scan_fmt!(i.trim(), "{[RL]}{}", char, isize).ok())
        .map(|(c, m)| {
            (
                if c == 'R' {
                    Rotate::Right
                } else {
                    Rotate::Left
                },
                m,
            )
        });

    let mut hist = HashSet::new();
    let mut p2 = None;

    let mut pos = (0, 0);
    hist.insert(pos);

    let mut dir = Direction::North;
    for (r, m) in directions {
        dir.update_dir(r);

        // println!("{:?}: {}", r, m);
        for pos in dir.update_pos(m as isize, &mut pos) {
            // println!("{:?}", pos);
            if !hist.insert(pos) && p2.is_none() {
                p2 = Some(pos);
            }
        }
    }

    let p1 = pos.0.abs() + pos.1.abs();
    let _ = aoc.submit_p1(p1);

    let p2 = p2.unwrap().0.abs() + p2.unwrap().1.abs();
    let _ = aoc.submit_p2(p2);
}
