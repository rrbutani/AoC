use aoc::*;
use grid::{Direction, SearchCtx as Ctx, SearchNext as Next, TwoDimensionalGrid};
use rayon::iter::ParallelIterator;

fn count_paths(c: &mut usize) -> impl FnMut(Ctx<u8>, Option<()>) -> Next + '_ {
    |ctx, _| match ctx.cell().checked_sub(1) {
        // stop if we've reached 0, increase the count
        None => {
            *c += 1;
            None
        }
        // continue in all directions where the height = `n - 1`
        Some(lower) => Some(
            Direction::ALL
                .into_iter()
                .filter_map(|d| ctx.apply(d))
                .filter(|&c| ctx.grid()[c] == lower)
                .map(|c| (c, ()))
                .collect(),
        ),
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2024, 10);
    let inp = aoc.get_input();
    let map = TwoDimensionalGrid::<u8>::from_str(&inp).unwrap();

    // note: uphill requirement (non-decreasing) means no cycles
    // gonna do the naïve thing: iterate over `9`s, count reachable `0`s
    let distinct_start_end_count = map
        .par_find(|&p| p == 9) // NOTE: parallelization doesn't help here..
        .map(|peak| {
            let mut bases = 0;
            map.floodfill([peak], count_paths(&mut bases));
            bases
        })
        .sum::<usize>();
    _ = aoc.submit_p1(distinct_start_end_count);

    // ... probably not the best way to do this but: short paths so whatever
    let mut paths = 0;
    map.search::<_, _, _, true>(map.find(|&p| p == 9), count_paths(&mut paths));
    _ = aoc.submit_p2(paths);
}
