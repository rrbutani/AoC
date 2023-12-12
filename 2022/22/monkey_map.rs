// #![feature(const_eval_limit)]
// #![const_eval_limit = "1000000"]

use std::ops::{Add, AddAssign, Div, Index, IndexMut, Mul, Not, Rem, RemAssign};

use derive_more as d;

use aoc::*;
use bimap::BiBTreeMap;
use owo_colors::OwoColorize;

#[rustfmt::skip]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default)]
#[derive(d::Add, d::AddAssign, d::Sub, d::SubAssign)]
pub struct Coord<Inner = usize> {
    x: Inner,
    y: Inner,
}

impl<T: Mul + Clone> Mul<T> for Coord<T> {
    type Output = Coord<<T as Mul<T>>::Output>;

    fn mul(self, rhs: T) -> Self::Output {
        Coord {
            x: self.x * rhs.clone(),
            y: self.y * rhs,
        }
    }
}

impl Coord<usize> {
    fn checked_add_signed(&self, offs: Coord<isize>) -> Option<Self> {
        let Coord { x, y } = self;
        let Coord { x: dx, y: dy } = offs;
        x.checked_add_signed(dx)
            .zip(y.checked_add_signed(dy))
            .map(|(x, y)| Coord { x, y })
    }
}

impl<T: Clone + Div<T, Output = T> + Mul<T, Output = T>> Coord<T> {
    fn snap_to(&self, grid_cell_len: T) -> Self {
        Self {
            x: (self.x.clone() / grid_cell_len.clone()) * grid_cell_len.clone(),
            y: (self.y.clone() / grid_cell_len.clone()) * grid_cell_len,
        }
    }
}

impl<T> From<(T, T)> for Coord<T> {
    fn from((x, y): (T, T)) -> Self {
        Self { x, y }
    }
}

impl<T> From<Coord<T>> for (T, T) {
    fn from(Coord { x, y }: Coord<T>) -> Self {
        (x, y)
    }
}

impl<T: Clone> Coord<T> {
    fn iter(
        x: impl Iterator<Item = T> + Clone,
        y: impl Iterator<Item = T> + Clone,
    ) -> impl Iterator<Item = Coord<T>> + Clone {
        y.cartesian_product(x).map(|(y, x)| Coord {
            y: y.clone(),
            x: x.clone(),
        })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default)]
pub enum Facing {
    #[default]
    Right = 0,
    Down = 1,
    Left = 2,
    Up = 3,
}

#[allow(bad_style)]
impl Facing {
    pub const Top: Self = Facing::Up;
    pub const Bottom: Self = Facing::Down;
}

impl Facing {
    pub const ALL: &[Facing] = {
        use Facing::*;
        &[Right, Down, Left, Up]
    };
}

impl Facing {
    // more like a 180˚ rotate..
    pub const fn flip(self) -> Facing {
        /*
        use Facing::*;
        match self {
            Right => Left,
            Down => Upp,
            Left => Right,
            Up => Down,
        }
        */
        Self::ALL[(self as usize + 2) % Self::ALL.len()]
    }

    const fn lateral_flip(self) -> Facing {
        use Facing::*;
        match self {
            Right => Left,
            Left => Right,
            x => x,
        }
    }

    pub const fn as_rotation(self) -> Rotate {
        use Facing::*;
        use Rotate::*;
        match self {
            Right => Clockwise,
            Down => Double,
            Left => CounterClockwise,
            Up => None,
        }
    }
}

impl Not for Facing {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.flip()
    }
}

impl Facing {
    pub const fn turn(self, turn: Turn) -> Self {
        let idx = ((self as usize) + Self::ALL.len()) as isize;
        let offs = match turn {
            Turn::Right => 1,
            Turn::Left => -1,
        };
        Self::ALL[((idx + offs) as usize) % Self::ALL.len()]
    }
}

impl Facing {
    pub const fn as_offs(self) -> Coord<isize> {
        use Facing::*;
        let (x, y) = match self {
            Right => (1, 0),
            Down => (0, 1),
            Left => (-1, 0),
            Up => (0, -1),
        };

        Coord { x, y }
    }
}

impl Display for Facing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Facing::*;
        if !f.alternate() {
            write!(
                f,
                "{}",
                match self {
                    Right => '>',
                    Down => 'v',
                    Left => '<',
                    Up => '^',
                }
            )
        } else {
            match self {
                Right => write!(f, "{}", '>'.red()),
                Down => write!(f, "{}", 'v'.yellow()),
                Left => write!(f, "{}", '<'.blue()),
                Up => write!(f, "{}", '^'.purple()),
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum Direction {
    Forward(usize),
    Turn(Turn),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum Turn {
    Right,
    Left,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
struct Directions(Vec<Direction>);

impl FromStr for Directions {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let mut directions = Vec::with_capacity(s.len());
        let mut it = s.chars().peekable();
        while let Some(c) = it.next() {
            let next = match c {
                'L' => Direction::Turn(Turn::Left),
                'R' => Direction::Turn(Turn::Right),
                n if c.is_numeric() => {
                    let mut num: usize = ((n as u8) - b'0').into();
                    while it.peek().filter(|c| c.is_numeric()).is_some() {
                        let c = it.next().unwrap() as u8;
                        num *= 10;
                        num += (c - b'0') as usize;
                    }

                    Direction::Forward(num)
                }
                _ => panic!("unexpected: {c}"),
            };
            directions.push(next);
        }

        Ok(Self(directions))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default)]
enum Cell {
    #[default]
    Empty,
    Wall,
    Open,
}

impl Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Cell::*;
        match self {
            Empty => write!(f, " "),
            Wall => write!(f, "{}", '#'.italic()),
            Open => write!(f, "{}", '.'.dimmed()),
        }
    }
}

impl TryFrom<char> for Cell {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use Cell::*;
        Ok(match value {
            ' ' => Empty,
            '#' => Wall,
            '.' => Open,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
enum Either<A, B> {
    Left(A),
    Right(B),
}

#[allow(unused)]
impl<A, B> Either<A, B> {
    fn map_left<C>(self, func: impl FnOnce(A) -> C) -> Either<C, B> {
        use Either::*;
        match self {
            Left(l) => Left(func(l)),
            Right(r) => Right(r),
        }
    }

    fn map_right<C>(self, func: impl FnOnce(B) -> C) -> Either<A, C> {
        use Either::*;
        match self {
            Left(l) => Left(l),
            Right(r) => Right(func(r)),
        }
    }
}

impl<A: Display, B: Display> Display for Either<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Either::*;
        match self {
            Left(l) => l.fmt(f),
            Right(r) => r.fmt(f),
        }
    }
}

/*

**Rotations and Flips**

We have these operations:
  - top-bottom flip (aka lateral flip or just _flip_)
  - vertical flip (aka edge flip or _spin_)
  - rotates: CW (+90), CCW (+270 or -90 -- equiv), None, 180 (aka double)

The algebra of rotates is simple: just % 360˚.

How these rotates interact with flips and spins is not obvious to me, however.

┏┷━━━━━┓  |  ┏━━━━━━┓   ┏━━━━━━┓   ┏━━━━━━┓ ┏┷━━━━━┓
┃      ┃  |  ┃  CW  ┠─  ┃Double┃   ┃ CCW  ┃ ┃ None ┃
┃      ┃  |  ┃      ┃   ┃      ┃  ─┨      ┃ ┃      ┃
┗━━━━━━┛  |  ┗━━━━━━┛   ┗━━━━━┯┛   ┗━━━━━━┛ ┗━━━━━━┛

          |  ┏━━━━━┷┓
          |  ┃ Spin ┃
          |  ┃      ┃
          |  ┗━━━━━━┛

          |  ┏━━━━━━┓
          |  ┃ Flip ┃
          |  ┃      ┃
          |  ┗┯━━━━━┛

S + Double = F
Double + S = F
S - Double = F

in other words, a flip is just a spin + double rotate (or: a spin is just a
flip + double rotate)

CW + Spin

CW + Flip + CW = Flip


going about this another way, here are all the different arrangements of the
grid:

      (1)      (2)        (3)         (4)

   ┏┷━━━━━┓  ┏━━━━━━┓   ┏━━━━━━┓   ┏━━━━━━┓
   ┃      ┃  ┃  CW  ┠─  ┃Double┃   ┃ CCW  ┃
   ┃      ┃  ┃      ┃   ┃      ┃  ─┨      ┃
   ┗━━━━━━┛  ┗━━━━━━┛   ┗━━━━━┯┛   ┗━━━━━━┛

   spin      flip       spin       flip
   ↓          ↓         ↓          ↓

   ┏━━━━━┷┓  ┏━━━━━━┓   ┏━━━━━━┓   ┏━━━━━━┓
   ┃ Spin ┃  ┃ Spin ┃   ┃ Spin ┃  ─┨ Spin ┃
   ┃      ┃  ┃  Cw  ┠─  ┃Double┃   ┃ CCW  ┃
   ┗━━━━━━┛  ┗━━━━━━┛   ┗┯━━━━━┛   ┗━━━━━━┛

      (5)      (6)        (7)         (8)

None + S = S
CW + F = S + CW  → CW + S + Double = S + CW ..... this tells us that addition is
  not a good model for this because we're not actually commutative

put another way, X + S does not always equal S + X:
  - N   + S = S +   N ✅
  - CW  + S = S +  CW ❌ #8 vs #6
  - D   + S = S +   D ✅
  - CCW + S = S + CCW ❌ #6 vs #8

*/

/// ```ignore
/// ┏┷━━━━━┓  |  ┏━━━━━━┓   ┏━━━━━━┓   ┏━━━━━━┓ ┏┷━━━━━┓
/// ┃      ┃  |  ┃  CW  ┠─  ┃Double┃   ┃ CCW  ┃ ┃ None ┃
/// ┃      ┃  |  ┃      ┃   ┃      ┃  ─┨      ┃ ┃      ┃
/// ┗━━━━━━┛  |  ┗━━━━━━┛   ┗━━━━━┯┛   ┗━━━━━━┛ ┗━━━━━━┛
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Copy, Clone)]
pub enum Rotate {
    /// ```ignore
    /// ┏┷━━━━━┓
    /// ┃ None ┃
    /// ┃      ┃
    /// ┗━━━━━━┛
    /// ```
    #[default]
    None = 0,
    /// ```ignore
    /// ┏━━━━━━┓
    /// ┃  CW  ┠─
    /// ┃      ┃
    /// ┗━━━━━━┛
    /// ```
    Clockwise = 1,
    /// ```ignore
    /// ┏━━━━━━┓
    /// ┃Double┃
    /// ┃      ┃
    /// ┗━━━━━┯┛
    /// ```
    Double = 2,
    /// ```ignore
    ///  ┏━━━━━━┓
    ///  ┃ CCW  ┃
    /// ─┨      ┃
    ///  ┗━━━━━━┛
    /// ```
    CounterClockwise = 3,
}

impl Rotate {
    pub const ALL: [Self; Self::CounterClockwise as usize + 1] = {
        use Rotate::*;
        [None, Clockwise, Double, CounterClockwise]
    };
    pub const LEN: usize = Self::ALL.len();

    const fn by_idx(i: usize) -> Self {
        Self::ALL[i % Self::ALL.len()]
    }
}

impl Rotate {
    pub const fn invert(self) -> Self {
        Self::by_idx(Self::LEN - (self as usize))
    }

    pub const fn plus(self, rhs: Self) -> Self {
        Self::by_idx(self as usize + rhs as usize)
    }
}

impl Add for Rotate {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.plus(rhs)
    }
}

impl AddAssign for Rotate {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Not for Rotate {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.invert()
    }
}

