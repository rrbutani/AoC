#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::fmt::{self, Display};
use std::{convert::TryInto, fmt::Debug};

// ((a * N) + 0) == ((b * N) + 1) == ((c * N) + 2) == ... == ((z * N) + S)
//
// solve for N

// alternatively:
//
//  x      ≡ 0 (mod a)
// (x + 1) ≡ 0 (mod b)
// (x + 2) ≡ 0 (mob c)
// etc.
//
// we can rewrite this as:
//
// x ≡      0  (mod a)
// x ≡ (b - 1) (mod b)
// x ≡ (c - 2) (mod c)
// etc. while making sure that (X - i) is still in (0..S)
//
// then this becomes a CRT problem which we can solve

fn main() {
    let mut aoc = AdventOfCode::new(2020, 13);
    let input: String = aoc.get_input();

    let mut lines = input.lines();
    let earliest = lines.next().unwrap().parse::<usize>().unwrap();
    let buses = lines
        .next()
        .unwrap()
        .split(',')
        .map(|l| l.parse::<usize>().ok());

    let next = buses
        .clone()
        .filter_map(|b| b)
        .map(|b| {
            (
                ((((earliest as f64 / b as f64).ceil() as usize) * b) - earliest),
                b,
            )
        })
        .min()
        .unwrap();
    let p1 = next.0 * next.1;
    println!("{}", p1);
    let _ = aoc.submit_p1(p1);

    // There's probably a better way to do this...
    // let mut p2 = None;
    // for i in 0.. {
    //     let mut works = true;
    //     for (offset, id) in buses.clone().enumerate().filter(|(_, b)| b.is_some()) {
    //         if (i + offset) % id.unwrap() != 0 {
    //             works = false;
    //             break;
    //         }
    //     }

    //     if works {
    //         p2 = Some(i); break;
    //     }
    // }
    // let p2 = p2.unwrap();
    // println!("{}", p2);
    // let _ = aoc.submit_p2(p2);

    fn modulo(n: isize, b: usize) -> usize {
        let b: isize = b.try_into().unwrap();

        TryInto::<usize>::try_into(((n % b) + b) % b).unwrap()
    }

    // for (i, b) in buses
    //     .clone()
    //     .enumerate()
    //     .filter_map(|(i, b)| b.map(|b| (i, b)))
    // {
    //     // println!("(N + {}) \t ≡ 0 (mod {})", i, b);
    //     println!("N ≡ {} (mod {})", modulo(b as isize - i as isize, b), b);
    //     // println!("");
    // }

    let congruences = buses
        .clone()
        .enumerate()
        .filter_map(|(i, b)| b.map(|b| (i, b)))
        .map(|(idx, b)| (modulo(b as isize - idx as isize, b), b))
        .inspect(|(rem, m)| println!("N ≡ {:4} (mod {:4})", rem, m));

    // let (rem, modulo) = congruences.next().unwrap();
    // let eq = Congruence {
    //     mul: 1,
    //     add: 0,
    //     rem,
    //     modulo
    // };

    // println!("{}", eq);

    // let (residues, moduli): (Vec<_>, Vec<_>) = congruences.unzip();
    // let p2 = solve((&residues, &moduli)).unwrap();
    let congruences: Vec<_> = congruences.collect();

    let p2 = solve(&congruences).unwrap();
    println!("{}", p2);
    let _ = aoc.submit_p2(p2);
}

/// Extended Euclidean Algorithm
///
/// Returns (gcd, x, y) such that: `a*x + b*y = gcd`.
///
/// Lifted nearly verbatim from [here][wp].
///
/// [wp]: https://en.wikipedia.org/wiki/Extended_Euclidean_algorithm#Example
//
// The recursive version actually stack overflows without `--release`!
#[allow(clippy::clippy::many_single_char_names)]
fn extended_gcd(a: usize, b: usize) -> (usize, isize, isize) {
    let (mut x, mut old_x) = (0isize, 1isize);
    let (mut r, mut old_r) = (b, a);

    while r != 0 {
        let quot = old_r / r;

        let (n_old_r, n_r) = (r, old_r - quot * r);
        let (n_old_x, n_x) = (x, old_x - quot.to::<isize>() * x);

        x = n_x;
        old_x = n_old_x;
        r = n_r;
        old_r = n_old_r;
    }

    let gcd = old_r;
    let x = old_x;

    let y = (r.to::<isize>() - x * a.to::<isize>())
        .checked_div(b.to())
        .unwrap_or(0);

    (gcd, x, y)
}

fn gcd(a: usize, b: usize) -> usize {
    extended_gcd(a, b).0
}

