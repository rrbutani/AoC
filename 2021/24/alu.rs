#!/usr/bin/env rustr

use aoc::*;
// use dashmap::{DashMap, DashSet};
use fxhash::{FxHashMap, FxHashSet};
use num_traits::{One, Zero};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt,
    hash::{BuildHasherDefault, Hash},
    iter,
    ops::{Add, Div, Mul, Rem},
};

type DefaultImm = isize;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Reg {
    W,
    X,
    Y,
    Z,
}

impl Display for Reg {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Reg::*;

        if fmt.alternate() {
            write!(
                fmt,
                "{}",
                match self {
                    W => "W",
                    X => "X",
                    Y => "Y",
                    Z => "Z",
                }
            )
        } else {
            write!(
                fmt,
                "{}",
                match self {
                    W => "w",
                    X => "x",
                    Y => "y",
                    Z => "z",
                }
            )
        }
    }
}

impl FromStr for Reg {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Reg::*;
        Ok(match s {
            "w" => W,
            "x" => X,
            "y" => Y,
            "z" => Z,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Either<Left, Right> {
    Left(Left),
    Right(Right),
}

impl<L: Display, R: Display> Display for Either<L, R> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Either::Left(l) => write!(fmt, "{}", l),
            Either::Right(r) => write!(fmt, "{}", r),
        }
    }
}

impl<L: FromStr, R: FromStr> FromStr for Either<L, R>
// where
//     L::Err: Into<R::Err>,
{
    type Err = R::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(l) = s.parse() {
            Ok(Either::Left(l))
        } else {
            s.parse().map(Either::Right)
        }
    }
}

// impl<T, L: TryFrom<T>, R: TryFrom<T>> TryFrom<T> for Either<L, R>
// where
//     L::Error: Into<R::Error>,
// {
//     type Error = R::Error;
//
//     fn try_from(inp: T) -> Result<Self, Self::Error> {
//         if let Ok(l) = inp.try_into() {
//             Ok(Either::Left(l))
//         } else {
//             inp.try_into().map(Either::Right)
//         }
//     }
// }

type E<L, R> = Either<L, R>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Instruction<Imm = DefaultImm> {
    Inp { dest: Reg },
    Add { dest: Reg, src: E<Reg, Imm> },
    Mul { dest: Reg, src: E<Reg, Imm> },
    Div { dest: Reg, src: E<Reg, Imm> },
    Mod { dest: Reg, src: E<Reg, Imm> },
    Eql { dest: Reg, src: E<Reg, Imm> },
}

impl<Imm: Display> Display for Instruction<Imm> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Instruction::*;
        match self {
            Inp { dest } => write!(fmt, "inp {}", dest),
            Add { dest, src } => write!(fmt, "add {} {}", dest, src),
            Mul { dest, src } => write!(fmt, "mul {} {}", dest, src),
            Div { dest, src } => write!(fmt, "div {} {}", dest, src),
            Mod { dest, src } => write!(fmt, "mod {} {}", dest, src),
            Eql { dest, src } => write!(fmt, "eql {} {}", dest, src),
        }
    }
}

impl<Imm> FromStr for Instruction<Imm>
where
    Either<Reg, Imm>: FromStr,
{
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Instruction::*;

        let mut it = s.split(' ');
        let mut n = || it.next().ok_or(());

        let res = match n()? {
            "inp" => Inp {
                dest: n()?.parse().map_err(|_| ())?,
            },
            other => {
                let dest = n()?.parse().map_err(|_| ())?;
                let src = n()?.parse().map_err(|_| ())?;
                match other {
                    "add" => Add { dest, src },
                    "mul" => Mul { dest, src },
                    "div" => Div { dest, src },
                    "mod" => Mod { dest, src },
                    "eql" => Eql { dest, src },
                    _ => return Err(()),
                }
            }
        };

        Ok(res)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Program<Imm = DefaultImm> {
    insns: Vec<Instruction<Imm>>,
}

impl<Imm: Display> Display for Program<Imm> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in self.insns.iter() {
            writeln!(f, "{}", i)?;
        }

        Ok(())
    }
}

impl<Imm: FromStr> FromStr for Program<Imm> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            insns: s.lines().map(|l| l.parse()).collect::<Result<_, ()>>()?,
        })
    }
}

impl<Imm: Clone> Program<Imm> {
    fn split_at_inputs(&self) -> Vec<Self> {
        let inps = self
            .insns
            .iter()
            .enumerate()
            .filter(|(_, insn)| matches!(insn, Instruction::Inp { .. }))
            .map(|(i, _)| i)
            .chain(iter::once(self.insns.len()));

        inps.tuple_windows()
            .map(|(start_idx, end_idx)| self.insns[start_idx..end_idx].to_vec())
            .map(|insns| Program { insns })
            .collect()
    }
}

