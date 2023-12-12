use std::ops::Index;

pub trait TupleGet<const N: usize> {
    type X;
    fn get(&self) -> &Self::X;
}

macro_rules! mapping_macro {
    () => {
        _0
    };
    ($A:tt) => {
        _1
    };
    ($A:tt $B:tt) => {
        _2
    };
    ($A:tt $B:tt $C:tt) => {
        _3
    };
    ($A:tt $B:tt $C:tt $D:tt) => {
        _4
    };
    ($A:tt $B:tt $C:tt $D:tt $E:tt) => {
        _5
    };
    ($A:tt $B:tt $C:tt $D:tt $E:tt $F:tt) => {
        _6
    };
    ($A:tt $B:tt $C:tt $D:tt $E:tt $F:tt $G:tt) => {
        _7
    };
    ($A:tt $B:tt $C:tt $D:tt $E:tt $F:tt $G:tt $H:tt) => {
        _8
    };
    ($A:tt $B:tt $C:tt $D:tt $E:tt $F:tt $G:tt $H:tt $I:tt) => {
        _9
    };
    ($A:tt $B:tt $C:tt $D:tt $E:tt $F:tt $G:tt $H:tt $I:tt $J:tt) => {
        _10
    };
    ($A:tt $B:tt $C:tt $D:tt $E:tt $F:tt $G:tt $H:tt $I:tt $J:tt $K:tt) => {
        _11
    };
}

macro_rules! tuple_impl {
    // porcelain
    (($($t:tt)*)) => {
        triangle!(($($t)*) | tuple_impl (@));
    };
    // per arity
    (@ $($t:tt)*) => {
        tuple_impl!(
            #
            ($($t)*)
            ()
            ($($t)* EOM)
        );
    };
    // plumbing; iterate from 1..=arity
    //
    // (note the `+` on remaining_rest and the extra token appended to the
    // list of ty_params fed as the initial "value" of `$remaining_rest` by ^;
    // this bit of trickery lets us avoid having to duplicate the impl below
    // in the "base case" of this macro without having to pull out the below
    // into its own macro)
    (# ($($ty_params:tt)*) ($($consumed:tt)*) ($remaining_first:tt $($remaining_rest:tt)+)) => {
        impl<$($ty_params,)*> TupleGet<{count!($($consumed)*)}> for ($($ty_params,)*) {
            type X = $remaining_first;

            #[allow(unused, bad_style)]
            #[inline]
            fn get(&self) -> &Self::X {
                let ($(
                    $ty_params
                ,)*) = self;
                $remaining_first
            }
        }

        // Unfortunately this doesn't work; using traits in the ty params of an
        // impl is a non-starter; `rustc` always treats these as foreign types.
        //
        // So we'll need a mechanism to do the mapping that's not type system based.
        // impl<$($ty_params,)*> Index<Num2Ty<{count!($($consumed)*)}>> for ($($ty_params,)*) {

        // Note that we use the alias in the fn signature; rustc is happy to unify these.
        impl<$($ty_params,)*> Index<mapping_macro!($($consumed)*)> for ($($ty_params,)*) {
            type Output = $remaining_first;

            #[allow(unused, bad_style)]
            #[inline]
            fn index(&self, _idx: Num2Ty<{count!($($consumed)*)}>) -> &Self::Output {
                let ($(
                    $ty_params
                ,)*) = self;
                $remaining_first
            }
        }

        tuple_impl!(
            # ($($ty_params)*) ($($consumed)* $remaining_first) ($($remaining_rest)+)
        );
    };
    (# ($($t:tt)*) ($($c:tt)*) (EOM) ) => {};
}

tuple_impl!((A B C D E F G H I J K L));

pub trait Num2TyMapping<const N: usize> {
    type Ty;
}
pub struct MapperType;
pub type Num2Ty<const N: usize> = <MapperType as Num2TyMapping<N>>::Ty;

// [num] -> ty
macro_rules! num_structs {
    ($($n:tt)*) => {$(
        paste::paste! {
            pub struct [<_ $n>];

            // This is "cleaner" but makes it so we need a wrapper type to
            // sidestep coherence.
            /*
            impl<Tup: TupleGet<$n>> Index<[<_ $n>]> for L<Tup> {
                type Output = Tup::X;
                fn index(&self, _idx: [<_ $n>]) -> &Self::Output {
                    self.0.get()
                }
            }
            */

            // So we use a macro to do the mapping and use this const generics
            // this to double check that we did it right:
            impl Num2TyMapping<$n> for MapperType { type Ty = [<_ $n>]; }
        }
    )*};
}

pub mod num_structs {
    use super::*;
    num_structs! { 0 1 2 3 4 5 6 7 8 9 10 11 }
}

use num_structs::*;

pub trait TupleGetExt {
    fn ref_to<const N: usize>(&self) -> &<Self as TupleGet<N>>::X
    where
        Self: TupleGet<N>,
    {
        self.get()
    }

    fn at<const N: usize>(&self) -> <Self as TupleGet<N>>::X
    where
        Self: TupleGet<N>,
        <Self as TupleGet<N>>::X: Copy,
    {
        *self.ref_to()
    }
}

impl<T> TupleGetExt for T {}
