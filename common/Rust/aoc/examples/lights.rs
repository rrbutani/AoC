
use std::{ops::RangeInclusive, str::FromStr};

#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Command {
    On,
    Off,
    Toggle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Instruction {
    cmd: Command,
    range: (RangeInclusive<usize>, RangeInclusive<usize>),
}

impl FromStr for Instruction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        let mut iter = s.split(" through ");

        let cmd = iter.next().unwrap();
        let end = iter.next().unwrap();

        let range_parse = |s: &str| {
            let mut iter = s.split(",");
            let start = iter.next().unwrap();
            let end = iter.next().unwrap();

            Ok((start.parse().map_err(|_| ())?, end.parse().map_err(|_| ())?))
        };

        let (cmd, start) = if let Some(start) = cmd.strip_prefix("turn on ") {
            (Command::On, start)
        } else if let Some(start) = cmd.strip_prefix("turn off ") {
            (Command::Off, start)
        } else if let Some(start) = cmd.strip_prefix("toggle ") {
            (Command::Toggle, start)
        } else {
            return Err(());
        };

        let (x1, y1) = range_parse(start)?;
        let (x2, y2) = range_parse(end)?;

        assert!(x2 >= x1 && y2 >= y1);

        Ok(Self {
            cmd,
            range: (x1..=x2, y1..=y2),
        })
    }
}

trait Exec<Ty> {
    fn exec(&self, element: &mut Ty);
}

impl Exec<bool> for Command {
    fn exec(&self, light: &mut bool) {
        use Command::*;
        *light = match self {
            On => true,
            Off => false,
            Toggle => !*light,
        };
    }
}

impl Exec<u8> for Command {
    fn exec(&self, light: &mut u8) {
        use Command::*;
        *light = match self {
            On => *light + 1,
            Off => light.checked_sub(1).unwrap_or(0),
            Toggle => *light + 2,
        };
    }
}

impl<T> Exec<Vec<Vec<T>>> for Instruction where Command: Exec<T> {
    fn exec(&self, grid: &mut Vec<Vec<T>>) {
        for x in self.range.0.clone() {
            for y in self.range.1.clone() {
                self.cmd.exec(&mut grid[y][x])
            }
        }
    }
}

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new(2015, 6);
    let input: String = aoc.get_input();

    let instructions = input.lines().map(|l| l.parse::<Instruction>().unwrap());
    let mut grid = vec![vec![false; 100_000]; 100_000];

    for insn in instructions.clone() {
        insn.exec(&mut grid);
    }

    let p1: usize = grid.iter().map(|r| r.iter().filter(|l| **l).count()).sum();
    aoc.submit_p1(p1);

    let mut grid = vec![vec![0u8; 100_000]; 100_000];
    for insn in instructions {
        insn.exec(&mut grid);
    }

    let p2: usize = grid.iter().map(|r| (*r).iter().map(|l| *l as usize).sum::<usize>()).sum();
    aoc.submit_p2(p2);
}
