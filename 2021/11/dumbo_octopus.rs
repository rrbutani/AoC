#!/usr/bin/env rustr

#![feature(generic_const_exprs)]

use std::{
    cell::Cell,
    marker::PhantomData,
    ops::{Deref, Index, IndexMut},
};

use aoc::*;
use num_traits::Zero;
use owo_colors::OwoColorize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TwoDGrid<
    T,
    XDim = usize,
    YDim = usize,
    DisplaySep = NoSep,
    ElemPrinter = DisplayElemPrinter,
> {
    // [row][col]; [y][x]
    inner: Vec<Vec<T>>,
    _idx: PhantomData<(XDim, YDim)>,
    _disp: PhantomData<(DisplaySep, ElemPrinter)>,
}

pub trait ElementPrinter<T> {
    fn print(elem: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct DisplayElemPrinter;
impl<T: Display> ElementPrinter<T> for DisplayElemPrinter {
    fn print(elem: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", elem)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ZeroBold;
impl<T: Display + Zero> ElementPrinter<T> for ZeroBold {
    fn print(elem: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if elem.is_zero() {
            write!(f, "{}", elem.bold())
        } else {
            write!(f, "{}", elem)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct DisplayForCellCopy<Inner>(PhantomData<Inner>);
impl<T: Copy, Inner: ElementPrinter<T>> ElementPrinter<Cell<T>> for DisplayForCellCopy<Inner> {
    fn print(elem: &Cell<T>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Inner::print(&elem.get(), f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct DisplayFirst<Inner>(PhantomData<Inner>);
impl<Inner: ElementPrinter<T>, T, U> ElementPrinter<(T, U)> for DisplayFirst<Inner> {
    fn print((a, _): &(T, U), f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Inner::print(a, f)
    }
}

pub trait DispSep {
    fn sep(f: &mut fmt::Formatter<'_>) -> fmt::Result;
}
impl<D: Default + Display> DispSep for D {
    fn sep(f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", D::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct NoSep;
impl Display for NoSep {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ChainSep<Car: DispSep, Cdr: DispSep>(PhantomData<(Car, Cdr)>);
impl<A: DispSep, B: DispSep> Display for ChainSep<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        A::sep(f)?;
        B::sep(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RepeatSep<S: DispSep, const N: usize>(PhantomData<S>);
impl<S: DispSep, const N: usize> Display for RepeatSep<S, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (0..N).try_for_each(|_| S::sep(f))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SpaceSep;
impl Display for SpaceSep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " ")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CommaSep;
impl Display for CommaSep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ",")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IndexPair<X, Y>(X, Y);
pub trait AsIndexPair<X, Y> {
    fn as_idx_pair(x: X, y: Y) -> (usize, usize);
    fn from_tuple(idx: (usize, usize)) -> (X, Y);
}
impl AsIndexPair<usize, usize> for IndexPair<usize, usize> {
    fn as_idx_pair(x: usize, y: usize) -> (usize, usize) {
        (x, y)
    }

    fn from_tuple((x, y): (usize, usize)) -> (usize, usize) {
        (x, y)
    }
}
trait AsIndexPairExt<X, Y> {
    fn idx(self) -> (usize, usize);
}
impl<X, Y> AsIndexPairExt<X, Y> for (X, Y)
where
    IndexPair<X, Y>: AsIndexPair<X, Y>,
{
    fn idx(self) -> (usize, usize) {
        let (x, y) = self;
        IndexPair::<X, Y>::as_idx_pair(x, y)
    }
}

impl<T, X, Y, D, P> Index<(X, Y)> for TwoDGrid<T, X, Y, D, P>
where
    IndexPair<X, Y>: AsIndexPair<X, Y>,
{
    type Output = T;

    fn index(&self, index: (X, Y)) -> &Self::Output {
        let (x, y) = index.idx();
        &self.inner[y][x]
    }
}

impl<T, X, Y, D, P> IndexMut<(X, Y)> for TwoDGrid<T, X, Y, D, P>
where
    IndexPair<X, Y>: AsIndexPair<X, Y>,
{
    fn index_mut(&mut self, index: (X, Y)) -> &mut Self::Output {
        let (x, y) = index.idx();
        &mut self.inner[y][x]
    }
}

impl<T, X, Y, D, P> TwoDGrid<T, X, Y, D, P> {
    pub fn from_iter(it: impl Iterator<Item = impl Iterator<Item = T>>) -> Self {
        let grid = it.map(|i| i.collect_vec()).collect_vec();

        if cfg!(debug_assertions) {
            assert!(grid.iter().map(|row| row.len()).all_equal())
        }

        Self {
            inner: grid,
            _idx: PhantomData,
            _disp: PhantomData,
        }
    }

    pub fn from_char(grid: &str, func: impl Fn(char) -> T) -> Self {
        // TODO:
        // doing `.map(func)` produces a misleading error message here; the issue is that
        // this creates a closure where `func` (a captured variable) is consumed per invocation
        // of the closure which means the closure isn't FnMut (which `map` requires)
        //
        // the issue is not, as rustc currently indicates, that the closure moves things out of
        // the closure's parent environment but that it moves things out of its own environment
        // i.e. the capture is not the issue; it's the usage of `func` within the closure

        Self::from_iter(grid.lines().map(|l: &str| l.trim().chars().map(&func)))
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner.iter().flat_map(|l| l.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.inner.iter_mut().flat_map(|l| l.iter_mut())
    }
}

impl<T, X, Y, D, P> TwoDGrid<T, X, Y, D, P>
where
    IndexPair<X, Y>: AsIndexPair<X, Y>,
{
    pub fn coord_iter(&self) -> impl Iterator<Item = (&T, (X, Y))> {
        self.inner.iter().enumerate().flat_map(|(y, row)| {
            row.iter()
                .enumerate()
                .map(move |(x, cell)| (cell, IndexPair::<X, Y>::from_tuple((x, y))))
        })
    }

    pub fn coord_iter_mut(&mut self) -> impl Iterator<Item = (&mut T, (X, Y))> {
        self.inner.iter_mut().enumerate().flat_map(|(y, row)| {
            row.iter_mut()
                .enumerate()
                .map(move |(x, cell)| (cell, IndexPair::<X, Y>::from_tuple((x, y))))
        })
    }
}

// [0] => NW
// [1] => N
// [2] => NE
// [3] => W
// [4] => E
// [5] => SW
// [6] => S
// [7] => SE
//
// 012
// 3 4
// 567

pub const fn rad_to_adj_arr_len(rad: usize) -> usize {
    let diam = rad * 2 + 1;
    diam * diam - 1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Adj<T, const RADIUS: usize>(pub [Option<T>; rad_to_adj_arr_len(RADIUS)])
where
    [(); rad_to_adj_arr_len(RADIUS)]: ;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Dir {
    NorthWest = 0,
    North = 1,
    NorthEast = 2,
    West = 3,
    East = 4,
    SouthWest = 5,
    South = 6,
    SouthEast = 7,
}

impl<T, const R: usize> Deref for Adj<T, R>
where
    [(); rad_to_adj_arr_len(R)]: ,
{
    type Target = [Option<T>; rad_to_adj_arr_len(R)];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const R: usize> Adj<T, R>
where
    [(); rad_to_adj_arr_len(R)]: ,
{
    fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.0.iter().filter_map(|i| i.as_ref())
    }
}

impl<T> Index<Dir> for Adj<T, 1> {
    type Output = Option<T>;

    fn index(&self, index: Dir) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl<T, X, Y, D, P> TwoDGrid<T, X, Y, D, P> {
    pub fn dim(&self) -> (usize, usize) {
        let (yl, xl) = (
            self.inner.len(),
            self.inner.last().map(|l| l.len()).unwrap_or_default(),
        );
        (xl, yl)
    }

    pub fn size(&self) -> usize {
        let (xl, yl) = self.dim();
        xl * yl
    }

    pub fn adj_iter<const RADIUS: usize>(&self) -> impl Iterator<Item = (&T, Adj<&T, RADIUS>)>
    where
        [(); rad_to_adj_arr_len(RADIUS)]: ,
    {
        let (xl, yl) = self.dim();
        let radius: isize = RADIUS.to();
        let axis_coord_iter = move |s: usize, lim| {
            ((-radius)..=radius).map(move |a| {
                (if a < 0 {
                    s.checked_sub(a.abs().to())
                } else {
                    s.checked_add(a.to())
                })
                .filter(|&s| s < lim)
            })
        };

        self.inner.iter().enumerate().flat_map(move |(y, row)| {
            row.iter().enumerate().map(move |(x, cell)| {
                // let adj = [x.checked_sub(1), Some(x), Some(x + 1).filter(|&x| x < xl)]
                //     .into_iter()
                //     .cartesian_product([y.checked_sub(1), Some(y), Some(y + 1).filter(|&y| y < yl)])
                //     .filter(|&(ex, why)| (ex, why) != (Some(x), Some(y)))
                //     .map(|(x, y)| x.zip(y))
                //     .map(|c| c.map(|(x, y)| &self.inner[y][x]))
                //     .arr();

                let adj = axis_coord_iter(x, xl)
                    .cartesian_product(axis_coord_iter(y, yl))
                    .filter(|&(ex, why)| (ex, why) != (Some(x), Some(y)))
                    .map(|(x, y)| x.zip(y))
                    .map(|c| c.map(|(x, y)| &self.inner[y][x]))
                    .arr::<{ rad_to_adj_arr_len(RADIUS) }>();

                (cell, Adj(adj))
            })
        })
    }

    // TODO
    // can yield indexes? or something
    // pub fn adj_iter_mut(&mut self) -> impl Iterator<Item = (&mut T, Adj<&mut T>)> {}
}

impl<T, X, Y, D: DispSep, P> Display for TwoDGrid<T, X, Y, D, P>
where
    P: ElementPrinter<T>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.inner.iter() {
            let mut first = true;
            for cell in row {
                if !first {
                    D::sep(f)?;
                }

                first = false;
                P::print(cell, f)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

// bool indicates whether a flash has already been propagated for this step or not
type Grid =
    TwoDGrid<Cell<(u8, bool)>, usize, usize, NoSep, DisplayForCellCopy<DisplayFirst<ZeroBold>>>;

trait Step {
    fn step(&mut self) -> usize;
}

impl Step for Grid {
    fn step(&mut self) -> usize {
        for c in self.iter_mut() {
            c.get_mut().0 += 1;
        }

        // it'd be better to do this as a graph but alas
        let mut flashed = 0;
        loop {
            let mut pending = false;
            for (c, adj) in self.adj_iter::<1>() {
                let (count, prop) = c.get();
                if count > 9 && !prop {
                    flashed += 1;

                    for a in adj.iter() {
                        let (count, prop) = a.get();
                        a.set((count + 1, prop));
                        if count >= 9 && !prop {
                            pending = true;
                        }
                    }

                    c.set((count, true));
                }
            }

            if !pending {
                break;
            }
        }

        for c in self.iter_mut() {
            let ref mut count = c.get_mut().0;
            if *count > 9 {
                *count = 0;
            }
            c.get_mut().1 = false;
        }

        flashed
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2021, 11);
    let inp = aoc.get_input();
    let mut g = Grid::from_char(&inp, |c| {
        Cell::new((char::to_digit(c, 10).unwrap().to(), false))
    });

    let flashes: usize = {
        let mut g = g.clone();
        (0..100).map(|_| g.step()).sum()
    };
    aoc.submit_p1(flashes).unwrap();

    let p2 = (1..).find(|_| g.step() == g.size()).unwrap();
    aoc.submit_p2(p2).unwrap();
}
