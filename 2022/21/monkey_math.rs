use std::{collections::HashMap, mem};

use aoc::*;

const DBG: bool = false;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Monkey<Key, T = isize> {
    Imm(T),
    Sym(Key),
    Add(Key, Key),
    Sub(Key, Key),
    Mul(Key, Key),
    Div(Key, Key),
}

impl<K, T> Monkey<K, T> {
    fn map_key<L>(self, mut func: impl FnMut(K) -> L) -> Monkey<L, T> {
        use Monkey::*;
        match self {
            Imm(imm) => Imm(imm),
            Sym(sym) => Sym(func(sym)),
            Add(l, r) => Add(func(l), func(r)),
            Sub(l, r) => Sub(func(l), func(r)),
            Mul(l, r) => Mul(func(l), func(r)),
            Div(l, r) => Div(func(l), func(r)),
        }
    }
}

impl<K: Clone, T> Monkey<K, T> {
    fn operands(&self) -> Option<(K, K)> {
        use Monkey::*;
        let (l, r) = match self {
            Imm(_) | Sym(_) => return None,
            Add(l, r) | Sub(l, r) | Mul(l, r) | Div(l, r) => (l, r),
        };

        Some((l.clone(), r.clone()))
    }
}

struct Monkeys<'s> {
    map: HashMap<&'s str, Monkey<&'s str, isize>>,
    cache: HashMap<&'s str, isize>,
    symbolic_cache: HashMap<&'s str, Expr<&'s str, isize>>,
}

impl<'s> Monkeys<'s> {
    fn new(s: &'s str) -> Self {
        let mut map = HashMap::with_capacity(s.lines().count());

        use Monkey::*;
        for (name, op) in s.lines().map(|l| l.split_once(": ").unwrap()) {
            let monkey = if !op.contains(' ') {
                Imm(op.parse().unwrap())
            } else {
                let (lhs, op, rhs) = op.split(' ').tuple::<3>();
                let op = match op {
                    "+" => Add,
                    "-" => Sub,
                    "*" => Mul,
                    "/" => Div,
                    unknown => panic!("unknown op: {unknown}"),
                };

                op(lhs, rhs)
            };

            map.insert(name, monkey);
        }

        Monkeys {
            cache: HashMap::with_capacity(map.len()),
            symbolic_cache: HashMap::with_capacity(map.len()),
            map,
        }
    }
}

impl<'s> Monkeys<'s> {
    fn compute(&mut self, name: &'s str) -> Option<isize> {
        if let Some(res) = self.cache.get(name) {
            return Some(*res);
        }

        use Monkey::*;
        let res = match self.map[name] {
            Imm(imm) => imm,
            Sym(sym) =>
            /* panic!("cannot compute; found symbol: {sym}") */
            {
                return None
            }
            Add(lhs, rhs) => self.compute(lhs)? + self.compute(rhs)?,
            Sub(lhs, rhs) => self.compute(lhs)? - self.compute(rhs)?,
            Mul(lhs, rhs) => self.compute(lhs)? * self.compute(rhs)?,
            Div(lhs, rhs) => self.compute(lhs)? / self.compute(rhs)?,
        };

        self.cache.insert(name, res);
        Some(res)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Expr<K, T = isize> {
    Imm(T),
    Var(K),
    Add(Box<Expr<K, T>>, Box<Expr<K, T>>),
    Sub(Box<Expr<K, T>>, Box<Expr<K, T>>),
    Mul(Box<Expr<K, T>>, Box<Expr<K, T>>),
    Div(Box<Expr<K, T>>, Box<Expr<K, T>>),
}

impl<K: Display, T: Display> Display for Expr<K, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Expr::*;
        match self {
            Imm(imm) => write!(f, "{imm}"),
            Var(var) => write!(f, "{var}"),
            Add(l, r) => write!(f, "({l} + {r})"),
            Sub(l, r) => write!(f, "({l} - {r})"),
            Mul(l, r) => write!(f, "({l} * {r})"),
            Div(l, r) => write!(f, "({l} / {r})"),
        }
    }
}

