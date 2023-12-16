use std::collections::HashSet;

use aoc::{iterator_map_ext::IterMapExt, AdventOfCode, FromStr, Itertools};
use rayon::prelude::*;
use strum::EnumString;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString)]
enum Cell {
    #[strum(serialize = ".")]
    Empty,
    #[strum(serialize = "/")]
    UpMirror,
    #[strum(serialize = "\\")]
    DownMirror,
    #[strum(serialize = "|")]
    VertSplitter,
    #[strum(serialize = "-")]
    HorizSplitter,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Contraption {
    grid: Vec<Vec<Cell>>,
    width: usize,
}

impl FromStr for Contraption {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let grid = s
            .lines()
            .map(|l| l.split_inclusive(|_| true).map_parse().collect_vec())
            .collect_vec();

        let width = grid[0].len();
        for row in &grid {
            assert_eq!(width, row.len());
        }
        Ok(Self { grid, width })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl TryFrom<usize> for Direction {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        use Direction::*;
        Ok(match value {
            0 => North,
            1 => East,
            2 => South,
            3 => West,
            _ => return Err(()),
        })
    }
}

impl Direction {
    fn rot(self, n: i8) -> Self {
        ((((self as u8) + 4).checked_add_signed(n).unwrap() % 4) as usize)
            .try_into()
            .unwrap()
    }

    fn as_offs(self) -> (isize, isize) {
        use Direction::*;
        // (row, col)
        match self {
            North => (-1, 0),
            East => (0, 1),
            South => (1, 0),
            West => (0, -1),
        }
    }

    fn apply(self, (row, col): Coord, (height, width): Coord) -> Option<Coord> {
        let (row_offs, col_offs) = self.as_offs();

        let row = row.checked_add_signed(row_offs).filter(|&r| r < height);
        let col = col.checked_add_signed(col_offs).filter(|&c| c < width);

        row.zip(col)
    }
}

type Coord = (usize, usize); // (row, col)

impl Contraption {
    fn count_energized(&self, start: Coord, dir: Direction) -> usize {
        self.find_energized(start, dir)
            .iter()
            .flat_map(|r| r.iter())
            .filter(|&&h| h)
            .count()
    }

    fn find_energized(&self, start: Coord, dir: Direction) -> Vec<Vec<bool>> {
        let mut hit = vec![vec![false; self.width]; self.grid.len()];
        let mut already_processed = HashSet::<(Coord, Direction)>::new();

        fn find_energized_inner(
            grid: &Vec<Vec<Cell>>,
            dimensions: Coord,
            (mut row, mut col): Coord,
            mut dir: Direction,
            hit: &mut Vec<Vec<bool>>,
            already_processed: &mut HashSet<((usize, usize), Direction)>,
        ) {
            use Cell::*;
            use Direction::*;
            loop {
                // If already processed, we're done:
                if !already_processed.insert(((row, col), dir)) {
                    return;
                }
                hit[row][col] = true;

                let dirs = match (grid[row][col], dir) {
                    (Empty, dir) => vec![dir],
                    (VertSplitter, North | South) => vec![dir],
                    (VertSplitter, East | West) => vec![North, South],
                    (HorizSplitter, North | South) => vec![East, West],
                    (HorizSplitter, East | West) => vec![dir],
                    (UpMirror, East | West) | (DownMirror, North | South) => vec![dir.rot(-1)], // CCW
                    (UpMirror, North | South) | (DownMirror, East | West) => vec![dir.rot(1)], // CW
                };

                let mut next_coords = dirs
                    .into_iter()
                    .filter_map(|d| d.apply((row, col), dimensions).map(|c| (c, d)));

                // bad workaround so we don't have to rely on tail-recursion...
                let Some(((next_row, next_col), next_dir)) = next_coords.next() else {
                    return;
                };

                // recurse for the rest:
                for (next, next_dir) in next_coords {
                    find_energized_inner(grid, dimensions, next, next_dir, hit, already_processed);
                }

                row = next_row;
                col = next_col;
                dir = next_dir;
            }
        }

        find_energized_inner(
            &self.grid,
            (self.grid.len(), self.width),
            start,
            dir,
            &mut hit,
            &mut already_processed,
        );
        hit
    }
}

const INP: &str = r#".|...\....
|.-.\.....
.....|-...
........|.
..........
.........\
..../.\\..
.-.-/..|..
.|....-|.\
..//.|...."#;

fn main() {
    let mut aoc = AdventOfCode::new(2023, 16);
    let inp = aoc.get_input();
    // let inp = INP;
    let contrap: Contraption = inp.parse().unwrap();

    let p1 = contrap.count_energized((0, 0), Direction::East);
    // dbg!(p1);
    _ = aoc.submit_p1(p1);

    let p2 = {
        let starting_points = [
            // Left
            (0..contrap.grid.len() - 1)
                .map(|r| ((r, 0), Direction::East))
                .collect_vec(),
            // Right
            (0..contrap.grid.len() - 1)
                .map(|r| ((r, contrap.width - 1), Direction::West))
                .collect_vec(),
            // Down
            (0..contrap.width)
                .map(|c| ((0, c), Direction::South))
                .collect_vec(),
            // Up
            (0..contrap.width)
                .map(|c| ((contrap.grid.len() - 1, c), Direction::North))
                .collect_vec(),
        ];

        starting_points
            .into_par_iter()
            .flat_map(|i| i.into_par_iter())
            .map(|(start, dir)| contrap.count_energized(start, dir))
            .max()
            .unwrap()
    };
    // dbg!(p2);
    _ = aoc.submit_p2(p2);
}
