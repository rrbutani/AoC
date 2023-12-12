use aoc::{iterator_map_ext::IterMapExt, AdventOfCode, Itertools};

// 1 5 7 10 11
//  4 2 3  1
//  -2 1 -2
//    3 -3
//     -6

// 10 13 16 21 30 45 68
//
// 10 + 3x + 0x^2 + 2*x^3

// (n + 1)
// ∫ (n + 1)
// = 0.5n² + n + c
//
// o(1) = 1 | 0 + 0 + 1
// o(1) = 3 | 1 + 1 + 1
// o(2) = 6 | 4 + 2 + 1 :(

// ∫ 1
// = n + C
// d/dx(1.5) = 2
// d/dx(2.5) = 3
//
// C = 0.5

// ∫ n + 0.5
// = n²/2 + 0.5n + C
// o(1) = 1 = 0
// o(2) = 3 = 0

////////////////////////////////////////////////

// [0] 10  13  16  21  30  45  68
// [1]    3   3   5   9  15  23
// [2]      0   2   4   6   8
// [3]        2   2   2   2
// [4]          0   0   0

// eq3(n) = 2
//
// eq2(n) = ∫ 2
// eq2(n) = 2n + C
// eq2(2) = 0 (2 * 2 - 4)
// eq2(3) = 2 (2 * 3 - 4)
// C = -4
// eq2(n) = 2n - 4
//
// eq1(n) = ∫ 2n - 4
// eq1(n) = n² - 4n + C
// eq1(1.5) = 3 (2.25 - 6 + C)
// eq1(2.5) = 3 (6.25 - 10 + C) //
// eq1(3.5) = 5 (6.25 - 10 + C) // 35/3,
// C = 6.75
// eq1(n) = n² - 4n + 6.75
//
// eq0(n) = ∫ n² - 4n + 6.75
// eq0(n) = 1/3*n³ - 2n² + 6.75n + C
// eq0(1) = 10 (1/3 - 2 + 6.75 + C)
// eq0(2) = 13 (8/3 - 8 + 13.5 + C)

// n^2 - 4n + (20/3) =

// nevermind, the above doesn't work because we don't know what point at which
// the derivatives correspond? .5 seems to have just worked by accident for the
// first example?
//
// not sure

/////

// lagrange polynomials..
// TODO(aoc2023)!

// out of time; going to just do it the naïve way...

fn find_next(inp: impl Iterator<Item = isize> + Clone) -> isize {
    if inp.clone().all(|i| i == 0) {
        0
    } else {
        let diffs = inp
            .clone()
            .tuple_windows()
            .map(|(a, b)| b - a)
            .collect_vec();
        find_next(diffs.into_iter()) + inp.last().unwrap()
    }
}

const INP: &str = "0 3 6 9 12 15
1 3 6 10 15 21
10 13 16 21 30 45";

fn main() {
    let mut aoc = AdventOfCode::new(2023, 9);
    let inp = aoc.get_input();
    // let inp = INP;
    let histories = inp
        .lines()
        .map(|l| l.split_whitespace().map_parse::<isize>().collect_vec())
        .collect_vec();

    let p1: isize = histories.iter().map(|h| find_next(h.iter().copied())).sum();
    _ = aoc.submit_p1(p1);

    let p2: isize = histories
        .iter()
        .map(|h| find_next(h.iter().rev().copied()))
        .sum();
    _ = aoc.submit_p2(p2);
}
