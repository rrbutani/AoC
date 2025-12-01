use aoc::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Equation {
    res: usize,
    nums: Vec<usize>,
}

impl FromStr for Equation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (res, nums) = s.split_once(": ").ok_or(())?;
        Ok(Self {
            res: res.parse().map_err(|_| ())?,
            nums: nums.split_whitespace().map_parse().collect(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Op {
    Add,
    Mul,

    // unlike `add` and `mul`, `concat` is not associative... however we can
    // still use our "eval rhs" search strategy
    Concat,
}

fn strip_suffix(num: usize, potential_suffix: usize) -> Option<usize> {
    fn next_pow_ten(n: usize) -> usize {
        match n {
            0 => 1,
            _ => 10 * next_pow_ten(n / 10),
        }
    }
    let base = next_pow_ten(potential_suffix);
    if num > potential_suffix && (num % base) == potential_suffix {
        Some(num / base)
    } else {
        None
    }
}

fn find_ops_inner<const USE_CONCAT: bool>(
    target: usize,
    rest: &[usize],
    op_slots: &mut [Op],
) -> bool {
    let f = find_ops_inner::<USE_CONCAT>;
    match rest {
        // if we ran out of numbers, no match (unless target is zero)
        [] => target == 0,
        // if we have a single number left, match only if it's equal to
        // the target:
        &[a] => {
            debug_assert_eq!(op_slots.len(), 0);
            target == a
        }
        // otherwise, take the next number and try both possibilities:
        &[ref rest @ .., z] => {
            let (this, op_slots) = op_slots.split_last_mut().unwrap();
            // only try `mul` if `target` is divisible by `a`:
            if target % z == 0 && f(target / z, rest, op_slots) {
                *this = Op::Mul;
                return true;
            }

            // only try `add` if `target` is greater than `a`:
            if let Some(target) = target.checked_sub(z) {
                if f(target, rest, op_slots) {
                    *this = Op::Add;
                    return true;
                }
            }

            if USE_CONCAT {
                if let Some(prefix) = strip_suffix(target, z) {
                    if f(prefix, rest, op_slots) {
                        *this = Op::Concat;
                        return true;
                    }
                }
            }

            false
        }
    }
}

impl Equation {
    fn find_ops<const CONCAT: bool>(&self) -> Option<Vec<Op>> {
        let mut ops = vec![Op::Add; self.nums.len() - 1];
        if find_ops_inner::<CONCAT>(self.res, &self.nums, &mut ops) {
            // self.print_with_ops(&ops);
            Some(ops)
        } else {
            None
        }
    }

    #[allow(unused)]
    fn print_with_ops(&self, ops: &[Op]) {
        assert_eq!(ops.len(), self.nums.len() - 1);
        eprint!("{} = ", self.res);
        for i in 0..self.nums.len() {
            if i != 0 {
                let o = match ops[i - 1] {
                    Op::Add => "+",
                    Op::Mul => "*",
                    Op::Concat => "||",
                };
                eprint!(" {o} ");
            }
            eprint!("{}", self.nums[i]);
        }
        eprintln!()
    }
}

const EX: &str = "190: 10 19
3267: 81 40 27
83: 17 5
156: 15 6
7290: 6 8 6 15
161011: 16 10 13
192: 17 8 14
21037: 9 7 18 13
292: 11 6 16 20";

fn main() {
    let mut aoc = AdventOfCode::new(2024, 7);
    let inp = aoc.get_input();
    // let inp = EX;
    let eqs = inp.lines().map_parse::<Equation>().collect_vec();

    let p1: usize = eqs
        .iter()
        .filter(|e| e.find_ops::<false>().is_some())
        .map(|e| e.res)
        .sum();
    dbg!(p1);
    _ = aoc.submit_p1(p1);

    let p2: usize = eqs
        .iter()
        .filter(|e| e.find_ops::<true>().is_some())
        .map(|e| e.res)
        .sum();
    dbg!(p2);
    _ = aoc.submit_p2(p2);
}

/*
6 8 6 15

rhs search:
[7290] = 6 _ 8 _ 6 _ 15
[7275] = 6 _ 8 _ 6 # + 15
 [486] = 6 _ 8 _ 6 # * 15
[7290] = 6 _ 8 _ 6 # || 15
  - must end in 15...

 [486] = 6 _ 8 _ 6 # * 15
  [48] = 6 _ 8 # || 6 * 15
  [48] = 6 * 8 #  || 6 * 15

[7290] = 6 * 8 || 6 * 15

---

lhs search:
[7290] = 6 _ 8 _ 6 _ 15
[7284] = (6 +) 8 _ 6 _ 15
[1215] = (6 *) 8 _ 6 _ 15
[7290] = (6 ||) 8 _ 6 _ 15
  - must start with lhs at this point
  - in this simple case (first number) that's known
    + but deeper in the search, the lhs value isn't known?

consider:
[1215] = (6 *) 8 || 6 _ 15
  - can't evaluate `1215 = <lhs> || new_target` because we don't have lhs
  - the issue is that we:
    + started with:
    + `7290 = 6 * 8 || 6 * 15`
    + aka:
    + `7290 = ((6 * 8) || 6) * 15`
    + and then tried to rewrite this as:
    + `7290 / 6 = ((8) || 6) * 15`
  - this kind of transformation is legal for addition and multiplication:
    + `e = a * b * c * d`
    + `e = (((a * b) * c) * d)`
    + `e / b = ((a * c) * d)`
    + i.e. it's associative
    * edit: nevermind; can't work with add *and* mul either due to the
      left-to-right eval order
    - but not for `||`

we can take advantage of the same precedence level for all ops and left to right
evaluation order; this means `rhs` for `||` will always be a number and we *can*
do this rewrite:
  - `target = ... || suffix`
  - `${atoi(itoa(target).removesuffix(itoa(suffix)))} = ...`
*/
