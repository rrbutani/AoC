use std::{collections::VecDeque, fmt, iter, path::Display};

use aoc::{iterator_map_ext::IterMapExt, AdventOfCode, FromStr, Itertools};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumString, strum::Display,
)]
enum Cell {
    #[strum(serialize = "|", to_string = "┃")]
    Vert, // |
    #[strum(serialize = "-", to_string = "━")]
    Horz, // -
    #[strum(serialize = "L", to_string = "┗")]
    NorthEast, // L
    #[strum(serialize = "J", to_string = "┛")]
    NorthWest, // J
    #[strum(serialize = "7", to_string = "┓")]
    SouthWest, // 7
    #[strum(serialize = "F", to_string = "┏")]
    SouthEast, // F
    #[strum(serialize = ".", to_string = " ")]
    Empty, // .
    // #[strum(serialize = "S", to_string = "\u{001b}[44;32mS\u{001b}[0m")]
    #[strum(serialize = "S", to_string = "\u{001b}[34mS\u{001b}[0m")]
    Start, // S
}

type Coord = (usize, usize); // (row, col)

impl Cell {
    pub fn adjacent(&self, (row, col): Coord) -> [Option<Coord>; 2] {
        use Cell::*;
        let (offs1, offs2) = match self {
            // row, col
            Vert => ((-1, 0), (1, 0)),
            Horz => ((0, -1), (0, 1)),
            NorthEast => ((-1, 0), (0, 1)),
            NorthWest => ((-1, 0), (0, -1)),
            SouthWest => ((1, 0), (0, -1)),
            SouthEast => ((1, 0), (0, 1)),
            Empty | Start => return [None; 2],
        };

        let apply_offs = |(row_offs, col_offs)| {
            row.checked_add_signed(row_offs)
                .and_then(|r| col.checked_add_signed(col_offs).map(|c| (r, c)))
        };

        [apply_offs(offs1), apply_offs(offs2)]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum IsEnclosed {
    OnLoop,
    Yes,
    No,
}

struct Grid {
    grid: Vec<Vec<Cell>>,
    distances: Option<Vec<Vec<Option<usize>>>>,
    enclosed: Option<Vec<Vec<IsEnclosed>>>,
    start: Coord,
    inferred_start_cell_kind: Cell,
}

impl FromStr for Grid {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let width = s.lines().next().unwrap().chars().count();
        let mut start = None;
        let grid = s
            .lines()
            .enumerate()
            .map(|(row, l)| {
                if l.len() != width {
                    panic!("row width is expected to be {width}, got {}: {l}", l.len());
                }

                l.split_inclusive(|_| true)
                    .map_parse()
                    .enumerate()
                    .inspect(|&(col, s)| {
                        if s == Cell::Start {
                            assert_eq!(start, None, "new: {:?}", (row, col));
                            start = Some((row, col));
                        }
                    })
                    .map(|(_, c)| c)
                    .collect_vec()
            })
            .collect_vec();

        let start @ (start_row, start_col) = start.unwrap();

        Ok(Self {
            grid,
            start,
            inferred_start_cell_kind: Cell::Start,
            distances: None,
            enclosed: None,
        })
    }
}

fn coord_iter<T: Copy>(grid: &Vec<Vec<T>>) -> impl Iterator<Item = (Coord, T)> + '_ {
    grid.iter().enumerate().flat_map(|(row_idx, row)| {
        row.iter()
            .enumerate()
            .map(move |(col_idx, &cell)| ((row_idx, col_idx), cell))
    })
}

#[rustfmt::skip]
impl Grid {
    pub fn height(&self) -> usize { self.grid.len() }
    pub fn width(&self) -> usize { self.grid[0].len() }

    pub fn cell_iter(&self) -> impl Iterator<Item = (Coord, Cell)> +'_ {
        coord_iter(&self.grid)
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // let furthest = self
        //     .distances
        //     .iter()
        //     .flat_map(|r| r.iter())
        //     .flat_map(|o| o)
        //     .max()
        //     .copied()
        //     .unwrap()
        //     .max(1);

        // let (furthest, furthest_coord) = self.furthest();
        // let furthest = furthest.max(1);
        let (furthest, furthest_coord) = self
            .distances
            .as_ref()
            .map(|d| Self::furthest_inner(self.start, d))
            .map(|(f, c)| (f.max(1), c))
            .unzip();

