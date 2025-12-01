use core::mem;
use std::{
    cmp::Ordering,
    fmt::Debug,
    iter::FusedIterator,
    ops::{Add, AddAssign, Index, IndexMut, Range, Sub, SubAssign},
    str::FromStr,
};

use itertools::Itertools;
use num_traits::One;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    const ALL: [Self; 3] = [Axis::X, Axis::Y, Axis::Z];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FixedOrRange<'a, T> {
    Fixed(&'a T),
    // hi is non-inclusive; i.e. lo..hi
    Range { lo: &'a T, hi: &'a T },
}

impl<'a, T: Clone + Add<T, Output = T> + One> FixedOrRange<'_, T> {
    fn as_range(&self) -> Range<T> {
        match *self {
            FixedOrRange::Fixed(x) => x.clone()..x.clone() + One::one(),
            FixedOrRange::Range { lo, hi } => lo.clone()..hi.clone(),
        }
    }
}

impl<T: Clone> FixedOrRange<'_, T> {
    pub fn to_owned(&self) -> FixedOrRangeOwned<T> {
        match *self {
            FixedOrRange::Fixed(x) => FixedOrRangeOwned::Fixed(x.clone()),
            FixedOrRange::Range { lo, hi } => {
                FixedOrRangeOwned::Range { lo: lo.clone(), hi: hi.clone() }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FixedOrRangeOwned<T> {
    Fixed(T),
    Range { lo: T, hi: T },
}

pub trait GetForAxis<T> {
    fn get_for_axis(&self, axis: Axis) -> FixedOrRange<T>;

    fn from_axis_parts(parts: Triple<FixedOrRangeOwned<T>>) -> Result<Self, ()>
    where
        Self: Sized;

    fn as_axis_parts(&self) -> Triple<FixedOrRange<T>> {
        Axis::ALL.map(|a| self.get_for_axis(a)).into()
    }

    fn as_axis_parts_owned(&self) -> Triple<FixedOrRangeOwned<T>>
    where
        T: Clone,
    {
        let arr: [_; 3] = self.as_axis_parts().into();
        arr.map(|p| p.to_owned()).into()
    }

    #[doc(hidden)]
    fn triple_using_ordering(&self, parts: [T; Axis::ALL.len()]) -> Triple<T>;

    fn as_cube(&self) -> Cube<T>
    where
        T: Clone + Add<T, Output = T> + One,
    {
        let [x, y, z] = Axis::ALL.map(|a| self.get_for_axis(a).as_range());

        Cube { x_lo: x.start, x_hi: x.end, y_lo: y.start, y_hi: y.end, z_lo: z.start, z_hi: z.end }
    }

    fn try_as_plane(&self) -> Result<Plane<T>, ()>
    where
        T: Clone + Add<T, Output = T> + One,
    {
        // try to collapse 1 axis down to a point
        todo!()
    }

    fn try_as_line(&self) -> Result<Line<T>, ()>
    where
        T: Clone + Add<T, Output = T> + One,
    {
        // try to collapse 2 axes down to a point
        todo!()
    }

    fn try_as_point(&self) -> Result<Point<T>, ()>
    where
        T: Clone + Add<T, Output = T> + One,
    {
        // try to collapse all axes down to a point
        todo!()
    }
}

////////////////////////////////////////////////////////////////////////////////

// represents a point
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Triple<T>(pub T, pub T, pub T);

impl<T: FromStr> FromStr for Triple<T>
where
    <T as FromStr>::Err: Debug,
{
    type Err = <T as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (a, b, c) = s.split(",").map(|s| s.trim()).collect_tuple().unwrap();

        // Ok(Self(a.parse()?, b.parse()?, c.parse()?))
        Ok(Self(a.parse().unwrap(), b.parse().unwrap(), c.parse().unwrap()))
    }
}

impl<T> From<(T, T, T)> for Triple<T> {
    fn from((a, b, c): (T, T, T)) -> Self {
        Self(a, b, c)
    }
}

impl<T> Into<(T, T, T)> for Triple<T> {
    fn into(self) -> (T, T, T) {
        (self.0, self.1, self.2)
    }
}

impl<T> From<[T; Axis::ALL.len()]> for Triple<T> {
    fn from([x, y, z]: [T; 3]) -> Self {
        (x, y, z).into()
    }
}

impl<T> Into<[T; Axis::ALL.len()]> for Triple<T> {
    fn into(self) -> [T; 3] {
        let Triple(x, y, z) = self;
        [x, y, z]
    }
}

impl<T> Index<Axis> for Triple<T> {
    type Output = T;

    fn index(&self, axis: Axis) -> &Self::Output {
        let Triple(x, y, z) = self;
        match axis {
            Axis::X => x,
            Axis::Y => y,
            Axis::Z => z,
        }
    }
}

impl<T> IndexMut<Axis> for Triple<T> {
    fn index_mut(&mut self, axis: Axis) -> &mut Self::Output {
        let Triple(x, y, z) = self;
        match axis {
            Axis::X => x,
            Axis::Y => y,
            Axis::Z => z,
        }
    }
}

///// Add /////

impl<T, A, R> Add<Triple<A>> for Triple<T>
where
    T: Add<A, Output = R>,
{
    type Output = Triple<R>;

    fn add(self, Triple(a, b, c): Triple<A>) -> Self::Output {
        let Triple(x, y, z) = self;
        Triple(x + a, y + b, z + c)
    }
}

impl<T, A, B, C, R> Add<(A, B, C)> for Triple<T>
where
    T: Add<A, Output = R>,
    T: Add<B, Output = R>,
    T: Add<C, Output = R>,
{
    type Output = Triple<R>;

    fn add(self, (a, b, c): (A, B, C)) -> Self::Output {
        let Triple(x, y, z) = self;
        Triple(x + a, y + b, z + c)
    }
}

impl<T, A> AddAssign<Triple<A>> for Triple<T>
where
    T: AddAssign<A>,
{
    fn add_assign(&mut self, Triple(a, b, c): Triple<A>) {
        let Triple(x, y, z) = self;
        *x += a;
        *y += b;
        *z += c;
    }
}

impl<T, A, B, C> AddAssign<(A, B, C)> for Triple<T>
where
    T: AddAssign<A> + AddAssign<B> + AddAssign<C>,
{
    fn add_assign(&mut self, (a, b, c): (A, B, C)) {
        let Triple(x, y, z) = self;
        *x += a;
        *y += b;
        *z += c;
    }
}

///// Sub /////

impl<T, A, R> Sub<Triple<A>> for Triple<T>
where
    T: Sub<A, Output = R>,
{
    type Output = Triple<R>;

    fn sub(self, Triple(a, b, c): Triple<A>) -> Self::Output {
        let Triple(x, y, z) = self;
        Triple(x - a, y - b, z - c)
    }
}

impl<T, A, B, C, R> Sub<(A, B, C)> for Triple<T>
where
    T: Sub<A, Output = R>,
    T: Sub<B, Output = R>,
    T: Sub<C, Output = R>,
{
    type Output = Triple<R>;

    fn sub(self, (a, b, c): (A, B, C)) -> Self::Output {
        let Triple(x, y, z) = self;
        Triple(x - a, y - b, z - c)
    }
}

impl<T, A> SubAssign<Triple<A>> for Triple<T>
where
    T: SubAssign<A>,
{
    fn sub_assign(&mut self, Triple(a, b, c): Triple<A>) {
        let Triple(x, y, z) = self;
        *x -= a;
        *y -= b;
        *z -= c;
    }
}

impl<T, A, B, C> SubAssign<(A, B, C)> for Triple<T>
where
    T: SubAssign<A> + SubAssign<B> + SubAssign<C>,
{
    fn sub_assign(&mut self, (a, b, c): (A, B, C)) {
        let Triple(x, y, z) = self;
        *x -= a;
        *y -= b;
        *z -= c;
    }
}

// todo: scalar ops: mul/divide

//////////

impl<T: Add<T, Output = T> + One + Clone> Triple<T> {
    fn as_line(self) -> Line<T> {
        Line {
            iteration_axis: Axis::X,
            a_lo: self.0.clone(),
            a_hi: self.0.clone() + One::one(),
            fixed_b: self.1,
            fixed_c: self.2,
        }
    }

    fn as_plane(self) -> Plane<T> {
        // note(perf): can impl directly if the optimizer doesn't boil this away
        self.as_line().as_plane()
    }
}
impl<T: Add<T, Output = T> + One + Clone> Into<Line<T>> for Triple<T> {
    fn into(self) -> Line<T> {
        self.as_line()
    }
}
impl<T: Add<T, Output = T> + One + Clone> Into<Plane<T>> for Triple<T> {
    fn into(self) -> Plane<T> {
        self.as_plane()
    }
}
impl<T: Add<T, Output = T> + One + Clone> Into<Cube<T>> for Triple<T> {
    fn into(self) -> Cube<T> {
        self.as_cube()
    }
}

impl<T> GetForAxis<T> for Triple<T> {
    fn get_for_axis(&self, axis: Axis) -> FixedOrRange<T> {
        let Self(x, y, z) = self;
        use {Axis::*, FixedOrRange::Fixed};
        match axis {
            X => Fixed(x),
            Y => Fixed(y),
            Z => Fixed(z),
        }
    }

    fn from_axis_parts(parts: Triple<FixedOrRangeOwned<T>>) -> Result<Self, ()>
    where
        Self: Sized,
    {
        use FixedOrRangeOwned::Fixed;
        match parts.into() {
            (Fixed(a), Fixed(b), Fixed(c)) => Ok(Triple(a, b, c)),
            _ => Err(()),
        }
    }

    fn triple_using_ordering(&self, [a, b, c]: [T; 3]) -> Triple<T> {
        Triple(a, b, c)
    }

    fn as_cube(&self) -> Cube<T>
    where
        T: Clone + Add<T, Output = T> + One,
    {
        self.clone().as_plane().as_cube()
    }

    fn try_as_plane(&self) -> Result<Plane<T>, ()>
    where
        T: Clone + Add<T, Output = T> + One,
    {
        Ok(self.clone().as_plane())
    }

    fn try_as_line(&self) -> Result<Line<T>, ()>
    where
        T: Clone + Add<T, Output = T> + One,
    {
        Ok(self.clone().as_line())
    }

    fn try_as_point(&self) -> Result<Point<T>, ()>
    where
        T: Clone + Add<T, Output = T> + One,
    {
        Ok(self.clone())
    }
}

pub type Point<T> = Triple<T>;

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Line<T> {
    iteration_axis: Axis,
    // like `lo..hi`; inclusive lower bound, exclusive upper
    a_lo: T,
    a_hi: T,
    fixed_b: T,
    fixed_c: T,
}

impl<T: Add<T, Output = T> + One + Clone> Line<T> {
    fn as_plane(self) -> Plane<T> {
        let Self { iteration_axis, a_lo, a_hi, fixed_b, fixed_c } = self;
        use Axis::*;
        let ((a_lo, a_hi), (b_lo, b_hi), (fixed_axis, fixed_c)) = match iteration_axis {
            X => {
                // a -> a, b -> b, c -> c (xyz -> xyz)
                ((a_lo, a_hi), (fixed_b.clone(), fixed_b + T::one()), (Z, fixed_c))
            }
            Y => {
                // a -> b, b -> a, c -> c (yxz -> xyz); could do yzx..
                ((fixed_b.clone(), fixed_b + One::one()), (a_lo, a_hi), (Z, fixed_c))
            }
            Z => {
                // a -> b, b -> c, c -> a (zxy -> yzx); could do xzy..
                ((fixed_c.clone(), fixed_c + One::one()), (a_lo, a_hi), (X, fixed_b))
            }
        };

        Plane { a_lo, a_hi, b_lo, b_hi, fixed_axis, fixed_c }
    }
}
impl<T: Add<T, Output = T> + One + Clone> Into<Plane<T>> for Line<T> {
    fn into(self) -> Plane<T> {
        self.as_plane()
    }
}
impl<T: Add<T, Output = T> + One + Clone> Into<Cube<T>> for Line<T> {
    fn into(self) -> Cube<T> {
        self.as_cube()
    }
}

impl<T> GetForAxis<T> for Line<T> {
    fn get_for_axis(&self, axis: Axis) -> FixedOrRange<T> {
        use {Axis::*, FixedOrRange::*};
        match (axis, self.iteration_axis) {
            (X, X) | (Y, Y) | (Z, Z) => Range { lo: &self.a_lo, hi: &self.a_hi },
            // if axis == first non-iteration axis
            (X, Y | Z) | (Y, X) => Fixed(&self.fixed_b),
            // second non-iteration axis
            (Z, X | Y) | (Y, Z) => Fixed(&self.fixed_c),
        }
    }

    fn from_axis_parts(parts: Triple<FixedOrRangeOwned<T>>) -> Result<Self, ()>
    where
        Self: Sized,
    {
        use {Axis::*, FixedOrRangeOwned::*};
        // exactly one axis should be a range
        let ((iteration_axis, (a_lo, a_hi)), fixed_b, fixed_c) = match parts.into() {
            (Range { lo, hi }, Fixed(y), Fixed(z)) => ((X, (lo, hi)), y, z),
            (Fixed(x), Range { lo, hi }, Fixed(z)) => ((Y, (lo, hi)), x, z),
            (Fixed(x), Fixed(y), Range { lo, hi }) => ((Z, (lo, hi)), x, y),
            _ => return Err(()),
        };

        Ok(Self { iteration_axis, a_lo, a_hi, fixed_b, fixed_c })
    }

    fn triple_using_ordering(&self, [a, b, c]: [T; 3]) -> Triple<T> {
        use Axis::*;
        match self.iteration_axis {
            X => Triple(a, b, c),
            Y => Triple(b, a, c),
            Z => Triple(b, c, a),
        }
    }

    fn as_cube(&self) -> Cube<T>
    where
        T: Clone + Add<T, Output = T> + One,
    {
        self.clone().as_plane().as_cube()
    }

    fn try_as_plane(&self) -> Result<Plane<T>, ()>
    where
        T: Clone + Add<T, Output = T> + One,
    {
        Ok(self.clone().as_plane())
    }

    fn try_as_line(&self) -> Result<Line<T>, ()>
    where
        T: Clone + Add<T, Output = T> + One,
    {
        Ok(self.clone())
    }
}

impl<T> IntoIterator for Line<T>
where
    LineIter<T>: Iterator<Item = Triple<T>>,
{
    type Item = Triple<T>;
    type IntoIter = LineIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        LineIter(self)
    }
}

// note: we'd like to use `core::iter::Step` here but its unstable
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineIter<T>(Line<T>);

impl<T> LineIter<T> {
    pub fn into_line(self) -> Line<T> {
        self.0
    }

    pub fn remaining_line(&self) -> Line<T>
    where
        T: Clone,
    {
        self.0.clone()
    }
}

impl<T> FusedIterator for LineIter<T> where Self: Iterator<Item = Triple<T>> {}

impl<T> Iterator for LineIter<T>
where
    T: Clone + Ord + Add<T, Output = T> + Sub<T, Output = T> + One,
{
    type Item = Triple<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let ref mut this = self.0;
        let a = match this.a_lo.cmp(&this.a_hi) {
            Ordering::Less => {
                let new_lo = this.a_lo.clone() + One::one();
                let lo = mem::replace(&mut this.a_lo, new_lo);
                lo
            }
            Ordering::Equal => return None,
            Ordering::Greater => unreachable!(),
        };

        let (b, c) = (this.fixed_b.clone(), this.fixed_c.clone());
        Some(this.triple_using_ordering([a, b, c]))
    }

    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }

    // Can only impl if we have `T: Try{Into,From}<usize>` ...
    /*
    fn size_hint(&self) -> (usize, Option<usize>) { todo!() }
    fn count(self) -> usize where Self: Sized { }
    fn nth(&mut self, n: usize) -> Option<Self::Item> { todo!() }
    */
}

