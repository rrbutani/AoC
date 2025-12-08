use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
    ops::Mul,
};

use aoc::{AdventOfCode, Itertools, Triple, TryConvert, iterator_map_ext::IterMapExt};

type Coord = Triple<u64>;

// note: not bother to take sqrt; just comparing, don't care about the actual value
fn dist(a: Coord, b: Coord) -> u64 {
    a.to::<[_; _]>()
        .into_iter()
        .zip(b.to::<[_; _]>().into_iter())
        .map(|(a, b)| a.abs_diff(b).pow(2))
        .sum()
}

// todo: quad trees?
//
// todo: grid compression?

// todo: do better than O(N ^ 2)?
fn group(coords: &[Coord], limit: usize) -> Vec<HashSet<Coord>> {
    let connections = coords
        .iter()
        .array_combinations()
        .map(|[&a, &b]| (dist(a, b), (a, b)))
        .sorted_by_key(|(d, _)| *d)
        .take(limit);

    let mut groups = Vec::with_capacity(limit / 8);
    let mut coord_to_group = HashMap::with_capacity(limit * 3 / 2);
    for (_, (a, b)) in connections {
        match (coord_to_group.get(&a), coord_to_group.get(&b)) {
            (None, None) => {
                // new group
                let id = groups.len();
                coord_to_group.insert(a, id);
                coord_to_group.insert(b, id);
                groups.push(HashSet::from([a, b]));
            }
            // add new to existing group
            (Some(&id), None) => {
                coord_to_group.insert(b, id);
                groups[id].insert(b);
            }
            (None, Some(&id)) => {
                coord_to_group.insert(a, id);
                groups[id].insert(a);
            }
            (Some(&id1), Some(&id2)) => {
                if id1 == id2 {
                    // already in same group, nothing to do
                } else {
                    // need to merge groups
                    let [g1, g2] = groups.get_disjoint_mut([id1, id2]).unwrap();
                    g1.extend(g2.iter());
                    for c in g2.drain() {
                        *coord_to_group.get_mut(&c).unwrap() = id1;
                    }
                }
            }
        }
    }

    groups.sort_by_key(|g| Reverse(g.len()));
    groups
}

fn main() {
    let mut aoc = AdventOfCode::new(2025, 8);
    let inp = aoc.get_input();
    let junction_box_coords = inp.lines().map_parse::<Coord>().collect_vec();

    let p1 = group(&junction_box_coords, 1_000)[0..3]
        .iter()
        .map(|g| g.len())
        .reduce(Mul::mul)
        .unwrap();
    _ = aoc.submit_p1(p1);
}