        for (row_idx, row) in self.grid.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                // If on path, color by distance from start:
                let dist = &self.distances.as_ref().and_then(|d| d[row_idx][col_idx]);
                if let Some(dist) = dist {
                    // scale 0..furthest to 200..0
                    let intensity = 200 - (200 * dist / furthest.unwrap());
                    write!(
                        f,
                        "\u{001b}[38;2;0;{green};0m",
                        green = (255 - 200) + intensity
                    )?;
                } else {
                    // if enclosed, highlight in blue
                    let enclosed = self
                        .enclosed
                        .as_ref()
                        .map(|e| e[row_idx][col_idx])
                        .unwrap_or(IsEnclosed::No);
                    if enclosed == IsEnclosed::Yes {
                        write!(f, "\u{001b}[48;2;0;0;255m")?;
                    } else {
                        // otherwise: unreachable; dimmed:
                        write!(f, "\u{001b}[2m")?;
                    }
                }

                // Color the furthest coordinate in red:
                if Some((row_idx, col_idx)) == furthest_coord {
                    // RED
                    write!(f, "\u{001b}[38;2;255;0;0m",)?;
                }

                cell.fmt(f)?;
                "\u{001b}[0m".fmt(f)?;
            }
            "\n".fmt(f)?;
        }

        // "\u{001b}[0m".fmt(f)?;
        Ok(())
    }
}

const VIS: bool = false; // true;

impl Grid {
    fn find_distances(&mut self) -> &Vec<Vec<Option<usize>>> {
        use Cell::*;

        // if not already computed:
        if self.distances.is_none() {
            // mark the start's distance appropriately...
            let (start_row, start_col) = self.start;
            let mut distances = vec![vec![None; self.width()]; self.height()];
            distances[start_row][start_col] = Some(0);

            let mut queue = VecDeque::with_capacity(self.width() * 2);

            // begin with all the nodes _adjacent_ to the start that lead to the
            // start
            let vert = Vert.adjacent(self.start);
            let horz = Horz.adjacent(self.start);
            queue.extend(
                vert.into_iter()
                    .chain(horz)
                    .into_iter()
                    .filter_map(|c| c)
                    .filter(|&(row, col)| {
                        let cell = self.grid[row][col];
                        cell.adjacent((row, col))
                            .into_iter()
                            .any(|c| c == Some(self.start))
                    })
                    .map(|c| (c, 1)),
            );

            // use the above to infer the starting cell's true kind:
            let start_adjacent = queue.iter().map(|&(c, _)| c).sorted().collect_vec();
            for kind in [Vert, Horz, NorthEast, NorthWest, SouthEast, SouthWest]
                .into_iter()
                .chain(iter::from_fn(|| panic!("no match for start")))
            {
                if kind
                    .adjacent(self.start)
                    .into_iter()
                    .filter_map(|c| c)
                    .sorted()
                    .eq(start_adjacent.iter().copied())
                {
                    self.inferred_start_cell_kind = kind;
                    break;
                }
            }

            while let Some(((row, col), new_dist)) = queue.pop_front() {
                if row >= self.height() || col >= self.width() {
                    continue;
                }

                // Update dist, if smaller:
                let dist = &mut distances[row][col];
                if let Some(old_dist) = dist {
                    if *old_dist <= new_dist {
                        continue;
                    }
                }
                *dist = Some(new_dist);

                // Visit adjacent nodes:
                queue.extend(
                    self.grid[row][col]
                        .adjacent((row, col))
                        .into_iter()
                        .filter_map(|c| c)
                        .map(|c| (c, new_dist + 1)),
                );

                if VIS {
                    // eprintln!("\u{001b}[2J");
                    eprintln!("{self}");
                    let mut s = String::new();
                    std::io::stdin().read_line(&mut s).unwrap();
                }
            }

            self.distances = Some(distances);
        }

        self.distances.as_ref().unwrap()
    }

    fn furthest_inner(start: Coord, distances: &Vec<Vec<Option<usize>>>) -> (usize, Coord) {
        let mut dist = 0;
        let mut coord = start;

        for ((r, c), cell) in coord_iter(distances) {
            if let Some(d) = cell {
                if d > dist {
                    dist = d;
                    coord = (r, c);
                }
            }
        }

        (dist, coord)
    }

    pub fn furthest(&mut self) -> (usize, Coord) {
        Self::furthest_inner(self.start, self.find_distances())
    }
}

