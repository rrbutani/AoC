use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashSet},
};

use aoc::*;
use grid::{Direction, TwoDimensionalGrid};
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, EnumString, Display)]
enum Cell {
    #[strum(to_string = "#")]
    Wall,
    #[strum(to_string = ".")]
    Empty,
    #[strum(to_string = "S")]
    Start,
    #[strum(to_string = "E")]
    End,
    #[strum(to_string = "{0}")]
    #[cfg_attr(not(debug_assertions), allow(unused))]
    Custom(&'static str),
}

type Map<C = Cell> = TwoDimensionalGrid<C>;
type PathMap = Map<[(usize, Vec<(Coord, Direction)>); Direction::ALL.len()]>;

#[rustfmt::skip]
fn find_shortest_paths_with_predecessors(map: &Map) -> PathMap {
    use {std::cmp::Ordering::*, Cell::*, Direction::*, Reverse as R};

    let start = map.find(|&c| c == Start).next().unwrap();
    let mut distances_with_prev = map.map(|_, _| {
        const E: (usize, Vec<(Coord, Direction)>) = (usize::MAX, vec![]);
        [E; Direction::ALL.len()]
    });

    let mut queue = BinaryHeap::from([(R(0), ((start, East), start, East))]);
    while let Some((R(cost), (prev, coord, dir))) = queue.pop() {
        let this = &mut distances_with_prev[coord][dir as usize];
        match this.0.cmp(&cost) {
            Less => continue, // we've already done better, move on
            Equal => { // record this path but no need to reevaluate neighbours
                this.1.push(prev);
                continue;
            },
            Greater => { // clear old paths; obviated by a new lowest cost path
                this.0 = cost;
                this.1.clear();
                this.1.push(prev);
            },
        }

        // try visiting our neighbours via this node, in case its cheaper; try
        // going forwards, and rotating CW/CCW:
        let this = (coord, dir);
        'forwards: {
            let Some(next) = dir.apply(coord) else { break 'forwards; };
            let Some(cell) = map.get(next) else { break 'forwards; };
            if let Wall = cell { break 'forwards; }

            queue.push((R(cost + 1), (this, next, dir)));
        }
        for rot in [-1, 1] {
            queue.push((R(cost + 1_000), (this, coord, dir.rotate(rot))));
        }
    }

    distances_with_prev
}

fn all_best_paths(map: &Map, pmap: &PathMap, end: Coord, scr: usize) -> usize {
    // go backwards from the end; count all cell+dir pairs that we encounter
    let mut encountered = HashSet::new();
    fn traverse(
        edges: &TwoDimensionalGrid<[(usize, Vec<(Coord, Direction)>); 4]>,
        curr @ (pos, dir): (Coord, Direction),
        encountered: &mut HashSet<(Coord, Direction)>,
    ) -> Vec<Vec<(Coord, Direction)>> {
        let mut paths = vec![];
        if encountered.insert(curr) {
            for &p in &edges[pos][dir as usize].1 {
                paths.extend(traverse(edges, p, encountered));
            }

            #[cfg(debug_assertions)]
            for path in &mut paths {
                path.push(curr);
            }
        } else {
            #[cfg(debug_assertions)]
            paths.push(vec![curr]);
        }
        paths
    }
    let mut paths = vec![];
    for &dir in Direction::ALL
        .iter()
        .filter(|&&d| pmap[end][d as usize].0 == scr)
    {
        paths.extend(traverse(&pmap, (end, dir), &mut encountered));
    }

    #[cfg(debug_assertions)]
    {
        // would be better if we did a gradient but alas
        let strings = (0..255)
            .map(|i| {
                Direction::ALL.map(|d| {
                    use owo_colors::OwoColorize;
                    let s = format!("{}", d.color(owo_colors::Rgb(0, i, 0)));
                    String::leak(s)
                })
            })
            .collect_vec()
            .leak();

        // print paths (kind of)
        for path in &paths {
            let mut map = map.clone();
            for (i, &(coord, dir)) in path.iter().enumerate() {
                let s = &strings[i * 255 / path.len()][dir as usize];
                map[coord] = Cell::Custom(s);
            }
            eprintln!("{map}\n");
        }
        eprintln!("----------\n");
    }

    // narrow to just cells:
    let cells = encountered.iter().map(|&(c, _)| c).collect::<HashSet<_>>();
    #[cfg(debug_assertions)]
    {
        let mut map = map.clone();
        for &c in &cells {
            map[c] = Cell::Custom("\u{001b}[34mO\u{001b}[0m");
        }
        eprintln!("{map}");
    }
    _ = (paths, map);

    cells.len()
}

fn main() {
    let mut aoc = AdventOfCode::new(2024, 16);
    let map = Map::from_str(&aoc.get_input()).unwrap();
    let end = map.find(|&c| c == Cell::End).next().unwrap();

    let paths = find_shortest_paths_with_predecessors(&map);
    let score = paths[end].iter().map(|(s, _)| *s).min().unwrap();
    _ = aoc.submit_p1(score);
    _ = aoc.submit_p2(all_best_paths(&map, &paths, end, score));
}