/// ```ignore
///       (1)      (2)        (3)         (4)
///
///   ┏┷━━━━━┓  ┏━━━━━━┓   ┏━━━━━━┓   ┏━━━━━━┓
///   ┃      ┃  ┃  CW  ┠─  ┃Double┃   ┃ CCW  ┃
///   ┃      ┃  ┃      ┃   ┃      ┃  ─┨      ┃
///   ┗━━━━━━┛  ┗━━━━━━┛   ┗━━━━━┯┛   ┗━━━━━━┛
///
///   spin      flip       spin       flip
///   ↓          ↓         ↓          ↓
///
///   ┏━━━━━┷┓  ┏━━━━━━┓   ┏━━━━━━┓   ┏━━━━━━┓
///   ┃ Spin ┃  ┃ Spin ┃   ┃ Spin ┃  ─┨ Spin ┃
///   ┃      ┃  ┃  Cw  ┠─  ┃Double┃   ┃ CCW  ┃
///   ┗━━━━━━┛  ┗━━━━━━┛   ┗┯━━━━━┛   ┗━━━━━━┛
///
///      (5)      (6)        (7)         (8)
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Copy, Clone)]
pub struct Transformation {
    spin: bool, // canonicalized as the spin being applied first..
    rotate: Rotate,
}

impl Transformation {
    fn all() -> impl Iterator<Item = Self> {
        [true, false]
            .into_iter()
            .cartesian_product(Rotate::ALL)
            .map(|(spin, rotate)| Transformation { spin, rotate })
    }
}

impl Transformation {
    pub const fn rotate(self, rot: Rotate) -> Self {
        let mut out = self;
        out.rotate = out.rotate.plus(rot);
        out
    }

    pub const fn spin(self) -> Self {
        Transformation {
            spin: !self.spin,
            rotate: self.rotate.invert(),
        }
    }

    pub const fn flip(self) -> Self {
        self.spin().rotate(Rotate::Double)
    }

    pub const fn combine(self, rhs: Self) -> Self {
        let mut out = self;
        if rhs.spin {
            out = out.spin();
        }
        out.rotate(rhs.rotate)
    }

    pub const fn invert(self) -> Self {
        // anything with a spin in it is actually its own inverse..
        let mut out = self;
        if !self.spin {
            out.rotate = out.rotate.invert();
        }
        out

        // this works too, I think:
        /*
        // reverse the order of operations (rotate, then spin) since we're
        // inverting:
        let mut out = Default::default();
        out = out.rotate(self.rotate.invert())
        if self.spin { out = out.spin(); }
        out
        */
    }
}

// using Rem instead of Add to indicate that this is *not* commutative..
impl Rem for Transformation {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        self.combine(rhs)
    }
}

impl RemAssign for Transformation {
    fn rem_assign(&mut self, rhs: Self) {
        *self = *self % rhs;
    }
}

impl Not for Transformation {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.invert()
    }
}

impl Transformation {
    /// Given the side this transformation info is supposed to start at, returns
    /// the side that applying this transformation info will take you to.
    pub const fn get_dest_side(&self, starting_side: Facing) -> Facing {
        // The spin will "invert" the side; if left or right, the side will
        // flip:
        let offset = if self.spin {
            // Facing::ALL.len() - starting_side as usize
            starting_side.lateral_flip() as usize
        } else {
            starting_side as usize
        };

        // Rotate applies as normal:
        let offset = offset + self.rotate as usize;

        Facing::ALL[offset % Facing::ALL.len()]
    }
}

impl Transformation {
    pub const NONE: Self = Transformation {
        spin: false,
        rotate: Rotate::None,
    };
    pub const CW: Self = Transformation {
        spin: false,
        rotate: Rotate::Clockwise,
    };
    pub const DOUBLE: Self = Transformation {
        spin: false,
        rotate: Rotate::Double,
    };
    pub const CCW: Self = Transformation {
        spin: false,
        rotate: Rotate::CounterClockwise,
    };

    pub const SPIN_NONE: Self = Transformation {
        spin: true,
        rotate: Rotate::None,
    };
    pub const SPIN_CW: Self = Transformation {
        spin: true,
        rotate: Rotate::Clockwise,
    };
    pub const SPIN_DOUBLE: Self = Transformation {
        spin: true,
        rotate: Rotate::Double,
    };
    pub const SPIN_CCW: Self = Transformation {
        spin: true,
        rotate: Rotate::CounterClockwise,
    };

    pub const FLIP: Self = Self::NONE.flip();
    pub const SPIN: Self = Self::NONE.spin();
}

#[cfg(test)]
mod test_transformation {
    use super::*;

    #[test]
    fn invert_identity_test() {
        for a in Transformation::all() {
            for b in Transformation::all() {
                let b_ = b.invert();
                let res = a % b_;
                if a == b {
                    assert_eq!(res, Transformation::default());
                } else {
                    assert_ne!(res, Transformation::default());
                }
            }
        }
    }

    #[test]
    fn rotate_identity_test() {
        for a in Transformation::all() {
            use Rotate::*;

            assert_eq!(a, a.rotate(None));

            // 90˚
            let a_ = a.rotate(Clockwise);
            assert_ne!(a, a_);

            // 180˚
            let a_ = a_.rotate(Clockwise);
            assert_ne!(a, a_);
            assert_eq!(a.rotate(Double), a_);
            assert_eq!(a.rotate(CounterClockwise).rotate(CounterClockwise), a_);

            // 270˚
            let a_ = a_.rotate(Clockwise);
            assert_ne!(a, a_);
            assert_eq!(a.rotate(CounterClockwise), a_);

            let a_ = a_.rotate(Clockwise);
            assert_eq!(a, a_);
        }
    }

    #[test]
    fn spin_identity_test() {
        for a in Transformation::all() {
            assert_ne!(a, a.spin());
            assert_eq!(a, a.spin().spin());
        }
    }

    #[test]
    fn flip_identity_test() {
        for a in Transformation::all() {
            assert_ne!(a, a.flip());
            assert_eq!(a, a.flip().flip());
        }
    }

    #[test]
    fn dest_side() {
        use Facing::*;
        use Transformation::{self as T};
        // use Transformation::{CCW, CW, DOUBLE, NONE, SPIN_CCW, SPIN_CW, SPIN_DOUBLE, SPIN_NONE};

        let t = |f: Facing, t: T, res: Facing| {
            assert_eq!(
                res,
                t.get_dest_side(f),
                "expected applying transformation {t:?} to {f:?} to yield {res:?}"
            );
        };

        t(Up, T::NONE, Up);
        t(Up, T::CW, Right);
        t(Up, T::DOUBLE, Down);
        t(Up, T::CCW, Left);
        t(Up, T::SPIN_NONE, Up);
        t(Up, T::SPIN_CW, Right);
        t(Up, T::SPIN_DOUBLE, Down);
        t(Up, T::SPIN_CCW, Left);

        t(Right, T::NONE, Right);
        t(Right, T::CW, Down);
        t(Right, T::DOUBLE, Left);
        t(Right, T::CCW, Up);
        t(Right, T::SPIN_NONE, Left);
        t(Right, T::SPIN_CW, Up);
        t(Right, T::SPIN_DOUBLE, Right);
        t(Right, T::SPIN_CCW, Down);

        t(Down, T::NONE, Down);
        t(Down, T::CW, Left);
        t(Down, T::DOUBLE, Up);
        t(Down, T::CCW, Right);
        t(Down, T::SPIN_NONE, Down);
        t(Down, T::SPIN_CW, Left);
        t(Down, T::SPIN_DOUBLE, Up);
        t(Down, T::SPIN_CCW, Right);

        t(Left, T::NONE, Left);
        t(Left, T::CW, Up);
        t(Left, T::DOUBLE, Right);
        t(Left, T::CCW, Down);
        t(Left, T::SPIN_NONE, Right);
        t(Left, T::SPIN_CW, Down);
        t(Left, T::SPIN_DOUBLE, Left);
        t(Left, T::SPIN_CCW, Up);
    }
}

/*

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default)]
enum Rotate {
    // the 4th possibility (connected on the same side) is only possible when
    // we're doing an orthogonal flip so it's represented as `(rotate = None,
    // flip = true)`
    //
    // update: nevermind!
    /// `None, edge_flip = true` => possible
    /// `None, edge_flip = false` => impossible!
    None = 0,

    /// `Clockwise, edge_flip = true` => possible
    /// `Clockwise, edge_flip = false` => possible
    Clockwise = 1,

    #[default]
    /// i.e. a flip that's parallel to the edge being translated and not
    /// _across_ it (i.e. that would be the other flip, an _edge_ flip).
    ///
    /// really we probably shouldn't be representing this as a rotation since
    /// it doesn't involve movement on the same two axes as [`Clockwise`] and
    /// [`CounterClockwise`]
    ///
    /// `Flip, edge_flip = true` => possible
    ///   + this is also possible! it's the translation you need to go from 1 to
    ///     6 in a net that looks like this:
    ///     ```text
    ///              ┏━━━┓
    ///              ┃ 1 ┃
    ///          ┏━━━╋━━━╋━━━┓
    ///          ┃ 2 ┃ 3 ┃ 4 ┃
    ///          ┗━━━╋━━━╋━━━┛
    ///              ┃ 5 ┃
    ///              ┣━━━┫
    ///              ┃ 6 ┃
    ///              ┗━━━┛
    ///     ```
    ///
    /// `Flip, edge_flip = false` => possible
    ///   + this is what we were calling "direct"; things that are already
    ///     connected in the 2D grid
    Flip = 2,

    /// `CounterClockwise, edge_flip = true` => possible
    /// `CounterClockwise, edge_flip = false` => possible
    CounterClockwise = 3,
    // NOTE: edge_flip + Flip is equiv to double rotate!
    // ergo
    // flip = double rotate + edge flip
    // edge_flip = double rotate + flip

    // starting to suspect there's an easier way to represent this that has to
    // do with (x, y, z) coords/yaw, pitch, roll rotations...
    //
    // one of these three doesn't affect the 2D face grids which is part of the
    // reason why we have this weird subset of states (7?)
}

*/

/*
impl Rotate {
    const ALL: [Rotate; Rotate::CounterClockwise as usize + 1] = {
        use Rotate::*;
        [None, Clockwise, Flip, CounterClockwise]
    };

    fn collapse(self, other: Rotate) -> Self {
        // use Rotate::*;
        // match (self, other) {
        //     (a, None) => a, // x + 0 = x
        //     (None, b) => b, // 0 + x = x
        //     (Clockwise, Clockwise) => Flip, // 1 + 1 = 2
        //     (Clockwise, Flip) => CounterClockwise, // 1 + 2 = 3
        //     (Clockwise, CounterClockwise) => None, // 1 + 3 = 0
        //     (Flip, Clockwise) => CounterClockwise, // 2 + 1 = 3
        //     (Flip, Flip) => None, // 2 + 2 = 0
        //     (Flip, CounterClockwise) => todo!(),
        //     (CounterClockwise, Clockwise) => todo!(),
        //     (CounterClockwise, Flip) => todo!(),
        //     (CounterClockwise, CounterClockwise) => todo!(),
        // }

        Self::ALL[(self as usize + (other as usize)) % Self::ALL.len()]
    }

    fn invert(self) -> Self {
        Self::ALL[(self as usize + 2) % Self::ALL.len()]
    }
} */
/*
impl Neg for Rotate {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.invert()
    }
}

impl Add for Rotate {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.collapse(rhs)
    }
} */

/* #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
enum TranslateKind {
    Direct,
    RotateRight,
    RotateLeft,
    Flip,
} */

/*
/// Information about how to translate a cube face edge to another cube face
/// edge, in 2D (where we assume that both edges correspond to the same edge in
/// the 3D representation of the cube).
*/

/// Information about how to transform the coordinates of an edge of a face to
/// map to the coordinates of a corresponding edge (i.e. the same edge in 3D
/// space) on another face.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct TransformationInfo {
    to: usize, // face idx -- TODO: can only be in 0..6

    // /// How we have to rotate the _source_ face to get to the face corresponding
    // /// to `to`.
    // rot: Rotate,
    // edge_flip: bool, // flip _orthogonal_; i.e. flips the edge that's being translated
    /// The transformation we need to apply to get from the source edge to the
    /// edge in `to`.
    trans: Transformation,
}