impl<'s> Monkeys<'s> {
    fn symbolic(&mut self, name: &'s str) -> Expr<&'s str, isize> {
        if let Some(e) = self.symbolic_cache.get(name) {
            return e.clone();
        }

        use Expr as E;
        use Monkey::*;
        let res = if let Some(res) = self.compute(name) {
            E::Imm(res)
        } else {
            let m = &self.map[name];
            match *m {
                Imm(imm) => E::Imm(imm),
                Sym(sym) => E::Var(sym),
                Add(_, _) | Sub(_, _) | Mul(_, _) | Div(_, _) => {
                    // since `compute` failed there's a variable somewhere here
                    // so we can't just compute..
                    //
                    // but we can apply our standard peephold optimizations and
                    // some of these may be able to eliminate the variable
                    let b = Box::new;
                    let m: Monkey<E<&str, isize>, isize> = m.map_key(|k| self.symbolic(k));
                    use E::Imm as I;
                    // TODO: leverage `simplify` instead..
                    match m {
                        Add(I(0), r) => r,
                        Add(l, I(0)) => l,
                        Add(l, r) => E::Add(b(l), b(r)),

                        Sub(l, I(0)) => l,
                        Sub(l, r) => E::Sub(b(l), b(r)),

                        Mul(I(0), _) | Mul(_, I(0)) => I(0),
                        Mul(I(1), x) | Mul(x, I(1)) => x,
                        Mul(l, r) => E::Mul(b(l), b(r)),

                        Div(I(0), _) => I(0),
                        Div(l, I(1)) => l,
                        Div(l, r) => E::Div(b(l), b(r)),
                        _ => unreachable!(),
                    }
                }
            }
        };

        self.symbolic_cache.insert(name, res.clone());
        res
    }
}

impl<K: Eq + std::hash::Hash> Expr<K, isize>
where
    Expr<K, isize>: Clone + Debug + Display,
{
    fn simplify(&mut self) -> Option<isize> {
        use Expr::*;
        let ret = match self {
            Imm(imm) => *imm,
            Var(_) => return None,
            Add(l, r) => {
                let (ls, rs) = (l.simplify(), r.simplify());
                if ls == Some(0) {
                    *self = mem::replace(&mut **r, Expr::Imm(0));
                    return rs;
                } else if rs == Some(0) {
                    *self = mem::replace(&mut **l, Expr::Imm(0));
                    return ls;
                } else {
                    ls.zip(rs).map(|(l, r)| l + r)?
                }
            }

            Sub(l, r) => {
                let ls = l.simplify();
                let rs = r.simplify();
                if rs == Some(0) {
                    *self = mem::replace(&mut **l, Expr::Imm(0));
                    return ls;
                } else {
                    ls.zip(rs).map(|(l, r)| l - r)?
                }
            }

            Mul(l, r) => {
                let ls = l.simplify();
                let rs = r.simplify();
                if ls == Some(0) || rs == Some(0) {
                    0
                } else if ls == Some(1) {
                    *self = mem::replace(&mut **r, Expr::Imm(0));
                    return rs;
                } else if rs == Some(1) {
                    *self = mem::replace(&mut **l, Expr::Imm(0));
                    return ls;
                } else {
                    ls.zip(rs).map(|(l, r)| l * r)?
                }
            }

            Div(l, r) => {
                let ls = l.simplify();
                let rs = r.simplify();
                if ls == Some(0) {
                    0
                } else if rs == Some(1) {
                    *self = mem::replace(&mut **l, Expr::Imm(0));
                    return ls;
                } else {
                    ls.zip(rs).map(|(l, r)| l / r)?
                }
            }
        };

        *self = Imm(ret);
        Some(ret)
    }

    fn is_imm(&self) -> bool {
        matches!(self, Expr::Imm(_))
    }
    fn get_imm(&self) -> Option<isize> {
        match self {
            Expr::Imm(i) => Some(*i),
            _ => None,
        }
    }

    // very primitve; not general purpose; assumes equal_to is an `Imm`; doesn't
    // support multiple vars or even multiple occurences of the var...
    //
    // just sophisticated enough to solve this problem and that's it
    fn solve(&self, equal_to: Self) -> HashMap<K, isize> {
        use Expr::*;
        // match
        let mut lhs = self.clone();
        let mut rhs = equal_to;
        let b = Box::new;
        assert!(rhs.is_imm());
        loop {
            lhs = match lhs {
                Imm(_) => unreachable!(),
                Var(v) => {
                    let Imm(res) = rhs else {
                        panic!("rhs is {rhs}");
                    };
                    break HashMap::from_iter([(v, res)].into_iter());
                }
                Add(l, r) => {
                    // (imm + ...) = x
                    // ->
                    // ... = x - imm
                    //
                    // (... + imm) = x
                    // ->
                    // ... = x - imm
                    let (new_lhs, sub) = if let Some(l) = l.get_imm() {
                        (r, l)
                    } else if let Some(r) = r.get_imm() {
                        (l, r)
                    } else {
                        panic!()
                    };

                    rhs = Sub(b(rhs), b(Imm(sub)));

                    *new_lhs
                }
                Sub(l, r) => {
                    // (imm - ...) = x
                    // ->
                    // imm - x = ... => ... = imm - x
                    //
                    // (... - imm) = x
                    // ->
                    // ... = x + imm
                    if let Some(l) = l.get_imm() {
                        rhs = Sub(b(Imm(l)), b(rhs));
                        *r
                    } else if let Some(r) = r.get_imm() {
                        rhs = Add(b(rhs), b(Imm(r)));
                        *l
                    } else {
                        panic!()
                    }
                }
                Mul(l, r) => {
                    // (imm * ...) = x
                    // ->
                    // ... = x / imm
                    //
                    // (... * imm) = x
                    // ->
                    // ... = x / imm
                    let (new_lhs, quot) = if let Some(l) = l.get_imm() {
                        (r, l)
                    } else if let Some(r) = r.get_imm() {
                        (l, r)
                    } else {
                        panic!()
                    };

                    rhs = Div(b(rhs), b(Imm(quot)));

                    *new_lhs
                }
                Div(l, r) => {
                    // (imm / ...) = x
                    // ->
                    // (imm / x) = ... => ... = (imm / x)
                    //
                    // (... / imm) = x
                    // ->
                    // ... = x * imm
                    if let Some(l) = l.get_imm() {
                        rhs = Div(b(Imm(l)), b(rhs));
                        *r
                    } else if let Some(r) = r.get_imm() {
                        rhs = Mul(b(rhs), b(Imm(r)));
                        *l
                    } else {
                        panic!()
                    }
                }
            };

            rhs.simplify();
            eprintln!("\n{lhs} == {rhs}");
        }
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2022, 21);
    let inp = aoc.get_input();
    // let inp = include_str!("ex");

    let mut m = Monkeys::new(&inp);
    let p1 = m.compute("root").unwrap();
    dbg!(p1);
    // aoc.submit_p1(p1).unwrap();

    let mut m = Monkeys::new(&inp);
    let (l, r) = m.map.get("root").unwrap().operands().unwrap();
    *m.map.get_mut("humn").unwrap() = Monkey::Sym("humn");
    let mut l = m.symbolic(l);
    let mut r = m.symbolic(r);

    l.simplify();
    r.simplify();
    eprintln!("{l}\n==\n{r}");
    assert!(r.is_imm());
    let vars = l.solve(r);
    let p2 = vars["humn"];
    dbg!(p2);
    aoc.submit_p2(dbg!(p2)).unwrap();
}
