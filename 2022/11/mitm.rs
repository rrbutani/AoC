use aoc::*;

use owo_colors::OwoColorize;
use std::{fmt, mem};

type I = usize;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Operation {
    Mul(I),
    Squared,
    Add(I),
    Doubled,
}

impl FromStr for Operation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rest = s.strip_prefix("  Operation: new = old ").ok_or(())?;
        let (op, rhs) = rest.split_once(' ').ok_or(())?;
        let rhs = match rhs {
            "old" => None,
            n => Some(n.parse().unwrap()),
        };
        use Operation::*;
        Ok(match (rhs, op) {
            (Some(num), "+") => Add(num),
            (None, "+") => Doubled,
            (Some(num), "*") => Mul(num),
            (None, "*") => Squared,
            _ => return Err(()),
        })
    }
}

impl Operation {
    fn apply(&self, old: I) -> I {
        use Operation::*;
        match self {
            Mul(rhs) => old * rhs,
            Squared => old * old,
            Add(rhs) => old + rhs,
            Doubled => old + old,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Test {
    Div(I),
}

impl FromStr for Test {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rest = s.strip_prefix("  Test: ").ok_or(())?;
        let (op, rhs) = rest.split_once(" by ").ok_or(())?;
        let rhs = rhs.parse().unwrap();
        use Test::*;
        Ok(match op {
            "divisible" => Div(rhs),
            _ => return Err(()),
        })
    }
}

impl Test {
    fn test(&self, val: I) -> bool {
        use Test::*;
        match self {
            Div(rhs) => (val % rhs) == 0,
        }
    }

    fn cond<T>((test, if_true, if_false): (Self, T, T), val: I) -> T {
        if test.test(val) {
            if_true
        } else {
            if_false
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Monkey {
    items: Vec<I>,
    op: Operation,
    test: (Test, /* true */ usize, /* false */ usize),
    inspected_count: usize,
}

impl FromStr for Monkey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.lines();
        let _ = s.next().unwrap().strip_prefix("Monkey ").unwrap();

        let items = s
            .next()
            .unwrap()
            .strip_prefix("  Starting items: ")
            .unwrap()
            .split(", ")
            .map(|i| i.parse().unwrap())
            .collect();
        let op = s.next().unwrap().parse().unwrap();
        let cond = s.next().unwrap().parse().unwrap();
        let if_true = s
            .next()
            .unwrap()
            .strip_prefix("    If true: throw to monkey ")
            .unwrap()
            .parse()
            .unwrap();
        let if_false = s
            .next()
            .unwrap()
            .strip_prefix("    If false: throw to monkey ")
            .unwrap()
            .parse()
            .unwrap();

        assert_eq!(s.next(), None);
        Ok(Monkey {
            items,
            op,
            test: (cond, if_true, if_false),
            inspected_count: 0,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct State {
    monkeys: Vec<Monkey>,
    lcm: I,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, m) in self.monkeys.iter().enumerate() {
            if f.alternate() {
                writeln!(f, "Monkey {i} inspected items {} times.", m.inspected_count)?;
            } else {
                writeln!(f, "Monkey {i}: {:?}", m.items)?;
            }
        }

        Ok(())
    }
}

impl FromStr for State {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let monkeys = s.split("\n\n").map(|m| m.parse().unwrap()).collect_vec();
        let lcm = monkeys
            .iter()
            .map(|m: &Monkey| {
                let Test::Div(d) = m.test.0;
                d
            })
            .product();
        Ok(State { monkeys, lcm })
    }
}

impl State {
    fn round(&mut self, dbg: bool, div_by: I) {
        macro_rules! d {
            ($($t:tt)*) => {
                if dbg {
                    eprintln!($($t)*);
                }
            };
        }

        for idx in 0..self.monkeys.len() {
            d!("Monkey {idx}:");
            for item in mem::take(&mut self.monkeys[idx].items) {
                d!("  Monkey inspects an item with a worry level of {item}.");
                let level = self.monkeys[idx].op.apply(item);
                d!("    Worry level: {item} -> {level}.");
                let level = level / div_by;
                let level = level % self.lcm;
                d!("    Monkey gets bored with item. Worry level is divided by 3 to {level}.");
                let res = Test::cond(self.monkeys[idx].test, level);
                if dbg {
                    let cond = self.monkeys[idx].test.0.test(level);
                    let Test::Div(div_by) = self.monkeys[idx].test.0;
                    let a;
                    let b;
                    d!(
                        "    Current worry level {} divisible by {div_by}.",
                        if cond {
                            a = "is".green();
                            &a as &dyn fmt::Display
                        } else {
                            b = "is not".red();
                            &b as _
                        }
                    );
                    d!("    Item with worry level {level} is thrown to monkey {res}.");
                }
                self.monkeys[res].items.push(level);
                self.monkeys[idx].inspected_count += 1;
            }
        }

        d!("\n{}", self);
    }

    fn monkey_business_level(&self) -> usize {
        self.monkeys
            .iter()
            .map(|m| m.inspected_count)
            .sorted()
            .rev()
            .take(2)
            .product()
    }
}

fn p1(s: &State, dbg: bool) -> usize {
    let mut s = s.clone();
    for _ in 0..20 {
        s.round(dbg, 3);
    }

    s.monkey_business_level()
}

fn p2(s: &State, dbg: bool) -> usize {
    let mut s = s.clone();
    for i in 0..10_000 {
        s.round(false, 1);
        if dbg && i % 1000 == 0 {
            eprintln!("== After round {i} ==");
            eprintln!("{s:#}");
        }
    }
    s.monkey_business_level()
}

fn main() {
    let mut aoc = AdventOfCode::new(2022, 11);
    let inp = aoc.get_input();
    // let inp = include_str!("ex");
    let s: State = inp.parse().unwrap();

    let dbg = false;
    aoc.submit_p1(dbg!(p1(&s, dbg))).unwrap();
    aoc.submit_p2(dbg!(p2(&s, dbg))).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ex() -> State {
        State::from_str(include_str!("ex")).unwrap()
    }

    #[test]
    fn p1() {
        assert_eq!(super::p1(&ex(), false), 10605);
    }

    #[test]
    fn p2() {
        assert_eq!(super::p2(&ex(), false), 2713310158);
    }
}
