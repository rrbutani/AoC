use aoc::*;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
struct Stack {
    inner: Vec<u8>,
}

impl Stack {
    fn push(&mut self, c: u8) {
        self.inner.push(c)
    }

    fn extend(&mut self, substack: Vec<u8>) {
        self.inner.extend(substack)
    }

    fn pop(&mut self) -> u8 {
        self.inner.pop().unwrap()
    }

    fn pop_n(&mut self, n: usize) -> Vec<u8> {
        self.inner.split_off(self.inner.len() - n)
    }

    fn top(&self) -> &u8 {
        self.inner.last().unwrap()
    }
}

// bad
fn ilog10(mut a: usize) -> usize {
    let mut count = 0;
    while a > 10 {
        count += 1;
        a /= 10;
    }

    count
}

struct Stacks<'a>(&'a [Stack]);
impl Debug for Stacks<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        let max = self.0.iter().map(|s| s.inner.len()).max().unwrap();

        let num_stacks = self.0.len();
        // let padding = num_stacks.ilog10();
        let padding = ilog10(num_stacks);
        let rpadding = padding / 2;
        let lpadding = padding - rpadding;

        for height in (0..max).rev() {
            for stack in self.0.iter() {
                write!(f, " ")?;
                for _ in 0..lpadding {
                    write!(f, " ")?;
                }

                if let Some(&c) = stack.inner.get(height) {
                    write!(f, "[{}]", c as char)?
                } else {
                    write!(f, "   ")?
                }

                for _ in 0..rpadding {
                    write!(f, " ")?
                }
            }
            writeln!(f)?
        }

        for idx in 0..num_stacks {
            write!(f, " {idx:^n$}", n = padding.max(3) as usize)?
        }

        writeln!(f)?;

        Ok(())
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2022, 5);
    let inp = aoc.get_input();

    let (stacks, directions) = inp.split_once("\n\n").unwrap();

    let stacks: Vec<Stack> = {
        let mut stacks = stacks.lines().rev();
        let num_stacks: usize = stacks
            .next()
            .unwrap()
            .trim_end()
            .rsplit_once(" ")
            .unwrap()
            .1
            .parse()
            .unwrap();

        let mut out = vec![Stack::default(); num_stacks];
        for line in stacks {
            for (idx, cell) in line.as_bytes().chunks(4).enumerate() {
                match cell {
                    [b'[', c, b']', ..] => out[idx].push(*c),
                    _ => {}
                }
            }
        }

        dbg!(Stacks(&out));

        out
    };

    let directions = directions.lines().map(|line| {
        if let ["move", n, "from", src, "to", dst] = line.split(' ').collect::<Vec<_>>()[..] {
            let n: u8 = n.parse().unwrap();
            let src: usize = src.parse().unwrap();
            let dst: usize = dst.parse().unwrap();

            let src = src - 1;
            let dst = dst - 1;

            (n, src, dst)
        } else {
            panic!("invalid direction: {line}");
        }
    });
    let ans = |stacks: Vec<Stack>| -> String { stacks.iter().map(|s| *s.top() as char).collect() };

    let p1 = {
        let mut stacks = stacks.clone();

        for (n, src, dst) in directions.clone() {
            for _ in 0..n {
                let x = stacks[src].pop();
                stacks[dst].push(x);
            }
        }

        ans(stacks)
    };
    aoc.submit_p1(p1).unwrap();

    let p2 = {
        let mut stacks = stacks.clone();

        for (n, src, dst) in directions.clone() {
            let x = stacks[src].pop_n(n as _);
            stacks[dst].extend(x);
        }

        ans(stacks)
    };
    aoc.submit_p2(p2).unwrap();
}
