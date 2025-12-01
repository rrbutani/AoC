use std::{
    collections::{HashSet, VecDeque},
    fmt::{self, Debug, Display, Write},
    marker::PhantomData,
    ops::{
        Add, AddAssign, Deref, DerefMut, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub,
        SubAssign,
    },
    str::FromStr,
};

use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use smallvec::SmallVec;

pub type Grid<C> = TwoDimensionalGrid<C>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TwoDimensionalGrid<C> {
    inner: Vec<Vec<C>>, // [row][col]; aka [y][x]
    width: usize,
}

// assumes single character cells!
impl<C: FromStr> FromStr for Grid<C>
where
    <C as FromStr>::Err: Debug,
{
    type Err = (); // TODO

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new_from_string(s, |l| {
            l.split_inclusive(|_| true).map(|c| c.parse::<C>().unwrap())
        }))
    }
}

impl<C> TwoDimensionalGrid<C> {
    pub fn new_from_iter<RowIter, ColIter>(rows: RowIter) -> Self
    where
        RowIter: Iterator<Item = ColIter>,
        ColIter: Iterator<Item = C>,
    {
        let grid = rows
            .map(|cols_in_row| cols_in_row.collect::<Vec<_>>())
            .collect::<Vec<_>>();

        let width = grid[0].len();
        for (idx, row) in grid.iter().enumerate() {
            debug_assert_eq!(width, row.len(), "row {idx} has unexpected length");
        }

        Self { inner: grid, width }
    }

    pub fn new_from_string<'s, Func, It>(s: &'s str, line_to_cells: Func) -> Self
    where
        Func: FnMut(&'s str) -> It,
        It: Iterator<Item = C>,
    {
        Self::new_from_iter(s.lines().map(line_to_cells))
    }

    pub fn map<R>(&self, mut func: impl FnMut(Coord, &C) -> R) -> TwoDimensionalGrid<R> {
        let mut out = Vec::with_capacity(self.inner.len());

        for (r, row) in self.inner.iter().enumerate() {
            let new_row = row
                .iter()
                .enumerate()
                .map(|(c, cell)| func((r, c).into(), cell))
                .collect();
            out.push(new_row);
        }

        TwoDimensionalGrid { inner: out, width: self.width }
    }

    pub fn new_with_dimensions(Coord { row: height, col: width }: Coord, default: C) -> Self
    where
        C: Clone,
    {
        TwoDimensionalGrid { inner: vec![vec![default; width]; height], width }
    }

    pub fn clear(&mut self)
    where
        C: Default,
    {
        for row in &mut self.inner {
            for cell in row {
                *cell = C::default();
            }
        }
    }
}

impl<C> TwoDimensionalGrid<C> {
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.inner.len()
    }
    pub fn dim(&self) -> Coord<usize> {
        (self.width(), self.height()).into()
    }
}

impl<C> TwoDimensionalGrid<C> {
    pub fn top_left(&self) -> Coord<usize> {
        (0, 0usize).into()
    }
    pub fn top_right(&self) -> Coord<usize> {
        (0, self.height() - 1).into()
    }
    pub fn bottom_right(&self) -> Coord<usize> {
        self.dim() - (1, 1)
    }
    pub fn bottom_left(&self) -> Coord<usize> {
        (self.width() - 1, 0).into()
    }
}

// should give an `Index` impl
impl<C> Deref for TwoDimensionalGrid<C> {
    type Target = [Vec<C>];

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}
// NOTE: we do not offer `DerefMut` because it gives access to the underlying
// `Vec`, allowing for `.clear()` which messes up our invariant (inner vector
// size matches width)

impl<C, Co: Into<Coord<usize>>> Index<Co> for TwoDimensionalGrid<C> {
    type Output = C;

    fn index(&self, co: Co) -> &Self::Output {
        let Coord { row, col } = co.into();
        &self.inner[row][col]
    }
}

impl<C, Co: Into<Coord<usize>>> IndexMut<Co> for TwoDimensionalGrid<C> {
    fn index_mut(&mut self, co: Co) -> &mut Self::Output {
        let Coord { row, col } = co.into();
        &mut self.inner[row][col]
    }
}

impl<C> TwoDimensionalGrid<C> {
    pub fn get(&self, coord: impl Into<Coord<usize>>) -> Option<&C> {
        let Coord { row, col } = coord.into();
        self.inner.get(row).and_then(|r| r.get(col))
    }

