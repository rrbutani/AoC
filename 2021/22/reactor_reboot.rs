#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::*;
use num_traits::{One, Zero};

use std::{
    collections::{HashMap, HashSet},
    fmt,
    mem::{replace, take},
    ops::{Add, Mul, RangeInclusive, Sub},
};

#[derive(Clone, PartialEq, Eq)]
struct ThreeDimensionalRange<T> {
    x: RangeInclusive<T>,
    y: RangeInclusive<T>,
    z: RangeInclusive<T>,
}

impl<T: Debug> Debug for ThreeDimensionalRange<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{}",
            format!(
                "ThreeDimensionalRange {{ x: {:?}, y: {:?}, z: {:?} }}",
                self.x, self.y, self.z
            )
        )
    }
}

impl<T: PartialOrd<T> + Clone> ThreeDimensionalRange<T> {
    fn bound(&mut self, r: RangeInclusive<T>) {
        fn limit<T: PartialOrd<T> + Clone>(inp: &mut RangeInclusive<T>, lim: &RangeInclusive<T>) {
            let (mut l, mut u) = inp.clone().into_inner();
            if lim.start() > &l {
                l = lim.start().clone();
            }
            if lim.end() < &u {
                u = lim.end().clone();
            }

            *inp = l..=u;
        }

        limit(&mut self.x, &r);
        limit(&mut self.y, &r);
        limit(&mut self.z, &r);
    }
}

impl<T> ThreeDimensionalRange<T>
where
    RangeInclusive<T>: Iterator<Item = T>,
    T: Clone,
{
    fn iter(&self) -> impl Iterator<Item = (T, T, T)> {
        self.x
            .clone()
            .cartesian_product(self.y.clone())
            .cartesian_product(self.z.clone())
            .map(|((x, y), z)| (x, y, z))
    }
}

impl<T: PartialOrd<T>> ThreeDimensionalRange<T> {
    fn contains(&self, (x, y, z): (T, T, T)) -> bool {
        self.x.contains(&x) && self.y.contains(&y) && self.z.contains(&z)
    }
}

impl<T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Copy + Zero + One + Ord>
    ThreeDimensionalRange<T>
{
    fn count(&self) -> T {
        if self.x.is_empty() || self.y.is_empty() || self.z.is_empty() {
            return T::zero();
        }

        let (lx, ux) = self.x.clone().into_inner();
        let (ly, uy) = self.y.clone().into_inner();
        let (lz, uz) = self.z.clone().into_inner();
        (ux - lx + T::one()) * (uy - ly + T::one()) * (uz - lz + T::one())
    }

    fn intersect(&self, other: &Self) -> Self {
        fn range_intersect<T: Copy + Ord + Sub<Output = T>>(
            un: &RangeInclusive<T>,
            deux: &RangeInclusive<T>,
        ) -> RangeInclusive<T> {
            (*un.start()).max(*deux.start())..=(*un.end()).min(*deux.end())
        }

        Self {
            x: range_intersect(&self.x, &other.x),
            y: range_intersect(&self.y, &other.y),
            z: range_intersect(&self.z, &other.z),
        }
    }
}

impl<T: FromStr> FromStr for ThreeDimensionalRange<T> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let [x, y, z]: [&str; 3] = s.split(',').collect::<Vec<_>>().try_into().unwrap();

        fn incl_range_parse<T: FromStr>(
            s: &str,
        ) -> Result<(RangeInclusive<T>, &str), Option<T::Err>> {
            let (axis, range) = s.split_once("=").ok_or(None)?;
            let (lower, upper) = range.split_once("..").ok_or(None)?;

            Ok((
                lower.parse().map_err(Some)?..=upper.parse().map_err(Some)?,
                axis,
            ))
        }

        let (x, x_tag) = incl_range_parse(x).map_err(|_| ())?;
        assert_eq!(x_tag, "x");
        let (y, y_tag) = incl_range_parse(y).map_err(|_| ())?;
        assert_eq!(y_tag, "y");
        let (z, z_tag) = incl_range_parse(z).map_err(|_| ())?;
        assert_eq!(z_tag, "z");

        Ok(Self { x, y, z })
    }
}

