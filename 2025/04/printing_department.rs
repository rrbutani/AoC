use aoc::{
    AdventOfCode, Coord, Itertools,
    grid::{ExtendedDirection, NextCoords},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(strum::EnumString, strum::Display)]
enum Cell {
    #[strum(serialize = ".")]
    Empty,
    #[strum(serialize = "@")]
    Roll,
}

type Grid = aoc::Grid<Cell>;

fn accessible_by_forklift<'g>(grid: &'g Grid) -> impl Iterator<Item = Coord> + use<'g> {
    grid.cell_iter()
        .filter(|&(_, &cell)| cell == Cell::Roll)
        .filter(move |&(coord, _)| {
            let adj_count = ExtendedDirection::ALL
                .iter()
                .filter_map(|d| d.get(coord, &grid))
                .filter(|&c| grid[c] == Cell::Roll)
                .count();

            adj_count < 4
        })
        .map(|(coord, _)| coord)
}

fn main() {
    let mut aoc = AdventOfCode::new(2025, 4);
    let grid: Grid = aoc.get_input().parse().unwrap();
    // eprintln!("{grid}");

    let mut accessible = accessible_by_forklift(&grid).collect_vec();
    _ = aoc.submit_p1(accessible.len());

    // TODO(perf): this is needlessly inefficient; on each iteration we should only consider cells
    // adjacent to something that was removed instead of doing a full scan..
    let mut removed = 0;
    let mut grid = grid;
    loop {
        if accessible.is_empty() {
            break;
        }

        removed += accessible.len();
        for c in accessible.drain(..) {
            grid[c] = Cell::Empty;
        }

        accessible.extend(accessible_by_forklift(&grid));
    }
    _ = aoc.submit_p2(removed);
}
