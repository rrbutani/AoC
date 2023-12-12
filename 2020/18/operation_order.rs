#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::marker::PhantomData;
use std::num::ParseIntError;
use std::ops::{Add, Mul};

trait ParseTy: Debug {}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct RightRecursive; // aka LeftAssociative
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct LeftRecursive; // aka RightAssociative
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct AddThenMultiply;

impl ParseTy for RightRecursive {}
impl ParseTy for LeftRecursive {}
impl ParseTy for AddThenMultiply {}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Expr<T: Mul + Add = usize, D: ParseTy = RightRecursive> {
    Paren {
        inner: Box<Expr<T, D>>,
    },
    Mul {
        lhs: Box<Expr<T, D>>,
        rhs: Box<Expr<T, D>>,
    },
    Add {
        lhs: Box<Expr<T, D>>,
        rhs: Box<Expr<T, D>>,
    },
    Literal(T, PhantomData<D>),
}

// impl<T: Mul + Add, D: ParseTy> Expr<T, D> {
//     // This should compile away to a nop!
//     #[inline]
//     fn cast(self) -> Expr<T, AddThenMultiply> {
//         use Expr::*;
//         match self {
//             Paren { inner } => Paren {
//                 inner: Box::new(inner.cast()),
//             },
//             Mul { lhs, rhs } => Mul {
//                 lhs: Box::new(lhs.cast()),
//                 rhs: Box::new(rhs.cast()),
//             },
//             Add { lhs, rhs } => Add {
//                 lhs: Box::new(lhs.cast()),
//                 rhs: Box::new(rhs.cast()),
//             },
//             Literal(inner, _) => Literal(inner, PhantomData),
//         }
//     }
// }

impl<T: Mul + Add + Display, D: ParseTy> Display for Expr<T, D> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Expr::*;
        match self {
            Paren { inner } => write!(fmt, "({})", inner),
            Mul { lhs, rhs } => write!(fmt, "{} * {}", lhs, rhs),
            Add { lhs, rhs } => write!(fmt, "{} + {}", lhs, rhs),
            Literal(inner, _) => write!(fmt, "{}", inner),
        }
    }
}

