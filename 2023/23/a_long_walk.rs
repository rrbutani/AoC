use std::collections::HashSet;

use aoc::{
    grid::{Direction, FloodfillFuncNextCoords},
    AdventOfCode, Coord, Grid,
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

fn longest_path(grid: &mut Grid<Cell>, respect_slopes: bool) -> usize {
    let mut paths_to_end = Vec::new();
    let (start, end) = (grid.top_left() + (0, 1), grid.bottom_right() - (0, 1));

    // Just doing BFS normally is likely to explode the state space way too
    // much but... the maze seems to consist largely of corridors so maybe
    // it'll be okay?
    //
    // (and, of course, part 2 makes us OOM)
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

    let p1 = longest_path(&mut grid, true);
    // _ = aoc.submit_p1(p1);

    // let p2 = longest_path(&mut grid, false);
    // dbg!(p2);
    // _ = aoc.submit_p2(p2);
}
