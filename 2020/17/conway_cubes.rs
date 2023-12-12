#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::collections::HashMap;
use std::hash::Hash;
use std::iter;
use std::mem;
use std::ops::Range;

trait Coord/* <'a> */: Sized {
    const DIMS: usize;

    // If we had const generics:
    // fn as_arr(&self) -> [usize; <Self as Coor>::DIMS];

    type Arr: AsRef<[isize]> + 'static;

    fn as_arr(&self) -> Self::Arr;
    fn new<I: Iterator<Item = isize>>(vals: I) -> Self;

    // It sure would be nice if we had existentials!
    #[inline]
    fn neighbours(&self) -> Box<dyn Iterator<Item = Self>> {
        let r = -1..=1isize;

        let curr = self.as_arr();

        let iter = (0..Self::DIMS)
            .map(|_| r.clone())
            .multi_cartesian_product()
            .filter(|v| !v.iter().all(|d| *d == 0))
            .map(move |v| {
                Self::new(
                    v.iter()
                        .enumerate()
                        .map(|(idx, val)| curr.as_ref()[idx] + val),
                )
            });

        Box::new(iter)
    }

    // type Dimensions: AsRef<[Range<isize>]> + AsMut<[Range<isize>]> + Clone + 'static;
    // // where
    // // &'a Self::Dimensions: TryFrom<&'a [Range<isize>]>,
    // // <&'a Self::Dimensions as TryFrom<&'a [Range<isize>]>>::Error: Debug;

    // fn new_dims</* 'a,  */ I: IntoIterator<Item = Range<isize>>>(dims: I) -> Self::Dimensions
    // where
    //     &'a Self::Dimensions: TryFrom<&'a [Range<isize>]>,
    //     <&'a Self::Dimensions as TryFrom<&'a [Range<isize>]>>::Error: Debug,


    type Dimensions: AsRef<[Range<isize>]> + AsMut<[Range<isize>]> + Clone + 'static;
    // where
    // &'a Self::Dimensions: TryFrom<&'a [Range<isize>]>,
    // <&'a Self::Dimensions as TryFrom<&'a [Range<isize>]>>::Error: Debug;

    fn new_dims</* 'a,  */ I: IntoIterator<Item = Range<isize>>>(dims: I) -> Self::Dimensions
    where
        &'static Self::Dimensions: TryFrom<&'static [Range<isize>]>,
        <&'static Self::Dimensions as TryFrom<&'static [Range<isize>]>>::Error: Debug,
    {
        let v = dims.into_iter().take(Self::DIMS).collect::<Vec<_>>();
        // let v = Box::leak(Box::new(v)); // Crime!
        let v: &'static [_] = unsafe { core::mem::transmute(&*v) }; // Crime!
        let d = AsRef::<[_]>::as_ref(v).to::<&Self::Dimensions>();
        Clone::clone(d)
    }

    fn expand_dims(dims: &mut Self::Dimensions) {
        fn expand_range(r: &mut Range<isize>) {
            *r = (r.start - 1)..(r.end + 1);
        }

        for r in dims.as_mut().iter_mut() {
            expand_range(r)
        }
    }

