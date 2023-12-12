use std::{
    collections::{HashMap, HashSet},
    iter,
};

use aoc::*;
use owo_colors::OwoColorize;

const INP: &str = "467..114..
...*......
..35..633.
......#...
617*......
.....+.58.
..592.....
......755.
...$.*....
.664.598..";

struct Grid {
    rows: Vec<Vec<char>>,
}

impl FromStr for Grid {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rows: Vec<Vec<_>> = s.lines().map(|l| l.chars().collect()).collect();

        let col_len = rows[0].len();
        for row in &rows {
            assert_eq!(col_len, row.len());
        }

        Ok(Grid { rows })
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct Span {
    row_num: usize,
    col_start: usize,
    col_end: usize,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct Number {
    num: usize,
    span: Span,
}

fn line(
    len: usize,
    left_right_pad: isize,
    row_offs: isize,
    start: (usize, usize),
) -> impl Iterator<Item = (usize, usize)> + Clone {
    (-left_right_pad..(len as isize + left_right_pad)).filter_map(move |col_offs| {
        Some((
            start.0.checked_add_signed(row_offs)?,
            start.1.checked_add_signed(col_offs)?,
        ))
    })
}

impl Number {
    // height is implicitly 1
    fn length(&self) -> usize {
        self.span.col_end + 1 - self.span.col_start
    }

    // (row, col)
    fn surrounding_coords(&self) -> impl Iterator<Item = (usize, usize)> {
        let start = (self.span.row_num, self.span.col_start);
        let range = |row_offs: isize| line(self.length(), 1, row_offs, start);

        // includes the coords for the number but that's okay
        range(-1).chain(range(0)).chain(range(1))
    }

    fn coords(&self) -> impl Iterator<Item = (usize, usize)> {
        let start = (self.span.row_num, self.span.col_start);
        let range = |row_offs: isize| line(self.length(), 0, row_offs, start);

        range(0)
    }
}

impl Grid {
    fn find_numbers(&self) -> impl Iterator<Item = Number> + '_ {
        let mut rows = self.rows.iter().enumerate();
        let mut curr_row: Option<(usize, iter::Peekable<_>)> = None;
        iter::from_fn(move || {
            loop {
                let (row_idx, row_it) = match &mut curr_row {
                    Some((row_idx, ref mut it)) => {
                        if it.peek().is_some() {
                            (row_idx, it)
                        } else {
                            curr_row = None;
                            continue;
                        }
                    }
                    _ => {
                        let Some((row_idx, next)) = rows.next() else {
                            return None;
                        };

                        // note: extra char to "flush"
                        curr_row = Some((
                            row_idx,
                            next.iter().chain(iter::once(&'.')).enumerate().peekable(),
                        ));
                        let (row_idx, ref mut it) = curr_row.as_mut().unwrap();
                        (row_idx, it)
                    }
                };

                let mut num = 0;
                let mut start_col = 0;
                for (col_idx, char) in row_it {
                    match char {
                        c if c.is_ascii_alphanumeric() => {
                            if num == 0 {
                                start_col = col_idx;
                            }
                            num = num * 10 + c.to_digit(10).unwrap() as usize;
                        }
                        _ if num == 0 => {
                            // haven't found a number yet, keep going!
                        }
                        _ if num != 0 => {
                            // end of number, yield:
                            return Some(Number {
                                num,
                                span: Span {
                                    row_num: *row_idx,
                                    col_start: start_col,
                                    col_end: col_idx - 1,
                                },
                            });
                        }
                        _ => unreachable!(),
                    }
                }
            }
        })
    }

    fn is_part_number(&self, num: &Number) -> bool {
        num.surrounding_coords()
            .filter(|&(r, c)| r < self.rows.len() && c < self.rows[0].len())
            .any(|(r, c)| {
                let cell: char = self.rows[r][c];
                !(cell.is_ascii_alphanumeric() || cell == '.')
            })
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2023, 3);
    let inp = aoc.get_input();
    // let inp = INP;

    let grid: Grid = inp.parse().unwrap();
    let p1: usize = grid
        .find_numbers()
        .filter(|n| grid.is_part_number(n))
        .map(|n| n.num)
        .sum();
    dbg!(p1);

    let mut grid_view = grid
        .rows
        .iter()
        .map(|r| r.iter().map(|c| String::from(*c)).collect_vec())
        .collect_vec();
    for num in grid.find_numbers() {
        let is_part_num = grid.is_part_number(&num);
        for (r, c) in num.surrounding_coords() {
            if r >= grid.rows.len() || c >= grid.rows[0].len() {
                continue;
            }
            grid_view[r][c] = if is_part_num {
                format!("{}", grid_view[r][c].green())
            } else {
                format!("{}", grid_view[r][c].red())
            };
        }
    }
    for row in grid_view {
        eprintln!("{}", row.join(""))
    }
    aoc.submit_p1(p1).unwrap();

    fn box_coords(center: (usize, usize)) -> impl Iterator<Item = (usize, usize)> + Clone {
        let range = |row_offs: isize| line(1, 1, row_offs, center);

        // includes the coords for the number but that's okay
        range(-1).chain(range(0)).chain(range(1))
    }

    let mut map: HashMap<(usize, usize), Number> =
        HashMap::with_capacity(grid.rows.len() * grid.rows[0].len());
    for num in grid.find_numbers() {
        let is_part_num = grid.is_part_number(&num);
        if !is_part_num {
            continue;
        }
        for coord in num.coords() {
            assert!(!map.contains_key(&coord));
            map.insert(coord, num.clone());
        }
    }

    let mut gear_ratio_sum = 0;
    for (r, row) in grid.rows.iter().enumerate() {
        for (c, &col) in row.iter().enumerate() {
            if col == '*' {
                // eprintln!("Potential gear at ({r},{c}):");
                let adj = box_coords((r, c));
                let adjacent_part_numbers = adj.filter_map(|c| map.get(&c)).collect::<HashSet<_>>();
                let adjacent_part_number_count = adjacent_part_numbers.len();
                if adjacent_part_number_count == 2 {
                    gear_ratio_sum += adjacent_part_numbers
                        .iter()
                        .map(|n| n.num)
                        .reduce(|a, b| a * b)
                        .unwrap();
                }
            }
        }
    }
    let p2 = gear_ratio_sum;
    dbg!(p2);
    aoc.submit_p2(p2).unwrap();
}
