use aoc::*;
use grid::{Direction, TwoDimensionalGrid};
use rayon::iter::ParallelIterator;
use std::{cmp::Reverse, collections::BinaryHeap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::Display)]
enum Cell {
    #[strum(to_string = "S")]
    Start,
    #[strum(to_string = "E")]
    End,
    #[strum(to_string = ".")]
    Empty,
    #[strum(to_string = "#")]
    Wall,
}
type Map<C = Cell> = TwoDimensionalGrid<C>;

// first, dijkstra's to find shortest path information for each cell
#[rustfmt::skip]
fn find_distances(map: &Map) -> Map<usize> {
    use {std::cmp::Ordering::*, Cell::*, Reverse as R};

    let end = map.find(|&c| c == End).next().unwrap();
    let mut distances = map.map(|_, _| usize::MAX);

    let mut queue = BinaryHeap::from([(R(0), end)]);
    while let Some((R(cost), coord)) = queue.pop() {
        let this = &mut distances[coord];
        match (*this).cmp(&cost) {
            Less | Equal => continue, // already done better, skip
            Greater => *this = cost,
        }

        // try visiting neighours via this node:
        for d in Direction::ALL {
            let Some(next) = d.apply(coord) else { continue; };
            let Some(cell) = map.get(next) else { continue; };
            if let Wall = cell { continue; }

            queue.push((R(cost + 1), next));
        }
    }

    distances
}

// next, for each coordinate, try cheating
fn find_cheats<'m, const TIME: usize>(
    map: &'m Map,
    distances: &'m Map<usize>,
) -> impl ParallelIterator<Item = ((Coord, Coord), usize)> + 'm {
    // NOTE: not considering whether each of these cheats are actually on a path
    // from `S -> E`? (everything seems to be reachable from `S` so it's fine..)
    map.par_cell_iter()
        .filter(move |&(coord, &cell)| match cell {
            Cell::Wall => {
                debug_assert_eq!(distances[coord], usize::MAX);
                return false;
            }
            _ => true,
        })
        .flat_map_iter(move |(coord, _)| {
            // for each coordinate, consider the diamond of coordinates that are
            // reachable in `TIME` steps:
            let range = -(TIME as isize)..=(TIME as isize);
            range
                .clone()
                .cartesian_product(range)
                .map(|(r_offs, c_offs)| {
                    let dist = r_offs.unsigned_abs() + c_offs.unsigned_abs();
                    let offs = Coord { row: r_offs, col: c_offs };
                    (offs, dist)
                })
                .filter(|&(_offs, dist)| {
                    // note: lazy approach; don't want to think about diamonds
                    // and coordinate math!
                    dist <= TIME
                })
                .filter_map(move |(offs, dist)| {
                    // only consider ends that are in bounds and not a wall:
                    let end = coord.checked_add_signed(offs)?;
                    let &end_cell = map.get(end)?;
                    if end_cell == Cell::Wall {
                        return None;
                    }

                    Some((end, dist))
                })
                .filter_map(move |(end, dist)| {
                    // check if the cost is reduced when using this cheat:
                    let cost_via_cheat = distances[end] + dist;
                    if cost_via_cheat < distances[coord] {
                        Some(((coord, end), distances[coord] - cost_via_cheat))
                    } else {
                        None
                    }
                })
                .inspect(|((a, b), savings)| {
                    #[cfg(debug_assertions)]
                    {
                        eprintln!("cheat{TIME} [{savings:4}]: {a}, {b}");
                    }
                    _ = ((a, b), savings);
                })
        })
}

fn main() {
    let mut aoc = AdventOfCode::new(2024, 20);
    let inp = aoc.get_input();
    let m = Map::from_str(&inp).unwrap();
    let d = find_distances(&m);

    _ = aoc.submit_p1(find_cheats::<2>(&m, &d).filter(|t| t.1 >= 100).count());
    _ = aoc.submit_p2(find_cheats::<20>(&m, &d).filter(|t| t.1 >= 100).count());
}
