#!/usr/bin/env rustr

use std::{io::Read, mem::replace};

use aoc::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Reading {
    x: isize,
    y: isize,
    z: isize,
}

impl TryFrom<&str> for Reading {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, ()> {
        let [x, y, z]: [isize; 3] = s
            .split(',')
            .map(|n| n.parse().unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| ())?;
        Ok(Self { x, y, z })
    }
}

impl Reading {
    /// 90º turns on each plane.
    ///
    /// (1, 2, 3) rot (-2,  0,  0) → (-1, -2,  3)
    /// (1, 2, 3) rot (-1,  0,  0) → (-2,  1,  3)
    /// (1, 2, 3) rot ( 1,  0,  0) → ( 2, -1,  3)
    /// (1, 2, 3) rot ( 2,  0,  0) → (-1, -2,  3)
    /// (1, 2, 3) rot ( 3,  0,  0) → (-2,  1,  3)
    ///
    /// (1, 2, 3) rot ( 0,  1,  0) → ( 1, -3,  2)
    ///
    /// (1, 2, 3) rot ( 1,  1,  0) → (-3, -1,  2)
    /// (1, 2, 3) rot ( 1,  1,  0) → ( 2, -3, -1)
    fn rotate(self, (xy, yz, xz): (isize, isize, isize)) -> Reading {
        use std::ops::{Add, Rem};
        fn modulo<T: Rem<Output = T> + Add<Output = T> + TryFrom<usize>, const DENOM: usize>(
            n: T,
        ) -> T {
            // We'd like to use checked operations here but we'd have to pull
            // in `num-traits` to do so.
            let d: T = DENOM
                .try_into()
                .map_err(|_| "expected denominator in range of T")
                .unwrap();
            (n % d + d) % d
        }

        fn plane(rot: isize, un: &mut isize, deux: &mut isize) {
            let rot = modulo::<_, 4>(rot);

            // A kind of cute (but pretty wasteful) way:
            // for i in 0..rot {
            //     let (x, y) = (*un, *deux);
            //     *un = y;
            //     *deux = -x;
            // }

            // Flattened:
            let (x, y) = (*un, *deux);
            let (x, y) = match rot {
                0 => (x, y),
                1 => (y, -x),
                2 => (-x, -y),
                3 => (-y, x),
                _ => unreachable!(),
            };
            *un = x;
            *deux = y;
        }
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2021, 19);
    let inp = aoc.get_input();
    let readings: Vec<Vec<Reading>> = inp
        .split("\n\n")
        .map(|dump| {
            let beacons = dump.lines();
            assert!(beacons.next().unwrap().contains("scanner"));
            beacons.map(|r| r.try_into().unwrap()).collect()
        })
        .collect();

    let (horiz, depth) = commands
        .clone()
        .fold((0, 0), |pos, c: Command| c.apply(pos));
    aoc.submit_p1(horiz * depth).unwrap();

    let (horiz, depth, _) = commands.fold((0, 0, 0), |pos, c| c.apply_with_aim(pos));
    aoc.submit_p2(horiz * depth).unwrap();
}
