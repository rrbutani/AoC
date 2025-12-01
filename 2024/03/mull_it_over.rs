use aoc::*;

// const EX1: &str = "xmul(2,4)%&mul[3,7]!@^do_not_mul(5,5)+mul(32,64]then(mul(11,8)mul(8,5))";
// const EX2: &str = "xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Item {
    Mul(usize, usize),
    Do,
    DoNot,
}

// `mul\([\d]+,[\d]+\)`
// `don\'t\(\)`
// `do\(\)`
//
// don't want to pull in `regex` or `nom` though; lets just write the state
// machine by hand...
//
// won't perform as well as a regular expression engine (no vectorization) but
// it's fine
fn parse(s: &str) -> impl Iterator<Item = Item> + Clone + '_ {
    #[derive(Default, Clone, Copy, PartialEq, Eq)]
    #[rustfmt::skip]
    enum State {
        #[default]
        Start,
        M, U, L,
        MulParen,
        FirstNum(usize),
        Comma { first: usize },
        SecondNum(usize, usize),

        D, O, DoParen,
        N, Apostrophe, T, DoNotParen,
    }
    use State::*;

    let mut state = State::default();

    const fn c2i(char: u8) -> usize {
        (char - b'0') as usize
    }

    // assuming ASCII
    s.as_bytes().iter().copied().filter_map(move |c| {
        state = match (c, state) {
            (b'm', Start) => M,
            (b'u', M) => U,
            (b'l', U) => L,
            (b'(', L) => MulParen,
            (c @ b'1'..=b'9', MulParen) => FirstNum(c2i(c)),
            (c @ b'0'..=b'9', FirstNum(n)) => FirstNum(n * 10 + c2i(c)),
            (b',', FirstNum(first)) => Comma { first },
            (c @ b'1'..=b'9', Comma { first }) => SecondNum(first, c2i(c)),
            (c @ b'0'..=b'9', SecondNum(f, n)) => SecondNum(f, n * 10 + c2i(c)),
            (b')', SecondNum(a, b)) => {
                state = Start;
                return Some(Item::Mul(a, b));
            }

            (b'd', Start) => D,
            (b'o', D) => O,
            (b'(', O) => DoParen,
            (b')', DoParen) => {
                state = Start;
                return Some(Item::Do);
            }

            (b'n', O) => N,
            (b'\'', N) => Apostrophe,
            (b't', Apostrophe) => T,
            (b'(', T) => DoNotParen,
            (b')', DoNotParen) => {
                state = Start;
                return Some(Item::DoNot);
            }

            _ => Start,
        };
        None
    })
}

fn main() {
    let mut aoc = AdventOfCode::new(2024, 3);
    let inp = aoc.get_input();
    let inp = parse(&inp);

    let p1: usize = inp
        .clone()
        .filter_map(|i| match i {
            Item::Mul(a, b) => Some((a, b)),
            _ => None,
        })
        .map(|(a, b)| a * b)
        .sum();
    _ = aoc.submit_p1(p1);

    let mut enabled = true;
    let p2: usize = inp
        .filter_map(|i| {
            match i {
                Item::Mul(a, b) if enabled => return Some((a, b)),
                Item::Mul(_, _) => {}
                Item::Do => enabled = true,
                Item::DoNot => enabled = false,
            }
            None
        })
        .map(|(a, b)| a * b)
        .sum();
    _ = aoc.submit_p2(p2);

    // let p1;
    // let p2;
}
