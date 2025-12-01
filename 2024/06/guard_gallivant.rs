use std::{collections::HashSet, ops::ControlFlow, str::FromStr};

use aoc::{
    grid::{
        Coord,
        Direction::{self, North},
        NextCoords, TwoDimensionalGrid,
    },
    AdventOfCode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumString)]
enum Cell {
    #[strum(serialize = ".")]
    Empty,
    #[strum(serialize = "#")]
    Obstacle,
    #[strum(serialize = "^")]
    Guard,
}

use Cell::*;
type Grid = TwoDimensionalGrid<Cell>;

const EX: &str = "....#.....
.........#
..........
..#.......
.......#..
..........
.#..^.....
........#.
#.........
......#...";

// fn walk(map: &Grid, mut pos: Coord, mut dir: Direction) -> HashSet<Coord> {
//     let mut visited = HashSet::from([pos]);

//     loop {
//         // move in direction; bail if we fall off the map:
//         let Some(next_coord) = dir.apply(pos).filter(|&c| map.get(c).is_some()) else { break };

//         match map[next_coord] {
//             // if we've hit an obstacle, rotate 90° and try again:
//             Cell::Obstacle => {
//                 dir = dir.rotate(1);
//             }
//             // otherwise, commit the new position and keep going:
//             Cell::Empty | Cell::Guard => {
//                 pos = next_coord;
//                 visited.insert(next_coord);
//             }
//         }
//     }

//     visited
// }

trait ControlFlowOrUnit<B, C = ()> {
    fn into(self) -> ControlFlow<B, C>;
}
impl<B, C> ControlFlowOrUnit<B, C> for ControlFlow<B, C> {
    fn into(self) -> ControlFlow<B, C> {
        self
    }
}
impl<B> ControlFlowOrUnit<B> for () {
    fn into(self) -> ControlFlow<B, ()> {
        ControlFlow::Continue(self)
    }
}

// todo: more efficient traversal
// todo: more efficient graph repr?
//
// see notes below
fn walk_with_map_getter<B, Cf: ControlFlowOrUnit<B>>(
    map: &Grid,
    mut pos: Coord,
    mut dir: Direction,
    mut get: impl FnMut(&Grid, Coord) -> Cell,
    mut callback: impl FnMut(Coord, Direction) -> Cf,
) -> Option<B> {
    loop {
        if let ControlFlow::Break(ret) = callback(pos, dir).into() {
            return Some(ret);
        }

        // move in direction; bail if we fall off the map:
        let Some(next_coord) = dir.get(pos, map) else {
            break;
        };

        match get(map, next_coord) {
            // if we've hit an obstacle, rotate 90° and try again:
            Cell::Obstacle => {
                dir = dir.rotate(1);
            }
            // otherwise, commit the new position and keep going:
            Cell::Empty | Cell::Guard => {
                pos = next_coord;
            }
        }
    }

    None
}

fn walk<Cf: ControlFlowOrUnit<()>, C: FnMut(Coord, Direction) -> Cf>(
    map: &Grid,
    p: Coord,
    d: Direction,
    callback: C,
) -> Option<()> {
    walk_with_map_getter(map, p, d, |g, c| g[c], callback)
}
fn walk_with_extra_obstacle<Cf: ControlFlowOrUnit<()>, C: FnMut(Coord, Direction) -> Cf>(
    map: &Grid,
    p: Coord,
    d: Direction,
    obstacle: Coord,
    callback: C,
) -> Option<()> {
    let getter = |g: &Grid, c| if c == obstacle { Cell::Obstacle } else { g[c] };
    walk_with_map_getter(map, p, d, getter, callback)
}

// fn would_loop_if_obstructed(map: &Grid, pos: Coord, dir: Direction, obstruction: Coord) -> bool {
//     if let None | Some(Cell::Guard) = map.get(obstruction) {
//         return false;
//     }

//     let mut map = map.clone();
//     map[obstruction] = Cell::Obstacle;

//     let mut seen = HashSet::new();
//     let looped = walk(&map, pos, dir, |p, d| {
//         if !seen.insert((p, d)) {
//             ControlFlow::Break(())
//         } else {
//             ControlFlow::Continue(())
//         }
//     });

//     looped.is_some()
// }

