use std::collections::{BTreeMap, HashMap};
use std::ops::Range;
use std::str::FromStr;

use aoc::{AdventOfCode, Itertools};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(usize)]
enum Element {
    Seed,
    Soil,
    Fertilizer,
    Water,
    Light,
    Temperature,
    Humidity,
    Location,
}

impl FromStr for Element {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Element::*;
        Ok(match s {
            "seed" => Seed,
            "soil" => Soil,
            "fertilizer" => Fertilizer,
            "water" => Water,
            "light" => Light,
            "temperature" => Temperature,
            "humidity" => Humidity,
            "location" => Location,
            other => panic!("invalid element: {}", other),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Map {
    from: Element,
    to: Element,
    // start of `from` -> (start of `to`, length of range)
    //
    // note: we're really just using `BTreeMap` as a sorted list..
    ranges: BTreeMap<usize, (usize, usize)>,
}

// two ranges, six possibilities re: overlap:
//
// a:      ┌──────────┐          | L, O, R (left, overlap, right)
// b: └──┘                       | ✔️       (4)
// b: └───────┘                  | ✔️  ✔️    (6)
// b: └──────────────────┘       | ✔️  ✔️  ✔️ (7)
// b:        └──────┘            |    ✔️    (2)
// b:          └──────────────┘  |    ✔️  ✔️ (3)
// b:                   └──────┘ |       ✔️ (1)
//
// note: 5 (left and right, no overlap) and 0 are not possible
fn range_overlap_components(
    a: Range<usize>,
    b: Range<usize>,
) -> (
    Option<Range<usize>>,
    Option<Range<usize>>,
    Option<Range<usize>>,
) {
    let left = b.start..((a.start).min(b.end));
    let overlap = (a.start.max(b.start))..(a.end.min(b.end));
    let right = (a.end.max(b.start))..b.end;

    let f = |r: Range<usize>| if r.is_empty() { None } else { Some(r) };
    (f(left), f(overlap), f(right))
}

#[cfg(test)]
mod range_tests {
    macro_rules! t {
        ($(
            $range_b_low:tt..$range_b_hi:tt ? $range_a_low:tt..$range_a_hi:tt =>
                ($exp_l:expr, $exp_o:expr, $exp_r:expr) as $name:ident
        ),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    let (left, overlap, right) = range_overlap_components(
                        $range_a_low..$range_a_hi,
                        $range_b_low..$range_b_hi,
                    );

                    assert_eq!($exp_l, left, "left: expected vs got");
                    assert_eq!($exp_o, overlap, "overlap: expected vs got");
                    assert_eq!($exp_r, right, "right: expected vs got");
                }
            )*
        };
    }

    use super::range_overlap_components;
    use Option::{None as N, Some as S};

    t! {
        0..2 ? 3..8 => (S(0..2), N, N) as only_left,
        0..3 ? 3..8 => (S(0..3), N, N) as only_left2,
        0..4 ? 3..8 => (S(0..3), S(3..4), N) as left_with_overlap,
        0..8 ? 3..8 => (S(0..3), S(3..8), N) as left_with_overlap2,
        0..9 ? 3..8 => (S(0..3), S(3..8), S(8..9)) as left_with_overlap_and_right,
        3..8 ? 3..8 => (N, S(3..8), N) as full_overlap,
        3..4 ? 3..8 => (N, S(3..4), N) as overlap_left_aligned,
        6..8 ? 3..8 => (N, S(6..8), N) as overlap_right_aligned,
        6..9 ? 3..8 => (N, S(6..8), S(8..9)) as overlap_with_right,
        8..9 ? 3..8 => (N, N, S(8..9)) as only_right,
    }
}

impl FromStr for Map {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (header, rest) = s.split_once(" map:\n").unwrap();
        let (from, to) = header.split_once("-to-").unwrap();
        let from = from.parse()?;
        let to = to.parse()?;

        let range_descriptions = rest.lines().map(|l| {
            l.split_whitespace()
                .map(|x| x.parse().unwrap())
                .collect_tuple()
                .unwrap()
        });
        let mut ranges = BTreeMap::new();
        for (to_start, from_start, range_len) in range_descriptions {
            // TODO: check that ranges aren't overlapping?
            ranges.insert(from_start, (to_start, range_len));
        }

        Ok(Self { from, to, ranges })
    }
}

impl Map {
    fn map(&self, inp: usize) -> usize {
        // each range can applies to input values `start..(start + range_len)`
        //
        // this means that we can exclude all ranges that have `start > inp`
        let applicable_ranges = self.ranges.range(..=inp);

        // now we can walk the range backwards to get a range that contains
        // `inp`
        //
        // assuming ranges are not overlapping, the moment we find a range
        // that doesn't apply (i.e. too low), our search is done
        for (&inp_start, &(out_start, range_len)) in applicable_ranges.rev() {
            let range = inp_start..(inp_start + range_len);
            if range.contains(&inp) {
                return inp - inp_start + out_start;
            } else {
                break;
            }
        }

        // fallback:
        return inp;
    }

