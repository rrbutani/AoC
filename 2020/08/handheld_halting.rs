#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::collections::HashSet;
// use std::ops::Index;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
struct State {
    acc: i16,
    pc: u16,
}

trait Exec {
    fn exec(&self, state: &mut State);
}

fn parse_op<T: FromStr>(s: &str, op: &str) -> Result<T, ()> {
    s.strip_prefix(op)
        .ok_or(())
        .and_then(|rest| rest.trim().parse::<T>().map_err(|_| ()))
}

macro_rules! op {
    ([$s:literal] $nom:ident($inner:ty) => |$self:ident, $state:ident| $block:block) => {
        #[derive(Debug, Clone, PartialEq, Eq)] pub struct $nom($inner);
        impl Exec for $nom {
            fn exec(&$self, $state: &mut State) $block
        }

        impl FromStr for $nom {
            type Err = ();
            fn from_str(s: &str) -> Result<$nom, ()> { parse_op::<$inner>(s, $s).map($nom) }
        }

    };
}

macro_rules! insn_ty {
    ($($variant:ident)*) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum Insn {
            $($variant($variant),)*
        }

        impl Exec for Insn {
            fn exec(&self, s: &mut State) {
                match self {
                    $(Insn::$variant(inner) => inner.exec(s),)*
                }
            }
        }

        impl FromStr for Insn {
            type Err = ();

            fn from_str(s: &str) -> Result<Insn, ()> {
                $(
                    if let Ok(inner) = s.parse::<$variant>() {
                        return Ok(Insn::$variant(inner));
                    }
                )*

                return Err(())
            }
        }
    };
}

macro_rules! ops {
    ($([$s:literal] $nom:ident($inner:ty) => |$self:ident, $state:ident| $block:block),* $(,)?) => {
        $(op! { [$s] $nom($inner) => |$self, $state| $block })*

        insn_ty! { $($nom)* }
    }
}

ops! {
    ["nop"] Nop(i16) => |self, s| { s.pc += 1; },
    ["jmp"] Jmp(i16) => |self, s| { s.pc = s.pc.wrapping_add(self.0 as u16); },
    ["acc"] Acc(i16) => |self, s| { s.acc += self.0; s.pc += 1; },
}

#[derive(Debug)]
struct Finished;

// fn step(state: &mut State, program: &impl Index<usize, Output = Insn>) -> Option<Finished> {
fn step(state: &mut State, program: &Vec<Insn>) -> Option<Finished> {
    program[state.pc as usize].exec(state);

    if state.pc as usize >= program.len() {
        Some(Finished)
    } else {
        None
    }
}

fn run_until_loop(mut state: State, program: &Vec<Insn>) -> Result<State, (Finished, State)> {
    let mut history = HashSet::new();

    loop {
        if !history.insert(state.pc) {
            break Ok(state);
        }

        if let Some(Finished) = step(&mut state, &program) {
            break Err((Finished, state));
        }
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2020, 08);
    let input: String = aoc.get_input();
    let mut program: Vec<_> = input.lines().map(|i| i.parse::<Insn>().unwrap()).collect();

    let p1 = run_until_loop(State::default(), &program).unwrap().acc;
    let _ = aoc.submit_p1(p1);

    // use Insn::*;
    let nops_and_jmps: Vec<usize> = program
        .iter()
        .enumerate()
        .filter(|(_, insn)| matches!(insn, Insn::Jmp(_) | Insn::Nop(_)))
        .map(|(idx, _)| idx)
        .collect();

    let flip = |insn: &mut Insn| match *insn {
        Insn::Jmp(Jmp(i)) => *insn = Insn::Nop(Nop(i)),
        Insn::Nop(Nop(i)) => *insn = Insn::Jmp(Jmp(i)),
        _ => {}
    };

    let mut p2 = None;
    for idx in nops_and_jmps {
        flip(&mut program[idx]);

        if let Err((Finished, s)) = run_until_loop(State::default(), &program) {
            p2 = Some(s.acc);
        }

        flip(&mut program[idx]);
    }
    let _ = aoc.submit_p2(p2.unwrap());
}
