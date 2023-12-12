use aoc::*;

use std::{
    collections::BTreeMap,
    ops::{Add, RangeBounds, RangeInclusive},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Pos {
    x: isize,
    y: isize,
}

impl FromStr for Pos {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, y) = s.split_once(", ").unwrap();
        let x = x.strip_prefix("x=").unwrap().parse().unwrap();
        let y = y.strip_prefix("y=").unwrap().parse().unwrap();

        Ok(Pos { x, y })
    }
}

impl Pos {
    fn tuning_frequency(&self) -> isize {
        self.x * 4_000_000 + self.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Report {
    sensor: Pos,
    closest_beacon: Pos,
}

impl FromStr for Report {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (sensor, beacon) = s.split_once(": ").unwrap();
        let sensor = sensor.strip_prefix("Sensor at ").unwrap().parse().unwrap();
        let beacon = beacon
            .strip_prefix("closest beacon is at ")
            .unwrap()
            .parse()
            .unwrap();

        Ok(Report {
            sensor,
            closest_beacon: beacon,
        })
    }
}

impl Report {
    // manhattan distance
    pub fn radius(&self) -> usize {
        let Pos { x: sx, y: sy } = self.sensor;
        let Pos { x: bx, y: by } = self.closest_beacon;

        sx.abs_diff(bx) + sy.abs_diff(by)
    }

    #[allow(unused)]
    pub fn exluded_all(&self) -> impl Iterator<Item = Pos> + '_ {
        self.excluded(|_| true)
    }

    #[allow(unused)]
    pub fn excluded<'s>(
        &'s self,
        row_filter: impl Fn(isize) -> bool + 's,
    ) -> impl Iterator<Item = Pos> + 's {
        // always squares, rotated 45º
        let rad = self.radius() as isize;
        let Pos { x, y } = self.sensor;

        (-rad..=rad)
            .filter(move |&y| row_filter(y))
            .flat_map(move |dy| {
                let remaining = rad - dy.abs();
                (-remaining..=remaining).map(move |dx| (x + dx, y + dy))
            })
            .map(|(x, y)| Pos { x, y })
    }

    pub fn excluded_for_row(
        &self,
        row: isize,
        exclude_beacon: bool,
    ) -> [Option<RangeInclusive<isize>>; 2] {
        let rad = self.radius();
        let Pos { x, y } = self.sensor;

        let offset = row.abs_diff(y);
        if offset > rad {
            [None, None]
        } else {
            let remaining = (rad - offset) as isize;
            let x_range = (-remaining + x)..=(remaining + x);

            if exclude_beacon
                && row == self.closest_beacon.y
                && x_range.contains(&self.closest_beacon.x)
            {
                // need to exclude the beacon which means: two ranges
                let ex = self.closest_beacon.x;
                let (a, b) = (*x_range.start(), *x_range.end());

                // tying ourselves to inclusive ranges means we need to handle
                // these annoying edge cases:
                match (ex == a, ex == b) {
                    (true, true) => [None, None],
                    (true, false) => [Some((a + 1)..=b), None],
                    (false, true) => [Some(a..=(b - 1)), None],
                    (false, false) => [Some(a..=(x - 1)), Some((x + 1)..=b)],
                }
            } else {
                // one range, nice and easy:
                [Some(x_range), None]
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Range {
    // map of non-overlapping ranges, keyed on starting point
    map: BTreeMap<isize, RangeInclusive<isize>>,
}

fn is_subset<T: ?Sized + Ord + Clone + num_traits::One + Add<T, Output = T>>(
    big: &impl RangeBounds<T>,
    small: &impl RangeBounds<T>,
) -> bool {
    use std::ops::Bound::*;

    let start = match (big.start_bound(), small.start_bound()) {
        (Included(a), Included(b)) | (Excluded(a), Excluded(b)) => a <= b,
        (Unbounded, _) => true,
        (Included(_) | Excluded(_), Unbounded) => false,
        (Included(a), Excluded(b)) => a.clone() <= (b.clone() + T::one()),
        (Excluded(a), Included(b)) => (a.clone() + T::one()) <= b.clone(),
    };
    let end = match (big.end_bound(), small.end_bound()) {
        (Included(a), Included(b)) | (Excluded(a), Excluded(b)) => a >= b,
        (Unbounded, _) => true,
        (Included(_) | Excluded(_), Unbounded) => false,
        (Included(a), Excluded(b)) => (a.clone() + T::one()) >= b.clone(),
        (Excluded(a), Included(b)) => a.clone() >= (b.clone() + T::one()),
    };

    start && end
}

impl Range {
    fn insert(&mut self, mut r: RangeInclusive<isize>) {
        // find all ranges that overlap with this one:
        //
        // i.e. all the ranges that have a starting point that is within `r`
        let overlapping = self
            .map
            .range(r.clone())
            .map(|(_, v)| v.clone())
            .collect_vec(); // copy so we can manipulate the map
        let mut it = overlapping.into_iter().peekable();

        fn join_overlapping(
            a: &RangeInclusive<isize>,
            b: &RangeInclusive<isize>,
        ) -> RangeInclusive<isize> {
            // up to the callee to ensure that these are overlapping and not
            // disjoint..

            let lower = a.start().min(b.start());
            let upper = a.end().max(b.end());
            *lower..=*upper
        }

        while let Some(o) = it.next() {
            // if we're a subset of the current range, we don't need to do
            // anything
            //
            // this also means that we're done because we know that we don't
            // overlap with any other ranges since the ranges in `map` are
            // non-overlapping:
            if is_subset(&o, &r) {
                assert_eq!(it.peek(), None);
                break;
            }

            // if `o` is a subset of us, we can subsume it:
            if is_subset(&r, &o) {
                self.map.remove(o.start()).unwrap();
            } else {
                // otherwise we need to do some range math.
                //
                // since we know that `o` is not a subset of `r` and `r` is not
                // a subset of `o` that means we've got ranges that look like
                // this:
                // ```
                //   |------------------|
                //               |------------------|
                // ```
                //
                // we want to just flatten this into one range:
                r = join_overlapping(&r, &o);

                self.map.remove(o.start()).unwrap();
            }
        }

        // the above checks for ranges whose starting point is within `r`.
        //
        // there's also the possibility that we overlapped with the end of a
        // previous range (range, _singular_, since ranges in `map` don't
        // overlap and therefore there can only be one range whose starting
        // point isn't in `r` but still overlaps with `r`)
        if let Some(last) = self
            .map
            .range(..=*r.start())
            .rev()
            .next()
            .map(|(_, v)| v.clone())
            .filter(|last| last.end() >= r.start())
        {
            r = join_overlapping(&r, &last);
            self.map.remove(last.start()).unwrap();
        }

        assert!(self.map.insert(*r.start(), r).is_none());

        // in retrospect I think this function can just be:
        //  - remove all of overlapping
        //  - construct a new range that's `it.map(|r| r.start()).min()..=it.map(|r| r.end()).max()`
        //  - do the "did we overlap with the end of a previous range check"
        //  - insert
    }

    fn len(&self) -> isize {
        self.map.values().map(|r| r.end() - r.start() + 1).sum()
    }

    fn clip(
        &self,
        within: RangeInclusive<isize>,
    ) -> impl Iterator<Item = RangeInclusive<isize>> + '_ {
        let last = self
            .map
            .range(..=within.start())
            .rev()
            .next()
            .map(|(_, v)| v.clone())
            .filter(|last| last.end() >= within.start());
        let last = [last].into_iter().flatten();

        let rest = self.map.range(within).map(|(_, v)| v.clone());
        last.chain(rest)
    }

    fn gaps(
        &self,
        within: RangeInclusive<isize>,
    ) -> impl Iterator<Item = RangeInclusive<isize>> + '_ {
        self.clip(within)
            .tuple_windows()
            .map(|(prev, next)| (prev.end() + 1)..=(next.start() - 1))
    }
}

// struct Grid {}

// impl Display for Grid {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
// }

fn range_for_row<'r>(
    reports: impl Iterator<Item = &'r Report>,
    row: isize,
    exclude_beacon: bool,
) -> Range {
    let mut range = Range::default();
    reports
        .flat_map(|r| r.excluded_for_row(row, exclude_beacon))
        .flatten()
        .for_each(|r| {
            // dbg!(&r);
            range.insert(r);
            // dbg!(&row);
        });
    range
}