    pub fn get_mut(&mut self, coord: impl Into<Coord<usize>>) -> Option<&mut C> {
        let Coord { row, col } = coord.into();
        self.inner.get_mut(row).and_then(|r| r.get_mut(col))
    }
}

impl<C: Display> Display for TwoDimensionalGrid<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: alt formatting should show row/col numbers!
        for row in &self.inner {
            for col in row {
                col.fmt(f)?;
            }
            Display::fmt("\n", f)?;
        }
        Ok(())
    }
}

/////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SingleChar(pub char);
impl FromStr for SingleChar {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let c = chars.next().ok_or(())?;
        if let Some(_) = chars.next() {
            return Err(());
        }

        Ok(SingleChar(c))
    }
}
impl From<&str> for SingleChar {
    fn from(value: &str) -> Self {
        value.parse().unwrap()
    }
}
impl fmt::Display for SingleChar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}
impl From<char> for SingleChar {
    fn from(value: char) -> Self {
        SingleChar(value)
    }
}
impl From<SingleChar> for char {
    fn from(SingleChar(value): SingleChar) -> Self {
        value
    }
}

/////////////

// generic floodfill function?
/*
fn floodfill(
    &mut self,
    start: impl Iterator<Item = Coord<usize>>, // note: can use `Some(...)` for a single starting point
    func: impl FnMut(
        /* curr */
        Coord,
        /* context object that allows either mutable access to the current Cell or read access to the whole grid */
        Ctx,
    ) -> Option<SmallVec<Direction | Coord>>,
) {
    // already visited nodes will not be visited again
    // coordinates given that are out of bounds will be ignored
}
*/

pub struct FloodfillFuncContext<C, G: AsRef<Grid<C>>> {
    grid: G,
    coord: Coord<usize>,
    _cell: PhantomData<C>,
}

impl<C, G: AsRef<Grid<C>>> FloodfillFuncContext<C, G> {
    pub fn grid(&self) -> &Grid<C> {
        self.grid.as_ref()
    }

    pub fn cell(&self) -> &C {
        &self.grid()[self.coord]
    }

    pub fn cell_mut(&mut self) -> &mut C
    where
        G: AsMut<Grid<C>>,
    {
        &mut self.grid.as_mut()[self.coord]
    }

    pub fn coord(&self) -> Coord<usize> {
        self.coord
    }

    pub fn apply(&self, dir: impl NextCoords<C>) -> Option<Coord<usize>> {
        dir.get(self.coord, self.grid())
    }
}

pub trait NextCoords<C>: Sized {
    fn get(self, from: Coord<usize>, grid: &Grid<C>) -> Option<Coord<usize>>;

    fn apply(self, from: Coord<usize>, grid: &Grid<C>) -> Option<Coord<usize>> {
        self.get(from, grid)
    }
}

// todo: scrap? just use `Grid::get` to filter...
impl<C> NextCoords<C> for Coord<usize> {
    fn get(self, _: Coord<usize>, grid: &Grid<C>) -> Option<Coord<usize>> {
        // coordinates given that are out of bounds will be ignored
        if self.row >= grid.height() || self.col >= grid.width() {
            None
        } else {
            Some(self)
        }
    }
}

impl<C> NextCoords<C> for Direction {
    fn get(self, from: Coord<usize>, grid: &Grid<C>) -> Option<Coord<usize>> {
        self.apply(from).and_then(|c| c.get(from, grid))
    }
}

impl<C> NextCoords<C> for ExtendedDirection {
    fn get(self, from: Coord<usize>, grid: &Grid<C>) -> Option<Coord<usize>> {
        self.apply(from).and_then(|c| c.get(from, grid))
    }
}

