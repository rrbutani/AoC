use crate::TryConvert;

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
pub fn extended_gcd(a: usize, b: usize) -> (usize, isize, isize) {
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

pub fn gcd(a: usize, b: usize) -> usize {
    extended_gcd(a, b).0
}

pub fn lcm(a: usize, b: usize) -> usize {
    (a * b) / gcd(a, b)
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

// TODO: modulo (like python)
// TODO: crt