fn modular_inverse(mul: usize, modulo: usize) -> Option<usize> {
    let (gcd, x, _) = extended_gcd(mul, modulo);

    if gcd != 1 {
        return None;
    }

    let modulo = modulo.to::<isize>();

    // The second `+ modulo % modulo` is so we get a positive number.
    Some(((x % modulo + modulo) % modulo).to())
}

/// `inp` is an slice containing (residue, modulo) pairs.
///
/// Essentially lifted from [here][rc].
///
/// [rc]: https://rosettacode.org/wiki/Chinese_remainder_theorem
fn solve(inp: &[(usize, usize)]) -> Result<usize, ()> {
    let residues = inp.iter().map(|(r, _)| *r);
    let moduli = inp.iter().map(|(_, m)| *m);

    // The moduli must be relatively prime.
    //
    // (in our case I think they're all _actually_ prime which makes sense since
    // that probably makes these inputs easier to generate)
    if moduli.clone().fold(1, gcd) != 1 {
        return Err(());
    }

    let product: usize = moduli.clone().product();
    let sum: usize = residues
        .zip(moduli)
        .map(|(r, m)| {
            let div = product / m;
            r * modular_inverse(div, m).unwrap() * div
        })
        .sum();

    Ok(sum % product)
}

// type Signed = isize;
// type S = Signed;
// type Unsigned = usize;
// type U = Unsigned;

// // fn modulo

// fn gcd(a: U, b: U) -> U {
//     match b > a {
//         true => gcd_inner(b, a),
//         false => gcd_inner(a, b),
//     }
// }

// // Euclidean Algorithm
// //
// // a > b
// fn gcd_inner(a: U, b: U) -> U {
//     match a % b {
//         0 => b,
//         rem => gcd_inner(b, rem),
//     }
// }

// // returns (gcd, x, y)
// // s.t.: a*x + b*y = gcd(x, y)
// fn egcd(a: S, b: S) -> (S, S, S) {
//     if a == 0 {
//         (b, 0, 1)
//     } else {
//         // Note that we're "swapping" a and b here; once we stop (i.e. once the
//         // remainder hits 0), we'll stop and return the other number.
//         let (g, x, y) = egcd(b % a, a);

//         // On the way back up we substitute back in:
//         (g, y - (b / a) * x, x)
//     }
// }

// fn modular_inverse(mul: S, modulo: S) -> Option<S> {
//     // // mul * x ≡ 1 (mod modulo)
//     // // (mul % modulo) * x ≡ 1 (mod modulo)
//     // // (mul % modulo) * x = 1 + modulo     # We add modulo so that we'll have

//     let (gcd, x, _) = egcd(mul as isize, modulo as isize);
//     if gcd != 1 {
//         dbg!(mul, modulo);
//         return None;
//     }

//     Some(((x % (modulo as isize) + (modulo as isize)) % (modulo as isize)) as isize)
// }

// // CRT
// fn solve(inp: &[(usize, usize)]) -> Result<usize, ()> {
//     let residues = inp.iter().map(|(r, _)| *r);
//     let moduli = inp.iter().map(|(_, m)| *m);

//     // The moduli must be relatively prime.
//     //
//     // (in our case I think they're all _actually_ prime which makes sense since
//     // that probably makes these inputs easier to generate)
//     if moduli.clone().fold(1, gcd) != 1 {
//         return Err(());
//     }

//     let p: usize = moduli.clone().product();
//     let divided = moduli.clone().map(|m| p / m);
//     let inverted = divided
//         .clone()
//         .zip(moduli.clone())
//         .map(|(d, m)| modular_inverse(d, m).unwrap());

//     let sum = residues
//         .zip(divided)
//         .zip(inverted)
//         .map(|((r, d), i)| r * d * i)
//         .sum::<usize>();

//     Ok(sum % p)
// }

// #[derive(Clone, Debug, PartialEq, Eq)]
// struct Congruence {
//     mul: U,
//     add: U,
//     rem: U,
//     modulo: U,
// }

// impl Congruence {
//     fn simplify(&mut self) {
//         // S = mul
//         // A = add
//         // R = rem
//         // M = modulo
//         // S * k + A ≡ R (mod M)
//         //
//         // into:
//         // 1 * k + 0 ≡ R (mod M)

//     }

//     fn solve(&self, (rem, modulo): (U, U)) -> Result<Self, ()> {
//         // S = mul
//         // A = add
//         // R = rem
//         // M = modulo
//         // S * k + A ≡ R (mod M)

//         // Need to be in:
//         // 1 * k + 0 ≡ R (mod M)
//         // form to continue:
//         if self.
//     }
// }

// impl Display for Congruence {
//     fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(fmt, "{}x + {} ≡ {} (mod {})", self.mul, self.add, self.rem, self.modulo)
//     }
// }