#[derive(Clone, Copy)]
pub struct Ref<'g, G: ?Sized>(&'g G);
impl<'g, G: ?Sized> Deref for Ref<'g, G> {
    type Target = G;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'g, G: ?Sized> AsRef<G> for Ref<'g, G> {
    fn as_ref(&self) -> &G {
        &self.0
    }
}
pub struct RefMut<'g, G: ?Sized>(&'g mut G);
impl<'g, G: ?Sized> Deref for RefMut<'g, G> {
    type Target = G;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'g, G: ?Sized> DerefMut for RefMut<'g, G> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<'g, G: ?Sized> AsRef<G> for RefMut<'g, G> {
    fn as_ref(&self) -> &G {
        &self.0
    }
}
impl<'g, G: ?Sized> AsMut<G> for RefMut<'g, G> {
    fn as_mut(&mut self) -> &mut G {
        &mut self.0
    }
}

pub type SearchCtx<'f, 'g, C> = FloodfillFuncContext<C, &'f mut Ref<'g, TwoDimensionalGrid<C>>>;
pub type SearchCtxMut<'f, 'g, C> =
    FloodfillFuncContext<C, &'f mut RefMut<'g, TwoDimensionalGrid<C>>>;

pub type SearchNext<NextCoord = Coord, Ctx = ()> = Option<SmallVec<[(NextCoord, Ctx); 4]>>;

impl<C> Grid<C> {
    // generic over `Ref`/`RefMut` for G
    fn search_inner<G, Func, NextCoord, Ctx, const REVISIT: bool>(
        mut grid: G,
        start: impl IntoIterator<Item = Coord<usize>>,
        mut func: Func,
    ) where
        G: AsRef<Self>,
        // for<'g> &'g mut G: Deref<Target = Self>,
        NextCoord: NextCoords<C>,
        Func: for<'f> FnMut(
            FloodfillFuncContext<C, &'f mut G>,
            Option<Ctx>,
        ) -> SearchNext<NextCoord, Ctx>,
    {
        let mut queue: VecDeque<_> = start.into_iter().map(|c| (c, None::<Ctx>)).collect();
        let mut visited: HashSet<_> = if REVISIT {
            HashSet::new()
        } else {
            queue.iter().map(|(c, _)| *c).collect()
        };
        let mut next_warn_threshold = 1000;

        while let Some((curr, ctx)) = queue.pop_front() {
            let res = func(
                // note: `&mut` is our way to scope down the borrow in case `G`
                // is a mutable borrow (can't be copied)
                //
                // undoubtedly a blemish..
                FloodfillFuncContext { grid: &mut grid, coord: curr, _cell: PhantomData },
                ctx,
            );

            for (next, next_ctx) in res.into_iter().flatten() {
                let Some(next) = next.get(curr, grid.as_ref()) else {
                    continue;
                };
                if !REVISIT && !visited.insert(next) {
                    continue;
                }
                queue.push_back((next, Some(next_ctx)));
            }

            if queue.len() > next_warn_threshold {
                eprintln!("WARNING: big queue size: {}", queue.len());
                next_warn_threshold *= 10;
            }
        }
    }

    // essentially BFS
    pub fn search<'g, Func, NextCoord, Ctx /* = () */, const REVISIT: bool>(
        &'g self,
        // note: can use `Some(...)` for a single starting point
        start: impl IntoIterator<Item = Coord<usize>>,
        func: Func,
    ) where
        NextCoord: NextCoords<C>,
        Func: for<'f> FnMut(SearchCtx<'f, 'g, C>, Option<Ctx>) -> SearchNext<NextCoord, Ctx>,
    {
        let this = Ref(self);
        Self::search_inner::<Ref<'g, Self>, _, _, _, REVISIT>(this, start, func)
    }

    // already visited nodes will not be visited again
    pub fn floodfill<'g, Func, NextCoord, Ctx /* = () */>(
        &'g self,
        // note: can use `Some(...)` for a single starting point
        start: impl IntoIterator<Item = Coord<usize>>,
        func: Func,
    ) where
        NextCoord: NextCoords<C>,
        Func: for<'f> FnMut(SearchCtx<'f, 'g, C>, Option<Ctx>) -> SearchNext<NextCoord, Ctx>,
    {
        self.search::<Func, NextCoord, Ctx, false>(start, func)
    }

    pub fn search_mut<'g, Func, NextCoord, Ctx, const REVISIT: bool>(
        &'g mut self,
        start: impl IntoIterator<Item = Coord<usize>>,
        func: Func,
    ) where
        NextCoord: NextCoords<C>,
        Func: for<'f> FnMut(SearchCtxMut<'f, 'g, C>, Option<Ctx>) -> SearchNext<NextCoord, Ctx>,
    {
        let this = RefMut(self);
        Self::search_inner::<RefMut<'g, Self>, _, _, _, REVISIT>(this, start, func)
    }

    pub fn floodfill_mut<'g, Func, NextCoord, Ctx>(
        &'g mut self,
        start: impl IntoIterator<Item = Coord<usize>>,
        func: Func,
    ) where
        NextCoord: NextCoords<C>,
        Func: for<'f> FnMut(SearchCtxMut<'f, 'g, C>, Option<Ctx>) -> SearchNext<NextCoord, Ctx>,
    {
        self.search_mut::<Func, NextCoord, Ctx, false>(start, func)
    }
}

/////////////

