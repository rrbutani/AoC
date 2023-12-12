#!/usr/bin/env rustr

/*#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::fmt::{self, Display};
use std::ops::RangeInclusive;
use std::str::FromStr;

fn main() {
    let mut aoc = AdventOfCode::new(2020, 2);
    let input: String = aoc.get_input();

    let list: Vec<Line<'_>> = input
        .lines()
        .map(TryInto::try_into)
        .collect::<Result<_, _>>()
        .unwrap();

    let p1 = list.iter().filter(|l| l.valid_for_count()).count();
    let _ = aoc.submit_p1(p1);

    let p2 = list.iter().filter(|l| l.valid_for_positions()).count();
    let _ = aoc.submit_p2(p2);
}

#[derive(Debug)]
struct StrError {
    msg: Cow<'static, str>,
}

impl Display for StrError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "Error: {}", self.msg.as_ref())
    }
}

impl Error for StrError {}

macro_rules! err {
    ($msg:literal) => { StrError { msg: Cow::Borrowed($msg) } };
    ($($msg:tt)*) => {
        StrError { msg: Cow::Owned(format!($($msg)*)) }
    };
}

#[derive(Debug)]
struct Line<'a> {
    password: &'a str,
    policy: Policy,
}

impl Line<'_> {
    fn valid_for_count(&self) -> bool {
        let count = self
            .password
            .chars()
            .filter(|c| c == &self.policy.required_char)
            .count() as u32;

        self.policy.required_count.contains(&count)
    }

    fn valid_for_positions(&self) -> bool {
        let (lower_idx, upper_idx) = self.policy.required_count.clone().into_inner();

        let mut pass = self.password.chars();

        let lower = pass
            .nth(lower_idx as usize - 1)
            .map(|c| c == self.policy.required_char)
            .unwrap_or(false);
        let upper = pass
            .nth((upper_idx - lower_idx) as usize - 1)
            .map(|c| c == self.policy.required_char)
            .unwrap_or(false);

        lower ^ upper
    }
}

// {Policy}: {password}
impl<'a> TryFrom<&'a str> for Line<'a> {
    type Error = Box<dyn Error>;

    fn try_from(s: &'a str) -> Result<Self, Box<dyn Error>> {
        let mut iter = s.split(":");

        let policy = iter.next().ok_or(err!("Expected a colon!"))?.parse()?;

        let password = iter
            .next()
            .ok_or(err!("Expected a password!"))
            .map(|p| p.trim_start())?;

        if let Some(_) = iter.next() {
            Err(err!("Expected no spaces in the password!").into())
        } else {
            Ok(Line { password, policy })
        }
    }
}

#[derive(Debug)]
struct Policy {
    required_char: char,
    required_count: RangeInclusive<u32>,
}

// {lower}-{upper} {char}
impl FromStr for Policy {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split(" ");

        let mut range_iter = iter
            .next()
            .ok_or(err!("Expected a range at the start!"))?
            .split("-");

        let lower = range_iter
            .next()
            .ok_or(err!("Expected a lower bound!"))?
            .parse()?;
        let upper = range_iter
            .next()
            .ok_or(err!("Expected an upper bound!"))?
            .parse()?;

        if let Some(_) = range_iter.next() {
            return Err(err!("Expected two numbers in a range!").into());
        }

        let mut required_iter = iter
            .next()
            .ok_or(err!("Expected a character for the range!"))?
            .chars();

        let required_char = required_iter
            .next()
            .ok_or(err!("Expected a character for the range!"))?;

        if let Some(_) = required_iter.next() {
            return Err(err!("Expected a *single() character for the range!").into());
        }

        Ok(Policy {
            required_char,
            required_count: lower..=upper,
        })
    }
}
*/

use aoc::{friends::*, AdventOfCode};

fn main() {
    let mut aoc = AdventOfCode::new(2020, 2);
    let input: String = aoc.get_input();

    let list: Vec<_> = input
        .lines()
        .map(|l| sf::scan_fmt!(l, "{}-{} {}: {}", usize, usize, char, String))
        .collect::<Result<_, _>>()
        .unwrap();

    let p1 = list
        .iter()
        .filter(|(l, u, c, pass)| (*l..=*u).contains(&pass.matches(*c).count()))
        .count();
    let _ = aoc.submit_p1(dbg!(p1));


    let p2 = list
        .iter()
        .filter(|(l, u, c, pass)| pass[*l..].starts_with(*c) ^ pass[*u..].starts_with(*c))
        .count();
    let _ = aoc.submit_p2(dbg!(p2));
}
