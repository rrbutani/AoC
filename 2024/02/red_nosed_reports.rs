use std::cmp::Ordering;

use aoc::*;

const INP: &'static str = "\
7 6 4 2 1
1 2 7 8 9
9 7 6 2 1
1 3 2 4 5
8 6 4 4 1
1 3 6 7 9
9 1 2
";

fn is_safe<const DAMPEN: bool>(mut levels: impl Iterator<Item = isize> + Clone) -> bool {
    let Some(first) = levels.next() else { return false };
    is_safe_inner(levels, first, None, DAMPEN)
}

fn is_safe_inner(
    mut levels: impl Iterator<Item = isize> + Clone,
    mut curr: isize,
    mut prev: Option<isize>,
    try_omitting: bool,
) -> bool {
    use Ordering::*;

    let mut prev_prev = None;
    while let Some(next) = levels.next() {
        match (next - curr, prev.map(|p| curr.cmp(&p)).unwrap_or(Equal)) {
            (1..=3, Greater | Equal) | (-3..=-1, Less | Equal) => {
                (prev_prev, prev, curr) = (prev, Some(curr), next);
            }
            _ if try_omitting => {
                // do search?
                //
                // upon hitting a Contradiction, attempt to:
                //   - skip the next number (i.e. `1 2 [7] 3`)
                //   - skip the current number (i.e. `9 [1] 2`)
                //   - skip the previous number (i.e. `2 1 [2] 3 4`)
                //     + TODO! we didn't handle this one...
                //
                // I believe we should never have to backtrack further than
                // that...

                let levels_copy = levels.clone();

                // try with skipping the next number:
                if is_safe_inner(levels, curr, prev, false) {
                    return true;
                }

                // if that didn't work, try skipping the current number:
                //
                // note that we can't go further backwards (since we have no way
                // to set `prev_prev` in this invocation) but that's fine: we'll
                // never attempt to omit more than 1 number:
                match prev {
                    // i.e. `curr` wasn't the first number...
                    //
                    // yuck; can't prepend `next` to `levels_copy` because... type
                    // recursion!
                    //
                    // just... handle the first element (next) out-of-band I
                    // guess? (duplicating logic...)
                    Some(curr) => {
                        // return is_safe_inner(levels_with_curr, curr, prev_prev, false)

                        let prev = prev_prev;
                        let (prev, curr) =
                            match (next - curr, prev.map(|p| curr.cmp(&p)).unwrap_or(Equal)) {
                                (1..=3, Greater | Equal) | (-3..=-1, Less | Equal) => {
                                    (Some(curr), next)
                                }
                                _ => return false,
                            };
                        return is_safe_inner(levels_copy, curr, prev, false);
                    }

                    // uh-oh, `curr` was the first number...
                    //
                    // to skip it we must do some munging (first val gets
                    // handled specially):
                    None => {
                        assert!(prev_prev.is_none());
                        return is_safe_inner(levels_copy, next, prev_prev, false);
                    }
                }
            }
            // note: 0 means not increasing or decreasing
            _ => return false,
        }
    }
    true
}

// fn is_safe_inner(levels: impl Iterator<Item = isize>, mut prev_cmp: Ordering) -> bool {
//     use Ordering::*;
//     levels.tuple_windows().all(|(a, b)| {
//         match (b - a, prev_cmp) {
//             (1..=3, Greater | Equal) => {
//                 prev_cmp = Greater;
//                 true
//             }
//             (-3..=-1, Less | Equal) => {
//                 prev_cmp = Less;
//                 true
//             }
//             // 0 means not increasing or decreasing
//             _ => false,
//         }
//     })
// }

fn is_safe_with_dampener_slow_and_bad(levels: impl Iterator<Item = isize> + Clone) -> bool {
    let ret = is_safe_with_dampener_slow_and_bad_inner(levels.clone());

    if ret != is_safe::<true>(levels.clone()) {
        eprintln!("for {:?} expected {ret}, got {}", levels.collect_vec(), !ret);
    }

    ret
}

fn is_safe_with_dampener_slow_and_bad_inner(levels: impl Iterator<Item = isize>) -> bool {
    let levels = levels.collect_vec();

    if is_safe::<false>(levels.iter().copied()) {
        return true;
    }

    for i in 0..levels.len() {
        if is_safe::<false>(
            levels
                .iter()
                .copied()
                .enumerate()
                .filter(|&(idx, _)| idx != i)
                .map(|(_, n)| n),
        ) {
            return true;
        }
    }

    return false;
}

fn main() {
    let mut aoc = AdventOfCode::new(2024, 2);
    let inp = aoc.get_input();
    // let inp = INP;

    let report = inp
        .lines()
        .map(|l| l.split_whitespace().map_parse::<isize>());

    let p1 = report.clone().map(is_safe::<false>).filter(|&b| b).count();
    _ = aoc.submit_p1(p1);

    // let p2 = report.map(is_safe::<true>).filter(|&b| b).count();
    let p2 = report
        .map(is_safe_with_dampener_slow_and_bad)
        .filter(|&b| b)
        .count();
    _ = aoc.submit_p2(p2);
}