impl<Imm: Clone> Program<Imm> {
    // symbol table, final reg values -> symbols
    fn symbolize(&self) -> (SymbolTable<Imm>, HashMap<Reg, AssignmentRef>) {
        use Instruction::*;
        use Reg::*;
        let mut table = SymbolTable::new();
        let mut curr = HashMap::from_iter(
            [
                (W, table.put_reg_assignment(W, Expr::ExistingReg(W))),
                (X, table.put_reg_assignment(X, Expr::ExistingReg(X))),
                (Y, table.put_reg_assignment(Y, Expr::ExistingReg(Y))),
                (Z, table.put_reg_assignment(Z, Expr::ExistingReg(Z))),
            ]
            .into_iter(),
        );

        for i in self.insns.iter() {
            match i {
                Inp { dest } => {
                    let inp = table.get_next_inp();
                    let aref = table.put_reg_assignment(*dest, Expr::Input(inp));
                    curr.insert(*dest, aref);
                }
                Add { dest, src }
                | Mul { dest, src }
                | Div { dest, src }
                | Mod { dest, src }
                | Eql { dest, src } => {
                    let lhs = Box::new(Expr::Ref(curr[dest]));
                    let rhs = Box::new(match src {
                        Either::Left(reg) => Expr::Ref(curr[reg]),
                        Either::Right(imm) => Expr::Imm(imm.clone()),
                    });

                    let expr = match i {
                        Add { .. } => Expr::Add { lhs, rhs },
                        Mul { .. } => Expr::Mul { lhs, rhs },
                        Div { .. } => Expr::Div { lhs, rhs },
                        Mod { .. } => Expr::Mod { lhs, rhs },
                        Eql { .. } => Expr::Cmp { lhs, rhs },
                        _ => unreachable!(),
                    };

                    curr.insert(*dest, table.put_reg_assignment(*dest, expr));
                }
            }
        }

        (table, curr)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct AssignmentRef {
    r: Reg,
    n: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct InpRef {
    n: usize,
}

impl Display for AssignmentRef {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}{}", self.r, self.n)
    }
}

impl Display for InpRef {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "in{}", self.n)
    }
}

#[derive(Debug, Clone)]
struct SymbolTable<Imm = DefaultImm> {
    table: HashMap<AssignmentRef, Expr<Imm>>,
    count_inp: usize,
    counts_regs: HashMap<Reg, usize>,
}

impl<Imm> SymbolTable<Imm> {
    fn new() -> Self {
        Self {
            table: HashMap::new(),
            count_inp: 0,
            counts_regs: HashMap::from_iter(
                [(Reg::W, 0), (Reg::X, 0), (Reg::Y, 0), (Reg::Z, 0)].into_iter(),
            ),
        }
    }

    fn put_reg_assignment(&mut self, r: Reg, exp: Expr<Imm>) -> AssignmentRef {
        let count = self.counts_regs[&r];
        *self.counts_regs.get_mut(&r).unwrap() += 1;
        let r = AssignmentRef { r, n: count };

        self.table.insert(r, exp);
        r
    }

