use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use aoc::{
    grid::{Direction, NextCoords},
    AdventOfCode, Coord, Display, Grid,
};
use smallvec::SmallVec;
use strum::{Display, EnumString};

impl TryFrom<Cell> for Direction {
    type Error = ();

    fn try_from(c: Cell) -> Result<Direction, ()> {
        use Cell::*;
        use Direction as D;
        Ok(match c {
            Up => D::Up,
            Right => D::Right,
            Down => D::Down,
            Left => D::Left,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumString, Display)]
enum Cell {
    #[strum(serialize = ".", to_string = "\u{001b}[31m.\u{001b}[0m")]
    Path,
    #[strum(serialize = "#", to_string = "\u{001b}[31m \u{001b}[0m")]
    Forest,
    #[strum(serialize = "^", to_string = "\u{001b}[32m^\u{001b}[0m")]
    Up,
    #[strum(serialize = ">", to_string = "\u{001b}[33m>\u{001b}[0m")]
    Right,
    #[strum(serialize = "v", to_string = "\u{001b}[34mv\u{001b}[0m")]
    Down,
    #[strum(serialize = "<", to_string = "\u{001b}[35m<\u{001b}[0m")]
    Left,
}

fn longest_path_naive(grid: &mut Grid<Cell>, respect_slopes: bool) -> usize {
    let mut paths_to_end = Vec::new();
    let (start, end) = (grid.top_left() + (0, 1), grid.bottom_right() - (0, 1));

    // Just doing BFS normally is likely to explode the state space way too
    // much but... the maze seems to consist largely of corridors so maybe
    // it'll be okay?
    //
    // (and, of course, part 2 makes us OOM. see below)
    grid.search::<_, _, _, true>([start].into_iter(), |ctx, path| {
        // we do our filtering eagerly; this function is only invoked on
        // coordinates that are valid:
        let mut path: HashSet<_> = path.unwrap_or_default();
        let new = path.insert(ctx.coord());
        debug_assert!(new);

        if ctx.coord() == end {
            paths_to_end.push(path.len() - 1);
            return None;
        }

        use Cell::*;
        let single;
        let next_directions = match *ctx.cell() {
            Path => Direction::ALL.as_slice(),
            Forest => unreachable!("{:?}", ctx.coord()),
            dir => {
                if respect_slopes {
                    single = [dir.try_into().unwrap()];
                    &single
                } else {
                    Direction::ALL.as_slice()
                }
            }
        };

        let next_coords: SmallVec<_> = next_directions
            .iter()
            .filter_map(|d| d.get(ctx.coord(), ctx.grid()))
            .filter(|&c| match ctx.grid()[c] {
                Forest => false,
                _ => true,
            })
            .filter(|c| !path.contains(c))
            .collect::<SmallVec<[Coord; 4]>>();

        // optimization to save a clone:
        match *next_coords {
            [] => None,
            [next] => Some(SmallVec::<_>::from_iter([(next, path)])),
            _ => Some(
                next_coords
                    .into_iter()
                    .map(|c: Coord| (c, path.clone()))
                    .collect(),
            ),
        }
    });

    // dbg!(&paths_to_end);
    paths_to_end.into_iter().max().unwrap()
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord)]
struct Corridor {
    start: Coord,
    end: Coord,
    len: usize,
}

impl Corridor {
    fn normalize(self) -> Self {
        if self.start < self.end {
            self
        } else {
            Self { start: self.end, end: self.start, len: self.len }
        }
    }
}

impl Hash for Corridor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let Self { start, end, .. } = self.normalize();

        start.hash(state);
        end.hash(state);
        // len.hash(state); // important!
    }
}

impl PartialEq for Corridor {
    fn eq(&self, other: &Self) -> bool {
        let a = self.normalize();
        let b = other.normalize();

        a.start == b.start && a.end == b.end // && a.len == b.len // important!
    }
}

impl Eq for Corridor {}

