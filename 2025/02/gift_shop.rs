use std::{cmp::Ordering, iter, num::NonZeroU8, ops::RangeInclusive};

use aoc::{AdventOfCode, Itertools, SmallVec};

struct InvalidIdIter {
    curr: u64,
    limit: u64,
    pattern_len: NonZeroU8,
}

impl InvalidIdIter {
    fn new(range: &RangeInclusive<u64>, pattern_len: NonZeroU8) -> Self {
        Self { curr: *range.start(), limit: *range.end(), pattern_len }
    }
}

fn base10_len(n: u64) -> u32 {
    //    1 -    9 -> 0
    //   10 -   99 -> 1
    //  100 -  999 -> 2
    // 1000 - 9999 -> 3
    n.ilog10() + 1
}

// Least Significant chunk first
fn number_into_chunks(mut n: u64, chunk_len: NonZeroU8) -> impl Iterator<Item = u64> {
    let mask = 10u64.pow(chunk_len.get() as _);
    debug_assert!(base10_len(n).is_multiple_of(chunk_len.get() as _), "{n}, {chunk_len}");

    iter::from_fn(move || {
        if n == 0 {
            return None;
        }
        let next = n % mask;
        n /= mask;
        Some(next)
    })
}

fn number_from_chunks(num: u64, chunk_len: NonZeroU8, num_chunks: u32) -> u64 {
    debug_assert_eq!(chunk_len.get() as u32, base10_len(num), "{num}, {chunk_len}");
    let mut out = 0;
    let mask = 10u64.pow(chunk_len.get() as _);
    for _ in 0..num_chunks {
        out = out * mask + num;
    }
    out
}

impl Iterator for InvalidIdIter {
    // number, (pattern-len chunk that's repeated, num occurences))
    type Item = (u64, (u64, u32));

    fn next(&mut self) -> Option<Self::Item> {
        while self.curr <= self.limit {
            let b10_len = base10_len(self.curr);
            let num_chunks = b10_len.div_ceil(self.pattern_len.get() as _);

            // if we cannot split `curr` evenly into at least two `pattern_len` sized chunks, skip
            // to the lowest number greater than `curr` that can be split evenly (into two or more
            // chunks):
            if b10_len % (self.pattern_len.get() as u32) != 0 || num_chunks == 1 {
                let lowest_num_for_chunk_size = 10u64.pow(self.pattern_len.get() as u32 - 1);
                self.curr = number_from_chunks(
                    lowest_num_for_chunk_size,
                    self.pattern_len,
                    num_chunks.max(2), // at least two chunks
                );
                continue;
            }

            // otherwise, split:
            let chunks =
                number_into_chunks(self.curr, self.pattern_len).collect::<SmallVec<[_; 10]>>();

            // we're interested in how `curr` compares to the number formed by repeating the most
            // significant chunk; we can either be:
            //   - equal to this number
            //   - less than this number
            //   - greater than this number
            use Ordering::*;
            let mut cmp = Equal;
            let most_significant_chunk = chunks.last().copied().unwrap();
            for c in chunks {
                // iterating from lsb to msb
                cmp = match (cmp, c.cmp(&most_significant_chunk)) {
                    (Less, Less) | (Equal, Equal) | (Greater, Greater) => cmp,
                    // if equal thus far but no longer:
                    (Equal, new @ Less) | (Equal, new @ Greater) => new,
                    // new data (msb) takes precedence
                    (Less, new @ Greater) | (Greater, new @ Less) => new,
                    // unless previously not equal and now equal:
                    (prev @ (Greater | Less), Equal) => prev,
                };
            }

            // if `self.curr` matched the pattern, tuck it away
            let val = (cmp == Equal).then_some(self.curr);

            // calculate the next value for `curr` using the most significant chunk (msc):
            //   - if `curr` is less than the msc repeated, then the msc repeated is the next value
            //   - if `curr` is greater than/equal to the msc repeated, increment the msc
            //     + edge case: if adding 1 would cause us to exceed the pattern len we just add
            //       1 to `curr`
            //       * this case should only arise when the most significant chunk is a bunch of 9s
            //         and when `cmp == Equal`
            //       * adding `1` should cause us to hit the first conditional of this loop, next
            //         time around
            let msc = most_significant_chunk;
            match cmp {
                Less => self.curr = number_from_chunks(msc, self.pattern_len, num_chunks),
                Equal | Greater => {
                    let msc = msc + 1;
                    if base10_len(msc) > self.pattern_len.get() as u32 {
                        assert!(cmp == Equal, "{}, {msc}, {}", self.curr, self.pattern_len);
                        self.curr += 1;
                    } else {
                        self.curr = number_from_chunks(msc, self.pattern_len, num_chunks);
                    }
                }
            }

            if let Some(val) = val {
                return Some((val, (msc, num_chunks)));
            }
        }

        return None;
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2025, 2);
    let inp = aoc.get_input();
    let ranges = inp
        .trim()
        .split(',')
        .map(|r| {
            let (start, end) = r.split_once('-').unwrap();
            let [start, end] = [start, end].map(|n| n.parse::<u64>().unwrap());
            assert!(start < end);
            start..=end
        })
        .collect_vec();

    let invalid_iter = ranges.iter().flat_map(|r| {
        (1..base10_len(*r.end()) as u8)
            .map(|pattern_len| InvalidIdIter::new(r, NonZeroU8::new(pattern_len).unwrap()))
    });

    let sum = invalid_iter
        .clone()
        .flatten()
        .filter(|(_val, (_repeated, count))| *count == 2) // only pairs
        .map(|(val, _)| val)
        .sum::<u64>();
    _ = aoc.submit_p1(sum);

    // TODO: can do a more efficient thing for filtering out duplicates than accumulating into a set
    //   - (for `repeated`, check if it itself matches any patterns; dynamic programming)
    let sum = invalid_iter
        .flatten()
        // .inspect(|(v, (pat, n))| eprintln!("{v}: {pat} x {n}"))
        .map(|(val, _)| val)
        .unique()
        .sum::<u64>();
    _ = aoc.submit_p2(sum);
}