fn coord_iter<T>(grid: &Vec<Vec<T>>) -> impl Clone + Iterator<Item = (Coord, &T)> + '_ {
    grid.iter().enumerate().flat_map(|(row_idx, row)| {
        row.iter()
            .enumerate()
            .map(move |(col_idx, cell)| ((row_idx, col_idx).into(), cell))
    })
}

// todo: clone
fn coord_iter_par<T: Sync>(
    grid: &Vec<Vec<T>>,
) -> impl Clone + ParallelIterator<Item = (Coord, &T)> + '_ {
    grid.par_iter().enumerate().flat_map(|(row_idx, row)| {
        row.par_iter()
            .enumerate()
            .map(move |(col_idx, cell)| ((row_idx, col_idx).into(), cell))
    })
}

impl<C> Grid<C> {
    pub fn cell_iter(&self) -> impl Clone + Iterator<Item = (Coord, &C)> + '_ {
        coord_iter(&self.inner)
    }

    pub fn find<'it>(
        &'it self,
        mut predicate: impl FnMut(&C) -> bool + 'it,
    ) -> impl Iterator<Item = Coord> + 'it {
        self.cell_iter()
            .filter(move |(_, c)| predicate(*c))
            .map(|(co, _)| co)
    }
}

impl<C: Sync> Grid<C> {
    pub fn par_cell_iter(&self) -> impl Clone + ParallelIterator<Item = (Coord, &C)> + '_ {
        coord_iter_par(&self.inner)
    }

    pub fn par_find<'it>(
        &'it self,
        predicate: impl Send + Sync + Fn(&C) -> bool + 'it,
    ) -> impl ParallelIterator<Item = Coord> + 'it {
        self.par_cell_iter()
            .filter(move |(_, c)| predicate(*c))
            .map(|(co, _)| co)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Coord<T = usize> {
    pub row: T,
    pub col: T,
}

impl<T: Display> Display for Coord<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&'(', f)?;
        self.row.fmt(f)?;
        Display::fmt(&", ", f)?;
        self.col.fmt(f)?;
        Display::fmt(&')', f)
    }
}

// impl:
//   - pairwise add, subtract, addassign, subassign
//   - scalar mul/divide
//   - checked add for usize with isize coord
//   - {try,}from/{try,}into from corresponding tuple forms
// funcs:
//   - move in dir (unbounded, just on num_traits limits)
//   - move in dir on grid type (bounded; usize)
//   -

///// Into/From /////

/*
impl<S, T> TryFrom<(S, S)> for Coord<T>
where
T: TryFrom<S>,
{
    type Error = <T as TryFrom<S>>::Error;

    fn try_from(value: (S, S)) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl<S, T> TryInto<(S, S)> for Coord<T>
where
T: TryInto<S>,
{
    type Error = <T as TryInto<S>>::Error;

    fn try_into(self) -> Result<(S, S), Self::Error> {
        todo!()
    }
}
*/

impl<S, T> From<(S, S)> for Coord<T>
where
    T: From<S>,
{
    fn from((row, col): (S, S)) -> Self {
        Self { row: row.into(), col: col.into() }
    }
}

impl<S, T> Into<(S, S)> for Coord<T>
where
    T: Into<S>,
{
    fn into(self) -> (S, S) {
        let Self { row, col } = self;
        (row.into(), col.into())
    }
}

///// Add /////

impl<T, A, B, R> Add<(A, B)> for Coord<T>
where
    T: Add<A, Output = R>,
    T: Add<B, Output = R>,
{
    type Output = Coord<R>;

    fn add(self, (r2, c2): (A, B)) -> Self::Output {
        let Coord { row: r, col: c } = self;
        Coord { row: r + r2, col: c + c2 }
    }
}

impl<T, S> Add<Coord<S>> for Coord<T>
where
    T: Add<S>,
{
    type Output = Coord<<T as Add<S>>::Output>;

    fn add(self, Coord { row: r2, col: c2 }: Coord<S>) -> Self::Output {
        let Coord { row: r, col: c } = self;
        Coord { row: r + r2, col: c + c2 }
    }
}

impl<T, A, B> AddAssign<(A, B)> for Coord<T>
where
    T: AddAssign<A>,
    T: AddAssign<B>,
{
    fn add_assign(&mut self, (r2, c2): (A, B)) {
        let Coord { row: ref mut r, col: ref mut c } = self;
        *r += r2;
        *c += c2;
    }
}