fn p1<'r>(reports: impl Iterator<Item = &'r Report>, row: isize) -> usize {
    range_for_row(reports, row, true).len().try_into().unwrap()
}

fn p2<'r>(
    reports: impl Iterator<Item = &'r Report> + Send + Sync + Clone,
    search_space: RangeInclusive<isize>,
) -> Option<Pos> {
    use rayon::iter::{IntoParallelIterator, ParallelIterator};
    let it = search_space.clone().into_par_iter().flat_map(|row| {
        let range = range_for_row(reports.clone(), row, false);
        range
            .gaps(search_space.clone())
            .flatten()
            .map(|x| Pos { x, y: row })
            .collect_vec() // bleh
    });

    // it.next()
    it.find_any(|_| true)
}

fn main() {
    let mut aoc = AdventOfCode::new(2022, 15);
    let inp = aoc.get_input();

    let reports = inp
        .lines()
        .map(|l| l.parse::<Report>().unwrap())
        .collect_vec();

    let p1 = p1(reports.iter(), 2000000);
    aoc.submit_p1(dbg!(p1)).unwrap();

    // TODO: the better approach is (maybe? actually this seems functionally
    // equivalent) to test the positions outside the edges of the squares formed
    // by the reports against all the other reports...
    //
    // but this runs in like a second so w/e
    let p2 = p2(reports.iter(), 0..=4_000_000);
    let p2 = dbg!(p2.unwrap()).tuning_frequency();
    aoc.submit_p2(dbg!(p2)).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reports() -> impl Iterator<Item = Report> {
        include_str!("ex")
            .lines()
            .map(|l| l.parse::<Report>().unwrap())
    }

    #[test]
    fn p1() {
        let r = reports().collect_vec();
        assert_eq!(super::p1(r.iter(), 10), 26);
    }

    #[test]
    fn p2() {
        let r = reports().collect_vec();
        let pos = super::p2(r.iter(), 0..=20).unwrap();
        let score = dbg!(pos).tuning_frequency();
        assert_eq!(score, 56000011);
    }
}