    fn get_next_inp(&mut self) -> InpRef {
        let n = self.count_inp;
        self.count_inp += 1;

        InpRef { n }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Expr<Imm = isize> {
    Input(InpRef),
    Ref(AssignmentRef),
    ExistingReg(Reg),
    Imm(Imm),
    Add { lhs: Box<Self>, rhs: Box<Self> },
    Mul { lhs: Box<Self>, rhs: Box<Self> },
    Div { lhs: Box<Self>, rhs: Box<Self> },
    Mod { lhs: Box<Self>, rhs: Box<Self> },
    Cmp { lhs: Box<Self>, rhs: Box<Self> },
}

trait ExprMapper<Imm> {
    type Out;

    fn early_return_hook(&mut self, _e: &Expr<Imm>) -> Option<Self::Out> {
        None
    }
    fn return_hook(&mut self, _e: &Expr<Imm>, _ret: &Self::Out) {}

    fn input(&mut self, inp: InpRef) -> Self::Out;
    fn existing_reg(&mut self, reg: Reg) -> Self::Out;
    fn imm(&mut self, imm: &Imm) -> Self::Out;
    fn add(&mut self, lhs: Self::Out, rhs: Self::Out) -> Self::Out;
    fn mul(&mut self, lhs: Self::Out, rhs: Self::Out) -> Self::Out;
    fn div(&mut self, lhs: Self::Out, rhs: Self::Out) -> Self::Out;
    fn rem(&mut self, lhs: Self::Out, rhs: Self::Out) -> Self::Out;
    fn cmp(&mut self, lhs: Self::Out, rhs: Self::Out) -> Self::Out;
}

impl<Imm> Expr<Imm> {
    fn map<M: ExprMapper<Imm>>(&self, table: Option<&SymbolTable<Imm>>, mapper: &mut M) -> M::Out {
        use Expr::*;

        let ret = if let Some(ret) = mapper.early_return_hook(self) {
            ret
        } else {
            match self {
                Input(inp) => mapper.input(*inp),
                Imm(imm) => mapper.imm(imm),
                Ref(ar) => table.unwrap().table[ar].map(table, mapper),
                ExistingReg(reg) => mapper.existing_reg(*reg),
                Add { lhs, rhs } => {
                    let lhs = lhs.map(table, mapper);
                    let rhs = rhs.map(table, mapper);
                    mapper.add(lhs, rhs)
                }
                Mul { lhs, rhs } => {
                    let lhs = lhs.map(table, mapper);
                    let rhs = rhs.map(table, mapper);
                    mapper.mul(lhs, rhs)
                }
                Div { lhs, rhs } => {
                    let lhs = lhs.map(table, mapper);
                    let rhs = rhs.map(table, mapper);
                    mapper.div(lhs, rhs)
                }
                Mod { lhs, rhs } => {
                    let lhs = lhs.map(table, mapper);
                    let rhs = rhs.map(table, mapper);
                    mapper.rem(lhs, rhs)
                }
                Cmp { lhs, rhs } => {
                    let lhs = lhs.map(table, mapper);
                    let rhs = rhs.map(table, mapper);
                    mapper.cmp(lhs, rhs)
                }
            }
        };

        mapper.return_hook(self, &ret);
        ret
    }
}

impl<Imm: Display> Display for Expr<Imm> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Expr::*;
        match self {
            Input(i) => write!(fmt, "{}", i),
            Ref(ar) => write!(fmt, "{}", ar),
            ExistingReg(r) => write!(fmt, "{:#}", r),
            Imm(imm) => write!(fmt, "{}", imm),
            Add { lhs, rhs } => write!(fmt, "({} + {})", *lhs, *rhs),
            Mul { lhs, rhs } => write!(fmt, "({} * {})", *lhs, *rhs),
            Div { lhs, rhs } => write!(fmt, "({} / {})", *lhs, *rhs),
            Mod { lhs, rhs } => write!(fmt, "({} % {})", *lhs, *rhs),
            Cmp { lhs, rhs } => write!(fmt, "({} == {})", *lhs, *rhs),
        }
    }
}