/*
impl TransformationInfo {
    /// Given the side this transformation info is supposed to start at, returns
    /// the side that applying this transformation info will take you to.
    fn get_dest_side(&self, starting_side: Facing) -> Facing {
        use Rotate::*;
        // let offset = match self.rot {
        //     // no rotation means we've got a direct connection which means we're
        //     // connected to the *opposite* side of the other face
        //     //
        //     // this is a 180˚ rotation; i.e. +2
        //     None => 2,
        //     // if we're rotating clockwise, that's another 90˚ meaning 270˚;
        //     // i.e. +3
        //     Clockwise => 3,
        //     // if we're rotating _counterclockwise_, that's another 180˚ meaning
        //     // 450˚ or +5... which wraps to +1 after a %4
        //     //
        //     // alternatively: CCW is -90˚ which, when added to the base 180˚
        //     // rotate yields 90˚ which is +1
        //     CounterClockwise => 1,
        // };

        /* let offset = match self.rot {
            // no rotation means we're connected to the same side
            None => 0,
            // clockwise means we've got to go 90˚ + 180˚ to invert:
            Clockwise => 3,
            // flip means we're dealing with the opposite side:
            Flip => 2,
            // counterclockwise means 270˚ + 180˚ to invert; 3 + 2 % 4 is just
            // 1:
            CounterClockwise => 1,
        }; */
        // we can represent the above more concisely as:
        // let offset = 4 - self.rot as usize;

        // // if we're doing an edge flip that's another 180˚:
        // let offset = if self.edge_flip {
        //     (offset + 2) % Facing::ALL.len()
        // } else {
        //     offset
        // };

        let offset = if self.edge_flip {
            (self.rot as usize) + 2
        } else {
            4 - self.rot as usize
        };

        Facing::ALL[(starting_side as usize + offset) % Facing::ALL.len()]
    }

    /// Returns the translation info we expect the opposite translation to have.
    ///
    /// (i.e. assuming `self` is the translation info to go from `idx ->
    /// self.to`, this function will return the translation info that's needed
    /// to go from `self.to -> idx`)
    fn get_dest_trans_info_for(&self, idx: usize) -> Self {
        let mut out = *self;
        out.to = idx;

        // just flip the rotation:
        use Rotate::*;
        out.rot = match out.rot {
            Clockwise => CounterClockwise,
            CounterClockwise => Clockwise,
            x => x,
        };
        // equivalent to `rot = 4 - out.rot`

        out
    }
}
*/

impl TransformationInfo {
    /// Returns the translation info we expect the opposite translation to have.
    ///
    /// (i.e. assuming `self` is the translation info to go from `idx ->
    /// self.to`, this function will return the translation info that's needed
    /// to go from `self.to -> idx`)
    fn get_dest_trans_info_for(&self, idx: usize) -> Self {
        let mut out = *self;
        out.to = idx;

        out.trans = out.trans.invert();

        out
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
struct Map {
    grid: Vec<Vec<Cell>>,                // row, col
    vis: Vec<Vec<Either<Cell, Facing>>>, // row, col
    dim: Coord,                          // x, y (cols, rows)
    pos: Coord,                          // x, y (col, row)
    dir: Facing,
    edge_len: usize,
    face_idxs: BiBTreeMap<Coord, usize>, // top left coord => face idx
    // cube_wrapping: [[TranslationInfo; 4]; 6],
    cube_face_mapping: [[TransformationInfo; 4]; 6], // face idx => translation info for the 4 directions (Right, Down, Left, Up)
}

impl Index<Coord> for Map {
    type Output = Cell;

    fn index(&self, Coord { x, y }: Coord) -> &Self::Output {
        &self.grid[y][x]
    }
}

impl IndexMut<Coord> for Map {
    fn index_mut(&mut self, Coord { x, y }: Coord) -> &mut Self::Output {
        &mut self.grid[y][x]
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (y, row) in self.grid.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if (Coord { x, y }) == self.pos {
                    write!(f, "{}", self.dir.green())
                } else if f.alternate() {
                    write!(f, "{:#}", self.vis[y][x])
                } else {
                    write!(f, "{cell}")
                }?
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

/// `face_idx` is a 1:1 mapping from the (2D) top-left coordinate of a
/// particular face to the index we're referring to the face by.
///
/// `edge_len` is the length of the cube's edges; i.e. each 2D face is expected
/// to have `edge_len` columns and `edge_len` rows.
///
/// Returns an array containing the four adjacent faces (and the translation
/// info to get to the adjacent face) for each of the six faces. Index `i` of
/// this array corresponds to the face described by `face_idx.get_by_right(i)`.
fn construct_face_mapping(
    face_idxs: &BiBTreeMap<Coord, usize>,
    edge_len: usize,
) -> [[TransformationInfo; 4]; 6] {
    assert_eq!(face_idxs.len(), 6);

    // We're constructing the full matrix of face <-> face edges.
    //
    // There are 12 edges in a cube but we have 24 entries in this array because
    // we have two entries per edge (i.e. `A <-> B`) so that we can store
    // translation info going in each direction; i.e. once going from face A ->
    // face B, once going from face B -> face A.
    //
    // At the end of this function we assert that such the translation info in
    // each direction for a particular edge is _symmetric_.
    let mut face_mapping: [[Option<TransformationInfo>; 4]; 6] = Default::default();
    let mut connected = 0;

    // First: mark each adjacent face that's in bound + present in
    // `face_mapping`
    //
    // these are all the "directly connected" faces; i.e. faces that are already
    // connected in the input "net" and thus require a parallel flip
    use Facing::*;
    for (coord, &idx) in face_idxs.iter() {
        for f in Facing::ALL {
            let offs = f.as_offs() * (edge_len as isize);

            if let Some(pos) = coord.checked_add_signed(offs) {
                // check if this is actually present:
                if let Some(&adj_idx) = face_idxs.get_by_left(&pos) {
                    let slot = &mut face_mapping[idx][*f as usize];
                    assert_eq!(*slot, None);
                    *slot = Some(TransformationInfo {
                        to: adj_idx,
                        trans: match f {
                            Up | Down => Transformation::FLIP,
                            Left | Right => Transformation::SPIN,
                        },
                    });
                    connected += 1;
                }
            }
        }
    }
    // seems to always be true (nets with more than 5 edges cannot be folded to
    // represent a square) but not sure how to prove..
    assert_eq!(connected, 5 * 2);

    // Next: fill in missing faces by traversing the cube.
    //
    // The idea is to, for each face in the matrix that's missing, take an
    // alternate path to get to that face.
    //
    // Consider the following net:
    // ```
    //         ┏━━━┓
    //         ┃ 1 ┃
    // ┏━━━┳━━━╋━━━┫
    // ┃ 2 ┃ 3 ┃ 4 ┃
    // ┗━━━┻━━━╋━━━╋━━━┓
    //         ┃ 5 ┃ 6 ┃
    //         ┗━━━┻━━━┛
    // ```
    //
    // For the sake of this example, let's (arbitrarily) call `4` the bottom of
    // the cube, `1` the front, `3` the left, etc:
    //
    // ```
    //               /––––––––>  2
    //           ________
    //          /⠡   1  /\
    // 3 <–––– /  ⠡..../..\
    //        /___⠌___/   / –––> 6
    //        \  ⠌  4 \  /
    //         \⠌______\/
    //             \
    //             \–––––––> 5
    // ```
    //
    // Looking at the cube above, we know that the face to 1's left is `3` but
    // this isn't immediately when looking at the 2D net because there's nothing
    // to `1`'s left in the net.
    //
    // Another path to get to the face that's left of a face is to: go down and
    // then left (for `1` this means going to 4 (down) and then going left to
    // 3).
    //
    // This is useful because looking at the 2D net we can easily see `1`'s down
    // (4) and `4`'s left (3).
    //
    // Alternatively going *up* and then left also gets us to a face's left side
    // (for `1` this means going to 2 (up) and then going left to 3 –– though
    // for this particular net this isn't useful).
    //
    // This generalizes to other directions too:
    //   - going right is equivalent to: down + right, up + right
    //   - going up is equivalent to: left + up, right + up
    //   - going down is equivalent to: left + down, right + down
    //
    // More generally: on a cube, we can go any number of steps in an
    // _orthogonal_ direction and still reach our destination (i.e. to go left
    // from `1` we can go up or down (i.e. on the faces `6`, `5`, `2`, `1`) as
    // many times as we want before going left and we'll always still reach `3`
    // because all of those faces share the same left face).
    //
    // Rather than list out all the rules and the paths they correspond to (i.e.
    // `right == (down)* + right`) we can just keep applying a few rules
    // iteratively.
    //
    // It doesn't (I think) matter what rule(s) we use here and I _suspect_
    // applying any rule that has a path of length 2 or greater would eventually
    // yield a full set of 24 edges but: to keep things simple we're just going
    // to go after the corners (i.e. the rules described above).
    let mut made_progress;
    loop {
        // break; // TODO!!
        made_progress = false;

        for (_coord, &idx) in face_idxs.iter() {
            for f in Facing::ALL {
                // use Facing::*;

                let slot = &mut face_mapping[idx][*f as usize];

                // if we already know what this adjacent face is, there's
                // nothing to do; move on:
                if slot.is_some() {
                    continue;
                }

                /*
                // corners:
                //  - if left is missing, try down+left (cw) / up+left (ccw)
                //  - if right is missing, try down+right (ccw) / up+right (cw)
                //  - if up is missing, try left+up (cw) / right+up (ccw)
                //  - if down is missing, try left+down (ccw) / right (ccw)
                // etc.
                // let alts = match f {
                    //     Left | Right => [Down, Up],
                    // }
                    // panic!();

                    // add to alt's dir

                    // add all three to get new dir
                */

                let alts = match f {
                    Right => &[
                        (&[Up, Right], Rotate::Clockwise),
                        (&[Down, Right], Rotate::CounterClockwise),
                    ],
                    Down => &[
                        (&[Left, Down], Rotate::Clockwise),
                        (&[Right, Down], Rotate::CounterClockwise),
                    ],
                    Left => &[
                        (&[Down, Left], Rotate::Clockwise),
                        (&[Up, Left], Rotate::CounterClockwise),
                    ],
                    Up => &[
                        (&[Right, Up], Rotate::Clockwise),
                        (&[Left, Up], Rotate::CounterClockwise),
                    ],
                };

                'path_loop: for (path, adj_rot) in alts {
                    let mut transformation = Transformation::NONE;

                    // let mut facing = starting_side;
                    let mut face_idx = idx;

                    // eprintln!("\nstart: {starting_face}, {_starting_side:?} [{path:?}]");
                    for &step in path.iter() {
                        // eprintln!("  - step: {step:?}");
                        // adjust step for the current transformation
                        let step = transformation.get_dest_side(step);
                        // eprintln!("    * adjusted for current transformation -> {step:?}");

                        let trans = face_mapping[face_idx][step as usize];
                        let Some(trans) = trans else {
                            // eprintln!("bailing on {path:?} from {face_idx} (start: {idx}, {f:?}) at step: {step:?}");
                            continue 'path_loop; /* bail */
                        };
                        // eprintln!("  - mapping[face: {face_idx}][{step:?}] -> {trans:?}");

                        transformation = transformation.combine(trans.trans);
                        // eprintln!("  - new transformation: {trans:?}");
                        face_idx = trans.to;
                        // eprintln!("  - now @ face: {face_idx}");
                        // eprintln!("  ––––––––––––––––––––––––––––––––––––––");
                    }

                    // transformation = Transformation {
                    //     spin: !transformation.spin,
                    //     rotate: transformation.rotate.plus(Rotate::Clockwise),
                    // };
                    let Transformation { spin, rotate } = transformation;
                    let transformation = Transformation {
                        spin: !spin,
                        rotate: rotate.plus(*adj_rot),
                    };

                    let trans_info = TransformationInfo {
                        trans: transformation,
                        to: face_idx,
                    };
                    // eprintln!("success! @ {idx}, {f:?} via path {path:?}");
                    face_mapping[idx][*f as usize] = Some(trans_info);

                    made_progress = true;
                    connected += 1;
                    break;
                }
            }
        }

        /* 12 edges in a cube; x2 for bidir */
        if connected == 24 {
            break;
        } else if !made_progress {
            panic!("stuck! at {connected} connected edges");
        }
    }

    let face_mapping = face_mapping.map(|arr| arr.map(|i| i.unwrap()));

    // Finally, we want to ensure that our matrix is internally consistent.
    assert_face_mapping_internally_consistent(&face_mapping);

    face_mapping
}

fn assert_face_mapping_internally_consistent(face_mapping: &[[TransformationInfo; 4]; 6]) {
    // For an cube edge shared by faces `A` and `B` we record separate entries
    // for `A -> B` and `B -> A` and we want to make sure that these entries are
    // symmetric (or, mirrored actually).
    for (src_face, directions) in face_mapping.iter().enumerate() {
        for (side, trans_info) in directions.iter().enumerate() {
            let side = Facing::ALL[side];
            let dest_face = trans_info.to;

            let dest_side = trans_info.trans.get_dest_side(side);
            let dest_trans_info = trans_info.get_dest_trans_info_for(src_face);

            assert_eq!(
                face_mapping[dest_face][dest_side as usize],
                dest_trans_info,
                "\nface {src_face}'s {side:?} has trans info: {trans_info:#?} ––> {dest_side:?}\n\
                \nexpected {dest_face}'s {dest_side:?} to have complementary trans info: {dest_trans_info:#?}"
            );
        }
    }
}

/// Given information about a cube (edge length, face indexes, face to face
/// translation info) and a coordinate and a movement direction, returns a new
/// coordinate and a new direction.
fn translate_cube_coord(
    last_valid_coord: Coord,
    movement_dir: Facing,
    _cube_info @ (edge_len, face_idxs, translation_info): (
        usize,
        &BiBTreeMap<Coord, usize>,
        &[[TransformationInfo; Facing::ALL.len()]; 6],
    ),
) -> (Coord, Facing) {
    // First, find out what face we're in:
    let top_left = last_valid_coord.snap_to(edge_len);
    let face_idx = *face_idxs
        .get_by_left(&top_left)
        .expect("last coordinate is a cube face");

    // Next, figure out what side of the face we hit:
    // use Facing::*;
    let coord_relative_to_face = last_valid_coord - top_left;
    let side = {
        let (x, y) = coord_relative_to_face.into();
        let possible_sides = [
            /* R */ x == edge_len - 1,
            /* D */ y == edge_len - 1,
            /* L */ x == 0,
            /* U */ y == 0,
        ];

        if !possible_sides.iter().any(|&x| x) {
            panic!("{last_valid_coord:?} doesn't appear to be on the edge of the face at {top_left:?} (edge len: {edge_len})");
        }

        if !possible_sides[movement_dir as usize] {
            panic!(
                "given that our movement direction is {movement_dir:?}, we \
                expected to hit the {movement_dir:?} side of the face at \
                {top_left:?} (index: {face_idx}) but this is not possible. \
                The given coordinate {last_valid_coord:?} only touches these \
                sides: {possible_sides:?}"
            );
        }

        movement_dir
    };
    /* let side = match coord_relative_to_face.into() {
        (_, 0) => /* top edge */ Up,
        (x, _) if x == edge_len - 1 => /* right edge */ Right,
        (_, y) if y == edge_len - 1 => /* bottom edge */ Down,
        (0, _) => /* left edge */ Left,
        _ => panic!("{last_valid_coord:?} doesn't appear to be on on edge of the face at {top_left:?} (edge len: {edge_len})"),
    };
    assert_eq!(
        side, movement_dir,
        "given that we've hit the {side:?} side, expected to be moving {side:?}"
    );*/

    // Get the translation info for the side:
    let TransformationInfo { to, trans } = translation_info[face_idx][side as usize];
    let dest_top_left = face_idxs.get_by_right(&to).unwrap();

    // This is invalid; we need _at least_ a rotate or a spin.
    assert_ne!(trans, Transformation::NONE); // TODO: is this right?

    // Flip/rotate the relative coords:
    let coord_relative_to_face: Coord = {
        // let Coord {
        //     x: mut rel_x,
        //     y: mut rel_y,
        // } = coord_relative_to_face;
        // let Transformation { spin, rotate } = trans;

        // todo!();

        /*
        // Apply the spin first!
        if spin {
            match side {
                /* y = 0 or edge_len - 1; flip x */
                Up | Down => rel_x = edge_len - 1 - rel_x,
                /* x = 0 or edge_len - 1; flip y */
                Left | Right => rel_y = edge_len - 1 - rel_y,
            }
        } */

        /*
        use Rotate::*;
        match rotate {
            // if we need to rotate the face cw to get to dest that means we
            // want to rotate the coordinates ccw to align with the dest:
            Clockwise => {
                // ccw
                /*
                match side {
                    // x = edge_len - 1, y is real
                    Right => { // -> top
                        rel_x = rel_y;
                        rel_y = 0;
                    }
                    // y = edge_len - 1, x is real
                    Down => { // -> right
                        rel_y = edge_len - 1 - rel_x;
                        rel_x = edge_len - 1;
                    }
                    // x = 0, y is real
                    Left => { // -> bottom
                        rel_x = rel_y;
                        rel_y = edge_len - 1;
                    }
                    // y = 0, x is real
                    Up => { // -> left
                        rel_y = edge_len - 1 - rel_x;
                        rel_x = 0;
                    }
                }
                */

                // rel_x:
                //  - rel_y
                //  - edge_len - 1 [ y = edge_len - 1 ]
                //  - rel_y
                //  - 0 [ y = 0 ]
                //
                // thus, rel_x = rel_y

                // rel_y:
                //  - 0 [ x = edge_len - 1 ]
                //  - edge_len - 1 - rel_x
                //  - edge_len - 1 [ x = 0 ]
                //  - edge_len - 1 - rel_x
                //
                // thus rel_y = edge_len - 1 - rel_x

                // so, the above can be written more concisely as:
                let (x, y) = (rel_x, rel_y);
                rel_x = y;
                rel_y = edge_len - 1 - x;
            }
            // likewise for counterclockwise; rotate the coordinates clockwise:
            CounterClockwise => {
                // cw
                /* match side {
                    // x = edge_len - 1, y is real
                    Right => {
                        // -> bottom
                        rel_x = edge_len - 1 - rel_y;
                        rel_y = edge_len - 1;
                    }
                    // y = edge_len - 1, x is real
                    Down => {
                        // -> left
                        rel_y = rel_x;
                        rel_x = 0;
                    }
                    // x = 0, y is real
                    Left => {
                        // -> top
                        rel_x = edge_len - 1 - rel_y;
                        rel_y = 0;
                    }
                    // y = 0, x is real
                    Up => {
                        // -> right
                        rel_y = rel_x;
                        rel_x = edge_len - 1;
                    }
                } */

                // rel_x:
                //  - edge_len - 1 - rel_y
                //  - 0 [y = edge_len - 1]
                //  - edge_len - 1 - rel_y
                //  - edge_len - 1 [ y = 0 ]
                //
                // can rewrite as:
                //  - edge_len - 1 - rel_y
                //  - edge_len - 1 - rel_y
                //  - edge_len - 1 - rel_y
                //  - edge_len - 1 - rel_y

                // rel_y:
                //  - edge_len - 1 [x = edge_len - 1]
                //  - rel_x
                //  - 0 [ x = 0 ]
                //  - rel_x
                //
                // can rewrite as:
                //  - x
                //  - x
                //  - x
                //  - x

                // This can be written more concisely as:
                let (x, y) = (rel_x, rel_y);
                rel_x = edge_len - 1 - y;
                rel_y = x;
            }