fn main() {
    let mut aoc = AdventOfCode::new(2024, 6);
    let map = Grid::from_str(&aoc.get_input()).unwrap();
    let guard = map
        .cell_iter()
        .filter_map(|(p, &c)| (c == Guard).then_some(p))
        .next()
        .unwrap();

    let mut visited = HashSet::new();
    walk(&map, guard, North, |p, _| _ = visited.insert(p));
    _ = aoc.submit_p1(visited.len());

    let mut would_induce_loop = HashSet::new();
    let mut seen = HashSet::new();
    let mut visited = HashSet::new();
    walk(&map, guard, North, |pos, dir| {
        seen.insert((pos, dir));
        visited.insert(pos);

        // consider: if there were an obstacle right in front of us and we were
        // forced to rotate right, would we loop?
        if let Some(in_front) = dir.get(pos, &map) {
            // can't place an obstacle anywhere the guard has already visited:
            if visited.contains(&in_front) {
                return;
            }

            let (seen, mut new_seen) = (&seen, HashSet::new());
            if let Some(()) =
                walk_with_extra_obstacle(&map, pos, dir.rotate(1), in_front, |p, d| {
                    if seen.contains(&(p, d)) || !new_seen.insert((p, d)) {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                })
            {
                would_induce_loop.insert(in_front);
            }

            // if looped.is_some() {
            //     // eprintln!(
            //     //     "obs at {in_front}; rot {pos}, {dir:?} -> {:?}: would loop\n",
            //     //     dir.rotate(1)
            //     // );
            //     would_induce_loop.insert(in_front);
            // }
        }
    });
    let p2 = would_induce_loop.len();
    dbg!(p2);
    _ = aoc.submit_p2(p2);

    // // brute force; assumes obstruction is placed at `t=0`
    // let mut count = 0;
    // for v in visited {
    //     if would_loop_if_obstructed(&map, guard, North, v) {
    //         count += 1;
    //     }
    // }
    // dbg!(count);
    // _ = aoc.submit_p2(p2);
}

// TODO: should be able to solve analytically?
//  - record obstacles as nodes, edges between them are directions (when either
//    the collision x or y is common); directed graph
//  - can connect any two nodes `N1(y1, x1)` and `N2(y2, x2)` by adding either:
//    + `N3(y1, x2)`
//    + `N3(y2, x1)`
//  - part 1 becomes walking the edges and adding up the distances?
//  - part 2 is finding all the nodes that you could introduce an node between
//    to induce a cycle
//
//  - there are a few wrinkles though:
//    + can't model each cell as a node since that doesn't properly account for
//      direction; instead have to model `(dir, coord)` as node?
//    + introducing a new "node" can do more than just add a path between two
//      other nodes; consider:
//      ```plain
//      .........F.................
//      ................G..........
//      .......A...................
//      .......+---------+B........
//      .......|.......H.|.........
//      .......^.........|.........
//      ........E........|.........
//      ...........D.....|.........
//      ```
//
//      adding obstacle `C` introduces multiple new edges, involving more than
//      two other nodes:
//      ```plain
//      .........F.................
//      .........+------+G..........
//      .......A.|......|..........
//      .......+-+-+C+--+.B........
//      .......|.|.|.|..H..........
//      .......^.|.|.|.............
//      ........E+-+.|.............
//      ...........D.|.............
//      ```
//        * A-South to C-West
//        * C-North to A-East (implied by above)
//        * C-West to D-North
//        * D-East to S-South (implied by above)
//        * H-North to C-East
//        * C-South to H-West (implied by above)
//
//    + even worse, introducing a new node can "break" existing edges; i.e.
//      `A-South to B-West` in the above...
//
// Not sure there's a way to elegantly handle all of the above when doing graph
// search for hypothetical nodes that would create cycles...

// A simpler optimization to do is at least using the graph representation
// described above to compute jumps, instead of manually iterating a 2D array to
// get from obstacle to obstacle; should reduce memory usage and compute for a
// sparse graph.
//
// Not going to bother with this though; brute force is fine for our input size!

// misc:
//  - took me an embarrassingly long time to realize:
//    + introducing a new obstacle can result in more than just 1 extra turn
//      (see above)
//    + you can't place an obstacle anywhere the guard has already visited
//      * i.e. the extra obstacle is placed at `t = 0`
//      * this matters for cells that get visited twice on the normal walk; i.e.
//        the `+`s in the ASCII view
//      * the example map does not have any loops that are made possible by
//        relaxing this constraint but the actual maps do