#[derive(Debug, Clone)]
struct RebootStep {
    on: bool,
    range: ThreeDimensionalRange<isize>,
}

impl FromStr for RebootStep {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (on, range) = s.split_once(" ").unwrap();

        let on = match on {
            "on" => true,
            "off" => false,
            _ => return Err(()),
        };

        Ok(RebootStep {
            on,
            range: range.parse()?,
        })
    }
}

impl RebootStep {
    /// Returns the old step's non-overlapping parts, the overlap, and the new
    /// step's non-overlapping parts.
    fn layer(self, other: Self) -> (Vec<Self>, Option<Self>, Vec<Self>) {
        // // In the best case (one is a subset of the other + same polarity) we end up with 1 range.
        // // In the worst case (subset + different polarity) we end up with

        // // If the ranges are the same, the newer step takes precedence, regardless of polarity:
        // if self.range == other.range {
        //     return (vec![], Some(other), vec![]);
        // }

        // eprintln!();
        // dbg!(&self, &other);
        // eprintln!();

        // No matter the inputs, the overlap region must be a single 3D range.
        // Let's find this range first:
        let overlap = self.range.intersect(&other.range);

        // If the overlap region is empty, there's nothing for us to do.
        if overlap.count() == 0 {
            return (vec![self], None, vec![other]);
        }

        // If the overlap region exactly matches the _older_ step, we can discard the older
        // step:
        //
        // Note that we return all of other as "unoverlapping"; this allows the layer-er to
        // potentially _subsume_ multiple older blocks into this block.
        //
        // We're guaranteed not to have the overlapping part overlap with anything
        // deeper in the layers and if it's overlapping by the time we hit the bottom it'll be
        // treated as effectively overlapping (i.e. new) anyways.
        //
        // This is a helpful optimization because it lets the layer-er clean up older layers
        // and keep from unecessarily fragmenting new ones.
        if overlap == self.range {
            return (vec![], None, vec![other]);
        }

        // |----------------------|
        // |                      |
        // |                      |
        // |    1          2      |
        // |                      |
        // |           |----------|-----------|
        // |           |          |           |
        // |    3      |   4      |           |
        // |           |          |           |
        // |-----------|----------|           |
        //             |                      |
        //             |                      |
        //             |                      |
        //             |                      |
        //             |----------------------|
        //
        // → x, ↓ y
        //
        // x: |----x1------&----x2-----|
        // y: |----y1------&----y2-----|
        // x2 * y2 (4) -> overlap region
        // x1 * y1 (1), x1 * y2 (3), x2 * y1 (2) -> non-overlap regions in the first rectangle
        //
        // we can actually fold (1) and (2) into a single rect (or (1) and (3)) but we're going
        // to ignore this for now
        //
        // ranges actually get split into three parts:
        // |--------------------------------------------|
        // |                                            |
        // |                                            |
        // |    1                 2               3     |
        // |                                            |
        // |           |----------------------|         |
        // |           |                      |         |
        // |    4      |          5           |   6     |
        // |           |                      |         |
        // |           |----------------------|         |
        // |                                            |
        // |    7                 8               9     |
        // |                                            |
        // |--------------------------------------------|
        //
        // x: |----x1---&----x2-----&----x3---|
        // x: |----y1---&----y2-----&----y3---|
        // x2 & y2 -> overlap region
        // x1 & y1 -> (1)
        // x2 & y1 -> (2)
        // x3 & y1 -> (3)
        // x1 & y2 -> (4)
        // x3 & y2 -> (6)
        // x1 & y3 -> (7)
        // x2 & y3 -> (8)
        // x3 & y3 -> (9)
        //
        // again, we can actually make fewer ranges (5 in this case), but to keep the implementation
        // simple we're not going to
        //
        // this approach generalizes to three dimensions.

        let find_axis_parts = |actual: &RangeInclusive<_>, overlap: &RangeInclusive<_>| {
            let (start, end) = actual.clone().into_inner();
            let (o_start, o_end) = overlap.clone().into_inner();

            let left = start..=(o_start - 1);
            let right = (o_end + 1)..=end;
            [left, overlap.clone(), right]
        };

        let non_overlapping_remnants = |step: &RebootStep| {
            find_axis_parts(&step.range.x, &overlap.x)
                .into_iter()
                .cartesian_product(find_axis_parts(&step.range.y, &overlap.y).into_iter())
                .cartesian_product(find_axis_parts(&step.range.z, &overlap.z).into_iter())
                .map(|((x, y), z)| RebootStep {
                    range: ThreeDimensionalRange { x, y, z },
                    on: step.on,
                })
                .filter(|s| s.range != overlap)
                .filter(|s| s.range.count() != 0)
                .collect::<Vec<_>>()
        };

        (
            non_overlapping_remnants(&self),
            Some(RebootStep {
                range: overlap.clone(),
                on: other.on,
            }),
            non_overlapping_remnants(&other),
        )
    }
}