    // we assume the ranges in `inp` are non-overlapping
    fn map_ranges(&self, mut inp: Vec<Range<usize>>) -> Vec<Range<usize>> {
        // let mut inp = inp.collect_vec();
        inp.sort_by_key(|r| r.start);

        let mut out = Vec::with_capacity(inp.len());

        // split ranges in `inp` so that they don't straddle multiple ranges
        // of the mapping:
        let mut mapping_ranges_it = self.ranges.iter().peekable();
        inp.reverse();

        while let Some(input_range) = inp.pop() {
            if let Some(&(&mapping_start, &(dest_start, mapping_len))) = mapping_ranges_it.peek() {
                let mapping = mapping_start..(mapping_start + mapping_len);
                let (left, overlap, right) = range_overlap_components(mapping, input_range);

                // Everything lower than the current mapping can be assumed to
                // have not matched anything and thus can be mapped to the
                // output directly:
                if let Some(left) = left {
                    out.push(left);
                }

                // Anything overlapping we can map right now:
                if let Some(overlap) = overlap {
                    let (lower, upper) = (
                        overlap.start - mapping_start + dest_start,
                        overlap.end - mapping_start + dest_start,
                    );
                    out.push(lower..upper);
                }

                // For anything to the right, we'll need to wait and see.
                //
                // This could be within a future (higher) mapping range.
                if let Some(right) = right {
                    inp.push(right);

                    // However, there being anything to the right is an
                    // indication that (assuming the elements of `inp` are
                    // non-overlapping and are sorted in increasing order) we
                    // can move on to the next mapping range:
                    mapping_ranges_it.next();
                }
            } else {
                // if there's another input (but no remaining mapping ranges),
                // map directly:
                out.push(input_range);
            }
        }

        // merge ranges in the output that are overlapping/adjacent:
        out.sort_by_key(|r| r.start);
        let mut merged = Vec::with_capacity(out.len());
        for range in out {
            let Some(prev) = merged.last_mut() else {
                merged.push(range);
                continue;
            };

            let (left, overlap, right) = range_overlap_components(prev.clone(), range.clone());

            // shouldn't be possible if `out` is sorted..
            if left.is_some() {
                panic!()
            }

            // if there's overlap between the ranges, we can merge:
            if overlap.is_some() {
                debug_assert!(prev.start <= range.start);
                *prev = prev.start..(range.end.max(prev.end));
            } else {
                assert_eq!(right, Some(range.clone()));
                merged.push(range);
            }
        }

        merged
    }
}

#[derive(Debug, Clone)]
struct Almanac {
    mappings: HashMap<Element, Map>,
}

impl FromStr for Almanac {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut mappings = HashMap::new();

        for section in s.split("\n\n") {
            let map: Map = section.parse()?;
            let from = map.from;
            let ret = mappings.insert(from, map);
            if let Some(prev) = ret {
                panic!(
                    "Duplicate mapping for {from:?}:\n  - {prev:?}\n  - {:?}",
                    &mappings[&from]
                );
            }
        }

        Ok(Self { mappings })
    }
}

impl Almanac {
    fn map(&self, val: usize, kind: Element) -> (usize, Element) {
        let map = &self.mappings[&kind];
        (map.map(val), map.to)
    }

    fn map_to(&self, mut val: usize, mut kind: Element, to: Element) -> usize {
        // eprintln!("{kind:?}: {val}");
        while kind != to {
            let (new_val, new_kind) = self.map(val, kind);
            val = new_val;
            kind = new_kind;
            // eprintln!("{kind:?}: {val}");
        }

        // eprintln!("\n\n");
        val
    }
}

impl Almanac {
    fn map_ranges(&self, ranges: Vec<Range<usize>>, kind: Element) -> (Vec<Range<usize>>, Element) {
        let map = &self.mappings[&kind];
        (map.map_ranges(ranges), map.to)
    }

    fn map_ranges_to(
        &self,
        mut ranges: Vec<Range<usize>>,
        mut kind: Element,
        to: Element,
    ) -> Vec<Range<usize>> {
        while kind != to {
            let (new_ranges, new_kind) = self.map_ranges(ranges, kind);

            ranges = new_ranges;
            kind = new_kind;
        }

        ranges
    }
}

const INP: &str = "seeds: 79 14 55 13

seed-to-soil map:
50 98 2
52 50 48

soil-to-fertilizer map:
0 15 37
37 52 2
39 0 15

fertilizer-to-water map:
49 53 8
0 11 42
42 0 7
57 7 4

water-to-light map:
88 18 7
18 25 70

light-to-temperature map:
45 77 23
81 45 19
68 64 13

temperature-to-humidity map:
0 69 1
1 0 69

humidity-to-location map:
60 56 37
56 93 4
";

fn main() {
    let mut aoc = AdventOfCode::new(2023, 5);
    let inp = aoc.get_input();

    let (seeds, almanac) = inp.split_once("\n\n").unwrap();
    let seeds = seeds
        .strip_prefix("seeds: ")
        .unwrap()
        .split_whitespace()
        .map(|s| s.parse().unwrap())
        .collect_vec();
    let almanac = almanac.parse::<Almanac>().unwrap();

    let p1 = seeds
        .iter()
        .map(|&s| almanac.map_to(s, Element::Seed, Element::Location))
        .min()
        .unwrap();
    aoc.submit_p1(p1).unwrap();

    let p2 = {
        let seed_ranges = seeds
            .chunks_exact(2)
            .map(|pair| {
                let &[start, len] = pair else { unreachable!() };
                let range = start..(start + len);
                range
            })
            .collect_vec();

        let location_ranges = almanac.map_ranges_to(seed_ranges, Element::Seed, Element::Location);
        let lowest = location_ranges.first().unwrap().start;
        lowest
    };
    aoc.submit_p2(p2).unwrap();
}
