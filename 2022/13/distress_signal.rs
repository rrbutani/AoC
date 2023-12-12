use aoc::*;

use owo_colors::OwoColorize;
use std::{
    cmp::Ordering,
    fmt::{self, Display},
    iter::{once, Peekable},
};

// started 54 minutes late..

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Message {
    Single(u8),
    List(Vec<Message>),
}

impl Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Message::*;
        match self {
            Single(x) => write!(f, "{x}"),
            List(list) => {
                write!(f, "[")?;
                for (idx, x) in list.iter().enumerate() {
                    write!(f, "{x}")?;
                    if idx != (list.len() - 1) {
                        write!(f, ",")?;
                    }
                }
                write!(f, "]")
            }
        }
    }
}

impl Message {
    fn parse(c: &mut Peekable<impl Iterator<Item = char>>) -> Self {
        match c.next().unwrap() {
            n if n.is_numeric() => {
                let mut out = (n as u8) - b'0';
                while let Some(n) = c.peek() {
                    if n.is_numeric() {
                        let n = c.next().unwrap();
                        out = out * 10 + ((n as u8) - b'0');
                    } else {
                        break;
                    }
                }

                Message::Single(out)
            }
            '[' => {
                let mut list = vec![];
                while let Some(n) = c.peek() {
                    match n {
                        ']' => {
                            let _ = c.next().unwrap();
                            return Message::List(list);
                        }
                        ',' => {
                            // permits multiple commas but oh well
                            let _ = c.next().unwrap();
                            continue;
                        }
                        _ => list.push(Self::parse(c)),
                    }
                }

                panic!("ran out of chars; expected end of list")
            }
            other => panic!("unexpected char: {other}"),
        }
    }
}

impl FromStr for Message {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars().peekable();
        Ok(Self::parse(&mut chars))
    }
}

impl Message {
    fn list_cmp<'s>(
        mut lhs: impl Iterator<Item = &'s Self>,
        mut rhs: impl Iterator<Item = &'s Self>,
    ) -> Ordering {
        loop {
            match (lhs.next(), rhs.next()) {
                (None, None) => break Ordering::Equal,
                (None, Some(_)) => break Ordering::Less,
                (Some(_), None) => break Ordering::Greater,
                (Some(l), Some(r)) => match l.cmp(r) {
                    Ordering::Equal => continue,
                    not_equal => break not_equal,
                },
            }
        }
    }
}

impl PartialOrd for Message {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Message::*;
        match (self, other) {
            (Single(a), Single(b)) => a.partial_cmp(b),
            (l @ Single(_), List(rhs)) => Some(Self::list_cmp(once(l), rhs.iter())),
            (List(lhs), r @ Single(_)) => Some(Self::list_cmp(lhs.iter(), once(r))),
            (List(lhs), List(rhs)) => Some(Self::list_cmp(lhs.iter(), rhs.iter())),
        }
    }
}

impl Ord for Message {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Pair {
    left: Message,
    right: Message,
}

impl Display for Pair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.left)?;
        writeln!(f, "{}", self.right)
    }
}

impl FromStr for Pair {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut l = s.lines();
        let left = l.next().ok_or(())?.parse()?;
        let right = l.next().ok_or(())?.parse()?;

        assert!(l.next().is_none());
        Ok(Self { left, right })
    }
}

impl Pair {
    fn correct_order(&self) -> bool {
        self.left < self.right
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Pairs {
    list: Vec<Pair>,
}

impl Display for Pairs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for p in &self.list {
            writeln!(f, "{p}\n")?;
        }

        Ok(())
    }
}

impl FromStr for Pairs {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Pairs {
            list: s
                .split("\n\n")
                .map(|pair| pair.parse())
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl Pairs {
    fn score(&self) -> usize {
        self.list
            .iter()
            .enumerate()
            .filter(|(_, p)| p.correct_order())
            .map(|(idx, _)| idx + 1)
            .sum()
    }

    fn all_messages(&self) -> impl Iterator<Item = &'_ Message> + '_ {
        self.list.iter().flat_map(|p| [&p.left, &p.right])
    }

    fn decoder_key(&self) -> usize {
        let div1: Message = "[[2]]".parse().unwrap();
        let div2: Message = "[[6]]".parse().unwrap();

        let messages = self
            .all_messages()
            .chain([&div1, &div2])
            .sorted()
            .collect_vec();
        messages
            .iter()
            .enumerate()
            .filter(|(_, &m)| m == &div1 || m == &div2)
            .map(|(i, _)| i + 1)
            .product()
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2022, 13);
    let inp = aoc.get_input();
    // let inp = include_str!("ex");

    let pairs: Pairs = inp.parse().unwrap();
    println!("{pairs}");

    let p1 = pairs.score();
    aoc.submit_p1(dbg!(p1)).unwrap();

    let p2 = pairs.decoder_key();
    aoc.submit_p2(dbg!(p2)).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs() -> Pairs {
        include_str!("ex").parse().unwrap()
    }

    #[test]
    fn p1() {
        assert_eq!(pairs().score(), 13);
    }

    #[test]
    fn p2() {
        assert_eq!(pairs().decoder_key(), 140);
    }

    // #[test]
    // fn p2() {
    //     assert_eq!(super::p2(&ex(), false), 2713310158);
    // }
}