// #[derive(Debug, Clone, Copy)]
// struct MapParseIter<I, T>(PhantomData<T>, I);

// impl<'a, I: Iterator<Item = &'a str>, T: FromStr> Iterator for MapParseIter<I, T>
// where
//     T::Err: Debug,
// {
//     type Item = T;

//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         self.1.next().map(|i| i.parse().unwrap())
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.1.size_hint()
//     }

//     fn count(self) -> usize
//     where
//         Self: Sized,
//     {
//         self.1.count()
//     }
// }

// #[derive(Debug, Clone)]
// struct MapTryIntoIter<I, T>(PhantomData<T>, I);

// impl<I: Iterator, T: TryFrom<I::Item>> Iterator for MapTryIntoIter<I, T>
// where
//     T::Error: Debug,
// {
//     type Item = T;

//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         self.1.next().map(|i| i.try_into().unwrap())
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.1.size_hint()
//     }

//     fn count(self) -> usize
//     where
//         Self: Sized,
//     {
//         self.1.count()
//     }
// }

// trait IterMapExt: Sized + Iterator {
//     fn map_parse<'a, T>(self) -> MapParseIter<Self, T>
//     where
//         Self: Iterator<Item = &'a str>,
//         T: FromStr,
//         T::Err: Debug,
//     {
//         MapParseIter(PhantomData, self)
//     }

//     fn map_into<T>(self) -> MapTryIntoIter<Self, T>
//     where
//         T: TryFrom<Self::Item>,
//         T::Error: Debug,
//     {
//         MapTryIntoIter(PhantomData, self)
//     }
// }

// impl<I: Iterator + Sized> IterMapExt for I {}

struct RangeFlattener {
    /// except, time instead of the z-axis
    layers: Vec<Vec<RebootStep>>,
}