impl<T: Mul<Output = T> + Add<Output = T> + Clone + Display + Debug, D: ParseTy> Expr<T, D> {
    fn eval(&self) -> T {
        use Expr::*;
        match self {
            Paren { inner } => inner.eval(),
            Mul { lhs, rhs } => lhs.eval() * rhs.eval(),
            Add { lhs, rhs } => lhs.eval() + rhs.eval(),
            Literal(inner, _) => inner.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ExprParseError {
    UnmatchedBrace,
    InvalidChar(char),
    NumberParseError(ParseIntError),
    UnexpectedEnd,
    MissingOperands,
    ExtraOperands,
}

fn skip_til_matching_paren_rev(iter: &mut impl Iterator<Item = (usize, char)>) -> Option<usize> {
    let mut level = 1;
    loop {
        match iter.next()? {
            (i, '(') => {
                level -= 1;
                if level == 0 {
                    return Some(i);
                }
            }
            (_, ')') => {
                level += 1;
            }
            (_, _) => {}
        }
    }
}

fn parse_rec_right<T: Mul + Add + FromStr<Err = ParseIntError>>(
    s: &str,
    mut it: impl Iterator<Item = (usize, char)>,
    rhs: Expr<T, RightRecursive>,
) -> Result<Expr<T, RightRecursive>, ExprParseError> {
    let res = loop {
        match it.next() {
            None => break rhs,
            Some((_, ' ')) => {}
            Some((op, '*')) => {
                break Expr::Mul {
                    lhs: Box::new(s[..op].parse()?),
                    rhs: Box::new(rhs),
                }
            }
            Some((op, '+')) => {
                break Expr::Add {
                    lhs: Box::new(s[..op].parse()?),
                    rhs: Box::new(rhs),
                }
            }
            Some((_, c)) => {
                return Err(ExprParseError::InvalidChar(c));
            }
        }
    };

    Ok(res)
}

impl<T: Mul + Add + FromStr<Err = ParseIntError>> FromStr for Expr<T, RightRecursive> {
    type Err = ExprParseError;

    fn from_str(s: &str) -> Result<Self, ExprParseError> {
        let len = s.len();
        let mut it = s
            .chars()
            .rev()
            .enumerate()
            .map(|(idx, c)| (len - idx - 1, c));

        let rhs = loop {
            match it.next().ok_or(ExprParseError::UnexpectedEnd)? {
                (end, ')') => {
                    let start = skip_til_matching_paren_rev(&mut it)
                        .ok_or(ExprParseError::UnmatchedBrace)?;

                    break Expr::Paren {
                        inner: Box::new(s[(start + 1)..end].parse()?),
                    };
                }

                (_, ' ') => {}

                (end, '0'..='9') => {
                    let mut start = end;
                    while let Some((idx, '0'..='9')) = it.next() {
                        start = idx;
                    }

                    break Expr::Literal(
                        s[start..=end]
                            .parse()
                            .map_err(ExprParseError::NumberParseError)?,
                        PhantomData,
                    );
                }

                (_, c) => {
                    return Err(ExprParseError::InvalidChar(c));
                }
            }
        };

        parse_rec_right(s, it, rhs)
    }

    //     loop {
    //         match s.next().ok_or(ExprParseError::Empty)? {
    //             (idx, '(') => {
    //                 let mut level = 1;
    //                 let (start, mut stop) = (idx, None);
    //                 while level != 0 {
    //                     match s.next().ok_or(ExprParseError::UnmatchedBrace)? {
    //                         (_, '(') => level += 1,
    //                         (i, ')') => {
    //                             level -= 1;
    //                             stop = Some(i);
    //                         }
    //                     }
    //                 }
    //             }
    //             (_, ' ') => { },
    //             (idx, '*') => {

    //             },
    //             (idx, '+') => {

    //             },
    //             (idx, '0'..='9') => {

    //             }
    //             (_, c) => return Err(ExprParseError::InvalidChar(c)),
    //         }

    //         if let None = s.next() {
    //             todo!()
    //         }
    //     }
    // }
}

fn skip_til_matching_paren(iter: &mut impl Iterator<Item = (usize, char)>) -> Option<usize> {
    let mut level = 1;
    loop {
        match iter.next()? {
            (_, '(') => level += 1,
            (i, ')') => {
                level -= 1;
                if level == 0 {
                    return Some(i);
                }
            }
            (_, _) => {}
        }
    }
}

impl<T: Mul + Add + FromStr<Err = ParseIntError>> FromStr for Expr<T, LeftRecursive> {
    type Err = ExprParseError;

    fn from_str(s: &str) -> Result<Self, ExprParseError> {
        let mut it = s.chars().enumerate();

        let lhs = loop {
            match it.next().ok_or(ExprParseError::UnexpectedEnd)? {
                (start, '(') => {
                    let end =
                        skip_til_matching_paren(&mut it).ok_or(ExprParseError::UnmatchedBrace)?;

                    break Expr::Paren {
                        inner: Box::new(s[(start + 1)..end].parse()?),
                    };
                }

                (_, ' ') => {}

                (start, '0'..='9') => {
                    let mut end = start;
                    while let Some((idx, '0'..='9')) = it.next() {
                        end = idx;
                    }

                    break Expr::Literal(
                        s[start..=end]
                            .parse()
                            .map_err(ExprParseError::NumberParseError)?,
                        PhantomData,
                    );
                }

                (_, c) => {
                    return Err(ExprParseError::InvalidChar(c));
                }
            }
        };

        let res = loop {
            match it.next() {
                None => break lhs,
                Some((_, ' ')) => {}
                Some((op, '*')) => {
                    break Expr::Mul {
                        lhs: Box::new(lhs),
                        rhs: Box::new(s[(op + 1)..].parse()?),
                    }
                }
                Some((op, '+')) => {
                    break Expr::Add {
                        lhs: Box::new(lhs),
                        rhs: Box::new(s[(op + 1)..].parse()?),
                    }
                }
                Some((_, c)) => {
                    return Err(ExprParseError::InvalidChar(c));
                }
            }
        };

        Ok(res)
    }

    //     loop {
    //         match s.next().ok_or(ExprParseError::Empty)? {
    //             (idx, '(') => {
    //                 let mut level = 1;
    //                 let (start, mut stop) = (idx, None);
    //                 while level != 0 {
    //                     match s.next().ok_or(ExprParseError::UnmatchedBrace)? {
    //                         (_, '(') => level += 1,
    //                         (i, ')') => {
    //                             level -= 1;
    //                             stop = Some(i);
    //                         }
    //                     }
    //                 }
    //             }
    //             (_, ' ') => { },
    //             (idx, '*') => {

    //             },
    //             (idx, '+') => {

    //             },
    //             (idx, '0'..='9') => {

    //             }
    //             (_, c) => return Err(ExprParseError::InvalidChar(c)),
    //         }

    //         if let None = s.next() {
    //             todo!()
    //         }
    //     }
    // }
}

impl<T: Mul + Add + FromStr<Err = ParseIntError>> FromStr for Expr<T, AddThenMultiply> {
    type Err = ExprParseError;

    // The Shunting-Yard Algorithm (just hardcoding left associativity):
    //
    // when you come across an operator with _lower_ (or equal) precedence
    // compared to the one on the top of the stack, "apply" the operator on the
    // top of the opertator stack
    fn from_str(s: &str) -> Result<Self, ExprParseError> {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
        enum BinOps {
            Mul = 0,
            Add = 1,
        }

        impl BinOps {
            fn to_expr<T: Mul + Add, D: ParseTy>(
                &self,
                stack: &mut Vec<Expr<T, D>>,
            ) -> Option<Expr<T, D>> {
                if stack.len() < 2 {
                    return None;
                }

                let rhs = Box::new(stack.pop()?);
                let lhs = Box::new(stack.pop()?);

                Some(match self {
                    Self::Mul => Expr::Mul { lhs, rhs },
                    Self::Add => Expr::Add { lhs, rhs },
                })
            }

            fn to_expr_inplace<T: Mul + Add, D: ParseTy>(
                &self,
                stack: &mut Vec<Expr<T, D>>,
            ) -> Result<(), ()> {
                let n = self.to_expr(stack).ok_or(())?;
                stack.push(n);
                Ok(())
            }
        }

        let mut op_stack = Vec::new();
        let mut expr_stack: Vec<Self> = Vec::new();

        fn push_bin_op<T: Mul + Add, D: ParseTy>(
            op: BinOps,
            exprs: &mut Vec<Expr<T, D>>,
            ops: &mut Vec<BinOps>,
        ) -> Result<(), ExprParseError> {
            // If there's an operator of equal or greater precedence in the
            // operator stack, it's time to pop it:
            while ops.last().map(|l| l >= &op).unwrap_or(false) {
                ops.pop()
                    .unwrap()
                    .to_expr_inplace(exprs)
                    .map_err(|()| ExprParseError::MissingOperands)?;
            }

            ops.push(op);
            Ok(())
        }

        let mut it = s.chars().enumerate();
        while let Some(n) = it.next() {
            match n {
                (_, ' ') => {}
                (start, '(') => {
                    let end =
                        skip_til_matching_paren(&mut it).ok_or(ExprParseError::UnmatchedBrace)?;
                    expr_stack.push(s[(start + 1)..end].parse()?);
                }
                (_, '*') => push_bin_op(BinOps::Mul, &mut expr_stack, &mut op_stack)?,
                (_, '+') => push_bin_op(BinOps::Add, &mut expr_stack, &mut op_stack)?,
                (start, '0'..='9') => {
                    let mut end = start;
                    while let Some((idx, '0'..='9')) = it.next() {
                        end = idx;
                    }

                    expr_stack.push(Expr::Literal(
                        s[start..=end]
                            .parse()
                            .map_err(ExprParseError::NumberParseError)?,
                        PhantomData,
                    ));
                }
                (_, c) => return Err(ExprParseError::InvalidChar(c)),
            }
        }

        while let Some(op) = op_stack.pop() {
            op.to_expr_inplace(&mut expr_stack)
                .map_err(|()| ExprParseError::MissingOperands)?;
        }

        match expr_stack.len() {
            0 => Err(ExprParseError::UnexpectedEnd),
            1 => Ok(expr_stack.drain(..).next().unwrap()),
            _ => Err(ExprParseError::ExtraOperands),
        }
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2020, 18);
    let input: String = aoc.get_input();

    fn sum<D: ParseTy>(input: &str) -> usize
    where
        Expr<usize, D>: FromStr<Err = ExprParseError>,
    {
        input
            .lines()
            .map(|e| e.parse::<Expr<usize, D>>().unwrap())
            .map(|e| e.eval())
            .sum()
    }

    aoc.submit_p1(sum::<RightRecursive>(&input)).unwrap();
    aoc.submit_p2(sum::<AddThenMultiply>(&input)).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! right_rec {
        ($nom:ident: $exp:literal => $val:literal) => {
            #[test]
            fn $nom() {
                let text = $exp;
                let expr = text.parse::<Expr>().unwrap();
                println!("{} vs {}", text, expr);

                assert_eq!(expr.eval(), $val);
            }
        };
    }

    right_rec!(t1: "2 * 3 + (4 * 5)" => 26);
    right_rec!(t2: "5 + (8 * 3 + 9 + 3 * 4 * 3)" => 437);
    right_rec!(t3: "5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))" => 12_240);
    right_rec!(t4: "((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2" => 13_632);

    macro_rules! add_then_mul {
        ($nom:ident: $exp:literal => $val:literal) => {
            #[test]
            fn $nom() {
                let text = $exp;
                let expr = text.parse::<Expr<usize, AddThenMultiply>>().unwrap();
                println!("{} vs {}", text, expr);

                assert_eq!(expr.eval(), $val);
            }
        };
    }

    add_then_mul!(r1: "1 + (2 * 3) + (4 * (5 + 6))" => 51);
    add_then_mul!(r2: "2 * 3 + (4 * 5)" => 46);
    add_then_mul!(r3: "5 + (8 * 3 + 9 + 3 * 4 * 3)" => 1445);
    add_then_mul!(r4: "5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))" => 669_060);
    add_then_mul!(r5: "((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2" => 23_340);
}

// 5 + 9 * (12 * 4)
// Mul ( Literal (9), Paren ( ... ) )

// ((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2
// Paren: (2 + 4 * 9) * (6 + 9 * 8 + 6) + 6
//   Paren: 2 + 4 * 9
//     2                  |
//     2                  | +
//     2 4                | +
//     (2 + 4)            | *
//     (2 + 4) 9          | *
//     ((2 + 4) * 9)      |
//   ((2 + 4) * 9)                               | *
//   Paren: 6 + 9 * 8 + 6
//     6                  |
//     6                  | +
//     6 9                | +
//     (6 + 9)            | *
//     (6 + 9) 8          | *
//     (6 + 9) 8          | * +
//     (6 + 9) 8 6        | * +
//     (6 + 9) (8 + 6)    | *
//     ((6 + 9) * (8 + 6))|
//   ((2 + 4) * 9) ((6 + 9) * (8 + 6))           | *
//   ((2 + 4) * 9) ((6 + 9) * (8 + 6))           | * +
//   ((2 + 4) * 9) ((6 + 9) * (8 + 6)) 6         | * +
//   ((2 + 4) * 9) (((6 + 9) * (8 + 6)) + 6)     | *
//   (((2 + 4) * 9) * (((6 + 9) * (8 + 6)) + 6)) |
// (((2 + 4) * 9) * (((6 + 9) * (8 + 6)) + 6))                  | +
// (((2 + 4) * 9) * (((6 + 9) * (8 + 6)) + 6)) 2                | +             // We're commutative so we could go
// ((((2 + 4) * 9) * (((6 + 9) * (8 + 6)) + 6)) + 2)            | +             // either way here but let's just
// ((((2 + 4) * 9) * (((6 + 9) * (8 + 6)) + 6)) + 2) 4          | +             // keep things simple and be left
// (((((2 + 4) * 9) * (((6 + 9) * (8 + 6)) + 6)) + 2) + 4)      | *             // associative.
// (((((2 + 4) * 9) * (((6 + 9) * (8 + 6)) + 6)) + 2) + 4) 2    | *
// ((((((2 + 4) * 9) * (((6 + 9) * (8 + 6)) + 6)) + 2) + 4) * 2)|
