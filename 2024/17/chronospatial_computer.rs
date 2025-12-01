use std::iter;

use aoc::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Instruction {
    /// `A ← A div 2 ** op:combo`
    Adv,
    /// `B ← B ^ op:literal`
    Bxl,
    /// `B ← op:combo % 8`
    Bst,
    /// `A != 0 → ip = op:literal`
    Jnz,
    /// `B ← B ^ C`
    Bxc,
    /// `emit op:combo % 8`
    Out,
    /// `B ← A div 2 ** op:combo`
    Bdv,
    /// `C ← A div 2 ** op:combo`
    Cdv,
}

type Word = u8; // note: 3-bits only...
type Var = usize;

impl TryFrom<Word> for Instruction {
    type Error = Word;

    fn try_from(value: Word) -> Result<Self, Self::Error> {
        use Instruction::*;
        Ok(match value {
            0 => Adv,
            1 => Bxl,
            2 => Bst,
            3 => Jnz,
            4 => Bxc,
            5 => Out,
            6 => Bdv,
            7 => Cdv,
            o => return Err(o),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Machine {
    a: Var,
    b: Var,
    c: Var,
    ip: Var,
}

impl FromStr for Machine {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (a, b, c) = s
            .lines()
            .map(|l| l.split_once(": ").unwrap().1.parse::<Var>().unwrap())
            .collect_tuple()
            .unwrap();

        Ok(Self { a, b, c, ip: 0 })
    }
}

impl Machine {
    fn lit(&self, op: Word) -> Var {
        op as Var
    }
    fn combo(&self, op: Word) -> Var {
        match op {
            0..=3 => self.lit(op),
            4 => self.a,
            5 => self.b,
            6 => self.c,
            7 => panic!("no `7` combo operands!"),
            _ => unreachable!(),
        }
    }

    fn run_until_halt(&mut self, program: &[Word]) -> Vec<Word> {
        use Instruction::*;
        let mut out = vec![];

        loop {
            let Some(insn) = program.get(self.ip).map(|&w| w.to()) else {
                break;
            };
            let op = if let Bxc = insn {
                0xFF
            } else {
                let Some(&o) = program.get(self.ip + 1) else {
                    break;
                };
                debug_assert!(o < 8);
                o
            };
            /* #[cfg(debug_assertions)]
            eprintln!("exec({:02X}): {insn:?} {op}", self.ip); */

            match insn {
                i @ (Adv | Bdv | Cdv) => {
                    let val = self.a / (2 as Var).pow(self.combo(op) as _);
                    let slot = match i {
                        Adv => &mut self.a,
                        Bdv => &mut self.b,
                        Cdv => &mut self.c,
                        _ => unreachable!(),
                    };
                    *slot = val;
                }
                Bxl => self.b = self.b ^ self.lit(op),
                Bst => self.b = self.combo(op) % 8,
                Jnz => {
                    if self.a != 0 {
                        self.ip = self.lit(op);
                        continue;
                    }
                }
                Bxc => self.b = self.b ^ self.c,
                Out => out.push((self.combo(op) % 8) as Word),
            }

            self.ip += 2;
        }

        out
    }
}

#[cfg(test)]
mod machine_tests {

    // todo: pretty

    use crate::Machine;

    #[test]
    fn a() {
        let mut machine = Machine::default();
        machine.c = 9;
        machine.run_until_halt(&[2, 6]);
        assert_eq!(machine.b, 1);
    }

    #[test]
    fn b() {
        let mut machine = Machine::default();
        machine.a = 10;
        let out = machine.run_until_halt(&[5, 0, 5, 1, 5, 4]);
        assert_eq!(out, [0, 1, 2]);
    }

    #[test]
    fn c() {
        let mut machine = Machine::default();
        machine.a = 2024;
        let out = machine.run_until_halt(&[0, 1, 5, 4, 3, 0]);
        assert_eq!(out, [4, 2, 5, 6, 7, 7, 7, 7, 3, 1, 0]);
        assert_eq!(machine.a, 0);
    }

    #[test]
    fn d() {
        let mut machine = Machine::default();
        machine.b = 29;
        machine.run_until_halt(&[1, 7]);
        assert_eq!(machine.b, 26);
    }

    #[test]
    fn e() {
        let mut machine = Machine::default();
        machine.b = 2024;
        machine.c = 43690;
        machine.run_until_halt(&[4, 0]);
        assert_eq!(machine.b, 44354);
    }
}

fn find_quine_dumb(machine: &Machine, program: &[Word]) -> Var {
    let mut a = 0;
    loop {
        let mut machine = machine.clone();
        machine.a = a;

        let out = machine.run_until_halt(program);
        if out == program {
            break a;
        }

        a += 1;

        if a % 1_000_000 == 0 {
            eprintln!("{a}");
        }
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2024, 17);
    let inp = aoc.get_input();
    let (machine, program) = inp.split_once("\n\n").unwrap();
    let machine: Machine = machine.parse().unwrap();
    let program: Vec<Word> = program
        .split_once(": ")
        .unwrap()
        .1
        .split(',')
        .map_parse()
        .collect_vec();

    let p1_alt = program_equiv(machine.a);
    let p1_corr = machine.clone().run_until_halt(&program);
    assert_eq!(p1_alt, p1_corr);
    dbg!(p1_alt);
    let p1 = machine
        .clone()
        .run_until_halt(&program)
        .into_iter()
        .join(",");
    _ = aoc.submit_p1(p1);

    // let p2 = find_quine_dumb(&machine, &program);
    let p2 = find_quine_little_less_dumb(&program);
    assert_eq!(program_equiv(p2), program);
    assert_eq!(Machine { a: p2, ..Default::default() }.run_until_halt(&program), program);

    _ = aoc.submit_p2(p2);
}

fn program_equiv(mut a: usize) -> Vec<u8> {
    let mut out = vec![];
    while a != 0 {
        let a0 = (a % 8) as u8;
        let a_up_to_10b = a & 0b11_1111_1111;

        // let lhs = (a0 & 0b010) ^ 0b111;
        // let rhs = a_up_to_10b / (1 << ((a0 ^ 0b010) as usize));
        // out.push((lhs ^ (rhs as u8)) % 8);

        /*         let b = a0;
        let b = b ^ 0b010;
        let c = a / 2usize.pow(b as _);
        let b = b ^ 0b111;
        let b = b ^ (c as u8);
        out.push(b % 8);
        */
        // let rhs = a_up_to_10b / (0b1 << (a0 ^ 0b010));

        let lhs = (a0 ^ 0b010) ^ 0b111;
        let rhs = a_up_to_10b >> (a0 ^ 0b010);
        out.push((lhs ^ (rhs as u8)) % 8);

        a = a / 8;
    }

    out
}

fn program_equiv_scalar(mut a: usize) -> usize {
    let mut out = 0;
    while a != 0 {
        let a0 = (a % 8) as u8;
        let a_up_to_10b = a & 0b11_1111_1111;

        // let lhs = (a0 & 0b010) ^ 0b111;
        // let rhs = a_up_to_10b / (1 << ((a0 ^ 0b010) as usize));
        // out.push((lhs ^ (rhs as u8)) % 8);

        /*         let b = a0;
        let b = b ^ 0b010;
        let c = a / 2usize.pow(b as _);
        let b = b ^ 0b111;
        let b = b ^ (c as u8);
        out.push(b % 8);
        */

        // let rhs = a_up_to_10b / (0b1 << (a0 ^ 0b010));

        let lhs = (a0 ^ 0b010) ^ 0b111;
        let rhs = a_up_to_10b >> (a0 ^ 0b010);
        let val = (lhs ^ (rhs as u8)) % 8;
        out = out << 3 | val as usize;

        a = a / 8;
    }

    out
}

// ```
// Program: 2,4,1,2,7,5,1,7,4,4,0,3,5,5,3,0
//
// 2,4, 1,2, 7,5, 1,7, 4,4 ,0,3, 5,5, 3,0
// ```
//
// ```
// Bst 4
// Bxl 2
// Cdv 5
// Bxl 7
// Bxc 4
// Adv 3
// Out 5
// Jnz 0
// ```
//
// ```
// Bst A
// Bxl 2
// Cdv B
// Bxl 7
// Bxc _
// Adv 3
// Out B
// Jnz 0
// ```

// ```asm
//                                # A is an array of integers, 3 bits apiece
//                                #
// B ← A % 8                      # put the first element of A in B
// B ← B ^ 0b010 (2)              # B = A[0] ^ 0b010
// C ← A / (2 ** B; 1 << B)       # C = A / (0b010 << (A[0] ^ 0b010))
// B ← B ^ 0b111 (7)              # B = (A[0] ^ 0b010) ^ 0b111
// B ← B ^ C                      # B = ((A[0] ^ 0b010) ^ 0b111) ^ (A / (0b010 << (A[0] ^ 0b010)))
// A ← A / (2 ** 3)               # drop the first element of A (shift)
// Out B                          # emit B
// Jnz 0                          # start over, if a isn't empty
// ```
//
// ```python
// B = ((A[0] ^ 0b010) ^ 0b111) ^ (A / (1 << (A[0] ^ 0b010)))
// ```
//
// ```python
// B = ((A[0] ^ 0b010) ^ 0b111) ^ (A >> (A[0] ^ 0b010))
// ```
//
// right hand side (`A >> ...`) is a way of looking ahead into `A`, a variable
// amount
//
// | `A[0]` | `A[0] ^ 0b010` |
// |:------:|:--------------:|
// | 0b000  | 0b010 (2)        -> A[2:5]
// | 0b001  | 0b011 (3)        -> A[3:6]
// | 0b010  | 0b000 (0)        -> A[0:3]
// | 0b011  | 0b001 (1)        -> A[1:4]
// | 0b100  | 0b110 (6)        -> A[5:8]
// | 0b101  | 0b111 (7)        -> A[7:10]
// | 0b110  | 0b100 (4)        -> A[4:7]
// | 0b111  | 0b101 (5)        -> A[5:8]
//
// left hand side is simpler (2nd column above inverted; like `7-{above}`)
// | `A[0]` | `(A[0] ^ 0b010) ^ 0b111`
// |:------:|:-----------------------:|
// | 0b000  | 0b101 (5)
// | 0b001  | 0b100 (4)
// | 0b010  | 0b111 (7)
// | 0b011  | 0b110 (6)
// | 0b100  | 0b001 (1)
// | 0b101  | 0b000 (0)
// | 0b110  | 0b011 (3)
// | 0b111  | 0b010 (2)
//
// maybe we can walk backwards?
//   - desired output is: `2,4,1,2,7,5,1,7,4,4,0,3,5,5,3,0`
//   - 16 nums -> A needs 48 bits
//
// out[15] = 0b0 (0)
//   + this can only be the case if the lhs and rhs are the same (xor)
//   + we know `A[15]` cannot be 0 — else the program would have ended already
//   + we know `A[16]` *must* be exactly 0 — else the program wouldn't end
//     * this means that `A[3:6]+` are all zero meaning `A[15]` vals in
//       `[1,4,5,6,7]` all yield a rhs of 0
//       - this would require a lhs of 0.. which is only possible for an `A[15]`
//         of `5`
//       - conveniently this is in our possible `A[15]` values
//   + just for completeness the other two possible rhs values would be `A[0:3]`
//     and `A[1:4]`; i.e. `A[15]` and `A[15] >> 1`
//     * `rhs = A[15]` requires `A[15] = 2` in which case `rhs = 2`
//       - lhs is only `2` when when `A[15]` is `7` — doesn't line up
//     * `rhs = A[15] >> 1` requires `A[15] = 3` in which case `rhs = 1`
//       - lhs is `1` when `A[15] = 4` — doesn't line up
// ergo, `A[15]` = `5`

// as much fun as unraveling these by hand is, now that we know the shape of
// the program and its input sensitivity we have enough to write a quick brute
// forcer that tacks on one digit at a time to the input `(A)` in order to
// divine the input value to produce the quine

// key bit is that there is lookahead but it's only sensitive to future digits
// in `A`, not past ones — by starting from the end of `A` we avoid the need to
// do any backtracking
//
// wait nevermind; that's not quite true... can have multiple options for a
// current num, the choice influences the next num
/*
fn find_quine_little_less_dumb(program: &[Word]) -> usize {
    let mut a: Vec<u8> = vec![];
    'next_a: while a.len() < program.len() {
        dbg!(&a);
        for next in 0..8 {
            let starting_a = a
                .iter()
                .copied()
                .chain([next])
                .fold(0, |acc, n| acc << 3 | (n as usize));
            let ret = program_equiv(starting_a);
            if ret.len() != a.len() + 1 {
                continue;
            }
            let corresp = &program[program.len() - ret.len()..];
            eprintln!("A[{:02}]={next}: {ret:?} vs {corresp:?}", program.len() - a.len() - 1);
            assert_eq!(ret[1..ret.len()], corresp[1..ret.len()]);
            if ret[0] == corresp[0] {
                a.push(next);
                continue 'next_a;
            }
        }

        panic!("stuck at a = {a:?}");
    }

    dbg!(&a);
    a.into_iter().fold(0, |acc, n| acc << 3 | n as usize)
}
*/

// generators in rust when :(
struct FindInputForSingleOutput<'p, It: Iterator<Item = usize>> {
    program: &'p [Word],
    target: &'p [Word],
    inner: It,
    current_search: Option<(usize, SmallVec<[u8; 8]>)>,
}

impl<I: Iterator<Item = usize>> FindInputForSingleOutput<'_, I> {
    fn level(&self) -> usize {
        debug_assert!(self.program.len() >= self.target.len());
        self.program.len() - self.target.len()
    }
}

impl<'p, It> Iterator for FindInputForSingleOutput<'p, It>
where
    It: Iterator<Item = usize>,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((prev_val, curr_vals)) = &mut self.current_search {
            let this = curr_vals.pop().unwrap();
            let ret = *prev_val << 3 | this as usize;

            if curr_vals.is_empty() {
                self.current_search = None;
            }

            return Some(ret);
        }

