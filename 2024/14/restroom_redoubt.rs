use std::ops::Mul;

use aoc::*;
use grid::TwoDimensionalGrid;
use num_utils::lcm;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Robot {
    pos: Coord,
    velocity: Coord<isize>,
}

impl FromStr for Robot {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (p, v) = s.split_once(' ').unwrap();
        fn duo<T: FromStr>(s: &str, pfx: &str) -> Option<Coord<T>> {
            // note: `(col, row)` in the problem text
            let (x, y) = s.strip_prefix(pfx)?.split_once(',')?;
            Some(Coord { row: y.parse().ok()?, col: x.parse().ok()? })
        }

        Ok(Self { pos: duo(p, "p=").unwrap(), velocity: duo(v, "v=").unwrap() })
    }
}

impl Robot {
    fn step_n(&self, n: usize, Coord { row: gy, col: gx }: Coord) -> Coord {
        let Coord { row: y, col: x } = self.pos;
        let Coord { row: dy, col: dx } = self.velocity * (n as isize);

        // convert to positive so it's easier to reason about
        //
        // (note: if this were python this would be unnecessary; negative lhs on
        // mod has the deisred semantics; `-4 % 10` is `6` not `-4`)
        let [x, y] = [(x, dx, gx), (y, dy, gy)].map(|(start, diff, limit)| {
            if let Some(new) = start.checked_add_signed(diff) {
                new
            } else {
                assert!(diff.is_negative());
                // if diff takes us below 0, "flatten" the diff by calculating
                // the distance to the limit (i.e. mod, inverted) and add that:
                //
                // i.e. 4 - 10 (limit 7)
                //   = -6 -> 1
                //   = +4 (10 % 7 -> 3; 7 - 3 -> 4)
                let diff = limit - ((diff.abs() as usize) % limit);
                start + diff
            }
        });

        // now wrap in bounds
        Coord { row: y % gy, col: x % gx }
    }
}

fn count_quadrants(
    _midpoint @ Coord { row: mid_row_idx, col: mid_col_idx }: Coord,
    coords: impl IntoIterator<Item = Coord>,
) -> [usize; 4] {
    use std::cmp::Ordering::*;

    let mut quadrants = [0; 4];
    let [ref mut a, ref mut b, ref mut c, ref mut d] = &mut quadrants;
    for Coord { row, col } in coords.into_iter() {
        let count = match (row.cmp(&mid_row_idx), col.cmp(&mid_col_idx)) {
            (Less, Less) => &mut *a,
            (Less, Greater) => b,
            (Greater, Greater) => c,
            (Greater, Less) => d,
            _ => continue,
        };

        *count += 1;
    }

    quadrants
}

fn find_period(grid_dim: Coord, robots: &[Robot]) -> usize {
    let periods = robots.iter().map(|r| {
        // the period of each robot is the number of cycles it takes for it to
        // return to its starting position
        //
        // i.e. `(s + period * delta) % limit = s`
        // i.e. period * delta == limit * ?
        // i.e. the lowest number `period` that, when multiplied by `delta`,
        // yields a multiple of `limit`
        // i.e. the lcm(delta, limit) divided by delta

        // period * 6 == 20 -> (6 * 20) / gcd(6, 20) -> 120 / 2 -> 60; per = 10
        // period * 4 == 7  -> (4 * 7)  / gcd(4, 7)  -> 28; period = 7
        let dir_period = |delta, limit| lcm(delta, limit) / delta;

        lcm(
            dir_period(r.velocity.row.unsigned_abs(), grid_dim.row),
            dir_period(r.velocity.col.unsigned_abs(), grid_dim.col),
        )
    });

    // really this is unnecessary; the answer is always `height * width`...
    //
    // todo: why is this?
    // answer: because the map is non-square and has prime dimensions!
    periods.reduce(lcm).unwrap()
}

