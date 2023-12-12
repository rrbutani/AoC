use std::{
    convert::identity,
    iter::once,
    ops::{Index, IndexMut},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
};

use aoc::*;
use owo_colors::OwoColorize;
use rayon::prelude::{ParallelBridge, ParallelIterator};

macro_rules! d {
    ($($tt:tt)*) => {
        if DBG {
            eprintln!($($tt)*);
        }
    };
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
enum Digit {
    DoubleMinus,
    Minus,
    Zero,
    One,
    Two,
}

impl From<Digit> for isize {
    fn from(value: Digit) -> Self {
        use Digit::*;
        match value {
            DoubleMinus => -2,
            Minus => -1,
            Zero => 0,
            One => 1,
            Two => 2,
        }
    }
}

impl Digit {
    // optional carry
    fn add(self, rhs: Self) -> (Option<Digit>, Digit) {
        let res = self.to::<isize>() + rhs.to::<isize>();
        use Digit::*;
        match res {
            -4 => (Some(Minus), One),
            -3 => (Some(Minus), Two),
            -2 => (None, DoubleMinus),
            -1 => (None, Minus),
            0 => (None, Zero),
            1 => (None, One),
            2 => (None, Two),
            3 => (Some(One), DoubleMinus),
            4 => (Some(One), Minus),
            _ => unreachable!(),
        }
    }
}

impl TryFrom<char> for Digit {
    type Error = char;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use Digit::*;
        Ok(match value {
            '=' => DoubleMinus,
            '-' => Minus,
            '0' => Zero,
            '1' => One,
            '2' => Two,
            c => return Err(c),
        })
    }
}

impl Display for Digit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Digit::*;
        write!(
            f,
            "{}",
            match self {
                DoubleMinus => '=',
                Minus => '-',
                Zero => '0',
                One => '1',
                Two => '2',
            }
        )
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
struct Snafu {
    digits: Vec<Digit>, // least significant at lower indexes; 0 = lsd
}

impl FromStr for Snafu {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Snafu {
            digits: s.chars().rev().map(|c| c.try_into().unwrap()).collect_vec(),
        })
    }
}

impl Display for Snafu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for d in self.digits.iter().rev() {
            Display::fmt(d, f)?;
        }

        Ok(())
    }
}

impl From<Snafu> for isize {
    #[allow(clippy::neg_multiply, clippy::identity_op)]
    fn from(s: Snafu) -> isize {
        use Digit::*;
        let mut out: isize = 0;
        let mut place = 1;
        for d in s.digits {
            match d {
                DoubleMinus => out += place * -2,
                Minus => out += place * -1,
                Zero => out += 0,
                One => out += place * 1,
                Two => out += place * 2,
            }

            place *= 5;
        }

        out
    }
}

impl From<usize> for Snafu {
    fn from(mut value: usize) -> Self {
        // base-5:
        //  a = 0, b = 1, c = 3, d = 4, e = 5
        //
        // snafu:
        //  a = -2, b = -1, c = 0, d = 1, e = 2

        // kind of rotated the number line in a weird way..
        use Digit::*;
        let to_digits = |v| match v {
            0 => vec![Zero],
            1 => vec![One],
            2 => vec![Two],
            3 => vec![/* One, */ DoubleMinus], // 5 - 2
            4 => vec![/* One, */ Minus],       // 5 - 1
            _ => unreachable!(),
        };

        if value == 0 {
            return Self {
                digits: to_digits(value),
            };
        }

        // let's just do it iteratively:
        let mut digits = vec![];

        while value != 0 {
            let r = value % 5;
            value /= 5;

            let extra = match r {
                0 | 1 | 2 => to_digits(r),
                3 | 4 => {
                    value += 1; // add 5 to offset these negative vals
                    to_digits(r)
                }
                _ => unreachable!(),
            };
            digits.extend(extra);
        }

        Self { digits }
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2022, 25);
    let inp = aoc.get_input();
    // let inp = include_str!("ex");

    let nums: Vec<Snafu> = inp.lines().map(|l| l.parse().unwrap()).collect_vec();
    // dbg!(&nums);

    let p1: Snafu = nums
        .iter()
        .cloned()
        .map(|n| n.to::<isize>())
        .sum::<isize>()
        .to::<usize>()
        .into();
    eprintln!("{p1}");
    aoc.submit_p1(p1).unwrap();
}
