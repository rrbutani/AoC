use std::{cmp::Ordering::*, ops::RangeInclusive};

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
        let mut ings = ings.lines().map_parse().collect_vec();

        ranges.sort_by_key(|r| *r.start());
        ings.sort();

        (ranges, ings)
    };

    // walk the sorted arrays together:
    let mut valid_ingredient_count = 0;
    let mut r = ranges.iter().peekable();
    'ing_loop: for ing in ingredient_ids {
        // skip ranges above this ingredient id:
        loop {
            let Some(range) = r.peek() else {
                // if we're out of ranges the ingredient id and all greater ids aren't valid
                break 'ing_loop;
            };
            match range.start().cmp(&ing) {
                Less => match range.end().cmp(&ing) {
                    Greater | Equal => break, // valid!
                    Less => {
                        // we've moved past this range; move to the next one and try again
                        _ = r.next().unwrap();
                    }
                },
                Equal => break,                // valid!
                Greater => continue 'ing_loop, // this ingredient isn't valid, move to the next
            }
        }

        valid_ingredient_count += 1;
    }
    _ = aoc.submit_p1(valid_ingredient_count);

    let mut total_range = 0;
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

        total_range += end + 1 - start;
    }
    _ = aoc.submit_p2(total_range);
}