            /*
            Flip => match side {
                Up | Down /* y is fixed */ => rel_y = edge_len - 1 - rel_y,
                Left | Right /* x is fixed */ => rel_x = edge_len - 1 - rel_x,
            },
            */
            None => {} // no rotation required!
            Clockwise => {
                let (x, y) = (rel_x, rel_y);
                rel_x = edge_len - 1 - y;
                rel_y = x;
            }
            CounterClockwise => {
                let (x, y) = (rel_x, rel_y);
                rel_x = y;
                rel_y = edge_len - 1 - x;
            }
            Double => {}
        }

        // x:
        //  [Non] x
        //  [Cwe] y
        //  [Flp, UD] x
        //  [Flp, LR] edge_len - 1 - x
        //  [Ccw] edge_len - 1 - y
        //
        // y:
        //  [Non] y
        //  [Cwe] edge_len - 1 - x
        //  [Flp, UD] edge_len - 1 - y
        //  [Flp, LR] y
        //  [Ccw] x

        // I believe its possible to compress the above further using ^ but I'm
        // going to stop here; more compact than this becomes hard to build
        // intuition for (imo).

        (rel_x, rel_y).into()

        */

        let Coord { x, y } = coord_relative_to_face;
        let Transformation { spin, rotate } = trans;

        // Apply the spin first!
        let (x, y) = if spin { (edge_len - 1 - x, y) } else { (x, y) };

        // And then the rotate:
        use Rotate::*;
        let (x, y) = match rotate {
            None => (x, y),
            Clockwise => (edge_len - 1 - y, x),
            Double => (edge_len - 1 - x, edge_len - 1 - y),
            CounterClockwise => (y, edge_len - 1 - x),
        };

        (x, y).into()
    };

    let new_coord = *dest_top_left + coord_relative_to_face;
    let new_facing = {
        // just: the opposite direction of the side of face we're now on:
        let dest_side = trans.get_dest_side(side);
        !dest_side
    };

    (new_coord, new_facing)
}

#[cfg(test)]
mod test_bleh {
    use super::*;

    #[test]
    #[ignore]
    fn test() {
        dbg!(EXAMPLE_TRANSLATE_INFO[3][Facing::Down as usize]);
        panic!();
    }
}

// 0R -> 5R'
//
//  == flip
//
// flip + flip + spin => spin
//
// (cw + cw = double) + flip + flip + spin => flip ✅
//
// flip + flip + spin + double => none ❌
//
// (flip + cw) + (flip + cw) + spin => flip ✅

#[allow(unused)]
const fn transformation_eq(exp: Transformation, got: Transformation) -> bool {
    if exp.rotate as usize != got.rotate as usize {
        let _got: usize = [][got.rotate as usize];
        let _expected: usize = [][exp.rotate as usize];
        panic!();
    }

    if exp.spin != got.spin {
        let _expected: usize = [][exp.spin as usize];
        let _got: usize = [][got.spin as usize];
        panic!();
    }

    true
}

static_assertions::const_assert!(transformation_eq(
    // Transformation::FLIP,
    // Transformation::FLIP
    //     .rotate(Rotate::Clockwise)
    //     .combine(Transformation::FLIP)
    //     .rotate(Rotate::Clockwise)
    //     .spin(),

    // Transformation::CCW.flip(),
    // Transformation::CCW.flip()
    //     .rotate(Rotate::Clockwise)
    //     .combine(Transformation::FLIP)
    //     .rotate(Rotate::Clockwise)
    //     .spin(),

    // Transformation::FLIP.flip().spin(),
    // Transformation::SPIN,

    // Transformation::CW.rotate(Rotate::Clockwise).flip().flip().spin(),
    // Transformation::FLIP,

    // Transformation::FLIP,
    // Transformation::CCW.flip().rotate(Rotate::CounterClockwise),
    Transformation::FLIP,
    Transformation::CCW /* !right */
        .flip()
        .rotate(Rotate::CounterClockwise) /* down path + (down when going right -> CCW) */ // ord?
        .flip()
        .rotate(Rotate::CounterClockwise) /* down path + (down when going right -> CCW) */
        .spin()
        .rotate(Rotate::Clockwise) /* right path + right */
                                   // .rotate(Rotate::CounterClockwise)
));

// 0L -> 2U' => spin + CCW
static_assertions::const_assert!(transformation_eq(
    Transformation::SPIN.rotate(Rotate::CounterClockwise),
    // Transformation::CW /* !left */
    //     .flip().rotate(Rotate::Clockwise) /* down path + (down when going left -> CW) */
    //     .spin().rotate(Rotate::CounterClockwise)
    Transformation::CW /* !left */
        .flip()
        .rotate(Rotate::Clockwise) /* down path + (down when going left -> CW) */
        .spin()
        .rotate(Rotate::CounterClockwise)
        .spin() // ????
));

// 4L -> 2D' => CW
static_assertions::const_assert!(transformation_eq(
    Transformation::CW,
    Transformation::CW /* !left */
        .flip()
        .rotate(Rotate::Clockwise) /* up path + (up with going left -> CW) */
        .spin()
        .rotate(Rotate::CounterClockwise) // .spin()
));

// 2D -> 4L => CCW
//
// down = right, down
// static_assertions::const_assert!(transformation_eq(
//     Transformation::CCW,
//     Transformation::NONE /* !down */
//         .spin()//.rotate(Rotate::None)
//         .flip()//.rotate(Rotate::None),
//     // Transformation::CCW,
// ));