impl RangeFlattener {
    fn new(steps: &[RebootStep]) -> Self {
        let mut layers: Vec<Vec<RebootStep>> = Vec::with_capacity(steps.len());

        let mut step_count = 0;
        for s in steps.iter() {
            step_count += 1;
            // println!("{}", step_count);
            // dbg!(&layers);
            // eprintln!("placing: {:?}", &s);
            let mut remaining_new_fragments = vec![(s.clone(), 0)];
            let mut new_layer = vec![];

            // layer on this new step, breaking older "layers" to not overlap
            // with this new step until we run out of non-overlapping fragments
            // (i.e. until each fragment has a "home")
            while let Some((f, min_layer_depth)) = remaining_new_fragments.pop() {
                // println!("{:3}: {}", step_count, remaining_new_fragments.len());
                // eprintln!(
                // "===================================================================\n{{"
                // );
                // eprintln!("  homing: {:?} into layer {}\n", f.range, min_layer_depth);
                let mut remaining_fragments_from_new_fragment = vec![f.clone()]; // a staging area to keep us from duplicating fragments

                // get the layer below where this fragment originated from and layer it on there:
                if let Some((depth, l)) = layers.iter_mut().rev().enumerate().nth(min_layer_depth) {
                    // we're going to rewrite everything in this layer to *not* overlap with the fragment

                    // eprintln!("layer {} at start: {:#?}", min_layer_depth, l);
                    // let mut count = 0;

                    // Need to layer every new fragment with everything in this layer before letting them
                    // move further down.
                    while let Some(frag) = remaining_fragments_from_new_fragment.pop() {
                        let current_layer = replace(l, Vec::with_capacity(l.len()));
                        let mut current_layer_iter = current_layer.into_iter();

                        // if a fragment goes through all the things in this layer without any overlap it's
                        // ready for the next layer:
                        let mut had_overlap = false;

                        while let Some(step) = current_layer_iter.next() {
                            let (old, overlap, new) = step.layer(frag.clone());
                            l.extend(old);

                            // any overlap we have is considered settled; it can't
                            // possibly overlap with anything below since this is
                            // the property we're maintaining
                            if let Some(o) = overlap {
                                new_layer.push(o.clone());

                                // drop the settled region from any fragments remaining for processing
                                for f in take(&mut remaining_fragments_from_new_fragment) {
                                    let (old, _, _) = f.layer(o.clone());
                                    remaining_fragments_from_new_fragment.extend(old);
                                }

                                had_overlap = true;

                                // if we got a hit, we need to restart using the remnants from the frag, if any:
                                remaining_fragments_from_new_fragment.extend(new);

                                // put all the unprocessed parts of the current layer back in place:
                                l.extend(current_layer_iter);

                                // and then break:
                                break;
                            } else {
                                // otherwise we continue and just try this same fragment on the next thing in this layer.
                            }
                        }

                        if !had_overlap {
                            remaining_new_fragments.push((frag, depth + 1));
                        }
                    }

                /*                     // layer onto each thing in the layer:
                for step in replace(l, Vec::with_capacity(l.len())) {
                    let frag = if let Some(f) = remaining_fragments_from_new_fragment.pop() {
                        f
                    } else {
                        eprintln!("\n  no more unhomed fragments from this fragment; moving to the next new fragment");
                        break;
                    };

                    eprintln!(
                        "\n  step[{}];\n      layering {:?}\n      onto     {:?}",
                        count, frag, step
                    );
                    count += 1;

                    let (old, overlap, new) = step.layer(frag.clone());
                    eprintln!("\n  updated old: {:#?}", old);
                    eprintln!("  overlap: {:?}", overlap);
                    eprintln!("  remaining from new: {:#?}", new);

                    // non-overlapping bits from the layer constitute the updated layer
                    l.extend(old);
                    // dbg!(depth, &l);

                    // any overlap we have is considered settled; it can't
                    // possibly overlap with anything below since this is
                    // the property we're maintaining
                    if let Some(o) = overlap {
                        new_layer.push(o.clone());

                        // drop the settled region from any fragments remaining for processing
                        for f in take(&mut remaining_fragments_from_new_fragment) {
                            let (old, _, _) = f.layer(o.clone());
                            remaining_fragments_from_new_fragment.extend(old);
                        }
                    }

                    // any non-overlapping bits from the fragment still need
                    // to find a home; we'll add them to the queue
                    remaining_fragments_from_new_fragment.extend(new);
                }

                eprintln!("\n\nlayer {} at end: {:#?}", min_layer_depth, l);

                // anything still left here at the end of this pass should be layered onto the new layer
                remaining_new_fragments.extend(
                    remaining_fragments_from_new_fragment
                        .into_iter()
                        .map(|s| (s, depth + 1)),
                ); */
                } else {
                    // eprintln!("no layer at depth {}!", min_layer_depth);
                    // eprintln!("\nadding {:?} to new layer", f);

                    // if we're out of layers (i.e. if we hit bottom) this
                    // fragment must not overlap with anything. so, we move it
                    // to the top most layer (the new one):
                    new_layer.push(f);
                }

                // eprintln!(
                // "\n\n}}\n==================================================================="
                // );
            }

            // finally, if the step turned things on, push the layer.
            //
            // we can elide layers full of "off"s because we ultimately only
            // need to count the things that are on; we're only really concerned
            // with "off"s that negate a prevoius on which we've accounted for
            // at this point by pushing the off through the layers and
            // fragmenting things
            if s.on {
                // eprintln!(
                // "+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++\n{{"
                // );
                // eprintln!("pushing new layer; removing overlap.");
                // eprintln!("orig: {:#?}", new_layer);
                // need to remove any overlap *within* this layer:
                let mut adjusted_layer: Vec<RebootStep> = Vec::with_capacity(new_layer.len());
                while let Some(new) = new_layer.pop() {
                    let mut made_it_through = true;
                    for existing in adjusted_layer.iter() {
                        let (_, overlap, new_fragments) = existing.clone().layer(new.clone());
                        if overlap.is_some() {
                            new_layer.extend(new_fragments);
                            made_it_through = false;
                            break;
                        }
                    }

                    if made_it_through {
                        adjusted_layer.push(new);
                    }
                }

                // eprintln!("\nadjusted: {:#?}", adjusted_layer);
                layers.push(adjusted_layer);

                // eprintln!(
                //     "\n\n}}\n+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++"
                // );
            } else {
                // we'll push an empty layer so our layers still line up nicely with the steps though
                layers.push(vec![])
            }
            // eprintln!("\n\n");
        }

        // dbg!(&layers);

        Self { layers }
    }

