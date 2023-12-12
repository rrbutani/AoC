#!/usr/bin/env rustr

use std::iter::Peekable;

use aoc::*;

#[derive(Debug, Clone, Copy)]
pub enum Invalid {
    InvalidClosingCharForPos { got: Closing, expected: Closing },
    UnexpectedChar(char),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Incomplete(Vec<Closing>);

use pair::{Closing, Opening};
mod pair {
    use super::*;

    pub type Opening = Pair<true>;
    pub type Closing = Pair<false>;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Pair<const OPENING: bool> {
        Square,
        Paren,
        Curly,
        Angle,
    }

    impl<const O: bool> Display for Pair<O> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.as_char())
        }
    }

    impl<const O: bool> Pair<O> {
        pub fn as_char(&self) -> char {
            use Pair::*;
            match O {
                true => match self {
                    Square => '[',
                    Paren => '(',
                    Curly => '{',
                    Angle => '<',
                },
                false => match self {
                    Square => ']',
                    Paren => ')',
                    Curly => '}',
                    Angle => '>',
                },
            }
        }
    }

    impl Closing {
        pub fn as_opening(self) -> Opening {
            use Pair::*;
            match self {
                Square => Square,
                Paren => Paren,
                Curly => Curly,
                Angle => Angle,
            }
        }

        pub fn value(&self) -> usize {
            use Pair::*;
            match self {
                Paren => 3,
                Square => 3 * 19,
                Curly => 3 * 19 * 21,
                Angle => 3 * 19 * 21 * 21,
            }
        }
    }

    impl Opening {
        pub fn as_closing(self) -> Closing {
            use Pair::*;
            match self {
                Square => Square,
                Paren => Paren,
                Curly => Curly,
                Angle => Angle,
            }
        }

        pub fn value(&self) -> usize {
            use Pair::*;
            match self {
                Paren => 1,
                Square => 2,
                Curly => 3,
                Angle => 4,
            }
        }
    }

    impl TryFrom<char> for Pair<true> {
        type Error = Invalid;

        fn try_from(value: char) -> Result<Opening, Invalid> {
            use Pair::*;
            Ok(match value {
                '[' => Square,
                '(' => Paren,
                '{' => Curly,
                '<' => Angle,
                other => return Err(Invalid::UnexpectedChar(other)),
            })
        }
    }

    impl TryFrom<char> for Pair<false> {
        type Error = Invalid;

        fn try_from(value: char) -> Result<Closing, Self::Error> {
            use Pair::*;
            Ok(match value {
                ']' => Square,
                ')' => Paren,
                '}' => Curly,
                '>' => Angle,
                other => return Err(Invalid::UnexpectedChar(other)),
            })
        }
    }
}

fn check(it: &mut Peekable<impl Iterator<Item = char>>) -> Result<Result<(), Incomplete>, Invalid> {
    let expect_next = |it: &mut Peekable<_>, p| {
        while let Some(&x) = {
            let n: Option<&char> = it.peek();
            n
        } {
            if x.try_to::<Closing>().is_ok() {
                break;
            } else if let Err(mut inc) = check(it)? {
                inc.0.push(p);
                return Ok(Err(inc));
            }
        }

        match it.next().map(|c: char| c.try_to::<Closing>()) {
            Some(Ok(x)) if x == p => Ok(Ok(())),
            Some(Ok(got) | Err(Invalid::InvalidClosingCharForPos { got, .. })) => {
                Err(Invalid::InvalidClosingCharForPos { got, expected: p })
            }
            Some(Err(err @ Invalid::UnexpectedChar(_))) => Err(err),
            None => Ok(Err(Incomplete(vec![p]))),
        }
    };

    if let Some(c) = it.next() {
        match expect_next(it, c.try_to::<Opening>()?.as_closing())? {
            Ok(()) => (),
            Err(inc) => return Ok(Err(inc)),
        }
    }

    Ok(Ok(()))
}

fn parse(it: &mut Peekable<impl Iterator<Item = char>>) -> Result<Result<(), Incomplete>, Invalid> {
    while it.peek().is_some() {
        match check(it) {
            Ok(Ok(())) => (),
            otherwise => return otherwise,
        }
    }

    Ok(Ok(()))
}

fn main() {
    let mut aoc = AdventOfCode::new(2021, 10);
    let inp = aoc.get_input();
    let inp = inp.lines().map(|l| parse(&mut l.trim().chars().peekable()));

    use Invalid::*;
    let p1 = inp
        .clone()
        .filter_map(|res| res.err())
        .map(|inv| match inv {
            InvalidClosingCharForPos { got, .. } => got,
            err => panic!("err: `{:?}`", err),
        })
        .map(|p| p.value())
        .sum::<usize>();
    aoc.submit_p1(p1).unwrap();

    let p2 = inp
        .filter_map(|res| res.ok())
        .filter_map(|res| res.err())
        .map(|inc| {
            inc.0
                .iter()
                .fold(0, |acc, c| acc * 5 + c.as_opening().value())
        })
        .median()
        .unwrap()
        .get();
    aoc.submit_p2(p2).unwrap();
}
