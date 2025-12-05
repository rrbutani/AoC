use std::ops::RangeInclusive;

use aoc::{AdventOfCode, Itertools, iterator_map_ext::IterMapExt};

fn main() {
    let mut aoc = AdventOfCode::new(2025, 5);
    let inp = aoc.get_input();
    let (ranges, ingredient_ids): (Vec<RangeInclusive<u64>>, Vec<u64>) = {
        let (ranges, ings) = inp.split_once("\n\n").unwrap();
        let mut ranges = ranges
            .lines()
            .map(|l| {
                let (l, r) = l.split('-').map_parse().collect_tuple().unwrap();
                l..=r
            })
            .collect_vec();
        let ings = ings.lines().map_parse().collect_vec();

        ranges.sort_by_key(|r| *r.start());
        (ranges, ings)
    };

    let p1 = ingredient_ids
        .iter()
        .filter(|i| ranges.iter().any(|r| r.contains(i)))
        .count();
    _ = aoc.submit_p1(p1);

    let mut coalesced_ranges = Vec::with_capacity(ranges.len());
    let mut r = ranges.iter().peekable();
    while let Some(this) = r.next() {
        let (start, mut end) = (*this.start(), *this.end());

        // while there's overlap, coalesce:
        while let Some(next) = r.peek()
            && *next.start() <= end
        {
            let potential_new_end = *r.next().unwrap().end();
            // need to use `max` in case `next` is entirely contained within our current range
            end = end.max(potential_new_end);
        }

        coalesced_ranges.push(start..=end);
    }
    let total_range = coalesced_ranges
        .iter()
        .map(|r| *r.end() + 1 - *r.start())
        .sum::<u64>();
    _ = aoc.submit_p2(total_range);
}