impl<
        I: Zero
            + One
            + Eq
            + Add<Output = I>
            + Mul<Output = I>
            + Div<Output = I>
            + Rem<Output = I>
            + Clone
            + Hash,
    > Expr<I>
{
    pub fn is_zero(&self, table: &SymbolTable<I>) -> bool {
        self.const_evaluate(table)
            .map(|i| i.is_zero())
            .unwrap_or(false)
    }

    pub fn is_one(&self, table: &SymbolTable<I>) -> bool {
        self.const_evaluate(table)
            .map(|i| i.is_one())
            .unwrap_or(false)
    }

    fn const_evaluate(&self, table: &SymbolTable<I>) -> Option<I> {
        #[derive(Debug, Default)]
        struct ConstEvaluate<I: Hash + Clone + PartialEq>(HashMap<Expr<I>, Option<I>>);
        impl<
                I: Zero
                    + One
                    + Eq
                    + Add<Output = I>
                    + Mul<Output = I>
                    + Div<Output = I>
                    + Rem<Output = I>
                    + Clone
                    + Hash,
            > ExprMapper<I> for ConstEvaluate<I>
        {
            type Out = Option<I>;

            fn early_return_hook(&mut self, e: &Expr<I>) -> Option<Self::Out> {
                self.0.get(e).cloned()
            }

            fn return_hook(&mut self, e: &Expr<I>, ret: &Self::Out) {
                self.0.insert(e.clone(), ret.clone());
            }

            fn input(&mut self, _inp: InpRef) -> Self::Out {
                None
            }

            fn existing_reg(&mut self, _reg: Reg) -> Self::Out {
                None
            }

            fn imm(&mut self, imm: &I) -> Self::Out {
                Some(imm.clone())
            }

            fn add(&mut self, lhs: Self::Out, rhs: Self::Out) -> Self::Out {
                match (lhs, rhs) {
                    (Some(lhs), Some(rhs)) => Some(lhs + rhs),
                    _ => None,
                }
            }

            fn mul(&mut self, lhs: Self::Out, rhs: Self::Out) -> Self::Out {
                match (lhs, rhs) {
                    (Some(lhs), Some(rhs)) => Some(lhs + rhs),
                    (Some(lhs), _) if lhs.is_zero() => Some(I::zero()),
                    (_, Some(rhs)) if rhs.is_zero() => Some(I::zero()),
                    _ => None,
                }
            }

            fn div(&mut self, lhs: Self::Out, rhs: Self::Out) -> Self::Out {
                match (lhs, rhs) {
                    (Some(lhs), Some(rhs)) => Some(lhs / rhs),
                    (lhs, Some(rhs)) if rhs.is_one() => lhs,
                    _ => None,
                }
            }

            fn rem(&mut self, lhs: Self::Out, rhs: Self::Out) -> Self::Out {
                match (lhs, rhs) {
                    (Some(lhs), Some(rhs)) => Some(lhs % rhs),
                    (_, Some(rhs)) if rhs.is_zero() => Some(I::zero()),
                    _ => None,
                }
            }

            fn cmp(&mut self, lhs: Self::Out, rhs: Self::Out) -> Self::Out {
                match (lhs, rhs) {
                    (Some(lhs), Some(rhs)) => Some(if lhs == rhs { I::one() } else { I::zero() }),
                    _ => None,
                }
            }
        }

        let mut c = ConstEvaluate(HashMap::new());
        self.map(Some(table), &mut c)
    }

    // here we go backwards (effectively getting us "dead code" elim for free)
    //
    // the resulting expr will have no `Ref`s.
    pub fn get_inlined_expr(table: &SymbolTable<I>, assignment: AssignmentRef) -> Self {
        Self::process_expr(table, &table.table[&assignment]).into_owned()
    }

    fn process_expr<'e>(table: &SymbolTable<I>, expr: &'e Expr<I>) -> Cow<'e, Self> {
        use Cow::Borrowed as B;
        use Cow::Owned as O;
        use Expr::*;

        // Not really efficient (we're not _propogating_ or even caching) but it's okay.
        if let Some(imm) = expr.const_evaluate(table) {
            return O(Expr::Imm(imm));
        }

        match expr {
            i @ Input(_) => B(i),
            Ref(a) => O(Self::get_inlined_expr(table, *a)),
            e @ ExistingReg(_) => B(e),
            i @ Imm(_) => B(i),
            Add { lhs, rhs } if lhs.is_zero(table) => Self::process_expr(table, rhs),
            Add { lhs, rhs } if rhs.is_zero(table) => Self::process_expr(table, lhs),
            Add { lhs, rhs } => O(Expr::Add {
                lhs: Self::process_expr(table, lhs).into_owned().into(),
                rhs: Self::process_expr(table, rhs).into_owned().into(),
            }),
            Mul { lhs, rhs } if lhs.is_one(table) => Self::process_expr(table, rhs),
            Mul { lhs, rhs } if rhs.is_one(table) => Self::process_expr(table, lhs),
            Mul { lhs, rhs } => O(Expr::Mul {
                lhs: Self::process_expr(table, lhs).into_owned().into(),
                rhs: Self::process_expr(table, rhs).into_owned().into(),
            }),
            Div { lhs, rhs } if rhs.is_one(table) => Self::process_expr(table, lhs),
            Div { lhs, rhs } => O(Expr::Div {
                lhs: Self::process_expr(table, lhs).into_owned().into(),
                rhs: Self::process_expr(table, rhs).into_owned().into(),
            }),
            Mod { lhs, rhs } => O(Expr::Mod {
                lhs: Self::process_expr(table, lhs).into_owned().into(),
                rhs: Self::process_expr(table, rhs).into_owned().into(),
            }),
            Cmp { lhs, rhs } => O(Expr::Cmp {
                lhs: Self::process_expr(table, lhs).into_owned().into(),
                rhs: Self::process_expr(table, rhs).into_owned().into(),
            }),
        }
    }

    fn eval(&self, old_reg_vals: &HashMap<Reg, I>, inputs: &[I]) -> I
// where
    //     I: Debug + Display, // TODO remove
    {
        struct Evaluate<'a, I> {
            old_reg_vals: &'a HashMap<Reg, I>,
            inputs: &'a [I],
        }

        impl<
                I: One
                    + Zero
                    + Eq
                    + Add<Output = I>
                    + Mul<Output = I>
                    + Div<Output = I>
                    + Rem<Output = I>
                    + Clone,
            > ExprMapper<I> for Evaluate<'_, I>
        {
            type Out = I;

            fn input(&mut self, inp: InpRef) -> Self::Out {
                self.inputs[inp.n].clone()
            }

            fn existing_reg(&mut self, reg: Reg) -> Self::Out {
                self.old_reg_vals[&reg].clone()
            }

            fn imm(&mut self, imm: &I) -> Self::Out {
                imm.clone()
            }

            fn add(&mut self, lhs: Self::Out, rhs: Self::Out) -> Self::Out {
                lhs + rhs
            }

            fn mul(&mut self, lhs: Self::Out, rhs: Self::Out) -> Self::Out {
                lhs * rhs
            }

            fn div(&mut self, lhs: Self::Out, rhs: Self::Out) -> Self::Out {
                lhs / rhs
            }

            fn rem(&mut self, lhs: Self::Out, rhs: Self::Out) -> Self::Out {
                lhs % rhs
            }

            fn cmp(&mut self, lhs: Self::Out, rhs: Self::Out) -> Self::Out {
                if lhs == rhs {
                    I::one()
                } else {
                    I::zero()
                }
            }
        }

        let mut e = Evaluate {
            old_reg_vals,
            inputs,
        };
        // no table since we expect to only be dealing with inlined exprs!
        //
        // if we weren't in a rust we'd make a `InlinedExpr` newtype that's only
        // produced by the inline function that is porcelain over this function
        // which we'd then update to take an option
        // let ret = self.map(None, &mut e);
        self.map(None, &mut e)

        // eprintln!(
        //     "{} (with {:?} and {:?}) = {}",
        //     self, old_reg_vals, inputs, ret
        // );

        // ret
    }
}