// 1U -> 0U' => Spin
//
// up = right, right, up
// (spin, spin, flip)
static_assertions::const_assert!(transformation_eq(
    Transformation::SPIN,
    Transformation::DOUBLE.spin().spin().flip()
));

// Attempt 1: `curr_trans.combine(trans_for_step)`
//
// ```
// up:    (down, flip) + (down, flip) + (down, flip) = down   ✅ // got Down
// 0 -> *
// left:  (down, flip) + (left, spin) = Up                    ❌ // got: Right; CCW to fix
// right: (down, flip) + (down, flip) + (right, spin) = Right ❌ // got: Left; Double to fix
// up:    (down, flip) + (left, spin) + (left, spin)  = Up    ❌ // got: Down; Double to fix
// 1 -> *
// up:   (right, spin) + (right, spin) + (up, flip)   = Up    ❌ // got: Down; Double to fix
// down: (right, spin) + (right, spin) + (down, flip) = Down  ❌ // got: Up; Double to fix
// 2 -> *
// up:   (right, spin) + (up, flip)   = Left                  ❌ // got: Down; CW to fix
// down: (right, spin) + (down, flip) = Left                  ❌ // got: Up; CCW to fix
// 3 -> *
// right: (down, flip) + (right, spin) = Up                   ❌ // got: Left; CW to fix
// 4 -> *
// left: (up, flip) + (left, spin) = Down                     ❌ // got: Right; CW to fix
// 5 -> *
// up: (left, spin) + (up, flip) = Right                      ❌ // got: Down; CW to fix
// ```
//
// Observations:
//   - all paths of length 3 are off by two rotations (need a double to fix),
//     all paths of length 2 are off by *one* rotation
//   - paths of length 2, grouped by the fix they require:
//     + CCW:
//       * down+left:  `left:  (down, flip) + (left, spin)`
//       * right+down: `down: (right, spin) + (down, flip)`
//     + CW:
//       * right+up:   `up:   (right, spin) + (up, flip)`
//       * down+right: `right: (down, flip) + (right, spin)`
//       * up+left:    `left: (up, flip) + (left, spin)`
//       * left+up:    `up: (left, spin) + (up, flip)`
//
// We should also include the empty paths as testcases:
//   ```
//   Up | Down => Transformation::FLIP,
//   Left | Right => Transformation::SPIN,
//   ```

const fn apply_path_transformation(
    _starting_dir: Facing,
    _step: Facing,
    curr_trans: Transformation,
    trans_for_step: Transformation,
) -> Transformation {
    curr_trans.combine(trans_for_step)
}

/* const */
fn traverse_path(start: Facing, path: &[(Facing, Transformation)]) -> Transformation {
    // let mut trans = Transformation::NONE;
    const fn trans_for_dir(dir: Facing, invert: bool) -> Transformation {
        use Facing::*;
        // let flip = match dir {
        //     Up | Down => true,     /* FLIP */
        //     Left | Right => false, /* SPIN */
        // };

        // let flip = if invert { !flip } else { flip };

        // if flip {
        //     Transformation::FLIP
        // } else {
        //     Transformation::SPIN
        // }

        if invert {
            match dir {
                Up => Transformation::SPIN,
                Down => Transformation::SPIN,
                // Left | Right => Transformation::FLIP,
                Left => Transformation::FLIP,
                Right => Transformation::FLIP,
            }
        } else {
            match dir {
                Up | Down => Transformation::FLIP,
                Left | Right => Transformation::SPIN,
            }
        }
    }

    let mut trans = trans_for_dir(start, false);

    let mut i = 0;
    while i < path.len() {
        let (step, trans_for_step) = path[i];
        i += 1;

        trans = trans.combine(trans_for_dir(step, true));
        trans = apply_path_transformation(start, step, trans, trans_for_step);
    }

    trans
}

// cargo test -p aoc22 --bin day22 -- path_relations |& grep OFF | sort | tee /dev/stderr | wc -l
#[cfg(test)]
mod test_path_relations {
    use super::*;

    macro_rules! test {
        ($nom:ident: $start:expr => $path:expr => $end_side:expr $(; exp$(($const:tt))?: $ans:expr)?) => {
            #[test]
            #[allow(unused)]
            fn $nom() {
                let start = $start;
                let path: &[(Facing, Transformation)] = &$path;
                let res = traverse_path(start, path);
                let end_side = $end_side;

                let expected_transformation = None::<Transformation>;
                $(
                    let expected_transformation = Some($ans);
                    if let Some(exp) = expected_transformation {
                        assert_eq!(
                            exp.get_dest_side(start),
                            end_side,
                            "test case is wrong, final expected transformation and end side don't match!"
                        );
                    }

                    $(
                        let _const_chk = std::stringify!($const);
                        static_assertions::const_assert_eq!(
                            $ans.get_dest_side($start) as usize,
                            $end_side as usize,
                        );
                    )?
                )?

                let got_side = res.get_dest_side(start);
                if got_side != end_side {
                    eprintln!(
                        "[OFF] {:20}: g: {got_side:5?}; e: {end_side:5?} | +{:?}",
                        core::stringify!($nom),
                        Rotate::by_idx((end_side.as_rotation() as usize + 4) - (got_side.as_rotation() as usize))
                    );
                }
                assert_eq!(
                    got_side,
                    end_side,
                    "\n\nExpected traversing path {path:#?}\
                    \nfrom `{start:?}` to yield a transformation taking us to \
                    `{end_side:?}` but instead we reached `{got_side:?}`;\
                    \n\ngot transformation: {res:?}\
                    \nexpected: {expected_transformation:?}"
                );

                if let Some(exp) = expected_transformation {
                    assert_eq!(exp.get_dest_side(start), end_side);
                    assert_eq!(res, exp);
                }
            }
        };
    }

    use Facing::{Down as D, Left as L, Right as R, Up as U, *};
    use Transformation as T;

    fn follow_path(
        starting_face: usize,
        _starting_side: Facing,
        info: &[[TransformationInfo; 4]; 6],
        path: &[Facing],
    ) -> (Transformation, usize) {
        let mut transformation = Transformation::NONE;

        // let mut facing = starting_side;
        let mut face_idx = starting_face;

        eprintln!("\nstart: {starting_face}, {_starting_side:?} [{path:?}]");
        for &step in path {
            eprintln!("  - step: {step:?}");
            // adjust step for the current transformation
            let step = transformation.get_dest_side(step);
            eprintln!("    * adjusted for current transformation -> {step:?}");

            let trans = info[face_idx][step as usize];
            eprintln!("  - mapping[face: {face_idx}][{step:?}] -> {trans:?}");

            transformation = transformation.combine(trans.trans);
            eprintln!("  - new transformation: {trans:?}");
            face_idx = trans.to;
            eprintln!("  - now @ face: {face_idx}");
            eprintln!("  ––––––––––––––––––––––––––––––––––––––");
        }

        // transformation = Transformation {
        //     spin: !transformation.spin,
        //     rotate: transformation.rotate.plus(Rotate::Clockwise),
        // };

        (
            transformation, //.spin().rotate(Rotate::CounterClockwise),
            face_idx,
        )
    }

    #[test]
    fn comprehensive() {
        // down + left + up -> transformation::none

        #[rustfmt::skip]
        let paths: &[(&[Facing], fn(Transformation) -> Transformation, Facing)] = &[
            // attempt 1, report:
            //  0: needs +CW
            //  1: needs +CW
            //  2: needs +CW
            //  3: needs +CW
            //  4: needs +CW
            //  5: needs +CW
            (&[Down, Left],
                |Transformation { spin, rotate} | Transformation { spin: !spin, rotate: rotate.plus(Rotate::Clockwise) },
                Left
            ),
            (&[Down, Right],
                |Transformation { spin, rotate} | Transformation { spin: !spin, rotate: rotate.plus(Rotate::CounterClockwise) },
                Right),

            // no idea what's going on here...


            (&[Up, Left],
                |Transformation { spin, rotate} | Transformation { spin: !spin, rotate: rotate.plus(Rotate::CounterClockwise) },
                // |t| t,
                Left
            ),
            (&[Up, Right],
                |Transformation { spin, rotate} | Transformation { spin: !spin, rotate: rotate.plus(Rotate::Clockwise) },
                Right
            ),

            (&[Left, Up],
                |Transformation { spin, rotate} | Transformation { spin: !spin, rotate: rotate.plus(Rotate::CounterClockwise) },
                Up
            ),
            (&[Left, Down],
                |Transformation { spin, rotate} | Transformation { spin: !spin, rotate: rotate.plus(Rotate::Clockwise) },
                Down,
            ),

            (&[Right, Up],
                |Transformation { spin, rotate} | Transformation { spin: !spin, rotate: rotate.plus(Rotate::Clockwise) },
                Up,
            ),
            (&[Right, Down],
                |Transformation { spin, rotate} | Transformation { spin: !spin, rotate: rotate.plus(Rotate::CounterClockwise) },
                Down,
            ),
        ];

        let mut errors = 0;
        let mut eq = |lhs, rhs| {
            if lhs != rhs {
                eprintln!("ERROR:\n  - expected {lhs:?}\n  - got {rhs:?}");
                errors += 1;
            }
        };
        let mut face_errors = 0;
        let mut eq_face = |lhs, rhs| {
            if lhs != rhs {
                eprintln!(
                    "ERROR:\n  - expected to reach face {lhs}\n  - reached face {rhs} instead"
                );
                face_errors += 1;
            }
        };

        for &(path, patch_func, equiv_dir) in paths {
            for face in 0..6 {
                let (trans, dest_face) =
                    follow_path(face, equiv_dir, &EXAMPLE_TRANSLATE_INFO, path);

                let trans = patch_func(trans);

                eq(
                    EXAMPLE_TRANSLATE_INFO[face][equiv_dir as usize].trans,
                    trans,
                );
                eq_face(
                    EXAMPLE_TRANSLATE_INFO[face][equiv_dir as usize].to,
                    dest_face,
                );
            }
        }

        if errors != 0 || face_errors != 0 {
            panic!("{errors} previous errors ({face_errors} face errors)");
        }
    }

    // Up | Down => Transformation::FLIP,
    // Left | Right => Transformation::SPIN,

    // ø -> *
    // up:    (down, flip) + (down, flip) + (down, flip) = flip (aka spin + double)
    test!(around: Up => [(D, T::FLIP), (D, T::FLIP), (D, T::FLIP)] => Down; exp(const): T::FLIP);
    test!(circle: Down => [(D, T::FLIP), (D, T::FLIP), (D, T::FLIP), (D, T::FLIP), (D, T::FLIP)] => Down; exp(const): T::NONE);

    // 0 -> *
    // down:  [] = Up (flip)
    // left:  (down, flip) + (left, spin) = Up
    // right: (down, flip) + (down, flip) + (right, spin) = Right
    // up:    (down, flip) + (left, spin) + (left, spin)  = Up
    test!(
        down: Down => [] => Up; // got: Down; Double to fix
        exp(const): EXAMPLE_TRANSLATE_INFO[0][D as usize].trans
    );
    test!(
        down_left: Left => [(D, T::FLIP), (L, T::SPIN)] => Up; // got: Right; CCW to fix
        exp(const): EXAMPLE_TRANSLATE_INFO[0][L as usize].trans
    );
    test!(
        down_down_right: Right => [(D, T::FLIP), (D, T::FLIP), (R, T::SPIN)] => Right; // got: Left; Double to fix
        exp(const): EXAMPLE_TRANSLATE_INFO[0][R as usize].trans
    );
    test!(
        down_left_left: Up => [(D, T::FLIP), (L, T::SPIN), (L, T::SPIN)] => Up; // got: Down; Double to fix
        exp(const): EXAMPLE_TRANSLATE_INFO[0][U as usize].trans
    );

