#[allow(unused_imports)]
use aoc::*;

use std::collections::HashMap;

// // trait Collectible {
// //     type Out<T>;
// //     fn do<I: Iterator>(it: I) -> Self::Out<I::Item>;
// // }

// trait ArrCollect: Iterator {
//     fn arr_collect<const N: usize>(self) -> [Self::Item; N];
// }

// struct Uninit<T>(*const T);
// impl<T> Uninit<T> {
//     const UNINIT: MaybeUninit<T> = MaybeUninit::uninit();
// }

// impl<It: Iterator> ArrCollect for It {
//     fn arr_collect<const N: usize>(self) -> [Self::Item; N] {
//         // // const U: MaybeUninit<It::Self> = MaybeUninit::uninit();
//         // use std::mem::{transmute, transmute_copy};
//         // let arr: MaybeUninit<[MaybeUninit<Self::Item>; N]> = MaybeUninit::uninit();
//         // let arr: [MaybeUninit<Self::Item>; N] = unsafe { arr.assume_init() };
//         let mut arr: [MaybeUninit<Self::Item>; N] = [Uninit::<Self::Item>::UNINIT; N];

//         /// If we panic while calling `next`, we'll unwind past this frame. In this case
//         /// we want to make sure any elements we've already consumed are dropped.
//         ///
//         /// Because the elements we've already pulled are in `MaybeUninit`s, they aren't
//         /// dropped without this glue.
//         struct PartiallyConsumedIteratorDropGuard<'a, T, const N: usize> {
//             consumed: usize,
//             finished: bool,
//             arr: &'a mut [MaybeUninit<T>; N],
//         }

//         impl<T, const N: usize> PartiallyConsumedIteratorDropGuard<'_, T, N> {
//             #[inline]
//             fn put(&mut self, i: T) {
//                 // Panicking here will cause this instance's destructor to run and for locals in this scope to
//                 // have their destructors run.
//                 //
//                 // Because this is prior to `i` bring moved into a MaybeUninit, it's destructor will run.
//                 // Previously extracted elements will also have their destructors run correctly when `self`
//                 // is destructed.
//                 if self.consumed == N {
//                     panic!(
//                         "too many elements in the source iterator! expected: {} elements",
//                         N
//                     );
//                 }

//                 self.arr[self.consumed].write(i);
//                 self.consumed += 1;
//             }
//         }

//         impl<T, const N: usize> Drop for PartiallyConsumedIteratorDropGuard<'_, T, N> {
//             fn drop(&mut self) {
//                 // If we finished gracefully (i.e. we're not running this Drop impl because
//                 // we're panicking) and got exactly the right number of elements, all is well.
//                 if self.finished && self.consumed == N {
//                     return;
//                 }

//                 // If we *are* panicking it doesn't matter how many elements we have. We want
//                 // to do the cleanup and exit.
//                 //
//                 // If we're not panicking but didn't get enough elements we want to panick here
//                 // so that our `arr` isn't used as an actual fully initialized array.
//                 //
//                 // Note that `finished` is actually redundant; we could just check `std::thread::panicking`.
//                 // However we choose not to because:
//                 //  - `#![no_std]` support
//                 //  - it seems prudent to avoid the (potential) TLS access that comes with `panicking()`; even
//                 //    with the optimiziations that it has it seems likely that the compiler will produce better
//                 //    code for this scheme that uses a boolean to track if we're "voluntarily" running Drop or not

//                 // We don't need to guard against panics in the Drop impls for T; if we
//                 // panic while we're here (i.e. already panicking) the process will just
//                 // abort.
//                 for i in 0..self.consumed {
//                     unsafe { std::ptr::drop_in_place(self.arr[i].as_mut_ptr()) }
//                 }

//                 if self.finished && self.consumed != N {
//                     panic!(
//                         "not enough elements in the source iterator! expected: {}, got {} elements",
//                         N, self.consumed
//                     );
//                 }
//             }
//         }

//         let mut sink = PartiallyConsumedIteratorDropGuard {
//             consumed: 0,
//             finished: false,
//             arr: &mut arr,
//         };
//         self.for_each(|i| sink.put(i));
//         sink.finished = true;

//         drop(sink);
//         unsafe { (&arr as *const _ as *const [Self::Item; N]).read() }
//     }
// }

// trait Collectible<I: Iterator> {
//     type Out;

//     fn gather(it: I) -> Self::Out;
// }

// struct Arr<const N: usize>;
// impl<I: Iterator, const N: usize> Collectible<I> for Arr<N> {
//     type Out = [I::Item; N];

//     fn gather(it: I) -> [I::Item; N] {
//         it.arr_collect()
//     }
// }

// struct Tuple<const N: usize>;
// /// `triangle!((a b c d) | foo (J))`:
// /// ```
// /// foo!(J)
// /// foo!(J a)
// /// foo!(J a b)
// /// foo!(J a b c)
// /// foo!(J a b c d)
// /// ```
// macro_rules! triangle {
//     (( $($i:tt)* ) | $next:tt $( ($($fwd:tt)+) )?) => {
//         triangle!(@ () # ($($i)*) | $next ($($($fwd)+)?));
//     };

//     (@ ($($consumed:tt)*) # ($t:tt $($rest:tt)*) | $next:tt ($($f:tt)*)) => {
//         $next!($($f)* $($consumed)*);
//         triangle!(@ ($($consumed)* $t) # ($($rest)*) | $next ($($f)*));
//     };

//     (@ ($($consumed:tt)*) # () | $next:tt ($($f:tt)*)) => {
//         $next!($($f)* $($consumed)*);
//     };
// }

// macro_rules! count {
//     ($a:tt $($t:tt)*) => {
//         (1 + count!($($t)*))
//     };
//     () => { 0 }
// }