impl<T, S> AddAssign<Coord<S>> for Coord<T>
where
    T: AddAssign<S>,
{
    fn add_assign(&mut self, Coord { row: r2, col: c2 }: Coord<S>) {
        let Coord { row: ref mut r, col: ref mut c } = self;
        *r += r2;
        *c += c2;
    }
}

///// Sub /////

impl<T, A, B, R> Sub<(A, B)> for Coord<T>
where
    T: Sub<A, Output = R>,
    T: Sub<B, Output = R>,
{
    type Output = Coord<R>;

    fn sub(self, (r2, c2): (A, B)) -> Self::Output {
        let Coord { row: r, col: c } = self;
        Coord { row: r - r2, col: c - c2 }
    }
}

impl<T, S> Sub<Coord<S>> for Coord<T>
where
    T: Sub<S>,
{
    type Output = Coord<<T as Sub<S>>::Output>;

    fn sub(self, Coord { row: r2, col: c2 }: Coord<S>) -> Self::Output {
        let Coord { row: r, col: c } = self;
        Coord { row: r - r2, col: c - c2 }
    }
}

impl<T, A, B> SubAssign<(A, B)> for Coord<T>
where
    T: SubAssign<A>,
    T: SubAssign<B>,
{
    fn sub_assign(&mut self, (r2, c2): (A, B)) {
        let Coord { row: ref mut r, col: ref mut c } = self;
        *r -= r2;
        *c -= c2;
    }
}

impl<T, S> SubAssign<Coord<S>> for Coord<T>
where
    T: SubAssign<S>,
{
    fn sub_assign(&mut self, Coord { row: r2, col: c2 }: Coord<S>) {
        let Coord { row: ref mut r, col: ref mut c } = self;
        *r -= r2;
        *c -= c2;
    }
}

///// Mul /////

impl<T, S: Clone, R> Mul<S> for Coord<T>
where
    T: Mul<S, Output = R> + Clone,
{
    type Output = Coord<R>;

    fn mul(self, scalar: S) -> Self::Output {
        let Coord { row, col } = self;
        Coord { row: row * scalar.clone(), col: col * scalar }
    }
}

impl<T, S: Clone> MulAssign<S> for Coord<T>
where
    T: MulAssign<S>,
{
    fn mul_assign(&mut self, scalar: S) {
        let Coord { ref mut row, ref mut col } = self;
        *row *= scalar.clone();
        *col *= scalar;
    }
}

///// Div /////

impl<T, S: Clone, R> Div<S> for Coord<T>
where
    T: Div<S, Output = R> + Clone,
{
    type Output = Coord<R>;

    fn div(self, scalar: S) -> Self::Output {
        let Coord { row, col } = self;
        Coord { row: row / scalar.clone(), col: col / scalar }
    }
}

impl<T, S: Clone> DivAssign<S> for Coord<T>
where
    T: DivAssign<S>,
{
    fn div_assign(&mut self, scalar: S) {
        let Coord { ref mut row, ref mut col } = self;
        *row /= scalar.clone();
        *col /= scalar;
    }
}

///// Neg /////
impl<T: Neg<Output = R>, R> Neg for Coord<T> {
    type Output = Coord<R>;

    fn neg(self) -> Self::Output {
        let Coord { row, col } = self;
        Coord { row: row.neg(), col: col.neg() }
    }
}

///// extras /////

// impl
// checked_add_signed

impl Coord<usize> {
    pub fn checked_add_signed(self, rhs: Coord<isize>) -> Option<Self> {
        let Coord { row, col } = self;
        let Coord { row: r2, col: c2 } = rhs;

        Some(Coord { row: row.checked_add_signed(r2)?, col: col.checked_add_signed(c2)? })
    }

    pub fn as_signed(self) -> Coord<isize> {
        Coord { row: self.row.try_into().unwrap(), col: self.col.try_into().unwrap() }
    }

    pub fn signed_diff(self, rhs: Self) -> Coord<isize> {
        self.as_signed() - rhs.as_signed()
    }
}

////////////////////////////////////////////////////////////////////////////////

// direction:
//   - as coord offset
//   - rotate

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Direction::*;
        // single char
        // ascii
        // TODO: arrow
        // TODO: U/R/D/L?
        let c = if f.alternate() {
            match self {
                North => 'N',
                East => 'E',
                South => 'S',
                West => 'W',
            }
        } else {
            match self {
                North => '^',
                East => '>',
                South => 'v',
                West => '<',
            }
        };
        f.write_char(c)
    }
}

