use std::mem::MaybeUninit;

use crate::macros::{cdr, count, triangle};

// trait Collectible {
//     type Out<T>;
//     fn do<I: Iterator>(it: I) -> Self::Out<I::Item>;
// }

trait ArrCollect: Iterator {
    fn arr_collect<const N: usize>(self) -> [Self::Item; N];
}

struct Uninit<T>(*const T);
impl<T> Uninit<T> {
    const UNINIT: MaybeUninit<T> = MaybeUninit::uninit();
}

impl<It: Iterator> ArrCollect for It {
    fn arr_collect<const N: usize>(self) -> [Self::Item; N] {
        // // const U: MaybeUninit<It::Self> = MaybeUninit::uninit();
        // use std::mem::{transmute, transmute_copy};
        // let arr: MaybeUninit<[MaybeUninit<Self::Item>; N]> = MaybeUninit::uninit();
        // let arr: [MaybeUninit<Self::Item>; N] = unsafe { arr.assume_init() };
        let mut arr: [MaybeUninit<Self::Item>; N] = [Uninit::<Self::Item>::UNINIT; N];

        /// If we panic while calling `next`, we'll unwind past this frame. In this case
        /// we want to make sure any elements we've already consumed are dropped.
        ///
        /// Because the elements we've already pulled are in `MaybeUninit`s, they aren't
        /// dropped without this glue.
        struct PartiallyConsumedIteratorDropGuard<'a, T, const N: usize> {
            consumed: usize,
            finished: bool,
            arr: &'a mut [MaybeUninit<T>; N],
        }

        impl<T, const N: usize> PartiallyConsumedIteratorDropGuard<'_, T, N> {
            #[inline]
            fn put(&mut self, i: T) {
                // Panicking here will cause this instance's destructor to run and for locals in this scope to
                // have their destructors run.
                //
                // Because this is prior to `i` bring moved into a MaybeUninit, it's destructor will run.
                // Previously extracted elements will also have their destructors run correctly when `self`
                // is destructed.
                if self.consumed == N {
                    panic!(
                        "too many elements in the source iterator! expected: {} elements",
                        N
                    );
                }

                self.arr[self.consumed].write(i);
                self.consumed += 1;
            }
        }

        impl<T, const N: usize> Drop for PartiallyConsumedIteratorDropGuard<'_, T, N> {
            fn drop(&mut self) {
                // If we finished gracefully (i.e. we're not running this Drop impl because
                // we're panicking) and got exactly the right number of elements, all is well.
                if self.finished && self.consumed == N {
                    return;
                }

                // If we *are* panicking it doesn't matter how many elements we have. We want
                // to do the cleanup and exit.
                //
                // If we're not panicking but didn't get enough elements we want to panick here
                // so that our `arr` isn't used as an actual fully initialized array.
                //
                // Note that `finished` is actually redundant; we could just check `std::thread::panicking`.
                // However we choose not to because:
                //  - `#![no_std]` support
                //  - it seems prudent to avoid the (potential) TLS access that comes with `panicking()`; even
                //    with the optimiziations that it has it seems likely that the compiler will produce better
                //    code for this scheme that uses a boolean to track if we're "voluntarily" running Drop or not

                // We don't need to guard against panics in the Drop impls for T; if we
                // panic while we're here (i.e. already panicking) the process will just
                // abort.
                for i in 0..self.consumed {
                    unsafe { std::ptr::drop_in_place(self.arr[i].as_mut_ptr()) }
                }

                if self.finished && self.consumed != N {
                    panic!(
                        "not enough elements in the source iterator! expected: {}, got {} elements",
                        N, self.consumed
                    );
                }
            }
        }

        let mut sink = PartiallyConsumedIteratorDropGuard {
            consumed: 0,
            finished: false,
            arr: &mut arr,
        };
        self.for_each(|i| sink.put(i));
        sink.finished = true;

        drop(sink);
        unsafe { (&arr as *const _ as *const [Self::Item; N]).read() }
    }
}

pub trait Collectible<I: Iterator> {
    type Out;

    fn gather(it: I) -> Self::Out;
}

pub struct Arr<const N: usize>;
impl<I: Iterator, const N: usize> Collectible<I> for Arr<N> {
    type Out = [I::Item; N];

    fn gather(it: I) -> [I::Item; N] {
        it.arr_collect()
    }
}

pub struct Tuple<const N: usize>;

macro_rules! tuple_impl {
    (($($t:tt)*)) => {
        triangle!(($($t)*) | tuple_impl (@));
    };
    (@ $($t:tt)*) => {
        impl<I: Iterator> Collectible<I> for Tuple<{count!($($t)*)}> {
            type Out = ($(
                cdr!($t I::Item)
            ,)*);

            #[allow(clippy::unused_unit, non_snake_case)]
            fn gather(it: I) -> Self::Out {
                let [$($t),*] = it.arr();
                ($(
                    $t
                ,)*)
            }
        }
    };
}

// trace_macros!(true);
tuple_impl!((A B C D E F));

pub trait IterCollectExt: Iterator + Sized {
    fn reshape<C: Collectible<Self>>(self) -> C::Out {
        C::gather(self)
    }

    fn arr<const N: usize>(self) -> <Arr<N> as Collectible<Self>>::Out {
        self.reshape::<Arr<N>>()
    }

    fn tuple<const N: usize>(self) -> <Tuple<N> as Collectible<Self>>::Out
    where
        Tuple<N>: Collectible<Self>,
    {
        self.reshape::<Tuple<N>>()
    }
}

impl<I: Iterator + Sized> IterCollectExt for I {}
