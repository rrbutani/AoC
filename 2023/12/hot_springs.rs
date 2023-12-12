#![allow(unstable_name_collisions)]

use std::{collections::HashMap, num::NonZeroUsize};

use aoc::{iterator_map_ext::IterMapExt, AdventOfCode, FromStr, Itertools};
use rayon::prelude::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumString, strum::Display,
)]
enum State {
    #[strum(serialize = ".")]
    Operational,
    #[strum(serialize = "#")]
    Damaged,
    #[strum(serialize = "?")]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ConditionRecord {
    records: Vec<State>,
    counts: Vec<NonZeroUsize>,
}

impl FromStr for ConditionRecord {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (records, counts) = s.split_once(' ').unwrap();

        Ok(Self {
            records: records.split_inclusive(|_| true).map_parse().collect(),
            counts: counts.split(',').map_parse().collect(),
        })
    }
}

const DEBUG: bool = false;
macro_rules! dprintln {
    ($($tt:tt)*) => { if DEBUG { eprintln!($($tt)*); } };
}
macro_rules! log {
    // TODO: support trailing comma after `$rest`...
    ($level:ident; $fmt:literal $(, $($rest:tt)+)?) => {
        dprintln!(std::concat!("{_pad:_indent$}", $fmt), $($($rest)+,)? _pad = "", _indent = $level * 2)
    };
}

#[allow(unused)]
fn fmt_states(states: &[State], counts: &[NonZeroUsize]) -> String {
    format!(
        "{} — {}",
        states.iter().map(|s| s.to_string()).collect_vec().join(""),
        counts.iter().map(|s| s.to_string()).collect_vec().join(",")
    )
}

// Perf notes:
//  - as of this writing, with parallelization, this solution takes ~100ms to
//    run: a significant portion of our total 1s budget
//  - maybe we should be going backwards?
//    + actually no; that doesn't help; you can still find out late that a path
//      doesn't work

impl ConditionRecord {
    fn find_arrangements<'r: 'hm, 'hm>(
        &'r self,
        shared_table: Option<&'hm mut HashMap<(&'r [State], &'r [NonZeroUsize]), usize>>,
        use_local_table_as_fallback: bool,
    ) -> usize {
        #[allow(unused)]
        fn try_arrangement_memoized<'r: 'hm, 'hm>(
            table: &'hm mut HashMap<(&'r [State], &'r [NonZeroUsize]), usize>,
            records: &'r [State],
            counts: &'r [NonZeroUsize],
            level: usize,
        ) -> usize {
            let evaluate = |table: &mut HashMap<_, _>| {
                let table = &mut *table; // reborrow
                try_arrangement_inner(records, counts, level, |r, c, l| {
                    try_arrangement_memoized(table, r, c, l)
                })
            };

            // // don't hash large inputs:
            // if records.len() > 200 {
            //     return evaluate(table);
            // }
            // NOTE: doesn't help

            if let Some(&res) = table.get(&(records, counts)) {
                res
            } else {
                let res = evaluate(table);
                table.insert((records, counts), res);

                res
            }
        }

        #[allow(unused)]
        fn try_arrangement(records: &[State], counts: &[NonZeroUsize], level: usize) -> usize {
            try_arrangement_inner(records, counts, level, |r, c, l| try_arrangement(r, c, l))
        }

        fn try_arrangement_inner<'r>(
            mut remaining_records: &'r [State],
            remaining_counts: &'r [NonZeroUsize],
            level: usize,
            mut recurse: impl FnMut(&'r [State], &'r [NonZeroUsize], usize) -> usize,
        ) -> usize {
            // fast fail checks (baseline serial: ~140ms):

            // → 130ms
            if (remaining_counts.len() * 2).saturating_sub(1) > remaining_records.len() {
                return 0;
            }
            // → 90ms alone;
            if remaining_counts
                .iter()
                .map(|n| n.get())
                .intersperse(1)
                .sum::<usize>()
                > remaining_records.len()
            {
                return 0;
            }

            if let Some((next_count, rest_of_counts)) = remaining_counts.split_first() {
                let (other_working_arrangements, next_count, remaining_records) = loop {
                    log!(level; "{}", fmt_states(remaining_records, remaining_counts));
                    match remaining_records.split_first() {
                        // peel off all leading operational records:
                        Some((State::Operational, rest)) => {
                            log!(level; "removing operational node");
                            remaining_records = rest;
                        }
                        // If there are no nodes left this arrangement doesn't
                        // work:
                        None => {
                            log!(level; "no nodes left, bailing");
                            return 0;
                        }
                        // If we hit an damaged node we must apply it to our
                        // count:
                        Some((State::Damaged, rest)) => {
                            log!(level; "damaged node, must apply");
                            break (0, next_count.get() - 1, rest);
                        }
                        // If we hit an unknown node we have a choice: we can
                        // choose to apply it towards the count or not.
                        //
                        // We'll explore both paths:
                        Some((State::Unknown, rest)) => {
                            log!(level; "unknown node, trying alternate path (interpreting as operational)");
                            let path_not_taken = recurse(rest, remaining_counts, level + 1);
                            break (path_not_taken, next_count.get() - 1, rest);
                        }
                    }
                };

                log!(level; "attempting to find {} nodes in: {}", next_count, fmt_states(remaining_records, rest_of_counts));

                // Now, find out if we can fulfill the remaining count with the
                // remaining records.
                //
                // We need `next_count` more nodes from `remaining_records` that
                // are damaged or unknown *and* we then need an operational node
                // (as a terminator) or the end of the list.
                if !(remaining_records.len() >= next_count) {
                    log!(level; "not enough remaining nodes ({} vs {}); bailing", remaining_records.len(), next_count);
                    return other_working_arrangements;
                }
                let (records_for_count, remaining_records) = remaining_records.split_at(next_count);
                if !records_for_count
                    .iter()
                    .all(|s| matches!(s, State::Damaged | State::Unknown))
                {
                    log!(level; "not enough remaining nodes ({} vs {}); bailing", remaining_records.len(), next_count);
                    return other_working_arrangements;
                }

                let remaining_records = match remaining_records.split_first() {
                    None => remaining_records, // okay; run can be terminated by end of list
                    Some((State::Operational | State::Unknown, rest)) => rest,
                    Some((State::Damaged, _rest)) => {
                        log!(level; "could not terminate run; bailing");
                        // not okay; run needs to be ended; this means this
                        // arrangement doesn't work:
                        return other_working_arrangements;
                    }
                };

                log!(level; "successfully found {} nodes; remaining state: {}\n", next_count, fmt_states(remaining_records, rest_of_counts));

                // If we've gotten here, we fulfilled this count!
                //
                // Now try the remaining counts on the remaining records:
                other_working_arrangements + recurse(remaining_records, rest_of_counts, level)
            } else {
                // no remaining counts; only valid if rest can be operational:
                if remaining_records
                    .iter()
                    .all(|&r| matches!(r, State::Operational | State::Unknown))
                {
                    log!(level; "no remaining counts and all remaining nodes can be interpreted as operational; success");
                    1
                } else {
                    log!(level; "bailing; no remaining counts but not all remaining nodes can be interpreted as operational: {}", fmt_states(remaining_records, remaining_counts));
                    0
                }
            }
        }

        // try_arrangement(&self.records, &self.counts, 0)

        // note: for part 1 this is actually _slower_...
        // let mut table = HashMap::new();
        // try_arrangement_memoized(&mut table, &self.records, &self.counts, 0)

        if let Some(mut table) = shared_table {
            try_arrangement_memoized(&mut table, &self.records, &self.counts, 0)
        } else if use_local_table_as_fallback {
            // note the size hint: cuts execution time in half for part 2
            let mut table = HashMap::with_capacity(self.records.len() * self.counts.len());
            try_arrangement_memoized(&mut table, &self.records, &self.counts, 0)
        } else {
            try_arrangement(&self.records, &self.counts, 0)
        }
    }
}