#[allow(bad_style)]
impl Direction {
    pub const Up: Self = Direction::North;
    pub const Right: Self = Direction::East;
    pub const Down: Self = Direction::South;
    pub const Left: Self = Direction::West;
}

impl Direction {
    pub const ALL: [Self; 4] =
        [Direction::North, Direction::East, Direction::South, Direction::West];
}

impl TryFrom<usize> for Direction {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        use Direction::*;
        Ok(match value {
            0 => North,
            1 => East,
            2 => South,
            3 => West,
            _ => return Err(()),
        })
    }
}

impl Direction {
    pub const fn rotate(self, n: i8) -> Self {
        let Some(new) = (self as u8 + (Self::ALL.len() as u8)).checked_add_signed(n) else {
            panic!()
        };

        Self::ALL[(new % (Self::ALL.len() as u8)) as usize]
    }

    pub const fn clockwise(self) -> Self {
        self.rotate(1)
    }
    pub const fn counterclockwise(self) -> Self {
        self.rotate(-1)
    }
}

impl From<Direction> for Coord<isize> {
    fn from(val: Direction) -> Self {
        use Direction::*;
        match val {
            North => (-1isize, 0),
            East => (0, 1),
            South => (1, 0),
            West => (0, -1),
        }
        .into()
    }
}

impl Direction {
    pub fn apply(self, Coord { row, col }: Coord<usize>) -> Option<Coord<usize>> {
        let Coord { row: r, col: c } = self.into();
        let row = row.checked_add_signed(r);
        let col = col.checked_add_signed(c);
        row.zip(col).map(Into::into)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExtendedDirection {
    North = 0,
    NorthEast = 1,
    East = 2,
    SouthEast = 3,
    South = 4,
    SouthWest = 5,
    West = 6,
    NorthWest = 7,
}

#[allow(bad_style)]
impl ExtendedDirection {
    pub const Up: Self = ExtendedDirection::North;
    pub const Right: Self = ExtendedDirection::East;
    pub const Down: Self = ExtendedDirection::South;
    pub const Left: Self = ExtendedDirection::West;
}

impl ExtendedDirection {
    pub const ALL: [Self; 8] = [
        Self::North,
        Self::NorthEast,
        Self::East,
        Self::SouthEast,
        Self::South,
        Self::SouthWest,
        Self::West,
        Self::NorthWest,
    ];

    pub const CARDINALS: [Self; 4] = [Self::North, Self::East, Self::South, Self::West];

    pub const DIAGONALS: [Self; 4] =
        [Self::NorthEast, Self::SouthEast, Self::SouthWest, Self::NorthWest];
}

impl From<Direction> for ExtendedDirection {
    fn from(value: Direction) -> Self {
        match value {
            Direction::North => Self::North,
            Direction::East => Self::East,
            Direction::South => Self::South,
            Direction::West => Self::West,
        }
    }
}

impl TryFrom<ExtendedDirection> for Direction {
    type Error = ExtendedDirection;

    fn try_from(value: ExtendedDirection) -> Result<Self, Self::Error> {
        use ExtendedDirection as E;
        match value {
            E::North => Ok(Direction::North),
            E::East => Ok(Direction::East),
            E::South => Ok(Direction::South),
            E::West => Ok(Direction::West),
            other => Err(other),
        }
    }
}

impl ExtendedDirection {
    pub const fn rotate(self, n: i8) -> Self {
        let Some(new) = (self as u8 + (Self::ALL.len() as u8)).checked_add_signed(n) else {
            panic!()
        };

        Self::ALL[(new % (Self::ALL.len() as u8)) as usize]
    }

    pub const fn clockwise(self) -> Self {
        self.rotate(1)
    }
    pub const fn counterclockwise(self) -> Self {
        self.rotate(-1)
    }
}

impl From<ExtendedDirection> for Coord<isize> {
    fn from(val: ExtendedDirection) -> Self {
        use ExtendedDirection::*;
        match val {
            North => (-1isize, 0),
            NorthEast => (-1, 1),
            East => (0, 1),
            SouthEast => (1, 1),
            South => (1, 0),
            SouthWest => (1, -1),
            West => (0, -1),
            NorthWest => (-1, -1),
        }
        .into()
    }
}

impl ExtendedDirection {
    pub fn apply(self, Coord { row, col }: Coord<usize>) -> Option<Coord<usize>> {
        let Coord { row: r, col: c } = self.into();
        let row = row.checked_add_signed(r);
        let col = col.checked_add_signed(c);
        row.zip(col).map(Into::into)
    }
}