    // 1 -> *
    // right: [] = Left (spin)
    // up:   (right, spin) + (right, spin) + (up, flip)   = spin
    // down: (right, spin) + (right, spin) + (down, flip) = spin
    test!(
        right: Right => [] => Left; // got: Right; Double to fix
        exp(const): EXAMPLE_TRANSLATE_INFO[1][R as usize].trans
    );
    test!(
        right_right_up: Up => [(R, T::SPIN), (R, T::SPIN), (U, T::FLIP)] => Up; // got: Down; Double to fix
        exp(const): EXAMPLE_TRANSLATE_INFO[1][U as usize].trans
    );
    test!(
        right_right_down: Down => [(R, T::SPIN), (R, T::SPIN), (D, T::FLIP)] => Down; // got: Up; Double to fix
        exp(const): EXAMPLE_TRANSLATE_INFO[1][D as usize].trans
    );

    // 2 -> *
    // left: [] = Right (spin)
    // up:   (right, spin) + (up, flip)   = spin + ccw
    // down: (right, spin) + (down, flip) = spin + cw
    test!(
        left: Left => [] => Right; // got: Left; Double to fix
        exp(const): EXAMPLE_TRANSLATE_INFO[2][L as usize].trans
    );
    test!(
        right_up: Up => [(R, T::SPIN), (U, T::FLIP)] => Left; // got Down; CW to fix
        exp(const): EXAMPLE_TRANSLATE_INFO[2][U as usize].trans
    );
    test!(
        right_down: Down => [(R, T::SPIN), (D, T::FLIP)] => Left; // got Up; CCW to fix
        exp(const): EXAMPLE_TRANSLATE_INFO[2][D as usize].trans
    );

    // 3 -> *
    // up:    [] = Down (spin)
    // right: (down, flip) + (right, spin) = Up
    test!(
        up: Up => [] => Down; // got: Down; Double to fix
        exp(const): EXAMPLE_TRANSLATE_INFO[3][U as usize].trans
    );
    test!(
        down_right: Right => [(D, T::FLIP), (R, T::SPIN)] => Up; // got: Left; CW to fix
        exp(const): EXAMPLE_TRANSLATE_INFO[3][R as usize].trans
    );

    // 4 -> *
    // left: (up, flip) + (left, spin) =
    // down: (up, flip) + (left, spin) + (left, spin) =
    test!(
        up_left: Left => [(U, T::FLIP), (L, T::SPIN)] => Down; // got: Right; CW to fix
        exp(const): EXAMPLE_TRANSLATE_INFO[4][L as usize].trans
    );
    /*test!(
        up_left_left:
        exp(const): EXAMPLE_TRANSLATE_INFO[4][D as usize].trans
    );*/

    // 5 -> *
    // up: (left, spin) + (up, flip) =
    test!(
        left_up: Up => [(L, T::SPIN), (U, T::FLIP)] => Right; // Got Down; CW to fix
        exp(const): EXAMPLE_TRANSLATE_INFO[5][U as usize].trans
    );
}

#[allow(non_snake_case)]
pub mod trans_info_helpers {
    use super::*;

    pub use super::Transformation as T;
    pub use Facing::*;
    pub use Rotate::*;

    #[derive(Copy, Clone)]
    pub enum TransformationRepr {
        Flip,
        Clockwise,
        CounterClockwise,

        Spin,
    }

    impl TransformationRepr {
        pub const fn apply_to(self, info: T) -> T {
            use TransformationRepr::*;
            match self {
                Flip => info.flip(),
                Clockwise => info.rotate(Rotate::Clockwise),
                CounterClockwise => info.rotate(Rotate::CounterClockwise),
                Spin => info.spin(),
            }
        }
    }

    #[derive(Copy, Clone)]
    pub enum Step {
        Imm {
            round: usize,
            to: usize,
            expected_dest_side: Option<Facing>,
            info: T,
        },
        // not actually what we want...
        /* Relative {
            round: usize,
            src: usize,
            src_side: Facing,
            to: usize,
            expected_dest_side: Option<Facing>,
            tacked_on_transformations: &'static [
                TransformationRepr
            ],
        }, */
        Computed {
            round: usize,
            directions: &'static [Facing],
            to: usize,
            expected_dest_side: Option<Facing>,
        },
    }

    pub const EMPTY: TransformationInfo = TransformationInfo {
        to: 10,
        trans: T::NONE,
    };

    impl Step {
        pub const fn round(self) -> usize {
            match self {
                Step::Imm { round, .. } => round,
                // Step::Relative { round, .. } => round,
                Step::Computed { round, .. } => round,
            }
        }

        pub const fn get(
            self,
            current_trans: &[[TransformationInfo; 4]; 6],
            src_face_idx: usize,
            _src_side: Facing,
        ) -> TransformationInfo {
            match self {
                Step::Imm { info, to, .. } => TransformationInfo { trans: info, to },
                /* Step::Relative { src, src_side, tacked_on_transformations, to, .. } => {
                    let info = current_trans[src][src_side as usize];
                    if info.to == EMPTY.to {
                        panic!("relative operation applied to placeholder; check your ordering?");
                    }
                    let mut trans = info.trans;
                    let mut i = 0;
                    while i < tacked_on_transformations.len() {
                        trans = tacked_on_transformations[i].apply_to(trans);
                    }

                    TransformationInfo { trans, to }
                }, */
                Step::Computed { directions, to, .. } => {
                    /* let mut trans = Transformation {
                        rotate: src_side.as_rotation(),
                        spin: false,
                    }; */
                    let mut trans = Transformation::NONE;
                    let mut face_idx = src_face_idx;

                    let mut i = 0;
                    while i < directions.len() {
                        let side = directions[i];
                        trans = match side {
                            Left | Right => trans,
                            Down | Up => trans, /* todo!() */
                        };

                        let info = current_trans[face_idx][side as usize];

                        if i == 0 {
                            // let _x: () = [][face_idx];
                            // let _x: () = [][side as usize];
                            // let _x: () = [][info.to];
                        }
                        if info.to == EMPTY.to {
                            // let _x: () = [][directions.len()];
                            // let _x: () = [][i];
                            // let _x: () = [][face_idx];
                            // let _x: () = [][side as usize];
                            // [()][info.to];
                            // [()][side as usize];
                            panic!(
                                "relative operation applied to placeholder; check your ordering?"
                            );
                        }

                        face_idx = info.to;
                        trans = trans.combine(info.trans);
                        i += 1;
                    }

                    if face_idx != to {
                        let _expected_dest: usize = [][to];
                        let _actual_dest: usize = [][face_idx];
                        panic!("directions did not lead us to the face index given in `to`");
                    }

                    TransformationInfo { trans, to }
                }
            }
        }

        pub const fn expected_dest_side(self) -> Option<Facing> {
            match self {
                /* Step::Relative { expected_dest_side, .. } | */
                Step::Imm {
                    expected_dest_side, ..
                }
                | Step::Computed {
                    expected_dest_side, ..
                } => expected_dest_side,
            }
        }

        pub const fn with_expected_dest_side(self, exp: Facing) -> Self {
            use Step::*;
            /* let slot = match self {
                Relative { expected_dest_side: ex @ Option::None, .. } | Imm { expected_dest_side: ex @ Option::None, .. } => {
                    &mut ex
                },
                Relative { expected_dest_side: Some(_), .. } | Imm { expected_dest_side: Some(_), .. } => panic!("expected_dest_side is already set!"),
            };
            *slot = Some(exp);
            self */
            match self {
                // x @ Relative { expected_dest_side: Option::None, round, src, src_side, to, tacked_on_transformations } => Relative { expected_dest_side: Some(exp), round, src, src_side, to, tacked_on_transformations },
                Imm {
                    expected_dest_side: Option::None,
                    round,
                    to,
                    info,
                } => Imm {
                    expected_dest_side: Some(exp),
                    round,
                    to,
                    info,
                },
                /* Relative { expected_dest_side: Some(_), .. } | */
                Computed {
                    expected_dest_side: Option::None,
                    round,
                    directions,
                    to,
                } => Computed {
                    round,
                    directions,
                    to,
                    expected_dest_side: Some(exp),
                },

                Imm {
                    expected_dest_side: Some(_),
                    ..
                }
                | Computed {
                    expected_dest_side: Some(_),
                    ..
                } => panic!("expected_dest_side is already set!"),
            }
        }

        pub const fn exp(self, e: Facing) -> Self {
            self.with_expected_dest_side(e)
        }
    }

    pub const fn apply_steps(steps: [[Step; 4]; 6]) -> [[TransformationInfo; 4]; 6] {
        let mut out = [[EMPTY; 4]; 6];

        let mut round = 0;
        let mut remaining = 24;
        while remaining != 0 {
            let mut face = 0;
            while face < 6 {
                let mut side = 0;
                while side < 4 {
                    let step = steps[face][side];
                    if step.round() == round {
                        remaining -= 1;
                        let transformation = step.get(&out, face, Facing::ALL[side]);
                        out[face][side] = transformation;

                        if let Some(expected_side) = step.expected_dest_side() {
                            let side = transformation.trans.get_dest_side(Facing::ALL[side]);
                            // assert_eq!(side as usize, expected_side as usize);
                            if (side as usize) != (expected_side as usize) {
                                let _side: usize = [][side as usize];
                                let _expected_side: usize = [][expected_side as usize];
                            }
                        }
                    }

                    side += 1;
                }

                face += 1;
            }

            round += 1;
        }

        out
    }
    pub const fn imm(iteration_round: usize, to: usize, trans: T) -> Step {
        Step::Imm {
            round: iteration_round,
            info: trans,
            to,
            expected_dest_side: Option::None,
        }
    }
    pub const fn R(to: usize, t: T) -> Step {
        imm(0, to, t)
    }
    pub const fn D(to: usize, t: T) -> Step {
        imm(0, to, t)
    }
    pub const fn L(to: usize, t: T) -> Step {
        imm(0, to, t)
    }
    pub const fn U(to: usize, t: T) -> Step {
        imm(0, to, t)
    }

    pub use {D as cD, L as cL, R as cR, U as cU}; // computed, not given; eventually replace with `rel`

    // const fn rel(round: usize, (src, src_side): (usize, Facing), transformations: &'static [TransformationRepr], to: usize) -> Step {
    //     Step::Relative { round, src, src_side, to, tacked_on_transformations: transformations, expected_dest_side: Option::None }
    // }
    // const fn _1(src: (usize, Facing), ts: &'static [TransformationRepr], to: usize) -> Step { rel(1, src, ts, to) }
    // const fn _2(src: (usize, Facing), ts: &'static [TransformationRepr], to: usize) -> Step { rel(2, src, ts, to) }
    // const fn _3(src: (usize, Facing), ts: &'static [TransformationRepr], to: usize) -> Step { rel(3, src, ts, to) }

    pub const fn com(round: usize, to: usize, directions: &'static [Facing]) -> Step {
        Step::Computed {
            round,
            directions,
            to,
            expected_dest_side: Option::None,
        }
    }

    pub const fn _1(to: usize, dir: &'static [Facing]) -> Step {
        com(1, to, dir)
    }
    pub const fn _2(to: usize, dir: &'static [Facing]) -> Step {
        com(2, to, dir)
    }
    pub const fn _3(to: usize, dir: &'static [Facing]) -> Step {
        com(3, to, dir)
    }

    pub const X: Step = Step::Imm {
        round: 1,
        to: 10,
        info: T::NONE,
        expected_dest_side: Option::None,
    };
}