#[test]
fn corridor_eq_hash() {
    let a = (0, 1usize).into();
    let b = (1, 0usize).into();

    let c_a = Corridor { start: a, end: b, len: 3 };
    let c_b = Corridor { start: b, end: a, len: 3 };
    assert_eq!(c_a, c_b);
    assert!(a < b);

    let mut hs = HashSet::new();
    assert!(hs.insert(c_a));
    assert!(!hs.insert(c_a));
    assert!(!hs.insert(c_b));
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Graph {
    nodes: HashSet<Coord>,
    edges: HashSet<Corridor>,
}

impl Display for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            // dot:
            "graph {\n".fmt(f)?;

            for Coord { row, col } in &self.nodes {
                writeln!(f, "  \"{row:03}_{col:03}\";")?;
            }
            writeln!(f)?;
            for Corridor {
                start: Coord { row: r1, col: c1 },
                end: Coord { row: r2, col: c2 },
                len,
            } in &self.edges
            {
                writeln!(f, "  \"{r1:03}_{c1:03}\" -- \"{r2:03}_{c2:03}\" [label=\"{len}\"];")?;
            }

            "}\n".fmt(f)?;
            Ok(())
        } else {
            unimplemented!()
        }
    }
}

impl Graph {
    fn next_map(&self) -> HashMap<Coord, Vec<(Coord, usize)>> {
        let mut map: HashMap<_, _> = self.nodes.iter().map(|&n| (n, Vec::new())).collect();

        for &Corridor { start, end, len } in &self.edges {
            map.get_mut(&start).unwrap().push((end, len));
            map.get_mut(&end).unwrap().push((start, len));
        }

        map
    }

    /*
    // dfs
    fn find_all_possible_paths(
        &self,
        start: Coord,
        end: Coord,
        next_map: &HashMap<Coord, Vec<(Coord, usize)>>,
        path_so_far: &mut Vec<Coord>,
        seen: &mut HashSet<Coord>,
        func: &mut impl FnMut(&Vec<Coord>),
        // paths: &mut Vec<Vec<Coord>>,
    ) {
        if start == end {
            func(&path_so_far);
            // paths.push(path_so_far.clone());
            return;
        }
        let nexts = next_map[&start].iter().map(|(c, _)| c);

        for &n in nexts {
            if seen.contains(&n) {
                continue;
            }

            path_so_far.push(n);
            seen.insert(n);
            self.find_all_possible_paths(n, end, next_map, path_so_far, seen, func);
            path_so_far.pop();
            seen.remove(&n);
        }
    }

    fn get_path_length(&self, path: &[Coord]) -> usize {
        path.windows(2)
            .map(|w| {
                let [start, end] = *w else { unreachable!() };
                self.edges
                    .get(&Corridor { start, end, len: 0 })
                    .unwrap()
                    .len
            })
            .sum()
    }
    */

    fn find_longest_path_inner(
        &self,
        start: Coord,
        end: Coord,
        next_map: &HashMap<Coord, Vec<(Coord, usize)>>,
        len_so_far: usize,
        seen: &mut HashSet<Coord>,
        longest_path: &mut usize,
    ) {
        if start == end {
            if len_so_far > *longest_path {
                *longest_path = len_so_far;
            }
            return;
        }

        // TODO: can parallelize here
        for &(n, edge_len) in &next_map[&start] {
            if seen.contains(&n) {
                continue;
            }

            seen.insert(n);
            self.find_longest_path_inner(
                n,
                end,
                next_map,
                len_so_far + edge_len,
                seen,
                longest_path,
            );
            seen.remove(&n);
        }
    }

    // fn find_longest_path(&self, start: Coord, end: Coord) -> (usize, Vec<Coord>) {
    fn find_longest_path(&self, start: Coord, end: Coord) -> usize {
        let next_map = self.next_map();

        /*
        // TODO: can probably use heuristics to prune this without enumerating
        // all the (1 million+) paths...
        // let mut paths = Vec::with_capacity(2_000_000);
        let mut path_so_far = vec![start];
        let mut seen = HashSet::from([start]);
        let mut longest_path_len = 0;
        self.find_all_possible_paths(
            start,
            end,
            &next_map,
            &mut path_so_far,
            &mut seen,
            &mut |p| {
                let len = self.get_path_length(p);
                if len > longest_path_len {
                    longest_path_len = len;
                }
            },
        );


        longest_path_len
        */

        // eprintln!("{} paths found", paths.len());

        // paths
        //     .into_iter()
        //     .map(|p| (self.get_path_length(&p), p))
        //     .max_by_key(|&(l, _)| l)
        //     .unwrap()

        // let mut path_lengths: HashMap<Coord, (usize, HashSet<Corridor>)> = self
        //     .nodes
        //     .iter()
        //     .map(|&n| (n, (0, HashSet::new())))
        //     .collect();

        // let mut queue = VecDeque::from([]);
        // while let Some(((to, extra_len), (from, path, len))) = queue.pop_front() {
        //     if path.contains(to) {
        //         continue;
        //     }
        //     let curr_len = path_lengths[to].0;
        //     let new_len = len + extra_len;
        //     if new_len <= curr_len
        // }

        // TODO: can probably use heuristics to prune this without enumerating
        // all the (1 million+) paths...
        let mut longest_path_len = 0;
        let mut seen = HashSet::from([start]);
        self.find_longest_path_inner(start, end, &next_map, 0, &mut seen, &mut longest_path_len);

        longest_path_len
    }
}

