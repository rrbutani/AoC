use std::{
    collections::{HashMap, HashSet},
    iter::successors,
};

use aoc::{grid::*, *};
use derive_more::Display;
use strum::EnumString;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
enum Cell {
    #[strum(serialize = ".")]
    #[display(".")]
    Empty,
    #[strum(default)]
    Antenna(SingleChar), // frequency
}
type Map = TwoDimensionalGrid<Cell>;

fn find_antinodes<const LIMIT_DIST: bool>(map: &Map, mut callback: impl FnMut(SingleChar, Coord)) {
    // find antennas by frequency:
    let mut by_freq = HashMap::<_, Vec<_>>::new();
    for (coord, &cell) in map.cell_iter() {
        match cell {
            Cell::Empty => continue,
            Cell::Antenna(c) => by_freq.entry(c).or_default().push(coord),
        }
    }

    // for each frequency, pair up the antennas:
    for (freq, antennas) in by_freq {
        for (&a1, &a2) in antennas.iter().tuple_combinations() {
            // calculate the offset between each pair, apply, yielding antinodes
            let diff = a2.signed_diff(a1);
            let succ = |start: Coord, delt| {
                let mut idx = 0;
                successors(Some(start), move |a| {
                    // limit to 1 location if requested
                    (!LIMIT_DIST || idx == 0).then_some(())?;
                    idx += 1;

                    // get next, if in range:
                    a.checked_add_signed(delt).filter(|&c| map.get(c).is_some())
                })
                // skip `start` if limiting distance..
                .skip(if LIMIT_DIST { 1 } else { 0 })
            };

            let (back, forward) = (succ(a1, -diff), succ(a2, diff));
            for anti in back.chain(forward) {
                callback(freq, anti);
            }
        }
    }
}

fn count_antinode_locs<const LIMIT: bool>(map: &Map) -> usize {
    let mut antinode_locations = HashSet::new();
    find_antinodes::<LIMIT>(map, |_, pos| _ = antinode_locations.insert(pos));

    #[cfg(debug_assertions)]
    {
        let mut map = map.clone();
        for &loc in &antinode_locations {
            map[loc] = Cell::Antenna('#'.into());
        }
        eprintln!("{map}");
    }

    antinode_locations.len()
}

fn main() {
    let mut aoc = AdventOfCode::new(2024, 8);
    let inp = aoc.get_input();
    let map = TwoDimensionalGrid::<Cell>::from_str(&inp).unwrap();

    let p1 = count_antinode_locs::<true>(&map);
    _ = aoc.submit_p1(p1);

    let p2 = count_antinode_locs::<false>(&map);
    _ = aoc.submit_p2(p2);
}