    #[inline]
    fn coord_iter(dims: &Self::Dimensions) -> Box<dyn Iterator<Item = Self>> {
        let iter = dims
            .as_ref()
            .iter()
            .cloned()
            .multi_cartesian_product()
            .map(|v| Self::new(v.iter().copied()));

        Box::new(iter)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Coord3 {
    x: isize,
    y: isize,
    z: isize,
}

impl From<(isize, isize, isize)> for Coord3 {
    fn from((x, y, z): (isize, isize, isize)) -> Self {
        Coord3 { x, y, z }
    }
}

impl Coord for Coord3 {
    const DIMS: usize = 3;
    type Arr = [isize; 3];

    fn as_arr(&self) -> [isize; 3] {
        [self.x, self.y, self.z]
    }

    fn new<I: Iterator<Item = isize>>(mut vals: I) -> Self {
        Self {
            x: vals.next().unwrap(),
            y: vals.next().unwrap(),
            z: vals.next().unwrap(),
        }
    }

    type Dimensions = [Range<isize>; 3];
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Coord4 {
    x: isize,
    y: isize,
    z: isize,
    w: isize,
}

impl From<(isize, isize, isize, isize)> for Coord4 {
    fn from((x, y, z, w): (isize, isize, isize, isize)) -> Self {
        Coord4 { x, y, z, w }
    }
}

impl Coord for Coord4 {
    const DIMS: usize = 4;
    type Arr = [isize; 4];

    fn as_arr(&self) -> [isize; 4] {
        [self.x, self.y, self.z, self.w]
    }

    fn new<I: Iterator<Item = isize>>(mut vals: I) -> Self {
        Self {
            x: vals.next().unwrap(),
            y: vals.next().unwrap(),
            z: vals.next().unwrap(),
            w: vals.next().unwrap(),
        }
    }

    type Dimensions = [Range<isize>; 4];
}

// impl Coord {
//     fn neighbours(&self) -> impl Iterator<Item = Coord> + '_ {
//         let r = -1..=1;

//         r.clone()
//             .cartesian_product(r.clone())
//             .cartesian_product(r)
//             .map(|((x, y), z)| (x, y, z))
//             .filter(|(x, y, z)| !(*x == 0 && *y == 0 && *z == 0))
//             .map(move |(x, y, z)| Coord {
//                 x: self.x + x,
//                 y: self.y + y,
//                 z: self.z + z,
//             })

//         // r.clone()
//         //     .zip(r.clone())
//         //     .zip(r)
//         //     .map(|((x, y), z)| (x, y, z))
//         //     .filter(|(x, y, z)| !(*x == 0 && *y == 0 && *z == 0))
//         //     .filter_map(move |(x, y, z)| {
//         //         let try_add = |base: usize, offs| {
//         //             base.to::<isize>()
//         //                 .checked_add(offs)
//         //                 .and_then(|n| n.try_to::<usize>().ok())
//         //         };
//         //         let x = try_add(self.x, x);
//         //         let y = try_add(self.y, y);
//         //         let z = try_add(self.z, z);

//         //         x.zip(y).zip(z).map(|((x, y), z)| Coord { x, y, z })
//         //     })
//     }
// }

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Cube {
    Inactive,
    Active,
}

impl Display for Cube {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            Cube::Active => '#',
            Cube::Inactive => '.',
        };

        write!(fmt, "{}", c)
    }
}

impl Default for Cube {
    fn default() -> Self {
        Cube::Inactive
    }
}

impl Cube {
    fn is_active(&self) -> bool {
        matches!(self, Cube::Active)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Grid</* 'a, */ C: Coord + Clone + Debug + Eq + Hash>
where
    <C as Coord>::Dimensions: Debug + Eq + Clone,
{
    dims: <C as Coord>::Dimensions,
    space: HashMap<C, Cube>,
    staging: HashMap<C, Cube>,
    // inner: Vec<Vec<Vec<Cube>>>,
    // staging: Vec<Vec<Vec<Cube>>>,
}

impl Display for Grid</* 'a,  */ Coord3> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [x, y, z] = &self.dims;

        for z in z.clone() {
            writeln!(fmt, "z={}", z)?;

            for y in y.clone() {
                for x in x.clone() {
                    write!(fmt, "{}", self.space.get(&Coord3 { x, y, z }).unwrap())?;
                }
                writeln!(fmt)?;
            }
            writeln!(fmt)?;
        }

        Ok(())
    }
}

impl</* 'a,  */ C: Coord + Clone + Debug + Eq + Hash> Grid</* 'a,  */ C>
where
    <C as Coord>::Dimensions: Debug + Eq + Clone,
{
    // fn coord_iter(&self) -> impl Iterator<Item = Coord> {
    //     let (x, y, z) = self.dims.clone();

    //     x.cartesian_product(y)
    //         .cartesian_product(z)
    //         .map(|((x, y), z)| Coord { x, y, z })
    // }

    // fn step(&mut self) {
    //     fn expand_range(r: &mut Range<isize>) {
    //         *r = (r.start - 1)..(r.end + 1);
    //     }
    //     expand_range(&mut self.dims.0);
    //     expand_range(&mut self.dims.1);
    //     expand_range(&mut self.dims.2);

    //     for coord in self.coord_iter() {
    //         let active_neighbours = coord
    //             .neighbours()
    //             .map(|c| self.space.get(&c).map(Cube::is_active).unwrap_or(false))
    //             .filter(|a| *a)
    //             .count();

    //         let cube = match (
    //             self.space.get(&coord).copied().unwrap_or(Cube::Inactive),
    //             active_neighbours,
    //         ) {
    //             (Cube::Active, 2) | (Cube::Active, 3) | (Cube::Inactive, 3) => Cube::Active,
    //             _ => Cube::Inactive,
    //         };

    //         self.staging.insert(coord, cube);
    //     }

    //     mem::swap(&mut self.staging, &mut self.space);
    // }

    fn step(&mut self) {
        C::expand_dims(&mut self.dims);

        for coord in C::coord_iter(&self.dims) {
            let active_neighbours = coord
                .neighbours()
                .map(|c| self.space.get(&c).map(Cube::is_active).unwrap_or(false))
                .filter(|a| *a)
                .count();

            let cube = match (
                self.space.get(&coord).copied().unwrap_or(Cube::Inactive),
                active_neighbours,
            ) {
                (Cube::Active, 2) | (Cube::Active, 3) | (Cube::Inactive, 3) => Cube::Active,
                _ => Cube::Inactive,
            };

            self.staging.insert(coord, cube);
        }

        mem::swap(&mut self.staging, &mut self.space);
    }

    fn active_cubes(&self) -> usize {
        self.space
            .iter()
            .map(|(_, c)| *c)
            .filter(Cube::is_active)
            .count()
    }
}

// Takes a slice assumed to be at z = 0.
impl</* 'a,  */ C: Coord + Clone + Debug + Eq + Hash> FromStr for Grid</* 'a,  */ C>
where
    <C as Coord>::Dimensions: Debug + Eq + Clone,
    &'static C::Dimensions: TryFrom<&'static [Range<isize>]>,
    <&'static C::Dimensions as TryFrom<&'static [Range<isize>]>>::Error: Debug,
{
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        let num_rows = s.lines().count();
        let num_cols = s.lines().next().ok_or(())?.chars().count();

        let x_offs = -(((num_rows - 1) / 2) as isize);
        let y_offs = -(((num_cols - 1) / 2) as isize);

        let mut space = HashMap::with_capacity(num_rows * num_cols);

        for (y, row) in s.lines().enumerate() {
            for (x, c) in row.chars().enumerate() {
                let cube = match c {
                    '#' => Cube::Active,
                    '.' => Cube::Inactive,
                    _ => unimplemented!(),
                };

                // space.insert(
                //     Coord {
                //         x: x as isize + x_offs,
                //         y: y as isize + y_offs,
                //         z: 0,
                //     },
                //     cube,
                // );

                space.insert(
                    C::new(
                        iter::once(x as isize + x_offs)
                            .chain(iter::once(y as isize + y_offs))
                            .chain(iter::repeat(0)),
                    ),
                    cube,
                );
            }
        }

        assert_eq!(space.len(), num_rows * num_cols);

        Ok(Grid {
            dims: C::new_dims(
                iter::once(x_offs..(x_offs + num_rows as isize))
                    .chain(iter::once(y_offs..(y_offs + num_cols as isize)))
                    .chain(iter::repeat(0..1)),
            ),
            staging: space.clone(),
            space,
        })
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2020, 17);
    let input: String = aoc.get_input();

    //     let input = ".#.
    // ..#
    // ###";

    let mut grid: Grid<Coord3> = input.parse().unwrap();
    // println!("Before any cycles:\n{}", grid);

    (0..6).for_each(|_| {
        grid.step();
        // println!("After Cycle {}:\n{}", i + 1, grid);
    });
    let p1 = grid.active_cubes();
    println!("{}", p1);
    let _ = aoc.submit_p1(p1);

    let mut grid: Grid<Coord4> = input.parse().unwrap();

    (0..6).for_each(|_| grid.step());
    let p2 = grid.active_cubes();
    println!("{}", p2);
    let _ = aoc.submit_p2(p2);
}
