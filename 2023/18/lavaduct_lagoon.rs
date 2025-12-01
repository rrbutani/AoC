use aoc::{
    iterator_map_ext::IterMapExt, itertools::MinMaxResult, AdventOfCode, Display, FromStr,
    Itertools,
};
use owo_colors::{OwoColorize, Rgb};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumString)]
enum Direction {
    #[strum(serialize = "U")]
    North,
    #[strum(serialize = "R")]
    East,
    #[strum(serialize = "D")]
    South,
    #[strum(serialize = "L")]
    West,
}

impl Direction {
    const ALL: [Self; 4] = {
        use Direction::*;
        [North, East, South, West]
    };
}

impl Direction {
    fn as_offs(self) -> (isize, isize) {
        use Direction::*;
        match self {
            North => (-1, 0),
            East => (0, 1),
            South => (1, 0),
            West => (0, -1),
        }
    }

    fn apply(self, (r, c): Coord, (h, w): Coord) -> Option<Coord> {
        let (dr, dc) = self.as_offs();
        let r = r.checked_add_signed(dr).filter(|&r| r < h);
        let c = c.checked_add_signed(dc).filter(|&c| c < w);
        r.zip(c)
    }
}

type Coord = (usize, usize);
type Color = (u8, u8, u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Step {
    len: usize,
    dir: Direction,
    color: Color,
}

impl FromStr for Step {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (dir, len, color) = s.split_whitespace().collect_tuple().unwrap();
        let dir = dir.parse().unwrap();
        let len = len.parse().unwrap();
        let color = {
            let color = color.strip_prefix("(#").unwrap().strip_suffix(")").unwrap();
            assert_eq!(color.len(), 6);
            let (r, rest) = color.split_at(2);
            let (g, b) = rest.split_at(2);

            (
                u8::from_str_radix(r, 16).unwrap(),
                u8::from_str_radix(g, 16).unwrap(),
                u8::from_str_radix(b, 16).unwrap(),
            )
        };

        Ok(Self { len, dir, color })
    }
}

impl Step {
    fn using_hex_codes(&self) -> Step {
        let (r, g, b) = self.color;
        let num = (r as usize) << 16 | (g as usize) << 8 | (b as usize);
        let len = num / 16;
        let dir = num % 16;
        let dir = match dir {
            0 => Direction::East,
            1 => Direction::South,
            2 => Direction::West,
            3 => Direction::North,
            _ => panic!("invalid dir: {dir}"),
        };

        Step { len, dir, color: (255, 0, 0) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Cell {
    Empty,
    Trench(Color),
    Inner,
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Cell::Empty => '.'.dimmed().fmt(f),
            Cell::Trench((r, g, b)) => '#'.color(Rgb(r, g, b)).fmt(f),
            Cell::Inner => '#'.dimmed().fmt(f),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Grid {
    // TODO: use from common
    grid: Vec<Vec<Cell>>,
    height: usize,
    width: usize,
}

impl Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in &self.grid {
            for c in r {
                c.fmt(f)?;
            }
            "\n".fmt(f)?;
        }

        Ok(())
    }
}

impl Grid {
    fn new(steps: &[Step]) -> Self {
        let coords = steps
            .iter()
            .flat_map(|s| (0..s.len).map(|_| (s.dir, s.color)))
            .scan((0isize, 0isize), |(r, c), (dir, color)| {
                let (dr, dc) = dir.as_offs();

                let nr = r.checked_add(dr);
                let nc = c.checked_add(dc);
                let (nr, nc) = nr.zip(nc).unwrap();
                *r = nr;
                *c = nc;

                Some(((nr, nc), color))
            })
            .collect_vec();

        let MinMaxResult::MinMax(&row_offset, &max_row) =
            coords.iter().map(|((r, _), _)| r).minmax()
        else {
            panic!()
        };
        let MinMaxResult::MinMax(&col_offset, &max_col) =
            coords.iter().map(|((_, c), _)| c).minmax()
        else {
            panic!()
        };

        // expand by 1 in all directions:
        let (row_offset, max_row) = (row_offset - 1, max_row + 1);
        let (col_offset, max_col) = (col_offset - 1, max_col + 1);

        let height = (max_row - row_offset + 1) as usize;
        let width = (max_col - col_offset + 1) as usize;
        let mut grid = vec![vec![Cell::Empty; width]; height];

        let coords = coords.into_iter().map(move |((r, c), color)| {
            (((r - row_offset) as usize, (c - col_offset) as usize), color)
        });

        for ((r, c), color) in coords {
            grid[r][c] = Cell::Trench(color);
        }

        Self { grid, height, width }
    }

    fn mark_enclosed(&mut self) -> &mut Self {
        // Now find everything inside the trench:
        // gonna just take advantage of the extra `± 1` on height/width to do
        // a floodfill from the outside and then invert
        //
        // in general this doesn't work — you can have isolated pockets;
        // something like the odd-even rule is the "right way" to do this
        //
        // but for the given inputs its fine
        fn floodfill(grid: &mut Vec<Vec<Cell>>, dimensions: Coord, (r, c): Coord) {
            use Cell::*;
            match grid[r][c] {
                Trench(_) | Inner => {}
                Empty => {
                    grid[r][c] = Inner;
                    Direction::ALL
                        .iter()
                        .filter_map(|d| d.apply((r, c), dimensions))
                        .for_each(|c| floodfill(grid, dimensions, c))
                }
            }
        }
        floodfill(&mut self.grid, (self.height, self.width), (0, 0));

        // now invert:
        use Cell::*;
        for row in self.grid.iter_mut() {
            for c in row.iter_mut() {
                *c = match *c {
                    Empty => Inner,
                    Inner => Empty,
                    Trench(c) => Trench(c),
                };
            }
        }

        self
    }

    fn count_enclosed(&self) -> usize {
        use Cell::*;
        self.grid
            .iter()
            .flat_map(|r| r.iter())
            .filter(|c| matches!(c, Trench(_) | Inner))
            .count()
    }

    /// https://en.wikipedia.org/wiki/Pick%27s_theorem
    /// https://en.wikipedia.org/wiki/Shoelace_formula
    fn enclosed_alt(&self) -> usize {
        todo!()
    }
}

const INP: &str = "R 6 (#70c710)
D 5 (#0dc571)
L 2 (#5713f0)
D 2 (#d2c081)
R 2 (#59c680)
D 2 (#411b91)
L 5 (#8ceee2)
U 2 (#caa173)
L 1 (#1b58a2)
U 2 (#caa171)
R 2 (#7807d2)
U 3 (#a77fa3)
L 2 (#015232)
U 2 (#7a21e3)
";

fn main() {
    let mut aoc = AdventOfCode::new(2023, 18);
    let inp = aoc.get_input();
    let inp = INP;
    let steps = inp.lines().map_parse::<Step>().collect_vec();
    let mut grid = Grid::new(&steps);

    let p1 = grid.mark_enclosed().count_enclosed();
    // _ = aoc.submit_p1(p1);

    eprintln!("{grid}");
}
