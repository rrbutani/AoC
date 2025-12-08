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
fn group(
    coords: &[Coord],
    // (conn_num, connected_junction_boxes, distinct_circuits, last connected pair)
    mut continue_func: impl FnMut(usize, usize, usize, (Coord, Coord)) -> bool,
) -> Vec<HashSet<Coord>> {
    let connections = coords
        .iter()
        .array_combinations()
        .map(|[&a, &b]| (dist(a, b), (a, b)))
        .sorted_by_key(|(d, _)| *d);

    let mut connected_junction_boxes = 0;
    let mut distinct_circuits = 0;

    let mut groups = Vec::with_capacity(coords.len() / 8);
    let mut coord_to_group = HashMap::with_capacity(coords.len());
    for (conn_num, (_, (a, b))) in connections.enumerate() {
        match (coord_to_group.get(&a), coord_to_group.get(&b)) {
            (None, None) => {
                // new group
                let id = groups.len();
                coord_to_group.insert(a, id);
                coord_to_group.insert(b, id);
                connected_junction_boxes += 2;

                groups.push(HashSet::from([a, b]));
                distinct_circuits += 1;
            }
            // add new to existing group
            (Some(&id), None) => {
                coord_to_group.insert(b, id);
                groups[id].insert(b);
                connected_junction_boxes += 1;
            }
            (None, Some(&id)) => {
                coord_to_group.insert(a, id);
                groups[id].insert(a);
                connected_junction_boxes += 1;
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
                    distinct_circuits -= 1;
                }
            }
        }

        if !continue_func(conn_num, connected_junction_boxes, distinct_circuits, (a, b)) {
            break;
        }
    }

    groups.sort_by_key(|g| Reverse(g.len()));
    groups
}

fn main() {
    let mut aoc = AdventOfCode::new(2025, 8);
    let inp = aoc.get_input();
    let junction_box_coords = inp.lines().map_parse::<Coord>().collect_vec();

    let p1 = group(&junction_box_coords, |i, _, _, _| i < 1_000)[0..3]
        .iter()
        .map(|g| g.len())
        .reduce(Mul::mul)
        .unwrap();
    _ = aoc.submit_p1(p1);

    let mut p2 = None;
    group(&junction_box_coords, |_, connected, circuit_count, (a, b)| {
        if connected < junction_box_coords.len() {
            return true; // keep going
        }

        if circuit_count > 1 {
            return true; // keep going
        }

        p2 = Some(a.0 * b.0);
        return false;
    });
    _ = aoc.submit_p2(p2.unwrap());
}
