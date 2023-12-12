#!/usr/bin/env rustr

use std::iter;

use aoc::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Line {
    start: (usize, usize),
    end: (usize, usize),
}

impl FromStr for Line {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (start, end) = s.split_once(" -> ").unwrap();
        Ok(Self {
            start: start.split(',').map_parse().tuple::<2>(),
            end: end.split(',').map_parse().tuple::<2>(),
        })
    }
}

impl Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{},{} -> {}{}",
            self.start.0, self.start.1, self.end.0, self.end.1
        )
    }
}

impl Line {
    pub fn is_parallel_to_axes(&self) -> bool {
        self.start.0 == self.end.0 || self.start.1 == self.end.1
    }

    fn stride_and_len(&self) -> Option<((isize, isize), usize)> {
        let Self {
            start: (x1, y1),
            end: (x2, y2),
        } = *self;

        let (x1, y1): (isize, isize) = (x1.to(), y1.to());
        let (x2, y2) = (x2.to(), y2.to());

        let len_x: isize = x1.checked_sub(x2).unwrap().abs();
        let len_y: isize = y1.checked_sub(y2).unwrap().abs();
        let x_stride = (x2 - x1).checked_div(len_x);
        let y_stride = (y2 - y1).checked_div(len_y);

        match (len_x, len_y) {
            (x, y) if x == y => Some(((x_stride.unwrap(), y_stride.unwrap()), len_x.to())),
            (0, 0) => None,
            (0, y) => Some(((0, y_stride.unwrap()), y.to())),
            (x, 0) => Some(((x_stride.unwrap(), 0), x.to())),
            _ => None,
        }
    }

    pub fn iter(&self) -> Option<impl Iterator<Item = (usize, usize)> + '_> {
        if let Some(((xs, ys), len)) = self.stride_and_len() {
            let mut len = Some(len);
            let Self { start: (x, y), .. } = *self;
            let (mut x, mut y): (isize, isize) = (x.to(), y.to());
            Some(iter::from_fn(move || {
                if let Some(ref mut l) = len {
                    let ret = Some((x.to(), y.to()));

                    x += xs;
                    y += ys;

                    if *l == 0 {
                        len.take();
                    } else {
                        *l -= 1;
                    }

                    ret
                } else {
                    None
                }
            }))
        } else {
            None
        }

        // let (stride, len) = if self.is_parallel_to_axes() {
        // } else if let Some(stride) = self.diagonal_stride() {
        //     Some((stride, self.start.0.to().checked))
        // } else {
        //     None
        // }?;

        // if self.is_parallel_to_axes() {
        //     let Self {
        //         start: (x1, y1),
        //         end: (x2, y2),
        //     } = *self;

        //     let func = move |n| if x1 == x2 { (x1, n) } else { (n, y1) };

        //     let base = if x1 == x2 {
        //         (y1.min(y2))..=(y1.max(y2))
        //     } else {
        //         (x1.min(x2))..=(x1.max(x2))
        //     };

        //     Some(base.map(func))
        // } else {
        //     None
        // }
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2021, 5);
    let lines = aoc.get_input().lines().map_parse::<Line>().collect_vec();

    // let (max_x, max_y) = (
    //     lines.iter().map(|l| l.start.0.max(l.end.0)).max().unwrap(),
    //     lines.iter().map(|l| l.start.1.max(l.end.1)).max().unwrap(),
    // );
    // let mut map = (0..=max_x)
    //     .map(|_| (0..=max_y).map(|_| 0u8).collect_vec())
    //     .collect_vec();

    let p1 = lines
        .iter()
        .filter(|l| l.is_parallel_to_axes())
        .flat_map(|l| l.iter().unwrap())
        .counts()
        .into_iter()
        .filter(|(_, count)| *count >= 2)
        .count();
    aoc.submit_p1(p1).unwrap();

    let p2 = lines
        .iter()
        .filter_map(|l| l.iter())
        .flatten()
        .counts()
        .into_iter()
        .filter(|(_, count)| *count >= 2)
        .count();
    aoc.submit_p2(p2).unwrap();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_line_iter() {
        assert_eq!(
            Line::from_str("0,9 -> 5,9")
                .unwrap()
                .iter()
                .unwrap()
                .collect_vec(),
            (0..=5).map(|x| (x, 9)).collect_vec()
        );
    }
}