impl Grid {
    fn find_enclosed(&mut self) -> &Vec<Vec<IsEnclosed>> {
        if self.enclosed.is_none() {
            let mut enclosed = vec![vec![IsEnclosed::No; self.width()]; self.width()];

            for (row, col) in (0..self.height()).cartesian_product(0..self.width()) {
                enclosed[row][col] = if let Some(_) = self.find_distances()[row][col] {
                    IsEnclosed::OnLoop
                } else {
                    // eprintln!("enc analysis: ({row}, {col})");
                    // https://en.wikipedia.org/wiki/Even%E2%80%93odd_rule
                    //
                    // let's do a horizontal check (we're row-major so
                    // iterating over columns in a row should yield better cache
                    // locality for big graphs?)
                    //
                    // could do lots of caching here but not going to bother
                    // for now
                    let crossings: usize = (0..col)
                        .map(|c| (row, c))
                        .map(|(row, col)| {
                            let cell = self.grid[row][col];
                            let kind = match cell {
                                Cell::Start => self.inferred_start_cell_kind,
                                // pipes that are not on the loop don't
                                // contribute to the count:
                                other if self.find_distances()[row][col].is_some() => other,
                                _ => Cell::Empty,
                            };

                            match kind {
                                // // ┏ + ┛ → in
                                // // ┏ + ┓ → out
                                // // ┏ + ┗ → indeterminate
                                // // ┏ + ┏ → not possible, in sequence?
                                //
                                // // ┛ + ┛ → not possible, in sequence?
                                // // ┛ + ┓ →
                                // // ┛ + ┗ →
                                // // ┛ + ┏ → indeterminate

                                // since we're doing a horizontal check, `—` doesn't
                                // contribute to the count
                                Cell::Horz | Cell::Start | Cell::Empty => 0,
                                // // ┏ and ┗ too (ends with horizontal)
                                // Cell::NorthEast | Cell::SouthEast => 1,
                                Cell::SouthWest | Cell::SouthEast => 0,

                                // Cell::NorthWest => 3,
                                // Cell::SouthWest => 3,

                                //
                                Cell::Vert | Cell::NorthWest | Cell::NorthEast => {
                                    // eprintln!("  - crosses: ({row}, {col})");
                                    1
                                }
                            }
                        })
                        .sum();

                    // eprintln!("  - total crossings: {crossings}\n");
                    if crossings % 2 == 1 {
                        IsEnclosed::Yes
                    } else {
                        IsEnclosed::No
                    }
                };
            }

            self.enclosed = Some(enclosed);
        }

        self.enclosed.as_ref().unwrap()
    }

    pub fn enclosed(&mut self) -> impl Iterator<Item = (Coord, IsEnclosed)> + '_ {
        coord_iter(self.find_enclosed())
    }
}

const INP: &str = "-L|F7
7S-7|
L|7||
-L-J|
L|-JF";

const INP2: &str = "..F7.
.FJ|.
SJ.L7
|F--J
LJ...";

const INP3: &str = ".F----7F7F7F7F-7....
.|F--7||||||||FJ....
.||.FJ||||||||L7....
FJL7L7LJLJ||LJ.L-7..
L--J.L7...LJS7F-7L7.
....F-J..F7FJ|L7L7L7
....L7.F7||L7|.L7L7|
.....|FJLJ|FJ|F7|.LJ
....FJL-7.||.||||...
....L---J.LJ.LJLJ...";

const INP4: &str = "FF7FSF7F7F7F7F7F---7
L|LJ||||||||||||F--J
FL-7LJLJ||||||LJL-77
F--JF--7||LJLJ7F7FJ-
L---JF-JLJ.||-FJLJJ7
|F|F-JF---7F7-L7L|7|
|FFJF7L7F-JF7|JL---7
7-L-JL7||F7|L7F-7F7|
L.L7LFJ|||||FJL7||LJ
L7JLJL-JLJLJL--JLJ.L";

fn main() {
    let mut aoc = AdventOfCode::new(2023, 10);
    let input = aoc.get_input();
    // let input = INP4;
    let mut grid: Grid = input.parse().unwrap();

    // eprintln!("{grid}");
    _ = aoc.submit_p1(grid.furthest().0);

    _ = aoc.submit_p2(
        grid.enclosed()
            .filter(|&(_, e)| e == IsEnclosed::Yes)
            .count(),
    );

    // grid.find_enclosed();
    eprintln!("{grid}");

    // let furthest = grid.furthest().0;
    // _ = aoc.submit_p1(furthest);
}

// For part 2:
//
// It didn't occur to me but a clever solution other people used is to replace
// each cell with a 3x3 grid with the original cell at the center:
//
//
// ```
//  ┃   ━   ┗   ┛   ┓   ┏
//
// to:
// .┃. ... .┃. .┃. ... ...
// .┃. ━━━ .┗━ ━┛. ━┓. .┏━
// .┃. ... ... ... .┃. .┃.
// ```
//
// This adds extra space through which otherwise enclosed (but not inside the
// loop) cells can be reached from the outside of the grid allowing you to do a
// flood-fill to identify these cells.

// https://en.wikipedia.org/wiki/Pick%27s_theorem
// https://en.wikipedia.org/wiki/Shoelace_formula
