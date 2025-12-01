use std::{cmp::Ordering, mem};

use aoc::*;

fn main() {
    let mut aoc = AdventOfCode::new(2024, 1);
    let inp = aoc.get_input();
    let (mut one, mut two): (Vec<usize>, Vec<_>) = inp
        .lines()
        .map(|l| {
            l.split_whitespace()
                .map(|n| usize::from_str(n).unwrap())
                .tuple::<2>()
        })
        .unzip();

    one.sort();
    two.sort();

    let p1: usize = one
        .iter()
        .zip(two.iter())
        .map(|(&a, &b)| (a as isize - b as isize).abs() as usize)
        .sum();
    _ = aoc.submit_p1(p1);

    // shiny, "inefficient"
    #[cfg(any())]
    let p2: usize = {
        let counts = two.into_iter().counts();
        one.into_iter()
            .map(|n| n * counts.get(&n).unwrap_or(&0))
            .sum()
    };

    // imperative-ish, faster?
    struct SortedIterRunLengths<T: Eq, It: Iterator<Item = T>> {
        it: It,
        last: Option<T>,
        len: usize,
    }
    impl<T: Eq, It: Iterator<Item = T>> Iterator for SortedIterRunLengths<T, It> {
        type Item = (T, usize);

        fn next(&mut self) -> Option<Self::Item> {
            match (self.it.next(), &mut self.last) {
                (None, None) => None,
                (None, Some(_)) => Some((self.last.take().unwrap(), mem::take(&mut self.len))),
                (Some(v), None) => {
                    self.len += 1;
                    self.last = Some(v);

                    self.next()
                }
                (Some(a), Some(b)) if &a == b => {
                    self.len += 1;
                    self.next()
                }
                (Some(new), Some(old)) => {
                    let old = mem::replace(old, new);
                    let ret = (old, self.len);

                    self.len = 1;
                    Some(ret)
                }
            }
        }
    }
    fn sorted_iter_run_lengths<T: Eq, It: Iterator<Item = T>>(
        it: It,
    ) -> SortedIterRunLengths<T, It> {
        SortedIterRunLengths { it, last: None, len: 0 }
    }

    let lhs_counts = sorted_iter_run_lengths(one.into_iter());
    let mut rhs_counts = sorted_iter_run_lengths(two.into_iter()).peekable();

    let p2: usize = lhs_counts
        .filter_map(|(v, lhs_count)| {
            let rhs_count = loop {
                // if there are no rhs counts remaining, there are no similarity
                // scores for the remaining `lhs_counts`, just yield `None`:
                let (next, rhs_count) = rhs_counts.peek()?;
                match next.cmp(&v) {
                    // skip values less than `v`
                    Ordering::Less => {
                        rhs_counts.next();
                    }
                    Ordering::Equal => {
                        break rhs_count;
                    }
                    // skipped past `v`, no similarity score for `v`
                    Ordering::Greater => return None,
                }
            };

            Some(v * lhs_count * rhs_count)
        })
        .sum();

    _ = aoc.submit_p2(p2);
}