fn longest_path_no_slopes(grid: &mut Grid<Cell>) -> usize {
    let (start, end) = (grid.top_left() + (0, 1), grid.bottom_right() - (0, 1));

    // The input really is just a set of corridors connected by some Ts and some
    // four way junctions (guarded by > and v slopes). In my input there are 34
    // of these Ts/junctions.
    //
    // Under part 1, the slopes ensure a single direction of travel through
    // these Ts and junctions.
    //
    // I think the thing to do is to represent the grid as a graph (with
    // corridors as undirected edges) and to then just ... ???

    let mut nodes = HashSet::from([start, end]);
    let mut edges: HashSet<Corridor> = HashSet::new();

    grid.search::<_, _, _, false>([start].into_iter(), |ctx, actual_start| {
        // eprintln!("\nstart corridor trace: {} (actual start: {actual_start:?})", ctx.coord());

        use Cell::*;
        let start = actual_start.unwrap_or_else(|| ctx.coord());
        let mut len = if actual_start.is_some() { 1 } else { 0 };

        assert!(!matches!(ctx.cell(), Cell::Forest));

        // continue until we hit a fork (or the end)
        let mut curr = ctx.coord();
        let mut path = HashSet::from([start, curr]);
        let ret = loop {
            if curr == end {
                break None;
            }

            let next_coords = Direction::ALL
                .iter()
                .filter_map(|d| d.get(curr, ctx.grid()))
                .filter(|&c| match ctx.grid()[c] {
                    Forest => false,
                    _ => true,
                })
                .filter(|c| !path.contains(c))
                .collect::<SmallVec<[_; 3]>>();

            match *next_coords {
                [] => unreachable!(),
                [one] => {
                    len += 1;
                    curr = one;
                    path.insert(curr);
                }
                _ => {
                    // fork in the road; this corridor is over
                    break Some(next_coords.into_iter().map(|n| (n, curr)).collect());
                }
            }
        };

        edges.insert(Corridor { start, end: curr, len });
        if !nodes.insert(curr) {
            // if we're previously arrived at `curr`, don't follow its children:
            return None;
        }

        ret
    });
    let g = Graph { nodes, edges };
    eprintln!("{g:#}");

    // let (len, path) = g.find_longest_path(start, end);
    // eprintln!("path: ({len}) {path:?}");
    let len = g.find_longest_path(start, end);
    len
}

const INP: &str = "#.#####################
#.......#########...###
#######.#########.#.###
###.....#.>.>.###.#.###
###v#####.#v#.###.#.###
###.>...#.#.#.....#...#
###v###.#.#.#########.#
###...#.#.#.......#...#
#####.#.#.#######.#.###
#.....#.#.#.......#...#
#.#####.#.#.#########v#
#.#...#...#...###...>.#
#.#.#v#######v###.###v#
#...#.>.#...>.>.#.###.#
#####v#.#.###v#.#.###.#
#.....#...#...#.#.#...#
#.#########.###.#.#.###
#...###...#...#...#.###
###.###.#.###v#####v###
#...#...#.#.>.>.#.>.###
#.###.###.#.###.#.#v###
#.....###...###...#...#
#####################.#";

fn main() {
    let mut aoc = AdventOfCode::new(2023, 23);
    let inp = aoc.get_input();
    // let inp = INP;
    let mut grid: Grid<Cell> = inp.parse().unwrap();

    eprintln!("{grid}");

    let p1 = longest_path_naive(&mut grid, true);
    _ = aoc.submit_p1(p1);

    // let p2 = longest_path_naive(&mut grid, false);

    let p2 = longest_path_no_slopes(&mut grid);
    _ = aoc.submit_p2(p2);
}