        let prev_val = self.inner.next()?;
        let mut valid_curr_vals = SmallVec::new();
        for val in 0..8 {
            let a = prev_val << 3 | (val as usize);
            let ret = program_equiv(a);
            #[cfg(debug_assertions)]
            #[rustfmt::skip]
            eprintln!(
                "A[{:02}]={val} (prev: {:#048b}): [{}] {ret:?} vs {:?}",
                self.level(), prev_val << 3,
                if ret == self.target { "match" } else { "     " },
                self.target,
            );
            if ret.len() == self.target.len() {
                debug_assert_eq!(ret[1..], self.target[1..]);
                if ret[0] == self.target[0] {
                    valid_curr_vals.push(val);
                }
            }
        }
        #[cfg(debug_assertions)]
        eprintln!();

        if !valid_curr_vals.is_empty() {
            self.current_search = Some((prev_val, valid_curr_vals));
        }
        self.next()
    }
}

fn find_quine_little_less_dumb_inner<'p>(
    program: &'p [Word],
    target: &'p [Word],
) -> impl Iterator<Item = usize> + 'p {
    match target {
        [] => Box::new(iter::once(0usize)) as Box<dyn Iterator<Item = usize>>,
        [_, rest @ ..] => Box::new(FindInputForSingleOutput {
            program,
            target,
            inner: find_quine_little_less_dumb_inner(program, rest),
            current_search: None,
        }),
    }
}

fn find_quine_little_less_dumb(program: &[Word]) -> usize {
    let a = find_quine_little_less_dumb_inner(program, program)
        /* next() */
        .min()
        .unwrap();

    for (idx, _) in program.iter().enumerate().rev() {
        eprint!("{} ", (a >> (idx * 3)) % 8);
    }
    eprintln!();

    a
}

// TODO: machine code gen (JIT)? (throw an optimizer at the assembly
// representation after lifting to get `program_equiv` instead of manually
// divining it)

// TODO: lower the program representation as a bunch of constraints and throw
// Z3 at it!

// TODO: test macro (pretty)

// TODO: pretty disassembler?
