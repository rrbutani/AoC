use std::mem;

use aoc::{grid::Direction, AdventOfCode, Coord, Grid};
use smallvec::SmallVec;
use strum::EnumString;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString, strum::Display)]
pub enum Cell {
    #[strum(serialize = "S")]
    Start,
    #[strum(serialize = ".")]
    Garden,
    #[strum(serialize = "#")]
    Rock,
}

fn p1(grid: &Grid<Cell>, start: Coord, steps: usize) -> usize {
    let mut curr = Vec::from([start]);
    let mut next = Vec::new();
    let grid = grid.clone();

    for _ in 0..steps {
        // if i % 1_000 == 0 {
        //     eprint!(".");
        // }
        // use `floodfill` to find the reachable next cells (and to dedupe):
        grid.floodfill(curr.drain(..), |ctx, state| {
            if let Some(()) = state {
                if let Cell::Garden | Cell::Start = *ctx.cell() {
                    next.push(ctx.coord());
                }

                // end the propagation
                None
            } else {
                // origin node; just travel:
                Some(SmallVec::from(Direction::ALL.map(|d| (d, ()))))
            }
        });

        mem::swap(&mut curr, &mut next);
        eprintln!("positions: {}", curr.len());
    }

    curr.len()
}

// const DEBUG: bool = true;

const INP: &str = "...........
.....###.#.
.###.##..#.
..#.#...#..
....#.#....
.##..S####.
.##..#...#.
.......##..
.##.#.####.
.##..##.##.
...........";

fn main() {
    let mut aoc = AdventOfCode::new(2023, 21);
    let inp = aoc.get_input();
    let inp = INP;

    let grid: Grid<Cell> = inp.parse().unwrap();
    let start = grid.find(|c| matches!(c, Cell::Start)).next().unwrap();

    // let p1 = p1(&grid, start, 64);
    // _ = aoc.submit_p1(p1);

    let p2 = p1(&grid, start, 26501365);
    dbg!(p2);
    // eprintln!("{grid}");
}

// p2:
//
// odd, even, odd, even ...
//  - each cell is reachable via EITHER an odd number of steps or an even number
//    of steps — not by both
//    + there can be multiple paths to a cell but they all have to have the same
//      polarity
//    + easiest proof I can think of is to consider the coordinate math: to get
//      from (for example) `0, 0` (even) to `0, 1` (odd) you have to do an odd
//      number of transforms (where your options are `-1, 0`, `0, 1`, `1, 0`,
//      `0, -1`)
//    + it follows from ^ that whether a cell is "odd" or "even" can be
//      determined from its coordinate (relative to the start)
//    + annoyingly the actual grid is 131 x 131 (odd numbers) so this polarity
//      flip flops for each duplicate
//      * though this is maybe unavoidable because there's a centered starting
//        point? not sure
//  - with a long enough step count everything is reachable (we can do a
//    floodfill to root out enclosed pockets but on the given inputs this
//    doesn't seem to come up); it's just polarity that determines which plots
//    are reachable final destinations and which aren't
//
// the question is: how to find the perimeter?
//
// for an empty grid it's just the "tilted square" you get with a fixed
// manhattan distance:
//  - if you can figure out where the perimeter is (relative to the start) you
//    can figure out how many repeats of the map are completely covered
//    + computing how many final destinations lie within these completely
//      covered tiles is (relatively) straight-forward; just odd/even stuff
//  - for tiles on which the perimeter falls you can manually do the stepping to
//    figure out how many final destinations there are
//    + for such tiles you want to use the point with the shortest distance from
//      the start as your starting point...
//
// with our actual input (very much not an empty grid) a few things aren't
// clear:
//  - how do we find the perimeter?
//  - how do we find the closest point (in distance) on a tile to the starting
//    point?
//
// the map does have an interesting property: it's padded on all four sides
//  - also: the vertical and horizontal that S lies on is completely clear!
//    + this is not true of the sample input though... nor are there
//      optimal-distance paths out of the sample input's tile in the 4
//      directions...
//  - I think this means that for traveling _between_ tiles we can assume that
//    the optimal path is available to us: i.e. just `len = rows + cols`
//  - so I think both finding the perimeter as it relates to tiles and finding
//    the closest point on such tiles to the starting point becomes easy: it's
//    the same as it was in our hypothetical "empty grid" case:
//    + perimeter tiles are ones where `X * G + Y * G` lies between `floor(LIM /
//      G) * G` and `ceil(LIM / G) * G`
//      * where `G` is the grid len, X and Y are the coordinates of the tile and
//        `LIM` is the step limit
//      * this should trace out a tilted square of its own
//    + re: finding the closest point; I think there are 8 cases to consider:
//      * lateral left/right: the tiles that you reach if you go in a
//        straight-line left/right; the closest point for these will be center
//        right and center left respectively
//      * vertical top/bottom: same as above but bottom middle and top middle
//      * left edge of the tilted square: this is a grid after all so I think
//        the bottom left would have to be the closest point
//      * right top edge: bottom left
//      * right bottom edge: top left
//      * left bottom edge: top right
//   + for the tiles on the edges I think the distance number will always be the
//     same (or will be one of two numbers)? we're drawing a tilted square on a
//     grid and asking about the cells where the line intersects. if the length
//     of the square's edges and the square's position exactly lines up with the
//     grid the line will go through the midpoint of tiles and we'll only have 1
//     distance number. otherwise we'll have two (the reason we can only have
//     two is because the slope of the line is fixed: 45 degrees)
//
// note: it may be incorrect to assume that all points within tiles that are
// enclosed within the perimeter (specifically, within tiles that are, say, 1
// tile back from the perimeter — not on the perimeter but just behind it) are
// reachable: if there are paths within the tile that require longer paths to
// reach (i.e. due rocks), there may not be enough steps left to walk such paths
// by the time we do the steps to get to the tile
//   - this is easy to test/account for though: we'd need to just keep drawing
//     tilted squares 1 tile further in and doing the manual calculation until
//     we hit tiles that *are* fully reachable
//
// note: 26501365/131 = ~2023 which is definitely not a coincidence (thanks
// eric)
//
// also not a coincidence: 26501365 % 131 = 65 which is half of the grid length
// which I think solves our "distance number" issue? (and also, I suspect, the
// issue about considering whether tiles 1-back from the perimeter are fully
// reachable)
//
// we'll probably need to run step simulations for 2023 * 4 tiles? though there
// is probably symmetry that we can exploit here...