#[cfg(feature = "llvm")]
mod llvm {
    use std::{collections::HashMap, fmt::Display, hash::Hash, mem, ops::Deref};

    use inkwell::{
        builder::Builder,
        context::Context as LlvmContext,
        execution_engine::{ExecutionEngine, JitFunction},
        module::Module,
        types::{IntType, StringRadix},
        values::IntValue,
        IntPredicate, OptimizationLevel,
    };
    use num_traits::{One, Zero};

    use crate::Reg;

    use super::InpRef;
    use super::{Expr, ExprMapper};

    pub(crate) struct Context<'ctx> {
        context: &'ctx LlvmContext,
        module: Module<'ctx>,
        builder: Builder<'ctx>,
        execution_engine: ExecutionEngine<'ctx>,
    }

    impl<'ctx> Context<'ctx> {
        pub(crate) fn new(ctx: &'ctx LlvmContext) -> Self {
            let module = ctx.create_module("alu");
            Self {
                context: ctx,
                builder: ctx.create_builder(),
                execution_engine: module
                    .create_jit_execution_engine(OptimizationLevel::Aggressive)
                    .unwrap(),
                module,
            }
        }
    }

    // (old_z, digit) -> z
    type StageFunction<Imm> = unsafe extern "C" fn(Imm, Imm) -> Imm;

    pub(crate) struct TrustMe<I>(I);
    unsafe impl<I> Send for TrustMe<I> {}
    unsafe impl<I> Sync for TrustMe<I> {}
    impl<I> Deref for TrustMe<I> {
        type Target = I;
        fn deref(&self) -> &I {
            &self.0
        }
    }

    impl<Imm: 'static + Display + One + Zero + Clone + Eq + Hash> Expr<Imm> {
        pub fn jit<'a>(
            &self,
            ctx: &'a Context<'a>,
            id: usize,
        ) -> TrustMe<JitFunction<'a, StageFunction<Imm>>> {
            let fn_name = format!("stage{}", id);
            dbg!(&fn_name);

            let imm_type = ctx
                .context
                .custom_width_int_type(mem::size_of::<Imm>() as u32 * 8);
            let fn_type = imm_type.fn_type(&[imm_type.into(), imm_type.into()], false);
            let function = ctx.module.add_function(&fn_name, fn_type, None);
            let basic_block = ctx.context.append_basic_block(function, "main");

            ctx.builder.position_at_end(basic_block);

            let z = function.get_nth_param(0).unwrap().into_int_value();
            let n = function.get_nth_param(1).unwrap().into_int_value();

            let mut table = HashMap::from_iter(
                [
                    (Expr::Input(InpRef { n: 0 }), n),
                    (Expr::ExistingReg(Reg::Z), z),
                ]
                .into_iter(),
            );
            struct Jit<'a, 'b, I> {
                context: &'a Context<'a>,
                table: &'b mut HashMap<Expr<I>, IntValue<'a>>,
                imm_type: IntType<'a>,
            }

            impl<'a, 'b, I: 'a + One + Zero + Clone + Display + Eq + Hash> ExprMapper<I> for Jit<'a, 'b, I> {
                type Out = IntValue<'a>;

                fn early_return_hook(&mut self, e: &Expr<I>) -> Option<IntValue<'a>> {
                    self.table.get(e).copied()
                }

                fn return_hook(&mut self, e: &Expr<I>, ret: &IntValue<'a>) {
                    self.table.insert(e.clone(), *ret);
                }

                fn input(&mut self, inp: InpRef) -> IntValue<'a> {
                    panic!("unhandled input! ({})", inp)
                }

                fn existing_reg(&mut self, reg: Reg) -> IntValue<'a> {
                    panic!("unprovided initial register! ({})", reg)
                }

                fn imm(&mut self, imm: &I) -> IntValue<'a> {
                    self.imm_type
                        .const_int_from_string(&format!("{}", imm), StringRadix::Decimal)
                        .unwrap()
                }

                fn add(&mut self, lhs: IntValue<'a>, rhs: IntValue<'a>) -> IntValue<'a> {
                    self.context.builder.build_int_nsw_add(lhs, rhs, "add")
                }

                fn mul(&mut self, lhs: IntValue<'a>, rhs: IntValue<'a>) -> IntValue<'a> {
                    self.context.builder.build_int_nsw_mul(lhs, rhs, "mul")
                }

                fn div(&mut self, lhs: IntValue<'a>, rhs: IntValue<'a>) -> IntValue<'a> {
                    self.context.builder.build_int_signed_div(lhs, rhs, "div")
                }

                fn rem(&mut self, lhs: IntValue<'a>, rhs: IntValue<'a>) -> IntValue<'a> {
                    self.context.builder.build_int_signed_rem(lhs, rhs, "rem")
                }

                fn cmp(&mut self, lhs: IntValue<'a>, rhs: IntValue<'a>) -> IntValue<'a> {
                    let c =
                        self.context
                            .builder
                            .build_int_compare(IntPredicate::EQ, lhs, rhs, "cmp");
                    self.context
                        .builder
                        .build_int_z_extend(c, self.imm_type, "zext")
                }
            }

            let mut j = Jit {
                context: ctx,
                table: &mut table,
                imm_type,
            };
            // no table since we expect to only be dealing with inlined exprs!
            //
            // if we weren't in a rust we'd make a `InlinedExpr` newtype that's only
            // produced by the inline function that is porcelain over this function
            // which we'd then update to take an option
            // let ret = self.map(None, &mut e);
            let ret = self.map(None, &mut j);
            ctx.builder.build_return(Some(&ret));

            TrustMe(unsafe { ctx.execution_engine.get_function(&fn_name) }.unwrap())
        }
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2021, 24);
    let program: Program = aoc.get_input().parse().unwrap();
    // println!("{} -", program);

    let stages = program.split_at_inputs();
    // stages.iter().for_each(|s| println!("{}\n", s));

    let stages: Vec<Expr> = stages
        .iter()
        .map(|s| {
            let (table, regs) = s.symbolize();
            Expr::get_inlined_expr(&table, regs[&Reg::Z])
        })
        .collect();

    // stages.iter().for_each(|s| println!("{}", s));

    // yields:
    // (       (~z * ((25 * ((((~z % 26) +  11) == in0) == 0)) + 1)) + ((in0 +  3) * ((((~z % 26) +  11) == in0) == 0)))
    // (       (~z * ((25 * ((((~z % 26) +  14) == in0) == 0)) + 1)) + ((in0 +  7) * ((((~z % 26) +  14) == in0) == 0)))
    // (       (~z * ((25 * ((((~z % 26) +  13) == in0) == 0)) + 1)) + ((in0 +  1) * ((((~z % 26) +  13) == in0) == 0)))
    // (((~z / 26) * ((25 * ((((~z % 26) +  -4) == in0) == 0)) + 1)) + ((in0 +  6) * ((((~z % 26) +  -4) == in0) == 0)))
    // (       (~z * ((25 * ((((~z % 26) +  11) == in0) == 0)) + 1)) + ((in0 + 14) * ((((~z % 26) +  11) == in0) == 0)))
    // (       (~z * ((25 * ((((~z % 26) +  10) == in0) == 0)) + 1)) + ((in0 +  7) * ((((~z % 26) +  10) == in0) == 0)))
    // (((~z / 26) * ((25 * ((((~z % 26) +  -4) == in0) == 0)) + 1)) + ((in0 +  9) * ((((~z % 26) +  -4) == in0) == 0)))
    // (((~z / 26) * ((25 * ((((~z % 26) + -12) == in0) == 0)) + 1)) + ((in0 +  9) * ((((~z % 26) + -12) == in0) == 0)))
    // (       (~z * ((25 * ((((~z % 26) +  10) == in0) == 0)) + 1)) + ((in0 +  6) * ((((~z % 26) +  10) == in0) == 0)))
    // (((~z / 26) * ((25 * ((((~z % 26) + -11) == in0) == 0)) + 1)) + ((in0 +  4) * ((((~z % 26) + -11) == in0) == 0)))
    // (       (~z * ((25 * ((((~z % 26) +  12) == in0) == 0)) + 1)) + ((in0 +  0) * ((((~z % 26) +  12) == in0) == 0)))
    // (((~z / 26) * ((25 * ((((~z % 26) +  -1) == in0) == 0)) + 1)) + ((in0 +  7) * ((((~z % 26) +  -1) == in0) == 0)))
    // (((~z / 26) * ((25 * ((((~z % 26) +   0) == in0) == 0)) + 1)) + ((in0 + 12) * ((((~z % 26) +   0) == in0) == 0)))
    // (((~z / 26) * ((25 * ((((~z % 26) + -11) == in0) == 0)) + 1)) + ((in0 +  1) * ((((~z % 26) + -11) == in0) == 0)))

    // takes a long while; yields an expr that's 230+ MB in plaintext
    // let all = {
    //     let (table, regs) = program.symbolize();
    //     println!("symbolized");
    //     Expr::get_inlined_expr(&table, regs[&Reg::Z])
    // };
    // println!("{}", all);

    // three routes forward that I can think of:
    // bin search
    // symbolic solve (z3 or other)
    // codegen (LLVM) + brute force

    // (       ( 0 * ((25 * (((( 0 % 26) +  11) ==   9) == 0)) + 1)) + ((  9 +  3) * (((( 0 % 26) +  11) ==   9) == 0)))
    // 12
    // (       (~z * ((25 * ((((~z % 26) +  14) == in0) == 0)) + 1)) + ((in0 +  7) * ((((~z % 26) +  14) == in0) == 0)))
    // (       (~z * ((25 * ((((~z % 26) +  13) == in0) == 0)) + 1)) + ((in0 +  1) * ((((~z % 26) +  13) == in0) == 0)))
    // (((~z / 26) * ((25 * ((((~z % 26) +  -4) == in0) == 0)) + 1)) + ((in0 +  6) * ((((~z % 26) +  -4) == in0) == 0)))
    // (       (~z * ((25 * ((((~z % 26) +  11) == in0) == 0)) + 1)) + ((in0 + 14) * ((((~z % 26) +  11) == in0) == 0)))
    // (       (~z * ((25 * ((((~z % 26) +  10) == in0) == 0)) + 1)) + ((in0 +  7) * ((((~z % 26) +  10) == in0) == 0)))
    // (((~z / 26) * ((25 * ((((~z % 26) +  -4) == in0) == 0)) + 1)) + ((in0 +  9) * ((((~z % 26) +  -4) == in0) == 0)))
    // (((~z / 26) * ((25 * ((((~z % 26) + -12) == in0) == 0)) + 1)) + ((in0 +  9) * ((((~z % 26) + -12) == in0) == 0)))
    // (       (~z * ((25 * ((((~z % 26) +  10) == in0) == 0)) + 1)) + ((in0 +  6) * ((((~z % 26) +  10) == in0) == 0)))
    // (((~z / 26) * ((25 * ((((~z % 26) + -11) == in0) == 0)) + 1)) + ((in0 +  4) * ((((~z % 26) + -11) == in0) == 0)))
    // (       (~z * ((25 * ((((~z % 26) +  12) == in0) == 0)) + 1)) + ((in0 +  0) * ((((~z % 26) +  12) == in0) == 0)))
    // ((( X / 26) * ((25 * (((( X % 26) +  -1) ==   Y) == 0)) + 1)) + ((  Y +  7) * (((( X % 26) +  -1) ==   Y) == 0)))
    // ((( X / 26) * ((25 * (((( X % 26) +   0) ==   Y) == 0)) + 1)) + ((  Y + 12) * (((( X % 26) +   0) ==   Y) == 0)))
    //
    // ((( X / 26) * ((25 * (((( X % 26) + -11) ==   Y) == 0)) + 1)) + ((  Y +  1) * (((( X % 26) + -11) ==   Y) == 0)))
    // -> -26 < old_z < 26 (inner cond must be false; etc.)

    // (X / 26 * ((25 * (((X % 26) + -11) != Y)) + 1)) = ((Y + 1) * (((X % 26) + -11) != Y))
    // can `(X / 26 * ((25 * (((X % 26) + -11) != Y)) + 1))` be -1?
    //   -26 + -X = X * ((25 * (((X % 26) + -11) != Y)))
    //   `(((X % 26) + -11) != Y)` can be either `0` or `1`
    //   -26 + -X = 25 * X or -26 + -X = 0
    //     -26 + -X = 25 -> X = -51
    //     X = -26

    // let's try: symbolic execution instead of pure symbolic stuff
    //
    // the last stage emits 0 for the following (old_z, inp) values:
    // 12 1
    // 13 2
    // 14 3
    // 15 4
    // 16 5
    // 17 6
    // 18 7
    // 19 8
    // 20 9
    //
    // so next we need to find for what values the previous stage emits 11..=20:
    // 0..=25 * 0..=8 where old_z != inp
    //
    // etc.
    //
    // it's not as satsifying as Solving the thing outright and I'm clearly missing whatever Art is involved that
    // makes it so that picking constant values for those 2 things in each equation makes it so that they accept
    // nice ranges like they do
    //
    // but it'll do

    // // stage => prev_z => input => next_z
    let mut output_maps: Vec<HashMap<isize, HashMap<u8, isize>>> =
        stages.iter().map(|_| HashMap::new()).collect_vec();
    let mut allowed_outputs = FxHashSet::from_iter([0].into_iter());

    // // stage => prev_z => input => next_z
    // let mut output_maps: Vec<DashMap<isize, DashMap<u8, isize>>> =
    //     stages.iter().map(|_| DashMap::new()).collect_vec();
    // // let mut allowed_outputs = FxHashSet::from_iter([0].into_iter());
    // let mut allowed_outputs =
    //     DashSet::<_, BuildHasherDefault<fxhash::FxHasher64>>::from_iter([0].into_iter());

    #[cfg(feature = "llvm")]
    let ctx = inkwell::context::Context::create();
    // #[cfg(feature = "llvm")]
    // let ctx = Box::leak(Box::new());
    #[cfg(feature = "llvm")]
    let fns = stages
        .iter()
        .enumerate()
        .map(|(idx, e)| {
            let ctx = Box::leak(Box::new(llvm::Context::new(&ctx)));
            e.jit(ctx, idx)
        })
        .collect_vec();

    // llvm, parallel:    1m 33s
    // llvm, single:     12m 26s
    // interp, parallel: 21m 55s
    // interp, single:   ...
    //
    // down to 41s with `fnv`
    // 31s with some other cleanup

    for (idx, stage) in stages.iter().enumerate().rev() {
        let mut next_allowed_outputs = FxHashSet::default();
        // let next_allowed_outputs = DashSet::<_, BuildHasherDefault<_>>::default();
        dbg!(&idx);
        let r = stages.len() - idx;
        let r = r.min(6);
        let r = 26isize.pow(r as u32);
        dbg!(r);
        #[cfg(feature = "llvm")]
        let func = fns.get(idx).unwrap();

        let vec: Vec<_> = (-r..=r)
            .into_par_iter()
            // .into_iter()
            .flat_map_iter(move |z| {
                (1..=9).map(move |inp| {
                    #[cfg(not(feature = "llvm"))]
                    let new_z = stage.eval(
                        &HashMap::from_iter([(Reg::Z, z)].into_iter()),
                        &[inp as isize],
                    );

                    #[cfg(feature = "llvm")]
                    let new_z = unsafe { func.call(z, inp as isize) };

                    (z, new_z, inp)
                })
            })
            .filter(|(_, new_z, _)| allowed_outputs.contains(new_z))
            .collect();
        vec.into_iter().for_each(|(z, new_z, inp)| {
            next_allowed_outputs.insert(z);
            output_maps[idx]
                .entry(z)
                .or_insert_with(HashMap::new)
                .insert(inp, new_z);
        });

        // Leaning on concurrent hashmaps is slower!
        /*
        let output_maps_ref = &output_maps;
        let next_allowed_outputs_ref = &next_allowed_outputs;

        (-r..=r)
            .into_par_iter()
            // .into_iter()
            .for_each(move |z| {
                for inp in 1..=9 {
                    #[cfg(not(feature = "llvm"))]
                    let new_z = stage.eval(
                        &HashMap::from_iter([(Reg::Z, z)].into_iter()),
                        &[inp as isize],
                    );

                    #[cfg(feature = "llvm")]
                    let new_z = unsafe { func.call(z, inp as isize) };

                    if allowed_outputs.contains(&new_z) {
                        next_allowed_outputs_ref.insert(z);
                        output_maps_ref[idx]
                            .entry(z)
                            .or_insert_with(DashMap::new)
                            .insert(inp, new_z);
                    }
                }
            }); */

        // for z in -r..=r {
        //     for inp in 1..=9 {
        //         let new_z = stage.eval(
        //             &HashMap::from_iter([(Reg::Z, z)].into_iter()),
        //             &[inp as isize],
        //         );
        //         // dbg!(z, &allowed_outputs);
        //         if allowed_outputs.contains(&new_z) {
        //             next_allowed_outputs.insert(z);
        //             output_maps[idx]
        //                 .entry(z)
        //                 .or_insert_with(HashMap::new)
        //                 .insert(inp, new_z);
        //         }
        //     }
        // }

        allowed_outputs = next_allowed_outputs;
    }

    // dbg!(&output_maps);

    let find_id = |search_func: Box<
        dyn for<'a> Fn(
            Box<dyn Iterator<Item = (&'a u8, &'a isize)>>,
        ) -> Option<(&'a u8, &'a isize)>,
    >| {
        let mut picked = Vec::with_capacity(stages.len());
        let mut z = 0;
        for (idx, map) in output_maps.iter().enumerate() {
            let possible = map
                .iter()
                .filter(move |(old_z, _)| **old_z == z)
                .flat_map(|(_, map)| map.iter());
            // .filter(move |r| *r.key() == z)
            // .flat_map(|r| r.value().iter())
            // .map(|r| r.pair());
            // dbg!(idx, &picked);
            let (inp, next_z) = search_func(Box::new(possible)).unwrap();
            picked.push(*inp);
            z = *next_z;
        }

        picked.iter().join("")
    };

    let p1 = find_id(Box::new(|it| it.max_by_key(|(inp, _next_z)| *inp)));
    let p2 = find_id(Box::new(|it| it.min_by_key(|(inp, _next_z)| *inp)));

    aoc.submit_p1(p1).unwrap();
    aoc.submit_p2(p2).unwrap();
    // dbg!(p1, p2);

    // for each stage pick the largest allowed input and propagate downward accordingly:
    // let mut picked = Vec::with_capacity(stages.len());
    // let mut z = 0;
    // for (idx, map) in output_maps.iter().enumerate() {
    //     let possible = map
    //         .iter()
    //         .filter(|(old_z, _)| **old_z == z)
    //         .flat_map(|(_, map)| map.iter());
    //     dbg!(idx, &picked);
    //     let (inp, next_z) = possible.max_by_key(|(inp, _next_z)| *inp).unwrap();
    //     picked.push(*inp);
    //     z = *next_z;
    // }

    // println!("{:?}", picked);
}