    fn on(&self) -> usize {
        self.layers
            .iter()
            .flat_map(|layer| layer.iter())
            .map(|s| s.range.count())
            .sum::<isize>()
            .try_into()
            .unwrap()
    }

    fn dump(&self) -> impl Iterator<Item = (isize, isize, isize)> + '_ {
        self.layers
            .iter()
            .flat_map(|layer| layer.iter())
            .flat_map(|s| s.range.iter())
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2021, 22);
    //     let inp = "\
    // on x=0..1,y=1..2,z=1..3
    // on x=0..2,y=1..2,z=1..2
    // on x=0..2,y=1..2,z=0..1
    // ";
    // on x=-22..28,y=-29..23,z=-38..16
    //     let inp = "\
    // on x=2..2,y=1..2,z=0..4
    // on x=2..2,y=1..2,z=3..5
    // off x=2..2,y=1..2,z=1..2
    // ";

    // on x=0..2,y=0..4,z=0..3
    // on x=0..3,y=2..3,z=2..5
    // on x=0..4,y=1..3,z=1..4

    let inp = "\
on x=-20..26,y=-36..17,z=-47..7
on x=-20..33,y=-21..23,z=-26..28
on x=-22..28,y=-29..23,z=-38..16
on x=-46..7,y=-6..46,z=-50..-1
on x=-49..1,y=-3..46,z=-24..28
on x=2..47,y=-22..22,z=-23..27
on x=-27..23,y=-28..26,z=-21..29
on x=-39..5,y=-6..47,z=-3..44
on x=-30..21,y=-8..43,z=-13..34
on x=-22..26,y=-27..20,z=-29..19
off x=-48..-32,y=26..41,z=-47..-37
on x=-12..35,y=6..50,z=-50..-2
off x=-48..-32,y=-32..-16,z=-15..-5
";

    //     let inp = "\
    // on x=-20..26,y=-36..17,z=-47..7
    // on x=-20..33,y=-21..23,z=-26..28
    // on x=-22..28,y=-29..23,z=-38..16
    // on x=-46..7,y=-6..46,z=-50..-1
    // on x=-49..1,y=-3..46,z=-24..28
    // on x=2..47,y=-22..22,z=-23..27
    // on x=-27..23,y=-28..26,z=-21..29
    // on x=-39..5,y=-6..47,z=-3..44
    // on x=-30..21,y=-8..43,z=-13..34
    // on x=-22..26,y=-27..20,z=-29..19
    // off x=-48..-32,y=26..41,z=-47..-37
    // on x=-12..35,y=6..50,z=-50..-2
    // off x=-48..-32,y=-32..-16,z=-15..-5
    // on x=-18..26,y=-33..15,z=-7..46
    // off x=-40..-22,y=-38..-28,z=23..41
    // on x=-16..35,y=-41..10,z=-47..6
    // off x=-32..-23,y=11..30,z=-14..3
    // on x=-49..-5,y=-3..45,z=-29..18
    // off x=18..30,y=-20..-8,z=-3..13
    // on x=-41..9,y=-7..43,z=-33..15
    // on x=-54112..-39298,y=-85059..-49293,z=-27449..7877
    // on x=967..23432,y=45373..81175,z=27513..53682
    // ";

    //     let inp = "\
    // on x=-5..47,y=-31..22,z=-19..33
    // on x=-44..5,y=-27..21,z=-14..35
    // on x=-49..-1,y=-11..42,z=-10..38
    // on x=-20..34,y=-40..6,z=-44..1
    // off x=26..39,y=40..50,z=-2..11
    // on x=-41..5,y=-41..6,z=-36..8
    // off x=-43..-33,y=-45..-28,z=7..25
    // on x=-33..15,y=-32..19,z=-34..11
    // off x=35..47,y=-46..-34,z=-11..5
    // on x=-14..36,y=-6..44,z=-16..29
    // on x=-57795..-6158,y=29564..72030,z=20435..90618
    // on x=36731..105352,y=-21140..28532,z=16094..90401
    // on x=30999..107136,y=-53464..15513,z=8553..71215
    // on x=13528..83982,y=-99403..-27377,z=-24141..23996
    // on x=-72682..-12347,y=18159..111354,z=7391..80950
    // on x=-1060..80757,y=-65301..-20884,z=-103788..-16709
    // on x=-83015..-9461,y=-72160..-8347,z=-81239..-26856
    // on x=-52752..22273,y=-49450..9096,z=54442..119054
    // on x=-29982..40483,y=-108474..-28371,z=-24328..38471
    // on x=-4958..62750,y=40422..118853,z=-7672..65583
    // on x=55694..108686,y=-43367..46958,z=-26781..48729
    // on x=-98497..-18186,y=-63569..3412,z=1232..88485
    // on x=-726..56291,y=-62629..13224,z=18033..85226
    // on x=-110886..-34664,y=-81338..-8658,z=8914..63723
    // on x=-55829..24974,y=-16897..54165,z=-121762..-28058
    // on x=-65152..-11147,y=22489..91432,z=-58782..1780
    // on x=-120100..-32970,y=-46592..27473,z=-11695..61039
    // on x=-18631..37533,y=-124565..-50804,z=-35667..28308
    // on x=-57817..18248,y=49321..117703,z=5745..55881
    // on x=14781..98692,y=-1341..70827,z=15753..70151
    // on x=-34419..55919,y=-19626..40991,z=39015..114138
    // on x=-60785..11593,y=-56135..2999,z=-95368..-26915
    // on x=-32178..58085,y=17647..101866,z=-91405..-8878
    // on x=-53655..12091,y=50097..105568,z=-75335..-4862
    // on x=-111166..-40997,y=-71714..2688,z=5609..50954
    // on x=-16602..70118,y=-98693..-44401,z=5197..76897
    // on x=16383..101554,y=4615..83635,z=-44907..18747
    // off x=-95822..-15171,y=-19987..48940,z=10804..104439
    // on x=-89813..-14614,y=16069..88491,z=-3297..45228
    // on x=41075..99376,y=-20427..49978,z=-52012..13762
    // on x=-21330..50085,y=-17944..62733,z=-112280..-30197
    // on x=-16478..35915,y=36008..118594,z=-7885..47086
    // off x=-98156..-27851,y=-49952..43171,z=-99005..-8456
    // off x=2032..69770,y=-71013..4824,z=7471..94418
    // on x=43670..120875,y=-42068..12382,z=-24787..38892
    // off x=37514..111226,y=-45862..25743,z=-16714..54663
    // off x=25699..97951,y=-30668..59918,z=-15349..69697
    // off x=-44271..17935,y=-9516..60759,z=49131..112598
    // on x=-61695..-5813,y=40978..94975,z=8655..80240
    // off x=-101086..-9439,y=-7088..67543,z=33935..83858
    // off x=18020..114017,y=-48931..32606,z=21474..89843
    // off x=-77139..10506,y=-89994..-18797,z=-80..59318
    // off x=8476..79288,y=-75520..11602,z=-96624..-24783
    // on x=-47488..-1262,y=24338..100707,z=16292..72967
    // off x=-84341..13987,y=2429..92914,z=-90671..-1318
    // off x=-37810..49457,y=-71013..-7894,z=-105357..-13188
    // off x=-27365..46395,y=31009..98017,z=15428..76570
    // off x=-70369..-16548,y=22648..78696,z=-1892..86821
    // on x=-53470..21291,y=-120233..-33476,z=-44150..38147
    // off x=-93533..-4276,y=-16170..68771,z=-104985..-24507
    // ";

    let inp = aoc.get_input();
    let steps: Vec<RebootStep> = inp.lines().map_parse().collect();

    let mut p1_steps = steps.clone();
    p1_steps.iter_mut().for_each(|s| s.range.bound(-50..=50));

    // naive:
    // let region = [[[false; 101]; 101]; 101];
    // for s in p1_steps {
    //     for (x, y, z) in s.iter() {
    //         region[x][y][z] = s.on;
    //     }
    // }

    let mut actual = HashSet::new();
    let mut point_to_step_map = HashMap::new();

    // better would be to do `(-50..=50)` x `(-50..=50)` x `(-50..=50)`
    // (i.e. per coord) and stop the search through the steps, backwards when we hit
    // a step
    let mut count = 0;
    let r = -50..=50;
    for (x, y, z) in (ThreeDimensionalRange {
        x: r.clone(),
        y: r.clone(),
        z: r,
    }
    .iter())
    {
        let on = p1_steps
            .iter()
            .enumerate()
            .rev()
            .filter(|(_, s)| s.range.contains((x, y, z)))
            .map(|(i, s)| {
                point_to_step_map.insert((x, y, z), i);
                s.on
            })
            .next()
            .unwrap_or(false);

        if on {
            actual.insert((x, y, z));
            // println!("{:4} {:4} {:4}", x, y, z);
            count += 1;
        }

        // if (9..=13).contains(&x) && (9..=13).contains(&y) && (9..=13).contains(&z) {
        //     // dbg!((x, y, z), on);
        //     eprintln!("({:3}, {:3}, {:3}) => {}", x, y, z, on);
        // }
    }

    // aoc.submit_p1(count).unwrap();

    // let p1 = RangeFlattener::new(&p1_steps);
    // let got: HashSet<_> = p1.dump().collect();
    // let count_l = p1.dump().count();
    // println!(
    //     "count:          {}\ncount, dedupped: {}",
    //     count_l,
    //     got.len()
    // );

    // if count_l != got.len() {
    //     let mut counts = HashMap::<_, usize>::new();
    //     for i in p1.dump() {
    //         *counts.entry(i).or_default() += 1;
    //     }

    //     counts.iter().filter(|(_, c)| **c > 1).for_each(|(k, c)| {
    //         let mut occurences = vec![];
    //         for (layer_idx, layer) in p1.layers.iter().enumerate() {
    //             for (s_idx, s) in layer.iter().enumerate() {
    //                 if s.range.contains(*k) {
    //                     occurences.push((layer_idx, s_idx));
    //                 }
    //             }
    //         }
    //         println!("{:?} repeated {} times ({:?})", k, c, occurences);
    //     })
    // }

    // println!(
    //     "\nin actual; missing from got: {:#?}",
    //     actual.difference(&got)
    // );
    // for c in actual.difference(&got) {
    //     println!("{:?} was set in step {}", c, point_to_step_map[c]);
    // }

    // println!("\nin got, wrongly so: {:#?}", got.difference(&actual));

    // let p1 = p1.on();
    // println!();
    // println!("{}", count);
    // println!("{} (ddp: {})", p1, got.len());

    let p2 = RangeFlattener::new(&steps);
    let p2 = p2.on();
    println!();
    println!("{}", p2);
    aoc.submit_p2(p2).unwrap();

    // aoc.submit_p2(horiz * depth).unwrap();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bounding() {
        let mut r: ThreeDimensionalRange<_> = "x=-54112..-39298,y=-85059..-49293,z=-27449..7877"
            .parse()
            .unwrap();

        r.bound(-50..=50);

        dbg!(r);

        Err::<u8, _>(()).unwrap();
    }
}
