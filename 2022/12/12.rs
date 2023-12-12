use aoc::*;

use owo_colors::OwoColorize;
use std::{
    cmp::Reverse,
    collections::{BTreeSet, BinaryHeap, HashMap},
    fmt,
    iter::Rev,
    mem,
    ops::Index,
};

type Pos = (usize, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Height(u8);

impl TryFrom<char> for Height {
    type Error = ();

    fn try_from(c: char) -> Result<Self, ()> {
        match c {
            'a'..='z' => Ok(Height(c as u8 - b'a')),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Grid {
    grid: Vec<Vec<Height>>,
    start: Pos,
    end: Pos,
    height: usize,
    width: usize,
}

impl Index<Pos> for Grid {
    type Output = Height;

    fn index(&self, (y, x): Pos) -> &Self::Output {
        &self.grid[y][x]
    }
}

impl FromStr for Grid {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let height = s.lines().count();
        let width = s.lines().next().unwrap().chars().count();

        let mut start = None;
        let mut end = None;
        let grid = s
            .lines()
            .enumerate()
            .map(|(row, l)| {
                l.chars()
                    .enumerate()
                    .map(|(col, mut c)| {
                        c = match c {
                            'S' => {
                                start = Some((row, col));
                                'a'
                            }
                            'E' => {
                                end = Some((row, col));
                                'z'
                            }
                            c => c,
                        };

                        c.try_into().unwrap()
                    })
                    .collect_vec()
            })
            .collect_vec();

        Ok(Grid {
            grid,
            start: start.unwrap(),
            end: end.unwrap(),
            height,
            width,
        })
    }
}

// TODO: display impl with bolds

// struct Explorer<'g> {
//     dist_to_end: HashMap<Pos, Option<usize>>,
//     grid: &'g Grid,
// }

// impl<'g> Explorer<'g> {
//     pub fn new(grid: &'g Grid) -> Self {
//         let mut dist_to_end = HashMap::new();
//         dist_to_end.insert(grid.end, Some(0));

//         Explorer { dist_to_end, grid }
//     }

//     fn dist(&mut self, coord: Pos) -> Option<usize> {
//         dbg!(coord);
//         if let Some(d) = self.dist_to_end.get(&coord) {
//             *d
//         } else {
//             let dist = [
//                 /* up    */ (1, 0),
//                 /* right */ (0, 1),
//                 /* down  */ (-1, 0),
//                 /* left  */ (0, -1),
//             ]
//             .into_iter()
//             .map(|(dy, dx)| {
//                 let (y, x) = coord;
//                 (y as isize + dy, x as isize + dx)
//             })
//             .filter(|(y, x)| {
//                 (0..(self.grid.height as isize)).contains(y)
//                     && (0..(self.grid.width as isize)).contains(x)
//             })
//             .map(|(y, x)| (y as usize, x as usize))
//             .filter(|&next| ((self.grid[coord].0 as i8) - (self.grid[next].0 as i8)).abs() <= 1)
//             .filter_map(|next| self.dist(next))
//             .min()
//             .map(|d| d + 1);

//             self.dist_to_end.insert(coord, dist);
//             dist
//         }
//     }
// }

fn dijkstra(
    g: &Grid,
    start: Pos,
    is_goal: impl Fn(Pos) -> bool,
    connected: impl Fn(Height, Height) -> bool,
) -> Option<usize> {
    // let mut dist_to_end: HashMap<_, usize> = HashMap::new();
    let mut dist_to_end = g
        .grid
        .iter()
        .map(|row| row.iter().map(|_| usize::MAX).collect_vec())
        .collect_vec();
    dist_to_end[start.0][start.1] = 0;

    let mut unexplored: BinaryHeap<_> = BinaryHeap::from([(0, start)]);

    let mut shortest = None;
    while let Some((cost, coord)) = unexplored.pop() {
        // eprintln!("cost: {cost}, coord: {coord:?}");
        if is_goal(coord) {
            shortest = Some(shortest.unwrap_or(usize::MAX).min(cost));
            // eprintln!("shortest path is now via: {coord:?}");
        }

        // if dist_to_end.get(&coord).map(|&c| c < cost).unwrap_or(false) {
        //     continue; // we've already done better so move on
        // }
        if dist_to_end[coord.0][coord.1] < cost {
            continue; // we've already done better so move on
        }

        let neighbours = [
            /* up    */ (1, 0),
            /* right */ (0, 1),
            /* down  */ (-1, 0),
            /* left  */ (0, -1),
        ]
        .into_iter()
        .map(|(dy, dx)| {
            let (y, x) = coord;
            (y as isize + dy, x as isize + dx)
        })
        .filter(|(y, x)| {
            (0..(g.height as isize)).contains(y) && (0..(g.width as isize)).contains(x)
        })
        // .inspect(|n| {
        //     eprintln!("  neigh: {n:?}");
        // })
        .map(|(y, x)| (y as usize, x as usize))
        .filter(|&next| connected(g[coord], g[next]));
        // .filter(|&next| ((g[coord].0 as i8) - (g[next].0 as i8)).abs() <= 1);

        for neighbour in neighbours {
            let next @ (next_cost, (y, x)) = (cost + 1, neighbour);

            if next_cost < dist_to_end[y][x] {
                unexplored.push(next);
                dist_to_end[y][x] = next_cost;
            }
        }
    }

    for (r, row) in g.grid.iter().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            let cell = (cell.0 + b'a') as char;
            if (r, c) == g.start || (r, c) == g.end {
                eprint!("{}", cell.bold())
            } else {
                let dist = dist_to_end[r][c];
                let color = if dist == usize::MAX {
                    owo_colors::Rgb(255, 0, 0)
                } else {
                    owo_colors::Rgb(0, (dist / 2).min(u8::MAX as usize) as u8, 0)
                };
                eprint!("{}", cell.color(color))
            }
        }

        eprintln!();
    }

    shortest
}

fn main() {
    let mut aoc = AdventOfCode::new(2022, 12);
    let inp = aoc.get_input();
    // let inp = include_str!("ex");

    let g = Grid::from_str(&inp).unwrap();
    // let mut exp = Explorer::new(&g);
    // let p1 = exp.dist(g.start).unwrap();
    let connected = |this: Height, next: Height| this.0 + 1 >= next.0;
    let p1 = dijkstra(&g, g.start, |p| p == g.end, connected).unwrap();
    dbg!(p1);
    let connected = |end: Height, this: Height| this.0 + 1 >= end.0;
    let p1 = dijkstra(&g, g.end, |p| p == g.start, connected).unwrap();
    dbg!(p1);
    // panic!();
    aoc.submit_p1(dbg!(p1)).unwrap();

    let p2 = dijkstra(&g, g.end, |p| g[p].0 == 0, connected).unwrap();
    aoc.submit_p2(dbg!(p2)).unwrap();

    // aoc.submit_p1(dbg!(p1(&s, dbg))).unwrap();
    // aoc.submit_p2(dbg!(p2(&s, dbg))).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ex() -> State {
        State::from_str(include_str!("ex")).unwrap()
    }

    #[test]
    fn p1() {
        assert_eq!(super::p1(&ex(), false), 10605);
    }

    #[test]
    fn p2() {
        assert_eq!(super::p2(&ex(), false), 2713310158);
    }
}