impl ConditionRecord {
    fn unfold(&self) -> Self {
        let records = [&*self.records; 5]
            .into_iter()
            .intersperse(&[State::Unknown])
            .flat_map(|i| i)
            .copied()
            .collect_vec();
        let counts = self.counts.repeat(5);

        Self { records, counts }
    }
}

const INP: &str = "???.### 1,1,3
.??..??...?##. 1,1,3
?#?#?#?#?#?#?#? 1,3,1,6
????.#...#... 4,1,1
????.######..#####. 1,6,5
?###???????? 3,2,1
";

fn main() {
    let mut aoc = AdventOfCode::new(2023, 12);
    let inp = aoc.get_input();
    // let inp = INP;
    let records: Vec<ConditionRecord> = inp.lines().map_parse().collect();

    // NOTE: not memoizing saves about 5 milliseconds (1ms instead of 6ms)..
    // NOTE: parallelization also (on this machine) confers no real benefit
    let p1: usize = records
        .par_iter()
        // .iter()
        // .skip(2)
        // .take(1)
        .map(|c| c.find_arrangements(None, false))
        // .inspect(|c| eprintln!("valid arrangements: {c}\n\n"))
        .sum();
    _ = aoc.submit_p1(p1);

    let records = records.iter().map(|r| r.unfold()).collect_vec();

    // NOTE: local table is faster (~150ms vs. ~195ms) *and* allows us to
    // parallelize.
    /*
    let mut table = HashMap::with_capacity(
        records
        .iter()
        .map(|r| r.counts.len() * r.records.len())
        .sum(),
    );
    */
    let p2: usize = records
        // .par_iter()
        .iter()
        .map(|c| c.find_arrangements(None, true))
        // .iter()
        // .map(|c| c.find_arrangements(Some(&mut table), true))
        // .inspect(|c| eprintln!("valid arrangements: {c}\n\n"))
        .sum();
    _ = aoc.submit_p2(p2);
}
