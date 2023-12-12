use std::iter::Map;

use aoc::*;
use owo_colors::OwoColorize;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[rustfmt::skip]
enum Level { L0, L1, L2, L3, L4, L5, L6, L7, L8, L9 }

impl TryFrom<u8> for Level {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'0'..=b'9' => Ok(unsafe { core::mem::transmute(value - b'0') }),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[rustfmt::skip]
enum Dir { Top, Right, Bottom, Left }

// struct Grid<'r, T> {
//     raw: &'r [T],
//     cols: usize,
// }

// impl<'r, T> Index<usize> for Grid<'r, T> {
//     type Output = [T]

//     fn index(&self, index: usize) -> &Self::Output {
//         todo!()
//     }
// }

#[allow(unused)]
const TESTCASE: &str = "30373
25512
65332
33549
35390
";

fn main() {
    let mut aoc = AdventOfCode::new(2022, 8);
    let inp = aoc.get_input();
    // let inp = TESTCASE;

    let rows = inp.lines().count();
    let cols = inp.lines().next().unwrap().as_bytes().len();
    assert_eq!(inp.len(), rows * (cols + 1));

    dbg!(rows);
    dbg!(cols);

    let grid: Vec<Vec<Level>> = inp
        .lines()
        .map(|l| {
            l.as_bytes()
                .iter()
                .map(|&c| c.try_into().unwrap())
                .collect_vec()
        })
        .collect_vec();
    let mut visibility = grid
        .iter()
        .map(|r| r.iter().map(|_| None).collect_vec())
        .collect_vec();

    // we're supposed to do caching / transitive smarts but the grid is small
    // enough (and I'm tired enough) that I'm not going to bother.
    let p1 = (0..rows)
        .cartesian_product(0..cols)
        // .filter(|&(r, c)| {
        //     any(
        //         [
        //             /* left  */ (0..c).map(&Box::new(|c| (r, c)) as _),
        //             /* right */ ((c + 1)..cols).map(&Box::new(|c| (r, c)) as _),
        //             /* top   */ (0..r).map(&Box::new(|r| (r, c)) as _),
        //             /* bot   */ ((r + 1)..rows).map(&Box::new(|c| (r, c)) as _),
        //         ],
        //         |mut range: Map<_, &dyn (Fn(_) -> _)>| {
        //             range.all(|(x, y): (usize, usize)| grid[x][y] < grid[r][c])
        //         },
        //     )
        // })
        .filter_map(|(r, c)| {
            use Dir::*;
            let dirs: [(_, Map<_, Box<dyn (Fn(_) -> _)>>); 4] = [
                (Left, (0..c).map(Box::new(|c| (r, c)) as _)),
                (Right, ((c + 1)..cols).map(Box::new(|c| (r, c)) as _)),
                (Top, (0..r).map(Box::new(|r| (r, c)) as _)),
                (Bottom, ((r + 1)..rows).map(Box::new(|r| (r, c)) as _)),
            ];

            for (dir, mut iter) in dirs {
                if iter.all(|(y, x): (usize, usize)| grid[y][x] < grid[r][c]) {
                    return Some((dir, (r, c)));
                }
            }

            None
        })
        .inspect(|&(dir, (r, c))| {
            visibility[r][c] = Some(dir);
            // eprintln!("({r}, {c}) [{:?}] is visible from {:?}", grid[r][c], dir);
        })
        .count();

    for (row, vis_row) in grid.iter().zip(visibility.iter()) {
        for (&cell, &vis) in row.iter().zip(vis_row.iter()) {
            use Dir::*;
            let val = cell as u8;
            match vis {
                Some(Top) => eprint!("{}", val.red()),
                Some(Right) => eprint!("{}", val.yellow()),
                Some(Left) => eprint!("{}", val.green()),
                Some(Bottom) => eprint!("{}", val.purple()),
                None => eprint!("{}", val.dimmed()),
            }
        }
        eprintln!()
    }

    aoc.submit_p1(dbg!(p1)).unwrap();

    // like part 1, not going to bother with caching; I want to go to sleep
    // soon.
    let p2 = (0..rows)
        .cartesian_product(0..cols)
        .map(|(r, c)| {
            let dirs: [Box<dyn Iterator<Item = (usize, usize)>>; 4] = [
                /* top   */ Box::new((0..r).rev().map(|r| (r, c))) as _,
                /* left  */ Box::new((0..c).rev().map(|c| (r, c))) as _,
                /* bot   */ Box::new(((r + 1)..rows).map(|r| (r, c))) as _,
                /* right */ Box::new(((c + 1)..cols).map(|c| (r, c))) as _,
            ];

            dirs.into_iter()
                .map(|d| {
                    let mut blocked = false;
                    let height = grid[r][c];
                    use std::cmp::Ordering::*;

                    d.take_while(|&(y, x)| {
                        if blocked {
                            return false;
                        }

                        match grid[y][x].cmp(&height) {
                            Less => true,
                            Equal | Greater => {
                                blocked = true;
                                true
                            } // can count this tree but then we have to stop
                        }
                        // we can't just +1 the count and ditch `blocked`
                        // because then we'll overestimate situations where we
                        // can see all the way to an edge
                    })
                    .count()
                })
                .product::<usize>()
        })
        .max()
        .unwrap();

    aoc.submit_p2(dbg!(p2)).unwrap();

    // let flat = inp
    //     .lines()
    //     .flat_map(|l| l.as_bytes().iter())
    //     .map(|&b| b.try_into())
    //     .collect::<Result<Vec<Level>, _>>()
    //     .unwrap();
    // assert_eq!(flat.len(), rows * cols);

    // let grid: &[[u8]] =
}