/// ```ignore
///         ┏━━━┓
///         ┃ 0 ┃
/// ┏━━━┳━━━╋━━━┫
/// ┃ 1 ┃ 2 ┃ 3 ┃
/// ┗━━━┻━━━╋━━━╋━━━┓
///         ┃ 4 ┃ 5 ┃
///         ┗━━━┻━━━┛
/// ```
///
/// ```ignore
///               /––––––––>  1
///           ________
///          /⠡   0  /\
/// 2 <–––– /  ⠡..../..\
///        /___⠌___/   / –––> 5
///        \  ⠌  3 \  /
///         \⠌______\/
///             \
///             \–––––––> 4
/// ```
#[rustfmt::skip]
#[allow(unused, bad_style)]
const EXAMPLE_TRANSLATE_INFO: [[TransformationInfo; 4]; 6] = {
    /*
    use TransformationInfo as T;
    use {T as R, T as D, T as L, T as U};
    use Rotate::{Flip, Flip as Flp, Clockwise as Cwe, CounterClockwise as Ccw, None, None as Non};

    const X: TransformationInfo = TransformationInfo { to: 9, rot: Non, edge_flip: false };
    */

    /*
    let _orig = [
        /* 0 */ [
            /* R */ X,
            D { to: 3, rot: Flip, edge_flip: false },
            /* L */ X,
            /* U */ X,
        ],
        /* 1 */ [
            R { to: 2, rot: Flip, edge_flip: false },
            /* D */ X,
            /* L */ X,
            /* U */ X,
        ],
        /* 2 */ [
            R { to: 3, rot: Flip, edge_flip: false },
            /* D */ X,
            L { to: 1, rot: Flip, edge_flip: false },
            /* U */ X,
        ],
        /* 3 */ [
            /* R */ X,
            D { to: 4, rot: Flip, edge_flip: false },
            L { to: 2, rot: Flip, edge_flip: false },
            U { to: 0, rot: Flip, edge_flip: false },
        ],
        /* 4 */ [
            R { to: 5, rot: Flip, edge_flip: false },
            /* D */ X,
            /* L */ X,
            U { to: 3, rot: Flip, edge_flip: false },
        ],
        /* 5 */ [
            /* R */ X,
            /* D */ X,
            L { to: 4, rot: Flip, edge_flip: false },
            /* U */ X,
        ],
    ];
    */

    /* [
        /* 0 */ [
            R { to: 5, rot: None, edge_flip: true }, // [2] via down + right; flp + cwe + (down, right) (aka cwe) -> (None, edge_flip)
            D { to: 3, rot: Flp, edge_flip: false },
            L { to: 2, rot: Ccw, edge_flip: false }, // [1] via down + left; flp + (down, left) -> Ccw
            U { to: 1, rot: None, edge_flip: true }, // [2] via left via Ccw + up => 2's left; ccw + flp + (left + up) -> (none, edge flip)
        ],
        /* 1 */ [
            R { to: 2, rot: Flp, edge_flip: false },
            D { to: 4, rot: None, edge_flip: true }, // [2] via right + down; flp + ccw + (right, down) -> flp + ccw + ccw -> (none, edge_flip)
            L { to: 5, rot: Ccw,  edge_flip: true }, // [3] via down + left => 4's right; (none, true) + flp + (down, left) (aka cwe) -> (ccw, true)
            U { to: 0, rot: None, edge_flip: true }, // [2] via right + up; flp + cwe + (right, up) -> (none, edge flip)
        ],
        /* 2 */ [
            R { to: 3, rot: Flp, edge_flip: false },
            D { to: 4, rot: Ccw, edge_flip: false }, // [1] via right + down; flp + flp + (right, down) -> ccw
            L { to: 1, rot: Flp, edge_flip: false },
            U { to: 0, rot: Cwe, edge_flip: false }, // [1] via right + up; flp + (right, up) -> Cwe
        ],
        /* 3 */ [
            R { to: 5, rot: Cwe, edge_flip: false }, // [1] via down + right; flp + flp + (down, right) -> Cw
            D { to: 4, rot: Flp, edge_flip: false },
            L { to: 2, rot: Flp, edge_flip: false },
            U { to: 0, rot: Flp, edge_flip: false },
        ],
        /* 4 */ [
            R { to: 5, rot: Flp, edge_flip: false },
            D { to: 1, rot: None, edge_flip: true }, // [2] via left via Cwe + down => 2's left; cwe + flp + (left, down) -> (none, edge flip)
            L { to: 2, rot: Cwe, edge_flip: false }, // [1] via up + left; flp + (up, left) -> Cwe
            U { to: 3, rot: Flp, edge_flip: false },
        ],
        /* 5 */ [
            R { to: 0, rot: None, edge_flip: true }, // [2] via up + right (ccw + right = top) => via 3's top; ccw + flp + (up, right) -> ccw + flp + ccw -> (none, edge flip)
            D { to: 1, rot: Cwe, edge_flip: true  }, // [2] via left + down; flp + (none, edge_flip) + (left, down) -> (cwe, edge_flip)
            L { to: 4, rot: Flp, edge_flip: false },
            U { to: 3, rot: Ccw, edge_flip: false }, // [1] via left + up; flp + flp + (left, up) -> Ccw
        ],
    ] */

    use trans_info_helpers::*;
    /* R D L U */
    apply_steps([
        /* 0 */ [
            cR(5, T::FLIP).exp(Right), // _1(5, &[Down, Down, Right]).exp(Right),
            D(3, T::FLIP).exp(Up),
            cL(2, T::CW.spin()).exp(Up), // _1(2, &[Down, Left]).exp(Up), // cL(2, T::FLIP.rotate(Clockwise)).exp(Up),
            cU(1, T::SPIN).exp(Up),
        ],
        /* 1 */ [
            R(2, T::SPIN).exp(Left),
            cD(4, T::SPIN).exp(Down),
            cL(5, T::SPIN.rotate(Clockwise)).exp(Down),
            cU(0, T::SPIN).exp(Up),
        ],
        /* 2 */ [
            R(3, T::SPIN).exp(Left),
            cD(4, T::CW.flip()).exp(Left),
            L(1, T::SPIN).exp(Right),
            cU(0, T::CCW.flip()).exp(Left),
        ],
        /* 3 */ [
            cR(5, T::SPIN.rotate(Clockwise)).exp(Up),
            D(4, T::FLIP).exp(Up),
            L(2, T::SPIN).exp(Right),
            U(0, T::FLIP).exp(Down),
        ],
        /* 4 */ [
            R(5, T::SPIN).exp(Left),
            cD(1, T::SPIN).exp(Down),
            cL(2, T::CCW.spin()).exp(Down), // _1(2, &[Up, Left]).exp(Down), // TODO: add translate test for this?
            U(3, T::FLIP).exp(Down),
        ],
        /* 5 */ [
            cR(0, T::FLIP).exp(Right),
            cD(1, T::CW.flip()).exp(Left),
            L(4, T::SPIN).exp(Right),
            cU(3, T::CW.flip()).exp(Right),
        ],
    ])
};

#[cfg(test)]
mod test_cube_mapping {
    use super::*;

    type CubeInfo<'c> = (
        usize,                        // edge len,
        &'c BiBTreeMap<Coord, usize>, // face_idxes
        &'c [[TransformationInfo; Facing::ALL.len()]; 6],
    );

    // ```
    //  0123456789ABCDEF
    // 0        ...A
    // 1        .#..
    // 2        #...
    // 3        ....
    // 4...#.......#
    // 5........#...
    // 6..#....#....
    // 7..........#.
    // 8        ...#....
    // 9        .....#..
    // A        .#......
    // B        ......#B
    // ```
    // ```
    //         ┏━━━┓
    //         ┃ 0 ┃
    // ┏━━━┳━━━╋━━━┫
    // ┃ 1 ┃ 2 ┃ 3 ┃
    // ┗━━━┻━━━╋━━━╋━━━┓
    //         ┃ 4 ┃ 5 ┃
    //         ┗━━━┻━━━┛
    // ```
    //
    fn with_example_cube_info<R>(func: impl FnOnce(CubeInfo<'_>) -> R) -> R {
        let edge_len = 4;
        let mut map = BiBTreeMap::new();
        map.extend([
            ((8, 0).into(), 0),
            ((0, 4).into(), 1),
            ((4, 4).into(), 2),
            ((8, 4).into(), 3),
            ((8, 8).into(), 4),
            ((12, 8).into(), 5),
        ]);

        func((edge_len, &map, &EXAMPLE_TRANSLATE_INFO))
    }

    #[test]
    fn example_translate_info() {
        assert_face_mapping_internally_consistent(&EXAMPLE_TRANSLATE_INFO);
    }

    mod example_cube_translate_tests {
        // these are the part 2 examples:
        use super::*;
        use Facing::*;
        const A: Coord = Coord { x: 11, y: 5 };
        const B: Coord = Coord { x: 14, y: 8 };
        const A_: Coord = Coord { x: 11, y: 4 };
        const B_: Coord = Coord { x: 15, y: 8 };
        const C: Coord = Coord { x: 10, y: 11 };
        const D: Coord = Coord { x: 1, y: 7 };

        macro_rules! test2 {
            ($nom:ident: $start:expr, $dir:expr => $end:expr, $dir2:expr) => {
                #[test]
                fn $nom() {
                    with_example_cube_info(|info| {
                        let test = |(start, start_dir), (end, end_dir)| {
                            assert_eq!(
                                translate_cube_coord(start, start_dir, info), (end, end_dir),
                                "Expected moving {start_dir:?} from {start:?} to leave us at {end:?} (facing {end_dir:?})"
                            );
                        };
                        let test2 = |(start, start_dir), (end, end_dir)| {
                            test((start, start_dir), (end, end_dir));
                            test((end, !end_dir), (start, !start_dir));
                        };

                        let start: Coord = ($start).into();
                        let end: Coord = ($end).into();

                        test2((start, $dir), (end, $dir2));
                    })
                }
            };
        }

        // test((a, Right), (b, Down));
        // test((b, Up), (a, Left));
        // test2((a, Right), (b, Down));
        test2!(ex1: A, Right => B, Down);

        // test2((a_, Right), (b_, Down));
        test2!(ex1_custom: A_, Right => B_, Down);

        // test((c, Down), (d, Up));
        // test((d, Down), (c, Up));
        // test2((c, Down), (d, Up));
        test2!(ex2: C, Down => D, Up);

        // 0; R D L U
        test2!(face0_r: (11, 0), Right => (15, 11), Left);
        test2!(face0_d: (8, 3), Down => (8, 4), Down);
        test2!(face0_l: (8, 3), Left => (7, 4), Down);
        test2!(face0_u: (8, 0), Up => (3, 4), Down);

        // 1
        test2!(face1_r: (3, 4), Right => (4, 4), Right);
        test2!(face1_d: (0, 7), Down => (11, 11), Up);
        test2!(face1_l: (0, 4), Left => (15, 11), Up);
        test2!(face1_u: (0, 4), Up => (11, 0), Down);

        // 2
        test2!(face2_r: (7, 4), Right => (8, 4), Right);
        test2!(face2_d: (4, 7), Down => (8, 11), Right);
        test2!(face2_l: (4, 7), Left => (3, 7), Left);
        test2!(face2_u: (4, 4), Up => (8, 0), Right);

        // 3
        test2!(face3_r: (11, 4), Right => (15, 8), Down);
        test2!(face3_d: (11, 7), Down => (11, 8), Down);
        test2!(face3_l: (8, 4), Left => (7, 4), Left);
        test2!(face3_u: (8, 4), Up => (8, 3), Up);

        // 4
        test2!(face4_r: (11, 8), Right => (12, 8), Right);
        test2!(face4_d: (11, 11), Down => (0, 7), Up);
        test2!(face4_l: (8, 11), Left => (4, 7), Up);
        test2!(face4_u: (8, 8), Up => (8, 7), Up);

        // 5
        test2!(face5_r: (15, 8), Right => (11, 3), Left);
        test2!(face5_d: (15, 11), Down => (0, 4), Right);
        test2!(face5_l: (12, 11), Left => (11, 11), Left);
        test2!(face5_u: (12, 8), Up => (11, 7), Left);
    }
}

impl FromStr for Map {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rows = s.lines().count();
        let cols = s.lines().map(|l| l.chars().count()).max().unwrap();
        let mut grid = vec![vec![Cell::default(); cols]; rows];

        for (r, row) in s.lines().enumerate() {
            for (c, cell) in row.chars().enumerate() {
                grid[r][c] = cell.try_into().unwrap();
            }
        }