impl<T> DoubleEndedIterator for LineIter<T>
where
    T: Clone + Ord + Add<T, Output = T> + Sub<T, Output = T> + One,
    Self: Iterator<Item = Triple<T>>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let ref mut this = self.0;
        let a = match this.a_lo.cmp(&this.a_hi) {
            Ordering::Less => {
                let new_hi = this.a_hi.clone() - One::one();
                let hi = mem::replace(&mut this.a_hi, new_hi);
                hi
            }
            Ordering::Equal => return None,
            Ordering::Greater => unreachable!(),
        };

        let (b, c) = (this.fixed_b.clone(), this.fixed_c.clone());
        Some(this.triple_using_ordering([a, b, c]))
    }

    // Can only impl if we have `T: Try{Into,From}<usize>` ...
    /* fn nth_back */
}

impl<T> ExactSizeIterator for LineIter<T>
where
    T: Clone + Ord + Sub<T, Output = T> + TryInto<usize>,
    Self: Iterator<Item = T>,
{
    fn len(&self) -> usize {
        (|| -> Option<usize> {
            let this = &self.0;
            match this.a_lo.cmp(&this.a_hi) {
                Ordering::Less => {
                    let len = (this.a_hi.clone() - this.a_lo.clone()).try_into().ok()?;
                    Some(len)
                }
                Ordering::Equal => Some(0),
                Ordering::Greater => unreachable!(),
            }
        })()
        .unwrap_or(0)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Plane<T> {
    a_lo: T,
    a_hi: T,
    b_lo: T,
    b_hi: T,
    fixed_axis: Axis,
    fixed_c: T,
}

impl<T: Add<T, Output = T> + One + Clone> Into<Cube<T>> for Plane<T> {
    fn into(self) -> Cube<T> {
        self.as_cube()
    }
}

impl<T> GetForAxis<T> for Plane<T> {
    fn get_for_axis(&self, axis: Axis) -> FixedOrRange<T> {
        use {Axis::*, FixedOrRange::*};
        // same idea as `Line<T>`'s impl but with the logic invert w.r.t to
        // fixed and range
        match (axis, self.fixed_axis) {
            (X, X) | (Y, Y) | (Z, Z) => Fixed(&self.fixed_c),
            // first iteration axis:
            (X, Y | Z) | (Y, X) => Range { lo: &self.a_lo, hi: &self.a_hi },
            // second non-iteration axis:
            (Z, X | Y) | (Y, Z) => Range { lo: &self.b_lo, hi: &self.b_hi },
        }
    }

    fn from_axis_parts(parts: Triple<FixedOrRangeOwned<T>>) -> Result<Self, ()>
    where
        Self: Sized,
    {
        use {Axis::*, FixedOrRangeOwned::*};
        // exactly two axes should be ranges; one fixed
        let ((fixed_axis, fixed_c), (a_lo, a_hi), (b_lo, b_hi)) = match parts.into() {
            (Fixed(x), Range { lo: y_lo, hi: y_hi }, Range { lo: z_lo, hi: z_hi }) => {
                ((X, x), (y_lo, y_hi), (z_lo, z_hi))
            }
            (Range { lo: x_lo, hi: x_hi }, Fixed(y), Range { lo: z_lo, hi: z_hi }) => {
                ((Y, y), (x_lo, x_hi), (z_lo, z_hi))
            }
            (Range { lo: x_lo, hi: x_hi }, Range { lo: y_lo, hi: y_hi }, Fixed(z)) => {
                ((Z, z), (x_lo, x_hi), (y_lo, y_hi))
            }
            _ => return Err(()),
        };

        Ok(Self { a_lo, a_hi, b_lo, b_hi, fixed_axis, fixed_c })
    }

    fn triple_using_ordering(&self, [a, b, c]: [T; 3]) -> Triple<T> {
        use Axis::*;
        match self.fixed_axis {
            X => Triple(c, a, b),
            Y => Triple(a, c, b),
            Z => Triple(a, b, c),
        }
    }
}

// TODO: proper line iterator?
// TODO: proper point iterator?

impl<T> Plane<T>
where
    T: Clone + Ord + Add<T, Output = T> + Sub<T, Output = T> + One,
{
    // todo: should we offer ways to customize the iteration direction?

    // note: wasteful impl if `T` isn't copy...
    pub fn lines(&self) -> impl Iterator<Item = Line<T>> + DoubleEndedIterator + FusedIterator {
        use {Axis::*, FixedOrRangeOwned::*};

        let mut parts = self.as_axis_parts_owned();
        let [first_iter_dim, second_iter_dim] = match self.fixed_axis {
            X => [Y, Z],
            Y => [X, Z],
            Z => [X, Y],
        };

        let outer_line = {
            let mut parts = parts.clone();
            // freeze the second iter dimension:
            parts[second_iter_dim] = Fixed(One::one());
            Line::from_axis_parts(parts).unwrap().into_iter()
        };

        // for each point on the outer line, yield a line:
        outer_line.map(move |triple| {
            // replace the first iter dimension with the fixed value at this
            // point in the outer iteration:
            parts[first_iter_dim] = Fixed(triple[first_iter_dim].clone());

            // the second iter dimension is still a range; now make a line:
            Line::from_axis_parts(parts.clone()).unwrap()
        })
    }

    pub fn points(&self) -> impl Iterator<Item = Point<T>> + DoubleEndedIterator + FusedIterator {
        self.lines().flatten()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cube<T> {
    // x: Range<T>,
    x_lo: T,
    x_hi: T,
    // y: Range<T>,
    y_lo: T,
    y_hi: T,
    // z: Range<T>,
    z_lo: T,
    z_hi: T,
}

impl<T> GetForAxis<T> for Cube<T> {
    fn get_for_axis(&self, axis: Axis) -> FixedOrRange<T> {
        use {Axis::*, FixedOrRange::Range};
        match axis {
            X => Range { lo: &self.x_lo, hi: &self.x_hi },
            Y => Range { lo: &self.y_lo, hi: &self.y_hi },
            Z => Range { lo: &self.z_lo, hi: &self.z_hi },
        }
    }

    fn from_axis_parts(parts: Triple<FixedOrRangeOwned<T>>) -> Result<Self, ()>
    where
        Self: Sized,
    {
        // must have three ranges:
        use FixedOrRangeOwned::Range as R;
        match parts.into() {
            (R { lo: x_lo, hi: x_hi }, R { lo: y_lo, hi: y_hi }, R { lo: z_lo, hi: z_hi }) => {
                Ok(Cube { x_lo, x_hi, y_lo, y_hi, z_lo, z_hi })
            }
            _ => Err(()),
        }
    }

    fn triple_using_ordering(&self, [x, y, z]: [T; 3]) -> Triple<T> {
        Triple(x, y, z)
    }
}

impl<T: TryInto<usize> + Sub<T, Output = T> + Clone> Cube<T> {
    pub fn size(&self) -> Result<usize, <T as TryInto<usize>>::Error> {
        let x_len = (self.x_hi.clone() - self.x_lo.clone()).try_into()?;
        let y_len = (self.x_hi.clone() - self.x_lo.clone()).try_into()?;
        let z_len = (self.x_hi.clone() - self.x_lo.clone()).try_into()?;

        Ok(x_len * y_len * z_len)
    }
}

impl<T: Clone> Cube<T> {
    pub fn x(&self) -> Range<T> {
        // should be equiv to: `self.get_for_axis(Axis::X).as_range()` (but
        // without the trait bounds since we know already have a non-empty
        // range)
        self.x_lo.clone()..self.x_hi.clone()
    }
    pub fn y(&self) -> Range<T> {
        self.y_lo.clone()..self.y_hi.clone()
    }
    pub fn z(&self) -> Range<T> {
        self.z_lo.clone()..self.z_hi.clone()
    }
    pub fn as_ranges(&self) -> Triple<Range<T>> {
        (self.x(), self.y(), self.z()).into()
    }
}

// pub type EdgeIter<T, F> = iter::Map<LineIter<T>, F>;

/// assuming x is lateral (left-right), y is "depth" (front-back), z is height
/// (top-bot)
impl<T: Clone> Cube<T> {
    // triple iter, edges
    //  - just join all the edges (exposed as sep methods), careful to drop
    //    duplicates
    //    + assuming x is lateral, y is "depth", z is height
    //    + `top_front` (..x, y_lo, z_hi)
    //    + `top_back` (..x, y_hi, z_hi)
    //    + `top_left` (x_lo, ..y, z_hi)
    //    + `top_right` (x_hi, ..y, z_hi)
    //    + `bot_front` (..x, y_lo, z_lo)
    //    + `bot_back` (..x, y_hi, z_lo)
    //    + `bot_left` (x_lo, ..y, z_lo)
    //    + `bot_right` (x_hi, ..y, z_lo)
    //    + `vert_front_left` (x_lo, y_lo, ..z)
    //    + `vert_front_right` (x_hi, y_lo, ..z)
    //    + `vert_back_left` (x_lo, y_hi, ..z)
    //    + `vert_back_right` (x_hi, y_hi, ..z)

    /*
    fn x_it(&self) -> LineIter<T> {
        LineIter { lo: self.x_lo.clone(), hi: self.x_hi.clone() }
    }
    fn y_it(&self) -> LineIter<T> {
        LineIter { lo: self.y_lo.clone(), hi: self.y_hi.clone() }
    }
    fn z_it(&self) -> LineIter<T> {
        LineIter { lo: self.z_lo.clone(), hi: self.z_hi.clone() }
    }

    const LO: bool = false;
    const HI: bool = true;

    // note: not using `impl Iterator<Item = Triple<T>> + ...` here so that
    // extra traits (i.e. fused, double ended, clone) are available to users
    pub fn edge_it(
        &self,
        axis: Axis,
        two: bool, // lo (false) or high (true)
        three: bool,
    ) -> EdgeIter<T, impl FnMut(T) -> Triple<T>> {
        use Axis::*;
        let (b, c) = match axis {
            X => (Y, Z),
            Y => (X, Z),
            Z => (X, Y),
        };
        let point = |axis, hi| {
            match (axis, hi) {
                (X, Self::LO) => &self.x_lo,
                (X, Self::HI) => &self.x_hi,
                (Y, Self::LO) => &self.y_lo,
                (Y, Self::HI) => &self.y_hi,
                (Z, Self::LO) => &self.z_lo,
                (Z, Self::HI) => &self.z_hi,
            }
            .clone()
        };
        let (b, c) = (point(b, two), point(c, three));

        let it = match axis {
            X => self.x_it(),
            Y => self.y_it(),
            Z => self.z_it(),
        };

        it.map(move |a| {
            let (b, c) = (b.clone(), c.clone());
            match axis {
                Axis::X => Triple(a, b, c),
                Axis::Y => Triple(b, a, c),
                Axis::Z => Triple(b, c, a),
            }
        })
    }

    // note: it's unfortunate that these functions have a different return type,
    // on paper..
    //
    // users will have to call `axis_it` directly to work around this (until we
    // get existential types — aka TAIT — that is)
    pub fn top_front_edge(&self) -> EdgeIter<T, impl FnMut(T) -> Triple<T>> {
        self.edge_it(Axis::X, Self::LO, Self::HI)
    }
    pub fn top_back_edge(&self) -> EdgeIter<T, impl FnMut(T) -> Triple<T>> {
        self.edge_it(Axis::X, Self::HI, Self::HI)
    }
    pub fn top_left_edge(&self) -> EdgeIter<T, impl FnMut(T) -> Triple<T>> {
        self.edge_it(Axis::Y, Self::LO, Self::HI)
    }
    pub fn top_right_edge(&self) -> EdgeIter<T, impl FnMut(T) -> Triple<T>> {
        self.edge_it(Axis::Y, Self::HI, Self::HI)
    }
    pub fn bot_front_edge(&self) -> EdgeIter<T, impl FnMut(T) -> Triple<T>> {
        self.edge_it(Axis::X, Self::LO, Self::LO)
    }
    pub fn bot_back_edge(&self) -> EdgeIter<T, impl FnMut(T) -> Triple<T>> {
        self.edge_it(Axis::X, Self::HI, Self::LO)
    }
    pub fn bot_left_edge(&self) -> EdgeIter<T, impl FnMut(T) -> Triple<T>> {
        self.edge_it(Axis::Y, Self::LO, Self::LO)
    }
    pub fn bot_right_edge(&self) -> EdgeIter<T, impl FnMut(T) -> Triple<T>> {
        self.edge_it(Axis::Y, Self::HI, Self::LO)
    }
    pub fn vert_front_left_edge(&self) -> EdgeIter<T, impl FnMut(T) -> Triple<T>> {
        self.edge_it(Axis::Z, Self::LO, Self::LO)
    }
    pub fn vert_front_right_edge(&self) -> EdgeIter<T, impl FnMut(T) -> Triple<T>> {
        self.edge_it(Axis::Z, Self::HI, Self::LO)
    }
    pub fn vert_back_left_edge(&self) -> EdgeIter<T, impl FnMut(T) -> Triple<T>> {
        self.edge_it(Axis::Z, Self::LO, Self::HI)
    }
    pub fn vert_back_right_edge(&self) -> EdgeIter<T, impl FnMut(T) -> Triple<T>> {
        self.edge_it(Axis::Z, Self::HI, Self::HI)
    }
    */

    const LO: bool = false;
    const HI: bool = true;

    /// info: `false` -> lo, `true` -> hi for each of `X`, `Y`, `Z`
    ///
    /// `Z`:
    ///   - `true`: top
    ///   - `false` bottom
    /// `Y`:
    ///   - `true`: back
    ///   - `false` front
    /// `X`:
    ///   - `true`: right
    ///   - `false` left
    pub fn corner(&self, info: impl Into<Triple<bool>>) -> Triple<T> {
        let (x, y, z) = info.into().into();
        [
            if x { &self.x_hi } else { &self.x_lo },
            if y { &self.y_hi } else { &self.y_lo },
            if z { &self.z_hi } else { &self.z_lo },
        ]
        .map(Clone::clone)
        .into()
    }
    //   - top_back_right
    //   - top_back_left
    //   - top_front_right
    //   - top_front_left
    //   - bot_back_right
    //   - bot_back_left
    //   - bot_front_right
    //   - bot_front_left
    pub fn top_back_right(&self) -> Point<T> {
        self.corner([Self::HI, Self::HI, Self::HI])
    }
    pub fn top_back_left(&self) -> Point<T> {
        self.corner([Self::LO, Self::HI, Self::HI])
    }
    pub fn top_front_right(&self) -> Point<T> {
        self.corner([Self::HI, Self::LO, Self::HI])
    }
    pub fn top_front_left(&self) -> Point<T> {
        self.corner([Self::LO, Self::LO, Self::HI])
    }
    pub fn bot_back_right(&self) -> Point<T> {
        self.corner([Self::HI, Self::HI, Self::LO])
    }
    pub fn bot_back_left(&self) -> Point<T> {
        self.corner([Self::LO, Self::HI, Self::LO])
    }
    pub fn bot_front_right(&self) -> Point<T> {
        self.corner([Self::HI, Self::LO, Self::LO])
    }
    pub fn bot_front_left(&self) -> Point<T> {
        self.corner([Self::LO, Self::LO, Self::LO])
    }

    fn get_part(&self, axis: Axis, hi: bool) -> T {
        use Axis::*;
        match (axis, hi) {
            (X, Self::LO) => &self.x_lo,
            (X, Self::HI) => &self.x_hi,
            (Y, Self::LO) => &self.y_lo,
            (Y, Self::HI) => &self.y_hi,
            (Z, Self::LO) => &self.z_lo,
            (Z, Self::HI) => &self.z_hi,
        }
        .clone()
    }

    // todo: nicer api? `enum EdgeAxisInfo { start, end, iterate }; Triple<_>`
    pub fn edge(
        &self,
        axis: Axis,
        two: bool, // lo (false) or high (true)
        three: bool,
    ) -> Line<T> {
        use {Axis::*, FixedOrRangeOwned::*};
        let ((lo, hi), b, c) = match axis {
            X => ((&self.x_lo, &self.x_hi), Y, Z),
            Y => ((&self.y_lo, &self.y_hi), X, Z),
            Z => ((&self.z_lo, &self.z_hi), X, Y),
        };
        let point = |axis, hi| Fixed(self.get_part(axis, hi));

        let mut parts = Triple::<Option<_>>::default();
        parts[axis] = Some(Range { lo: lo.clone(), hi: hi.clone() });
        parts[b] = Some(point(b, two));
        parts[c] = Some(point(c, three));

        let parts: [_; 3] = parts.into();
        let parts = parts.map(|p| p.unwrap()).into();

        Line::from_axis_parts(parts).unwrap()
    }

    // 12 edges
    pub fn top_front_edge(&self) -> Line<T> {
        self.edge(Axis::X, Self::LO, Self::HI)
    }
    pub fn top_back_edge(&self) -> Line<T> {
        self.edge(Axis::X, Self::HI, Self::HI)
    }
    pub fn top_left_edge(&self) -> Line<T> {
        self.edge(Axis::Y, Self::LO, Self::HI)
    }
    pub fn top_right_edge(&self) -> Line<T> {
        self.edge(Axis::Y, Self::HI, Self::HI)
    }
    pub fn bot_front_edge(&self) -> Line<T> {
        self.edge(Axis::X, Self::LO, Self::LO)
    }
    pub fn bot_back_edge(&self) -> Line<T> {
        self.edge(Axis::X, Self::HI, Self::LO)
    }
    pub fn bot_left_edge(&self) -> Line<T> {
        self.edge(Axis::Y, Self::LO, Self::LO)
    }
    pub fn bot_right_edge(&self) -> Line<T> {
        self.edge(Axis::Y, Self::HI, Self::LO)
    }
    pub fn vert_front_left_edge(&self) -> Line<T> {
        self.edge(Axis::Z, Self::LO, Self::LO)
    }
    pub fn vert_front_right_edge(&self) -> Line<T> {
        self.edge(Axis::Z, Self::HI, Self::LO)
    }
    pub fn vert_back_left_edge(&self) -> Line<T> {
        self.edge(Axis::Z, Self::LO, Self::HI)
    }
    pub fn vert_back_right_edge(&self) -> Line<T> {
        self.edge(Axis::Z, Self::HI, Self::HI)
    }

    /*
    pub fn edges(&self) -> impl Iterator<Item = Triple<T>> {
        todo!()
    }
    */

    // triple iter, contained
    // todo

    /////////////////////////////////////////

    // face iters:
    //
    // assuming x is lateral (left-right), y is "depth" (front-back), z is height (top-bot)

    pub fn face(
        &self,
        fixed_axis: Axis,
        high: bool, // lo if false, hi if true
    ) -> Plane<T> {
        use FixedOrRangeOwned::*;
        let mut parts = self.as_axis_parts_owned();
        parts[fixed_axis] = Fixed(self.get_part(fixed_axis, high));

        Plane::from_axis_parts(parts).unwrap()
    }

    // 6 faces

    /// z high
    pub fn top_face(&self) -> Plane<T> {
        self.face(Axis::Z, Self::HI)
    }
    /// z low
    pub fn bot_face(&self) -> Plane<T> {
        self.face(Axis::Z, Self::LO)
    }
    /// y low
    pub fn front_face(&self) -> Plane<T> {
        self.face(Axis::Y, Self::HI)
    }
    /// y high
    pub fn back_face(&self) -> Plane<T> {
        self.face(Axis::Y, Self::LO)
    }
    /// x low
    pub fn left_face(&self) -> Plane<T> {
        self.face(Axis::X, Self::HI)
    }
    /// x high
    pub fn right_face(&self) -> Plane<T> {
        self.face(Axis::X, Self::LO)
    }
}

// TODO: proper plane iterator?
// TODO: line iterator?
// TODO: point iterator?

impl<T> Cube<T>
where
    T: Clone + Add<T, Output = T> + Sub<T, Output = T> + Ord + One,
    LineIter<T>: Iterator<Item = Triple<T>>,
{
    // note: wasteful... (especially it `T: !Copy`)
    pub fn planes(
        &self,
        axis: Axis,
    ) -> impl Iterator<Item = Plane<T>> + DoubleEndedIterator + FusedIterator {
        use FixedOrRangeOwned::Fixed;

        // we'll iterate over the given axis
        let mut parts = self.as_axis_parts_owned();
        let outer_line = {
            let mut parts = parts.clone();
            // freeze the "other" two axes:
            for a in Axis::ALL {
                if a == axis {
                    continue;
                }
                parts[a] = Fixed(One::one());
            }

            Line::from_axis_parts(parts).unwrap().into_iter()
        };

        // for each point on the outer line, yield a plane:
        outer_line.map(move |triple| {
            // replace the given axis with the fixed value at this point in the
            // iteration over that axis:
            parts[axis] = Fixed(triple[axis].clone());

            // the remaining dimensions are still ranges; now make a plane:
            Plane::from_axis_parts(parts.clone()).unwrap()
        })
    }
}

//////////////////

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CubeFromPairOfTriples<
    const SEP: char = '~',
    const INCLUSIVE_UPPER_BOUND: bool = true,
    T: FromStr + Ord = isize,
>(Cube<T>);

impl<const S: char, const I: bool, T: FromStr + Ord> CubeFromPairOfTriples<S, I, T> {
    pub fn get(self) -> Cube<T> {
        let Self(cube) = self;
        // in order and non-empty:
        debug_assert!(cube.x_lo < cube.x_hi);
        debug_assert!(cube.y_lo < cube.y_hi);
        debug_assert!(cube.z_lo < cube.z_hi);

        cube
    }
}

// non-inclusive upper bound
impl<const SEP: char, T> FromStr for CubeFromPairOfTriples<SEP, false, T>
where
    T: FromStr + Ord + Clone,
    <T as FromStr>::Err: Debug,
{
    type Err = (); // TODO

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (first, second) = s.split_once(SEP).unwrap();
        let [first, second] = [first, second].map(|s| s.parse::<Triple<T>>().unwrap());

        let Triple(x1, y1, z1) = first;
        let Triple(x2, y2, z2) = second;

        // want `std::cmp::minmax` but it's not stable...
        let min_max = |a, b| {
            if a < b {
                (a, b)
            } else {
                (b, a)
            }
        };

        let (x_lo, x_hi) = min_max(x1, x2);
        let (y_lo, y_hi) = min_max(y1, y2);
        let (z_lo, z_hi) = min_max(z1, z2);

        Ok(Self(Cube { x_lo, x_hi, y_lo, y_hi, z_lo, z_hi }))
    }
}

// inclusive upper bound; need to increment by 1 to be in "non-inclusive" form
impl<const SEP: char, T> FromStr for CubeFromPairOfTriples<SEP, true, T>
where
    T: FromStr + Ord + Clone,
    <T as FromStr>::Err: Debug,
    T: One,
    T: Add<T, Output = T>,
{
    type Err = (); // TODO

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let CubeFromPairOfTriples(mut cube) = CubeFromPairOfTriples::<SEP, false, T>::from_str(s)?;

        cube.x_hi = cube.x_hi + One::one();
        cube.y_hi = cube.y_hi + One::one();
        cube.z_hi = cube.z_hi + One::one();

        Ok(Self(cube))
    }
}

#[test]
fn test_cube_parse() {
    let cube = CubeFromPairOfTriples::<'~', true, isize>::from_str("2,2,2~2,2,2")
        .unwrap()
        .get();
    assert_eq!(cube, Cube { x_lo: 2, x_hi: 3, y_lo: 2, y_hi: 3, z_lo: 2, z_hi: 3 });
}

////////////////////////////////////////////////////////////////////////////////
