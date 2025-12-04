use aoc::{
    AdventOfCode, Itertools,
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

fn main() {
    let mut aoc = AdventOfCode::new(2025, 4);
    let mut grid: Grid = aoc.get_input().parse().unwrap();

    let mut adjacent_counts: aoc::Grid<u8> = grid.map(|coord, _| {
        ExtendedDirection::ALL
            .iter()
            .filter_map(|d| d.get(coord, &grid))
            .filter(|&c| grid[c] == Cell::Roll)
            .count() as u8
    });

    let accessible_by_forklift = grid
        .cell_iter()
        .filter(|&(_, &cell)| cell == Cell::Roll)
        .filter(|&(c, _)| adjacent_counts[c] < 4)
        .map(|(c, _)| c)
        .collect_vec();
    _ = aoc.submit_p1(accessible_by_forklift.len());

    // For each roll we remove, consider adjacent rolls for removal; continue until fixed point:
    let mut removed = 0;
    grid.search_mut::<_, _, _, true>(accessible_by_forklift, |mut ctx, _| {
        // remove the roll at the current cell
        *ctx.cell_mut() = Cell::Empty;
        removed += 1;

        // update counts for adjacent rolls; if now < 4, mark for removal:
        let next_remove = ExtendedDirection::ALL
            .iter()
            .filter_map(|d| d.get(ctx.coord(), ctx.grid()))
            .filter(|&c| ctx.grid()[c] == Cell::Roll)
            .filter(|&c| {
                let prev = adjacent_counts[c];
                adjacent_counts[c] = prev - 1;

                prev == 4
            })
            .map(|roll_remove_coord| (roll_remove_coord, ()))
            .collect();

        Some(next_remove)
    });
    _ = aoc.submit_p2(removed);
}
