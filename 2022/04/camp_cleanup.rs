// #![rustfmt::skip]

use aoc::*;
use std::ops::RangeInclusive;

fn is_subset<T>(a: &RangeInclusive<T>, b: &RangeInclusive<T>) -> bool
where
    T: PartialOrd<T>,
{
    (a.start() <= b.start()) && (a.end() >= b.end())
}

fn is_disjoint<T>(a: &RangeInclusive<T>, b: &RangeInclusive<T>) -> bool
where
    T: PartialOrd<T>,
{
    b.start() > a.end()
}

#[rustfmt::skip]
fn main(){sub!(|i| -> (usize, usize) {

    let inp = i.lines().map(|l| {
        let [ one, two ] = &l.split(',').map(|r| {
            let (a, b) = r.split_once('-').unwrap();

            a.parse::<u32>().unwrap()..=b.parse().unwrap()
        }).collect::<Vec<_>>()[..] else {
            panic!()
        };

        (one.clone(), two.clone())
    }).map(|(a, b)| {
        if a.start() > b.start() {
            (b, a)
        } else {
            (a, b)
        }
    });

    let p1 = inp.clone().filter(|(a, b)| is_subset::<u32>(&a, &b) || is_subset::<u32>(&b, &a)).count();
    let p2 = inp.clone().filter(|(a, b)| !is_disjoint(&a, &b)).count();

    (p1, p2)
});}
