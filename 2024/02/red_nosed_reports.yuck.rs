use std::cmp::Ordering;

use aoc::*;

#[rustfmt::skip]
fn main() {
    let mut aoc = AdventOfCode::new(2024, 3);
    let inp = aoc.get_input();

    fn diff((a, b): (u8, u8)) -> (Ordering, u8) { (a.cmp(&b), a.abs_diff(b)) }
    let reps = inp.lines().map(|l| {
        l.split_whitespace().map_parse().tuple_windows().map(diff).collect_vec()
    }).collect_vec();
    let count = |f| reps.iter().filter_map(f).count();

    let okay = |p, (n, diff)| (n == p && (1..=3).contains(&diff)).then_some(p);
    let safe = |v: &Vec<(_, _)>| v.iter().copied().try_fold(v[0].0, okay);
    let p1 = count(safe);

    let safe_with_skip = |v| {
        safe(v) ||
    };
}
