#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use itertools::Itertools;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Instruction {
    Mask(Mask),
    Write { addr: u64, val: u64 },
}

impl FromStr for Instruction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        if let Some(val) = scan_fmt!(s, "mask = {}", String) {
            Ok(Instruction::Mask(val.parse()?))
        } else if let (Some(addr), Some(val)) = scan_fmt!(s, "mem[{}] = {}", u64, u64) {
            Ok(Instruction::Write { addr, val })
        } else {
            Err(())
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Mask {
    ones: u64,
    zeros: u64,
    raw: Option<String>,
}

impl Mask {
    fn apply(&self, val: u64) -> u64 {
        (val & self.zeros) | self.ones
    }

    fn possibilities(&self) -> impl Iterator<Item = Mask> {
        // dbg!(self);

        let Mask { ones, .. } = *self;
        let zeros = u64::max_value(); // argh!

        self.raw
            .as_ref()
            .unwrap()
            .chars()
            .rev()
            .enumerate()
            .filter(|(_idx, c)| *c == 'X')
            .map(|(idx, _c)| vec![(idx as u8, false), (idx as u8, true)].into_iter())
            .multi_cartesian_product()
            .map(move |v| {
                let (mut ones, mut zeros) = (ones, zeros);
                for (idx, b) in &v {
                    // println!("idx: {}, b: {}", idx, b);
                    // println!("ones:  {:036b}", ones);
                    // println!("zeros: {:036b}", zeros);
                    // println!();

                    match *b {
                        true => {
                            ones |= 1 << idx;
                            // zeros |= 1 << idx;
                        }
                        false => {
                            zeros &= !(1 << idx);
                            // ones &= !(1 << idx);
                        }
                    }
                }

                Mask {
                    ones,
                    zeros,
                    raw: None,
                }
            })
        // .inspect(|mask| {
        //     dbg!(mask);
        // })

        // let mut possibilities: Box<dyn Iterator<Item = &[(bool, u8)]>> = Box::new(iter::empty());

        // for (idx, c) in floating_bits {}
    }
}

impl Default for Mask {
    fn default() -> Self {
        Self {
            ones: 0,
            zeros: u64::max_value(),
            raw: None,
        }
    }
}

impl FromStr for Mask {
    type Err = ();

    // XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X
    fn from_str(s: &str) -> Result<Self, ()> {
        if s.len() != 36 {
            return Err(());
        }

        let ones = u64::from_str_radix(&s.replace('X', "0"), 2).map_err(|_| ())?;
        let zeros = u64::from_str_radix(&s.replace('X', "1"), 2).map_err(|_| ())?;

        Ok(Mask {
            ones,
            zeros,
            raw: Some(s.to_string()),
        })
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2020, 14);
    let input: String = aoc.get_input();

    //     let input = "mask = 000000000000000000000000000000X1001X
    // mem[42] = 100
    // mask = 00000000000000000000000000000000X0XX
    // mem[26] = 1";

    let insns = input.lines().map(|l| l.parse::<Instruction>().unwrap());

    let mut memory: HashMap<u64, u64> = HashMap::new();
    let mut mask = Mask::default();

    for insn in insns.clone() {
        match insn {
            Instruction::Mask(m) => mask = m,
            Instruction::Write { addr, val } => {
                memory.insert(addr, mask.apply(val));
            }
        }
    }

    let p1: u64 = memory.iter().map(|(_, val)| *val).sum();
    println!("{}", p1);
    let _ = aoc.submit_p1(p1);

    // Brute Force:
    let mut memory: HashMap<u64, u64> = HashMap::new();
    let mut mask = Mask::default();

    for insn in insns {
        match insn {
            Instruction::Mask(m) => mask = m,
            Instruction::Write { addr, val } => {
                for mask in mask.possibilities() {
                    memory.insert(mask.apply(addr), val);
                }
            }
        }
    }

    let p2: u64 = memory.iter().map(|(_, val)| *val).sum();
    // println!("{}", p2);
    let _ = aoc.submit_p2(p2);

    // Better:
}

/// So, we've got these weird "super-position" addresses:
/// ```ignore
/// mask     = X10X1X0
/// addr     = 1001011
/// superpos = X10X1X1
/// ```
///
/// When _flattened_ they turn into 2 ^ N different addresses, where N = the
/// number of Xs in the address:
/// ```ignore
/// superpos: X10X1X1
/// addresses:
///  - 0100101
///  - 0100111
///  - 0101101
///  - 0101111
///  - 1100101
///  - 1100111
///  - 1101101
///  - 1101111
/// ```
///
/// Rather than try to flatten each super-position address and actually
/// _execute_ the program, we want to figure out how many addresses two
/// super-position addresses have in common; i.e. their _intersection_.
///
/// We can do this by "unifying" super-position addresses and then counting the
/// number of Xs to figure out the cardinality (if unification doesn't fail).
///
/// The rules for unification (a commutative operation for us) are:
/// ```ignore
/// (_ indicates a unification failure)
///  0 | 0 ⇒ 0
///  0 | 1 ⇒ _
///  0 | X ⇒ 0
///  1 | 1 ⇒ 1
///  1 | X ⇒ 1
///  X | X ⇒ X
/// ```
///
/// Here are some examples:
/// ```ignore
///   001X ∩ X0XX ⇒ 001X
/// | 001X ∩ X0XX | = 2
///
///   X10X1X1 ∩ X0XXXXX ⇒ X_0X1X1
/// | X10X1X1 ∩ X0XXXXX | = 0
/// ```
///
/// This is useful because we can use this to determine how many addresses each
/// super-position address will cause it's corresponding value to be written
/// into that _aren't_ overwritten by super-position addresses that come later.
///
/// For our small examples above with only 2 super-position addresses this is
/// straightforward but to make this work for longer sequences of super-position
/// addresses we need some [more math][inc-exc].
///
/// When we have only 2 super-position addresses (A and B), figuring out how
/// many addresses the first super-position's value will be written to (and not
/// overwritten) just means subtracting out the addresses that overlap with the
/// second super-position: `||A|| = |A| - |A ∩ B|`
///
/// But if we've got 3 super-position addresses (A, B, and C) things are
/// trickier. We can't just subtract out the addresses that are in A and B
/// **and** the addresses that are in B and C because we will unintentionally
/// _double count_ some of these addresses if A, B, and C all share some
/// addresses.
///
/// Here's an example where 00110 gets double counted:
/// ```ignore
/// Naïve:
/// A = 001XX
/// B = 00X10
/// C = 0011X // B and C share 00110; neither is a subset of the other
///
/// || A || = |A| - |A ∩ B| - |A ∩ C|
/// || A || =  4  - |00110| - |0011X|
/// || A || =  4  -    1    -    2
/// || A || =  1
///
/// Actual:
/// [00000]:
/// [00001]:
/// [00010]: B
/// [00011]:
/// [00100]: A
/// [00101]: A
/// [00110]: C
/// [00111]: C
///
/// || A || = 2
/// ```
///
/// The solution is to compensate for this "double counting" by adding back in
/// addresses that are shared betwen B and C (and also are in A). Altogether
/// that means to find the number of addresses that super-position A's value
/// will end up in we need to calulate:
/// ```ignore
/// || A || = |A| - |A ∩ B| - |A ∩ C| + |A ∩ B ∩ C|
/// ```
///
/// This is the [_inclusion-exclusion principle_][inc-exc].
///
/// Generalized to N sets, it tells us that to find the cardinality of the
/// portion of some set (A) that _doesn't overlap with any of the other N-1
/// sets_ we need to alternate between adding and subtracting the cardinalities
/// of all the possible subsets of the N sets that contain A where whether we
/// add or subtract depends on whether the cardinality of the subset is even or
/// odd.
///
/// Written out for some values of N:
/// ```ignore
/// N = 1: || A || = |A|
/// N = 2: || A || = |A| - |A ∩ B|
/// N = 3: || A || = |A| - |A ∩ B| - |A ∩ C| + |A ∩ B ∩ C|
/// N = 4: || A || = |A| - |A ∩ B| - |A ∩ C| - |A ∩ D| + |A ∩ B ∩ C| + |A ∩ B ∩ D| + |A ∩ C ∩ D| - |A ∩ B ∩ C ∩ D|
/// ...
/// ```
///
/// The number of terms we have follows this pattern:
/// ```ignore
/// N = 1: 1
/// N = 2: 1 + 1
/// N = 3: 1 + 2 + 1
/// N = 4: 1 + 3 + 3 + 1
/// N = S: 1 + (1..S).map(|i| nCr(S - 1, i)).sum()
///
/// This gives us: O(N^2)
/// ```
///
/// Note that we calculate the number of terms and **not** the number of unions
/// because each new term introduces at most 1 new union; i.e. to calculate
/// `|A ∩ B ∩ C ∩ D|` we only have to perform 1 union since we can reuse a
/// cached value of `|A ∩ B ∩ C|`.
///
/// Another thing to consider is that we have to do this for each of the N
/// super-position addresses that we have.
///
/// Here's an example with `S = 4`:
/// ```ignore
/// N = 1: || D || = |D|
/// N = 2: || C || = |C| - |C ∩ D|
/// N = 3: || B || = |B| - |B ∩ C| - |B ∩ D| + |B ∩ C ∩ D|
/// N = 4: || A || = |A| - |A ∩ B| - |A ∩ C| - |A ∩ D| + |A ∩ B ∩ C| + |A ∩ B ∩ D| + |A ∩ C ∩ D| - |A ∩ B ∩ C ∩ D|
/// ```
///
/// This pattern makes it so that reusing cached terms doesn't _really_ help us
/// out so really our complexity becomes the summation of S `N^2` terms where
/// S is our number of masks:
/// ```ignore
/// (1..S).map(|i| i * i).sum()
/// ```
///
/// This has a [closed form][sum-seq-squares] of `(N * (N+1) * (2N+1))/6` and is
/// thus `O(N^3)` where N is the number of super-position masks (460 for our
/// input).
///
/// Comparatively, the brute-force solution's runtime is proportional to the
/// number of Xs in each mask (which is then multiplied by the number of
/// addresses that use that mask).
///
/// This makes it trickier to quickly estimate how the brute-force solution will
/// perform on our input but let's say that all the masks have an average of 7
/// Xs (in reality they seem to vary between 4 and 9 Xs and it's worth noting
/// that since the complexity is 2^N where N is the number of Xs we don't want
/// to take a linear average). This is an average that likely _favors_ the brute
/// force solution; the linear average is 6.55. Anyways, split across the 460
/// writes this means a total of (2 ^ 7) * 460 writes ⇒ 58880 writes.
///
/// For our super-position address solution we get: 460^3 ⇒ 97336000 (or
/// 32551210 if we use the actual formula instead of the estimate) which is
/// something like 2 or 3 orders of magnitude worse (and ignores the fact that
/// the per-operation cost of this solution will likely be an order of magnitude
/// more expensive than that of the brute force solution — it'll take more
/// cycles to calculate a set intersection than to do a write). 😬
///
/// We do save having to traverse "memory" at the end though. Not that it
/// matters at all.
///
/// [inc-exc]: https://en.wikipedia.org/wiki/Inclusion%E2%80%93exclusion_principle
/// [sum-seq-squares]: https://proofwiki.org/wiki/Sum_of_Sequence_of_Squares
fn seemingly_better_but_probably_actually_worse() {}

// 101010
// Mask: X1001X
// ones: 010010
// zero: 1001X
