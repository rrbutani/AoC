use aoc::{
    grid::{ExtendedDirection, TwoDimensionalGrid},
    *,
};
use show_match::ShowMatch;

#[cfg(debug_assertions)]
mod show_match {
    use std::collections::HashSet;

    use aoc::{grid::TwoDimensionalGrid, Coord};

    #[derive(Debug, Clone)]
    pub struct ShowMatch<'g> {
        grid: &'g TwoDimensionalGrid<char>,
        coords: HashSet<Coord>,
    }

    impl<'g> ShowMatch<'g> {
        pub fn new(grid: &'g TwoDimensionalGrid<char>) -> Self {
            Self { grid, coords: HashSet::new() }
        }

        pub fn push(&mut self, coord: Coord) {
            self.coords.insert(coord);
        }

        pub fn matched(self) {
            let mut new_grid = self.grid.clone();
            for r in 0..new_grid.height() {
                for c in 0..new_grid.width() {
                    let coord = (r, c).into();
                    if self.coords.contains(&coord) {
                        continue;
                    } else {
                        new_grid[coord] = '.';
                    }
                }
            }

            eprintln!("{new_grid}\n");
        }
    }
}

#[cfg(not(debug_assertions))]
mod show_match {
    use aoc::grid::{Coord, TwoDimensionalGrid};

    #[derive(Debug, Clone)]
    pub struct ShowMatch<'g> {
        _g: Option<&'g ()>,
    }

    impl<'g> ShowMatch<'g> {
        pub fn new(_grid: &'g TwoDimensionalGrid<char>) -> Self {
            ShowMatch { _g: None }
        }
        pub fn push(&mut self, _coord: Coord) {}
        pub fn matched(self) {}
    }
}

fn check_match<'g>(
    grid: &'g TwoDimensionalGrid<char>,
    word: &str,
    offs: ExtendedDirection,
    mut curr: Coord,
    mut m: ShowMatch<'g>,
) -> Option<ShowMatch<'g>> {
    let mut chars = word.chars().peekable();
    loop {
        match (chars.next(), grid[curr]) {
            (Some(n), c) if n == c => {
                // matched!
                m.push(curr);

                // if we matched the whole string, return true
                if chars.peek().is_none() {
                    break Some(m);
                }

                // if there are chars left, keep going, if still in bounds
                curr = if let Some(curr) = offs.apply(curr).filter(|&c| grid.get(c).is_some()) {
                    curr
                } else {
                    break None;
                };
            }
            // didn't match, move to the next starting coord and try again
            (Some(_), _) => break None,
            // shouldn't get here unless `word` is empty..
            (None, _) => unreachable!(),
        }
    }
}

pub fn check_x_match<'g>(
    grid: &'g TwoDimensionalGrid<char>,
    word: &str,
    dir: ExtendedDirection,
    curr: Coord,
    m: ShowMatch<'g>,
) -> Option<ShowMatch<'g>> {
    // it's an X if the current dir matches AND one of:
    //   - dir rotated 90° matches
    //   - dir rotated 270° matches (i.e. 90° but backwards)

    // len must be odd
    assert!(word.len() % 2 == 1, "{word} ({}; must have odd len)", word.len());

    let halfway_len = (word.len() - 1) / 2;
    let go_to_corner = |new_dir: ExtendedDirection| {
        let mut pos = curr;
        // uhhhhh? maybe: don't just burn CPU cycles??
        for _ in 0..halfway_len {
            pos = dir.apply(pos).unwrap();
        }

        let reversed_new_dir = new_dir.rotate(4);
        for _ in 0..halfway_len {
            pos = reversed_new_dir.apply(pos).unwrap()
        }

        pos
    };

    if let Some(m) = check_match(grid, word, dir, curr, m) {
        let cw = dir.rotate(2);
        if let Some(m) = check_match(grid, word, cw, go_to_corner(cw), m.clone()) {
            return Some(m);
        }

        let ccw = dir.rotate(-2);
        return check_match(grid, word, ccw, go_to_corner(ccw), m);
    }

    None
}

pub fn search<const X: bool>(
    grid: &TwoDimensionalGrid<char>,
    word: &str,
    offs: ExtendedDirection,
) -> usize {
    assert!(!word.is_empty());

    let chk = if X { check_x_match } else { check_match };

    // naïve; doesn't skip intelligently
    grid.cell_iter()
        .filter(|&(p, _)| {
            let m = ShowMatch::new(grid);
            if let Some(m) = chk(grid, word, offs, p, m) {
                m.matched();
                return true;
            }

            false
        })
        .count()
}

const EX1: &str = "\
..X...
.SAMX.
.A..A.
XMAS.S
.X....";
const EX2: &str = "\
MMMSXXMASM
MSAMXMSMSA
AMXSXMAAMM
MSAMASMSMX
XMASAMXAMM
XXAMMXXAMA
SMSMSASXSS
SAXAMASAAA
MAMMMXMMMM
MXMXAXMASX";

fn main() {
    let mut aoc = AdventOfCode::new(2024, 4);
    let inp = aoc.get_input();
    let grid = TwoDimensionalGrid::<char>::from_str(&inp).unwrap();

    let p1: usize = ExtendedDirection::ALL
        .iter()
        .map(|&d| search::<false>(&grid, "XMAS", d))
        .sum();
    _ = aoc.submit_p1(p1);

    use ExtendedDirection as E;
    let p2: usize = /* ExtendedDirection::DIAGONALS */ [E::NorthEast, E::SouthWest]
        .iter()
        .map(|&d| search::<true>(&grid, "MAS", d))
        .sum();
    _ = aoc.submit_p2(p2);
}
