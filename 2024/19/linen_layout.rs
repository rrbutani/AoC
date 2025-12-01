use std::collections::HashMap;

use aoc::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Design<'p> {
    pattern: &'p str,
}

// impl FromStr for Design {
//     type Err = ();

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         debug_assert!(s.chars().all(|c| c.is_ascii_lowercase()));
//         Ok(Self { pattern: s.to_string() })
//     }
// }

impl<'p> Design<'p> {
    #[cfg(any())]
    fn using_towels<'t>(&'p self, towels: &'t [Design]) -> usize {
        let mut memo = HashMap::new(); // sharing across calls doesn't help much

        // greedy algorithm; try biggest first and keep going; O(N^len(design))
        fn inner<'t, 'p>(
            memo: &mut HashMap<*const u8, usize>,
            target: &'p str,
            towels: &'t [Design<'t>],
        ) -> usize {
            if target.is_empty() {
                return 1;
            }
            if let Some(&count) = memo.get(&target.as_ptr()) {
                return count;
            }
            let mut count = 0;

            // count += towels
            //     .iter()
            //     .map(|t| {
            //         target
            //             .strip_suffix(&t.pattern)
            //             .map(|rest| inner(memo, rest, &towels))
            //             .unwrap_or_default()
            //     })
            //     .sum::<usize>();

            for t in towels {
                if let Some(rest) = target.strip_suffix(&t.pattern) {
                    count += inner(memo, rest, towels);
                }
            }

            memo.insert(target.as_ptr(), count);
            count
        }

        inner(&mut memo, &self.pattern, &towels)
    }

    // #[cfg(any())]
    fn using_towels<'t>(&'p self, towels: &'t [Design]) -> usize {
        let mut memo = vec![None; self.pattern.len() + 1];

        // greedy algorithm; try biggest first and keep going; O(N^len(design))
        fn inner<'t, 'p>(
            memo: &mut Vec<Option<usize>>,
            target: &'p str,
            towels: &'t [Design<'t>],
        ) -> usize {
            if target.is_empty() {
                return 1;
            }
            if let Some(count) = memo[target.len()] {
                return count;
            }
            let mut count = 0;

            // count += towels
            //     .iter()
            //     .map(|t| {
            //         target
            //             .strip_suffix(&t.pattern)
            //             .map(|rest| inner(memo, rest, &towels))
            //             .unwrap_or_default()
            //     })
            //     .sum::<usize>();

            for t in towels {
                if let Some(rest) = target.strip_suffix(&t.pattern) {
                    count += inner(memo, rest, towels);
                }
            }

            memo[target.len()] = Some(count);
            count
        }

        inner(&mut memo, &self.pattern, &towels)
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2024, 19);
    let inp = aoc.get_input();
    let (towels, designs) = inp.split_once("\n\n").unwrap();
    // let towels = towels.split(", ").map_parse::<Design>().collect_vec();
    // let designs = designs.lines().map_parse::<Design>().collect_vec();
    let towels = towels
        .split(", ")
        .map(|s| Design { pattern: s })
        .collect::<Vec<_>>();
    let designs = designs
        .lines()
        .map(|s| Design { pattern: s })
        .collect::<Vec<_>>();

    let towels = {
        // sort by length (helps a bit)
        let mut towels = towels;
        towels.sort_unstable_by_key(|d| d.pattern.len());
        // towels.reverse();
        towels
    };

    let arrangements: Vec<_> = designs
        .par_iter()
        .map(|d| d.using_towels(&towels))
        .collect();

    _ = aoc.submit_p1(arrangements.iter().filter(|&&c| c != 0).count());
    _ = aoc.submit_p2(arrangements.iter().sum::<usize>());

    // dbg!(arrangements.iter().filter(|&&c| c != 0).count());
    // dbg!(arrangements.iter().sum::<usize>());
}

// TODO: make an actually fast version with automata?
