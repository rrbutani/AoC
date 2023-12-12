use aoc::*;

use owo_colors::OwoColorize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Instruction {
    Nop,
    Addx { imm: isize },
}

impl FromStr for Instruction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Instruction::*;
        match s {
            "noop" => Ok(Nop),
            l if l.starts_with("addx") => {
                let (_, num) = l.split_once(' ').ok_or(())?;
                Ok(Instruction::Addx {
                    imm: num.parse().map_err(|_| ())?,
                })
            }
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Cpu {
    x: isize,
    cycle: usize,
}

impl Default for Cpu {
    fn default() -> Self {
        Self { x: 1, cycle: 0 }
    }
}

impl Cpu {
    fn tick(&mut self, callback: Option<impl FnOnce(&Self)>) {
        self.cycle += 1;
        if let Some(func) = callback {
            func(self)
        }
    }

    fn tick_fn(&mut self, callback: impl FnOnce(&Self)) {
        self.tick(Some(callback))
    }

    pub fn exec(
        &mut self,
        instructions: impl Iterator<Item = Instruction>,
        mut func: impl FnMut(&Self),
    ) {
        use Instruction::*;

        for ins in instructions {
            match ins {
                Nop => self.tick_fn(&mut func),
                Addx { imm } => {
                    self.tick_fn(&mut func);
                    self.tick_fn(&mut func);
                    self.x += imm;
                }
            }
        }
    }
}

fn parse(inp: &str) -> impl Iterator<Item = Instruction> + '_ {
    inp.lines().map(|l| l.parse().unwrap())
}

fn p1(inp: impl Iterator<Item = Instruction>) -> isize {
    let mut cpu = Cpu::default();
    let mut out = 0;
    let mut inc = |c: &Cpu| {
        out += c.x * (c.cycle as isize);
    };
    cpu.exec(inp, |c| match c.cycle {
        20 | 60 | 100 | 140 | 180 | 220 => inc(c),
        _ => {}
    });

    out
}

fn readline(prompt: &str) -> String {
    eprint!("{prompt}: ");
    let mut out = String::new();
    std::io::stdin().read_line(&mut out).unwrap();
    out.trim_end().to_owned()
}

fn main() {
    let mut aoc = AdventOfCode::new(2022, 10);
    let inp = aoc.get_input();

    aoc.submit_p1(p1(parse(&inp))).unwrap();

    let p2 = {
        let mut cpu = Cpu::default();
        cpu.exec(parse(&inp), |c| {
            let pos = ((c.cycle - 1) % 40) as isize;
            if ((c.x - 1)..=(c.x + 1)).contains(&pos) {
                eprint!("{}", '#'.green())
            } else {
                eprint!("{}", '.'.dimmed())
            }

            if c.cycle % 40 == 0 {
                println!()
            }
        });

        readline("WHAT DO YOU SEE")
    };
    aoc.submit_p2(p2).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ex1() {
        let ins = parse(include_str!("./ex1"));
        let mut cpu = Cpu::default();
        cpu.exec(ins, |c| eprintln!("{:?}", c));

        assert_eq!(cpu.x, -1);
        assert_eq!(cpu.cycle, 5);
    }

    #[test]
    fn ex2() {
        let res = p1(parse(include_str!("./ex2")));
        assert_eq!(res, 13140);
    }
}