// macro_rules! cdr {
//     ($a:tt $($b:tt)+) => { $($b)+ }
// }

// macro_rules! tuple_impl {
//     (($($t:tt)*)) => {
//         triangle!(($($t)*) | tuple_impl (@));
//     };
//     (@ $($t:tt)*) => {
//         impl<I: Iterator> Collectible<I> for Tuple<{count!($($t)*)}> {
//             type Out = ($(
//                 cdr!($t I::Item)
//             ,)*);

//             fn gather(it: I) -> Self::Out {
//                 let [$($t),*] = it.arr();
//                 ($(
//                     $t
//                 ,)*)
//             }
//         }
//     };
// }

// // trace_macros!(true);
// tuple_impl!((A B C D E F));

// trait Ext: Iterator + Sized {
//     fn reshape<C: Collectible<Self>>(self) -> C::Out {
//         C::gather(self)
//     }

//     fn arr<const N: usize>(self) -> <Arr<N> as Collectible<Self>>::Out {
//         self.reshape::<Arr<N>>()
//     }

//     fn tuple<const N: usize>(self) -> <Tuple<N> as Collectible<Self>>::Out
//     where
//         Tuple<N>: Collectible<Self>,
//     {
//         self.reshape::<Tuple<N>>()
//     }
// }

// impl<I: Iterator + Sized> Ext for I {}

// fn main() {
//     println!("Hello, world!");

//     struct F {
//         a: [u8; 3],
//         b: u8,
//     }
//     impl Drop for F {
//         fn drop(&mut self) {
//             self.a = [0, 0, 0]
//         }
//     }
//     let j = F { a: [3, 0, 0], b: 3 };
//     let F { a, .. } = j;

//     // let [_, _, _, _]: [u8; 4] = (0..3).reshape::<Arr<4>>();
//     let [a, b, c, d] = (0..4).arr();
//     println!("{} {} {} {}", a, b, c, d);
//     let (_, _, c, _) = (0..4).tuple::<4>();

//     println!("wef");
// }

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new(2021, 14);
    let input: String = aoc.get_input();

    // let input = "NNCB

    // CH -> B
    // HH -> N
    // CB -> H
    // NH -> C
    // HB -> C
    // HC -> B
    // HN -> C
    // NN -> C
    // BH -> H
    // NC -> B
    // NB -> B
    // BN -> B
    // BB -> N
    // BC -> B
    // CC -> N
    // CN -> C";

    let mut input = input.lines();

    let template: Vec<char> = input.next().map(|t| t.chars().collect()).unwrap();
    input.next();

    let rules: HashMap<(char, char), char> = input
        .map(|l| {
            // let [inp, _, res]: [&str; 3] = l.split(" ").collect::<Vec<_>>().try_into().unwrap();
            let [inp, _, res] = l.split(' ').arr();
            let [p1, p2] = inp.chars().arr();
            let [res] = res.chars().arr();
            ((p1, p2), res)
        })
        .collect();

    // let graphviz: String = rules.iter().map(|((a, b), c)| {
    //     format!("  {a}{b} -> {a}{b};\n  {a}{b} -> {b}{c};", a = a, b = b, c = c)
    // }).collect();
    // let graphviz = format!("digraph G {{\n{}\n}}", graphviz);
    // print!("{}", graphviz);

    // let mut w = template.clone();
    // for i in 0..10 {
    //     w = step_naive(&*w, &rules);
    // }

    // let counts = w.iter().counts();
    // let ((min, min_c), (max, max_c)) = counts.iter().minmax_by_key(|(k, v)| **v).into_option().unwrap();
    // let p1 = max_c - min_c;
    // println!("{}", p1);

    // print!("{}", p1);
    aoc.submit_p1(dbg!(solve(&template, &rules, 10)));
    aoc.submit_p2(dbg!(solve(&template, &rules, 40)));
}

// fn step_naive(inp: &[char], rules: &HashMap<(char, char), char>) -> Vec<char> {
//     // let mut out = Vec::with_capacity(inp.len() * 2 - 1);

//     inp.windows(2) // I want `array_windows`!
//         .flat_map(|w| [w[0], rules[&(w[0], w[1])]])
//         .chain(iter::once(*inp.last().unwrap()))
//         .collect()
// }

fn solve(template: &[char], rules: &HashMap<(char, char), char>, steps: usize) -> usize {
    let mut state = HashMap::with_capacity(rules.len());
    for w in template.windows(2) {
        *state.entry((w[0], w[1])).or_default() += 1;
    }

    for _ in 0..steps {
        state = step(&state, rules);
    }

    let mut counts: HashMap<char, usize> = HashMap::new();
    for ((a, b), count) in state.iter() {
        *counts.entry(*a).or_default() += count;
        *counts.entry(*b).or_default() += count;
    }

    // Everything is double counted except for the first and last char:
    *counts.get_mut(template.last().unwrap()).unwrap() += 1;
    *counts.get_mut(template.first().unwrap()).unwrap() += 1;

    let ((_, min_c), (_, max_c)) = counts
        .iter()
        .minmax_by_key(|(_, c)| **c)
        .into_option()
        .unwrap();
    (max_c - min_c) / 2
}

fn step(
    state: &HashMap<(char, char), usize>,
    rules: &HashMap<(char, char), char>,
) -> HashMap<(char, char), usize> {
    let mut out = HashMap::with_capacity(state.len());
    for ((a, b), count) in state.iter() {
        let c = rules[&(*a, *b)];
        *out.entry((*a, c)).or_default() += count;
        *out.entry((c, *b)).or_default() += count;
    }

    out
}

// https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=fcd6f52d0230243d4435349243b2ce5b