        let surface_area = grid
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&c| c != Cell::Empty)
            .count();
        assert_eq!(surface_area % 6, 0, "surface area: {surface_area}");
        let edge_len = ((surface_area / 6) as f64).sqrt() as usize;
        assert_eq!(surface_area / 6, edge_len * edge_len);
        assert_eq!(rows % edge_len, 0);
        assert_eq!(cols % edge_len, 0);

        // first open
        let pos: Coord = grid
            .iter()
            .enumerate()
            .flat_map(|(r, row)| row.iter().enumerate().map(move |(c, cell)| ((c, r), cell)))
            .filter(|(_, &cell)| cell == Cell::Open)
            .map(|(coord, _)| coord.into())
            .next()
            .unwrap();

        let dir = Default::default();

        let mut vis = grid
            .iter()
            .map(|row| row.iter().cloned().map(Either::Left).collect_vec())
            .collect_vec();
        vis[pos.y][pos.x] = Either::Right(dir);

        // assign each face an index and create a 1:1 mapping from top-left
        // corner of the face (in the 2D grid) to its index:
        let mut face_idxs = BiBTreeMap::new();
        let mut idx = 0;
        for tile in Coord::iter(0..(cols / edge_len), 0..(rows / edge_len)) {
            let c = tile * edge_len;
            let Coord { x, y } = c;

            if grid[y][x] == Cell::Empty {
                continue;
            }

            face_idxs.insert(c, idx);
            idx += 1;
            // eprintln!("tile ({}, {}), @ ({x}, {y})", tile.x, tile.y);
        }
        assert_eq!(idx, 6);

        // TODO: coord type that you can multiply by a scalar..
        /*         let mut face_mapping: [[Option<TranslationInfo>; 4]; 6] = Default::default();
               let mut connected = 0;

               // mark each adjacent face that's in bound + present in `face_mapping`
               for (coord, &idx) in face_idx.iter() {
                   for f in Facing::ALL {
                       let offs = f.as_offs() * (edge_len as isize);

                       if let Some(pos) = coord.checked_add_signed(offs) {
                           // check if this is actually present:
                           if let Some(&adj_idx) = face_idx.get_by_left(&pos) {
                               let slot = &mut face_mapping[idx][*f as usize];
                               assert_eq!(*slot, None);
                               *slot = Some(TranslationInfo {
                                   to: adj_idx,
                                   rot: Rotate::None,
                               });
                               connected += 1;
                           }
                       }
                   }
               }
               assert_eq!(connected, 10); // not sure if this is actually always true?

               let mut made_progress;
               loop {
                   made_progress = false;

                   for (coord, &idx) in face_idx.iter() {
                       for f in Facing::ALL {
                           use Facing::*;

                           let slot = &mut face_mapping[idx][*f as usize];

                           // if missing:
                           if slot.is_some() {
                               continue;
                           }

                           // corners:
                           //  - if left is missing, try down+left (cw) / up+left (ccw)
                           //  - if right is missing, try down+right (ccw) / up+right (cw)
                           //  - if up is missing, try left+up (cw) / right+up (ccw)
                           //  - if down is missing, try left+down (ccw) / right (ccw)
                           // etc.
                           let alts = match f {
                               Left | Right => [Down, Up],
                           }
                       }
                   }

                   /* 12 edges in a cube; x2 for bidir */
                   if connected == 24 {
                       break;
                   } else if !made_progress {
                       panic!("stuck! at {connected} connected edges");
                   }
               }

               let face_mapping = face_mapping.map(|arr| arr.map(|i| i.unwrap()));

               panic!();
        */

        let cube_face_mapping = construct_face_mapping(&face_idxs, edge_len);

        Ok(Map {
            vis,
            pos,
            grid,
            dim: (cols, rows).into(),
            dir,
            edge_len,
            face_idxs,
            cube_face_mapping,
        })
    }
}

impl Map {
    // move forward 1 step in the current facing, returning false if we've
    // hit a wall..
    fn step(&mut self) -> bool {
        use Cell::*;

        let Coord { x: lx, y: ly } = self.dim;
        let Coord { x: dx, y: dy } = self.dir.as_offs();

        let one_step = |Coord { x, y }| {
            let x = ((x as isize) + dx).try_into().unwrap_or(lx - 1) % lx;
            let y = ((y as isize) + dy).try_into().unwrap_or(ly - 1) % ly;

            Coord { x, y }
        };

        // move 1 in the direction requested, wrapping around the edges of
        // the grid
        let pos = one_step(self.pos);
        match self[pos] {
            // if we hit a wall, we're done; can't move:
            Wall => false,
            // if we hit an open space we're also done;
            Open => {
                // update the position and return true
                self.pos = pos;
                true
            }
            // if we hit an empty space things are trickier..
            Empty => {
                // we want to wrap around to the first non-empty space
                // encountered when starting from the wall opposite the
                // direction we tried to move
                //
                // we can't model this as simply repeating the "move in the
                // direction we're facing until we wrap on a wall" logic
                // because of gaps; i.e.
                // ```
                // .......
                // ..A....
                //     ...
                //     ...
                // ..^....
                // ..B....
                // ```
                //
                // `^` in the above wraps to `B`, not `A`
                //
                // instead we model this as snapping back to a wall,
                // applying our step until we hit a non-open position

                // note: this is a good place to introduce some caching..

                let Coord {
                    x: mut nx,
                    y: mut ny,
                } = pos;
                use Facing::*;
                match self.dir {
                    // snap to left; i.e. x = 0
                    Right => nx = 0,
                    // snap to top; i.e. y = 0
                    Down => ny = 0,
                    // snap to right; i.e. x = lx - 1
                    Left => nx = lx - 1,
                    // snap to bottom; i.e. y = ly - 1
                    Up => ny = ly - 1,
                }
                let mut pos = (nx, ny).into();

                // now, keep stepping til we hit something not empty:
                loop {
                    match self[pos] {
                        Empty => pos = one_step(pos),
                        Wall => break false,
                        Open => {
                            self.pos = pos;
                            break true;
                        }
                    }
                }
            }
        }
    }
}

impl Map {
    // move forward 1 step in the current facing, returning false if we've hit
    // a wall
    //
    // `facing` is updated if we've moved across an edge of the cube
    fn step_cube(&mut self) -> bool {
        use Cell::*;

        // TODO: contains for `Range<Coord>` ?
        let Coord { x: lx, y: ly } = self.dim;

        // Try to move in the direction requested:
        //
        // bail if we:
        //  - go off the end of the grid
        //  - hit empty space
        let (new_pos, new_dir) = if let Some(c) = self
            .pos
            .checked_add_signed(self.dir.as_offs())
            .filter(|Coord { x, .. }| (0..lx).contains(x))
            .filter(|Coord { y, .. }| (0..ly).contains(y))
            .filter(|&c| !matches!(self[c], Cell::Empty))
        {
            (c, self.dir)
        } else {
            // if we hit empty space/need to wrap do so, respecting the cube:
            translate_cube_coord(
                self.pos,
                self.dir,
                (self.edge_len, &self.face_idxs, &self.cube_face_mapping),
            )
        };

        match self[new_pos] {
            // if we hit a wall, we're done; we can't move here so don't update
            // pos. just return
            Wall => false,
            // if we hit an open space we can move here!
            Open => {
                self.pos = new_pos;
                self.dir = new_dir;
                true
            }
            Empty => unreachable!(),
        }

        /*         let (lx, ly) = self.dim;
        let (dx, dy) = self.dir.as_offs();

        let one_step = |(x, y)| {
            let x = ((x as isize) + dx).try_into().unwrap_or(lx - 1) % lx;
            let y = ((y as isize) + dy).try_into().unwrap_or(ly - 1) % ly;

            (x, y)
        };

        // move 1 in the direction requested, wrapping around the edges of
        // the grid
        let pos = one_step(self.pos);
        match self[pos] {
            // if we hit a wall, we're done; can't move:
            Wall => false,
            // if we hit an open space we're also done;
            Open => {
                // update the position and return true
                self.pos = pos;
                true
            }
            // if we hit an empty space things are trickier..
            Empty => {
                // we want to wrap around to the first non-empty space
                // encountered when starting from the wall opposite the
                // direction we tried to move
                //
                // we can't model this as simply repeating the "move in the
                // direction we're facing until we wrap on a wall" logic
                // because of gaps; i.e.
                // ```
                // .......
                // ..A....
                //     ...
                //     ...
                // ..^....
                // ..B....
                // ```
                //
                // `^` in the above wraps to `B`, not `A`
                //
                // instead we model this as snapping back to a wall,
                // applying our step until we hit a non-open position

                // note: this is a good place to introduce some caching..

                let (mut nx, mut ny) = pos;
                use Facing::*;
                match self.dir {
                    // snap to left; i.e. x = 0
                    Right => nx = 0,
                    // snap to top; i.e. y = 0
                    Down => ny = 0,
                    // snap to right; i.e. x = lx - 1
                    Left => nx = lx - 1,
                    // snap to bottom; i.e. y = ly - 1
                    Up => ny = ly - 1,
                }
                let mut pos = (nx, ny);

                // now, keep stepping til we hit something not empty:
                loop {
                    match self[pos] {
                        Empty => pos = one_step(pos),
                        Wall => break false,
                        Open => {
                            self.pos = pos;
                            break true;
                        }
                    }
                }
            }
        } */
        // todo!()
    }
}

impl Map {
    pub fn apply_direction(&mut self, dir: Direction, cube: bool) {
        use Direction::*;
        match dir {
            Forward(x) => {
                for _ in 0..x {
                    let res = if cube { self.step_cube() } else { self.step() };
                    if !res {
                        return;
                    }

                    // vis:
                    let Coord { x, y } = self.pos;
                    self.vis[y][x] = Either::Right(self.dir);
                }
            }
            Turn(t) => {
                self.dir = self.dir.turn(t);

                // vis:
                let Coord { x, y } = self.pos;
                self.vis[y][x] = Either::Right(self.dir);
            }
        }
    }
}

impl Map {
    fn password(&self) -> usize {
        let Coord { x: c, y: r } = self.pos;
        1000 * (r + 1) + 4 * (c + 1) + (self.dir as usize)
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2022, 22);
    let inp = aoc.get_input();
    // let inp = include_str!("ex");

    let (map, directions) = inp.split_once("\n\n").unwrap();
    let map: Map = map.parse().unwrap();
    let directions: Directions = directions.parse().unwrap();

    // eprintln!("map:\n{map}\ndirections: {directions:?}\n\n----------------------\n");

    let p1 = {
        let mut map = map.clone();
        for d in &directions.0 {
            map.apply_direction(*d, false);
            // eprintln!("\n{map}\n----------------------------------");
        }

        eprintln!("\nfinal:\n{map:#}");
        map.password()
    };

    dbg!(p1);
    aoc.submit_p1(p1).unwrap();

    let p2 = {
        let mut map = map;
        for d in &directions.0 {
            map.apply_direction(*d, true);
        }

        eprintln!("\nfinal:\n{map:#}");
        map.password()
    };
    dbg!(p2);
    aoc.submit_p2(p2).unwrap();

    // https://gist.github.com/rkirov/b914c9c10a146ec5ee538e65949f6bc1
    // https://www.reddit.com/r/adventofcode/comments/zuso8x/2022_day_22_part_3/
    let p3 = {
        // let inp = reqwest::blocking::get("https://gist.githubusercontent.com/rkirov/b914c9c10a146ec5ee538e65949f6bc1/raw/0f27f35a5bc2dbfb730e156089e2a72e53a54529/input.txt")
        //     .unwrap()
        //     .text()
        //     .unwrap();

        let inp = include_str!("p3.txt");
        inp.split("\n\n")
            .chunks(2)
            .into_iter()
            .map(|mut it| {
                // let [map, directions] = arr else {
                //     panic!()
                // };
                let map = it.next().unwrap();
                let directions = it.next().unwrap();

                let directions: Directions = directions.parse().unwrap();
                let mut map: Map = map.parse().unwrap();

                for d in &directions.0 {
                    map.apply_direction(*d, true);
                }

                map.password()
            })
            .sum::<usize>()
    };
    const P3_ANS: usize = 2415853;
    assert_eq!(p3, P3_ANS);
}

/*

        ...#
        .#..
        #...
        ....
...#.......#
........#...
..#....#....
..........#.
        ...#....
        .....#..
        .#......
        ......#.

 - - 1 -
 2 3 4 -
 - - 5 6

1's left:
  left = down + left => down (direct) + left (direct) + cw => 3 cw (ccw to go from 1l -> 3)

1's top:
  top = left + up + cw => left (3 cw) + up
    means:
      take left (3), apply adjustment ccw (-1)
        up of ^ is now left (ccw, -1)
          yields 2; add another cw => cw + cw = flip


*/

// part 3: https://gist.github.com/rkirov/b914c9c10a146ec5ee538e65949f6bc1
// https://www.youtube.com/watch?v=60z_hpEAtD8 clifford algebra
// https://en.wikipedia.org/wiki/Rotation_matrix
