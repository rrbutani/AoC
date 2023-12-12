#!/usr/bin/env rustr

use aoc::*;

enum Command {
    Forward(usize),
    Down(usize),
    Up(usize),
}

impl TryFrom<&str> for Command {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, ()> {
        Ok(
            match s
                .split_once(" ")
                .and_then(|(dir, magnitude)| magnitude.parse::<usize>().ok().map(|m| (dir, m)))
                .ok_or(())?
            {
                ("forward", n) => Command::Forward(n),
                ("down", n) => Command::Down(n),
                ("up", n) => Command::Up(n),
                _ => return Err(()),
            },
        )
    }
}

impl Command {
    fn apply(&self, (horiz, depth): (usize, usize)) -> (usize, usize) {
        use Command::*;
        match self {
            Forward(n) => (horiz + n, depth),
            Down(n) => (horiz, depth + n),
            Up(n) => (horiz, depth - n),
        }
    }

    fn apply_with_aim(&self, (horiz, depth, aim): (usize, usize, usize)) -> (usize, usize, usize) {
        use Command::*;
        match self {
            Forward(n) => (horiz + n, depth + aim * n, aim),
            Down(n) => (horiz, depth, aim + n),
            Up(n) => (horiz, depth, aim - n),
        }
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2021, 2);
    let inp = aoc.get_input();
    let commands = inp.lines().map(|l| l.try_into().unwrap());

    let (horiz, depth) = commands
        .clone()
        .fold((0, 0), |pos, c: Command| c.apply(pos));
    aoc.submit_p1(horiz * depth).unwrap();

    let (horiz, depth, _) = commands.fold((0, 0, 0), |pos, c| c.apply_with_aim(pos));
    aoc.submit_p2(horiz * depth).unwrap();
}
