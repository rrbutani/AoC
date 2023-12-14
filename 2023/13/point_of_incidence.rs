use std::num::NonZeroUsize;

use aoc::{iterator_map_ext::IterMapExt, AdventOfCode, Display, FromStr, Itertools};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumString, strum::Display,
)]
enum Cell {
    #[strum(serialize = ".")]
    Ash,
    #[strum(serialize = "#")]
    Rock,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Pattern {
    grid: Vec<Vec<Cell>>,
}

impl FromStr for Pattern {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let grid = s
            .lines()
            .map(|l| l.split_inclusive(|_| true).map_parse().collect_vec())
            .collect_vec();

        let width = grid[0].len();
        for (r, row) in grid.iter().enumerate() {
            assert_eq!(row.len(), width, "row {} width", r);
        }

        Ok(Self { grid })
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.grid {
            for c in row {
                c.fmt(f)?;
            }
            "\n".fmt(f)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ReflectionLine {
    Vertical(usize),
    Horizontal(usize),
}

impl Into<usize> for ReflectionLine {
    fn into(self) -> usize {
        use ReflectionLine::*;
        match self {
            Vertical(i) => i,
            Horizontal(i) => 100 * i,
        }
    }
}

fn check_vert_reflection<T: PartialEq>(
    grid: &Vec<Vec<T>>,
    reflect_idx: NonZeroUsize, // in [1, width)
    height: usize,
    width: usize,
    discrepancy_limit: usize,
) -> usize {
    assert!(reflect_idx.get() < width);

    let mut discrepancies = 0;
    for col in 0..reflect_idx.get() {
        let translated_col = (reflect_idx.get() - 1 - col) + reflect_idx.get();
        if translated_col >= width {
            continue;
        }
        for row in 0..height {
            if grid[row][col] != grid[row][translated_col] {
                // eprintln!(
                //     "[vert({:2})] mismatch at row: {}; col: {} vs col: {}",
                //     reflect_idx.get(),
                //     row,
                //     col,
                //     translated_col
                // );
                discrepancies += 1;
                if discrepancies > discrepancy_limit {
                    return discrepancies;
                }
            }
        }
    }

    discrepancies
}

fn check_horiz_reflection<T: PartialEq>(
    grid: &Vec<Vec<T>>,
    reflect_idx: NonZeroUsize, // in [1, height)
    height: usize,
    width: usize,
    discrepancy_limit: usize,
) -> usize {
    assert!(reflect_idx.get() < height);

    let mut discrepancies = 0;
    for row in 0..reflect_idx.get() {
        let translated_row = (reflect_idx.get() - 1 - row) + reflect_idx.get();
        if translated_row >= height {
            continue;
        }
        for col in 0..width {
            if grid[row][col] != grid[translated_row][col] {
                discrepancies += 1;
                if discrepancies > discrepancy_limit {
                    return discrepancies;
                }
            }
        }
    }

    discrepancies
}

impl Pattern {
    fn width(&self) -> usize {
        self.grid[0].len()
    }
    fn height(&self) -> usize {
        self.grid.len()
    }

    fn find_line_with_n_discrepancies(&self, n: usize) -> ReflectionLine {
        use ReflectionLine::*;
        let (height, width) = (self.height(), self.width());
        let try_reflect = |func: fn(&_, _, _, _, _) -> usize, limit, ctor: fn(_) -> _| {
            for i in 1..limit {
                let idx = NonZeroUsize::new(i).unwrap();
                if func(&self.grid, idx, height, width, n) == n {
                    return Some(ctor(i));
                }
            }

            None
        };

        try_reflect(check_horiz_reflection, height, Horizontal)
            .or_else(|| try_reflect(check_vert_reflection, width, Vertical))
            .unwrap()
    }

    pub fn find_reflection_line(&self) -> ReflectionLine {
        self.find_line_with_n_discrepancies(0)
    }

    pub fn find_one_off(&self) -> ReflectionLine {
        self.find_line_with_n_discrepancies(1)
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2023, 13);
    let inp = aoc.get_input();
    let patterns = inp.split("\n\n").map_parse().collect_vec();

    let p1: usize = patterns
        .iter()
        .map(|p: &Pattern| p.find_reflection_line())
        .map(Into::<usize>::into)
        .sum();
    _ = aoc.submit_p1(p1);

    let p2: usize = patterns
        .iter()
        .map(|p: &Pattern| p.find_one_off())
        .map(Into::<usize>::into)
        .sum();
    _ = aoc.submit_p2(p2);
}