fn show(robots: &[Robot], grid_dim: Coord, n: usize) {
    let mut grid = TwoDimensionalGrid::new_with_dimensions(grid_dim, ' ');
    for r in robots {
        grid[r.step_n(n, grid_dim)] = 'x';
    }

    println!("{n}:\n{grid}\n");
}

fn search_for_cycle_with_tree(grid_dim: Coord, robots: &[Robot]) -> usize {
    // assumption is that the tree is centered and is symmetric along a y axis
    //
    // edit: whoops, problem says _most_ of the robots, not all, need to
    // quantify the symmetry and use a heuristic (or compare scores)...

    // note: another heuristic would have been to look for straight lines but...
    // I didn't know beforehand that the tree would have a box around it or that
    // it would be filled in (assumed it would be an outline only)

    // note: one more potential heuristic would have been to consider the
    // positions of the robots, i.e. entropy; in the cycle where they are
    // drawing a tree we expect most of the robots to be clustered together

    // note: yet another heuristic is to find the cycles where all robots have a
    // unique position... if you consider how the inputs are likely crafted it
    // makes sense why this is likely to be a good heuristic but I don't think
    // this is something we can reasonably know a priori

    const MIN_TREE_WIDTH_ASSUMPTION: usize = 10;
    let offs: usize = MIN_TREE_WIDTH_ASSUMPTION / 2;

    let mut grid = TwoDimensionalGrid::new_with_dimensions(grid_dim, 0u16);
    'cycle: for i in 0..find_period(grid_dim, robots) {
        grid.clear();
        for c in robots.iter().map(|r| r.step_n(i, grid_dim)) {
            grid[c] += 1;

            if grid[c] == 2 {
                // cheating... we know that, for the inputs produced, only
                // cycles where all robots have a unique position will have the
                // tree — this eliminates all the other cycles and reduces the
                // runtime for part 2 dramatically
                continue 'cycle;
            }
        }

        // for each cycle, sweep the y axes and assess symmetry:
        for mid_x in offs..(grid_dim.col - offs) {
            let mut symm_count = 0;
            for x in 0..mid_x {
                let rhs_x = (mid_x - x) + mid_x; // mirrored across
                if rhs_x >= grid.width() {
                    continue;
                }

                for y in 0..grid.height() {
                    let [left, right] = [(y, x), (y, rhs_x)].map(Coord::from);
                    match (grid[left], grid[right]) {
                        (0, _) | (_, 0) => continue,
                        (l, r) => symm_count += l + r,
                    }
                }
            }

            let total = robots.len();
            let ratio = symm_count as f64 / total as f64;
            if ratio > 0.50 {
                eprintln!("{symm_count} of {total} => {ratio} at x = {mid_x}");
                show(robots, grid_dim, i);
                return i;
            }
        }
    }

    panic!("nothing found!")
}

fn main() {
    let mut aoc = AdventOfCode::new(2024, 14);
    let (inp, dim) = (aoc.get_input(), Coord { col: 101, row: 103 });
    let robots = inp.lines().map_parse::<Robot>().collect_vec();

    assert!(dim.row % 2 == 1 && dim.col % 2 == 1);
    let mid = dim / 2;
    let quads = count_quadrants(mid, robots.iter().map(|r| r.step_n(100, dim)));
    let p1 = quads.into_iter().fold(1, Mul::mul);
    _ = aoc.submit_p1(p1);

    let p2 = search_for_cycle_with_tree(dim, &robots);
    _ = aoc.submit_p2(p2);
}

// yeah okay, eric got me pretty good; thought part 2 was gonna be "hah, large N
// now; suffer ye who wrote an imperative solution"...
//
// the efficient `step_n` did end up being useful but not as useful as I thought
// — since we're sweeping `n` anyways it doesn't confer any speedup (just some
// memory savings)
//   - actually having a grid is *more* efficient when looking for symmetry
//     though
//   - rewriting my part 2 to actually have a grid instead of collecting
//     `HashMap`s of coordinates brought the runtime from ~25s to ~3s
//     + still not good though...
