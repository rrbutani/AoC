use std::{cmp::Ordering, collections::HashMap};

use aoc::*;
use grid::{Direction, TwoDimensionalGrid};
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
enum Cell {
    #[strum(to_string = ".")]
    Safe,
    #[strum(to_string = "#")]
    Corrupted,
}
type Grid = TwoDimensionalGrid<Cell>;

fn find_shortest_path_out(memory_space: &Grid, exit: Coord) -> Option<usize> {
    let mut steps_to_exit = None;
    memory_space.floodfill([(0usize, 0).into()], |ctx, curr_dist| {
        if ctx.cell() == &Cell::Corrupted {
            return None;
        }
        if ctx.coord() == exit {
            steps_to_exit = Some(curr_dist.unwrap());
            return None;
        }

        let dist = curr_dist.unwrap_or_default() + 1;
        Some(SmallVec::from_buf(Direction::ALL.map(|d| (d, dist))))
    });
    steps_to_exit
}

fn shortest_path_out(bytes: impl Iterator<Item = Coord>, exit: Coord) -> usize {
    let dim = exit + Coord::<usize>::from((1usize, 1));
    let memory_space = {
        let mut grid = Grid::new_with_dimensions(dim, Cell::Safe);
        for b in bytes {
            grid[b] = Cell::Corrupted
        }
        grid
    };

    #[cfg(debug_assertions)]
    eprintln!("{memory_space}");

    find_shortest_path_out(&memory_space, exit).unwrap()
}

fn main() {
    let mut aoc = AdventOfCode::new(2024, 18);
    let inp = aoc.get_input();
    let env_num = |n, def| {
        std::env::var(n)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(def)
    };
    let dim: usize = env_num("AOC24_18_DIM", 70);
    let lim: usize = env_num("AOC24_18_LIM", 1024);

    let exit = Coord::from((dim, dim));
    let byte_positions = inp
        .lines()
        .map(|l| l.split(',').map_parse::<usize>().collect_tuple().unwrap())
        .map(|(x, y)| Coord { row: y, col: x });

    let p1 = shortest_path_out(byte_positions.clone().take(lim), exit);
    _ = aoc.submit_p1(p1);

    let p2 = {
        let dim = exit + Coord::<usize>::from((1usize, 1));
        let mut grid = Grid::new_with_dimensions(dim, Cell::Safe);

        // NOTE: we can maybe do something clever w.r.t. to incrementally
        // updating shortest path information per cell as new obstacles are
        // being placed rather than recomputing? not gonna bother for now though

        // lazy approach; `slice::partition_point` doesn't expose index
        let bytes = byte_positions.collect_vec();
        let to_idx = bytes
            .iter()
            .enumerate()
            .map(|(i, &b)| (b, i))
            .collect::<HashMap<_, _>>();
        debug_assert_eq!(to_idx.len(), bytes.len(), "bytes should be unique");
        let mut last_idx = 0;

        #[rustfmt::skip]
        let blocker_idx = bytes.partition_point(|b| {
            let idx = to_idx[b];
            // update state of `grid` such that it has everything up to & b:
            match idx.cmp(&last_idx) {
                Ordering::Greater => for i in (last_idx + 1)..=idx {
                    grid[bytes[i]] = Cell::Corrupted;
                },
                Ordering::Less => for i in (idx + 1)..=last_idx {
                    grid[bytes[i]] = Cell::Safe;
                },
                Ordering::Equal => {},
            }
            last_idx = idx;

            find_shortest_path_out(&grid, exit).is_some()
        });

        let Coord { row: y, col: x } = bytes[blocker_idx];
        format!("{x},{y}")
    };
    _ = aoc.submit_p2(p2);
}

// I totally thought part two was gonna be "okay, now find the shortest path out
// if you run *while* the memory is being corrupted"...
