use std::{collections::HashMap, iter::repeat_n, mem};

use aoc::*;
use fxhash::FxHashMap;
use grid::Direction;

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Numeric {
    _0, _1, _2, _3, _4, _5, _6, _7, _8, _9, _A,
}
impl TryFrom<char> for Numeric {
    type Error = char;
    #[rustfmt::skip]
    fn try_from(value: char) -> Result<Self, Self::Error> {
        use Numeric::*;
        Ok(match value {
            '0' => _0, '1' => _1, '2' => _2, '3' => _3, '4' => _4, '5' => _5,
            '6' => _6, '7' => _7, '8' => _8, '9' => _9, 'A' => _A,
            other => return Err(other)
        })
    }
}
impl Numeric {
    #[rustfmt::skip]
    fn as_num(self) -> Option<usize> {
        use Numeric::*;
        Some(match self {
            _0 => 0, _1 => 1, _2 => 2, _3 => 3, _4 => 4, _5 => 5,
            _6 => 6, _7 => 7, _8 => 8, _9 => 9, _A => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Directional {
    Move(Direction),
    Press, // aka `A`
}
impl From<Direction> for Directional {
    fn from(value: Direction) -> Self {
        Self::Move(value)
    }
}

//

// segment is a sequence of operations starting at `A`, ending with a press of
// `A` (TODO...)
type Segment = Vec<Directional>;

// sequence of segments
type Chain = Vec<Segment>;

// choices for chains; alternates; pick one
//
// represents reorderings
type ChainChoices = Vec<Chain>;

// chaining of chains to form a path
type Path = Vec<ChainChoices>;

// // sequence of segments is: a path; segments can only be chained; not reordered
// type Path = Vec<Segment>;

// // bunch of potential paths, with all reorderings flattened out
// //
// // TODO: may be more efficient to push down choices into Path; i.e.
// // `Path = Vec<Segment | Choices of Segment>`? this is simpler though
// type PathChoices = Vec<Path>;

// type PathSegmentChoices = Vec<PathSegment>;

//------------------------------------------------------------------------------

trait Steppable: Copy {
    const EMPTY_SPACE: Coord;
    fn as_coord(self) -> Coord;

    // not technically a segment but whatever..
    //
    // list of sequences that will get you from `self` to `to` (and press `to`)
    #[rustfmt::skip] // stop eating my comments!!! >:-(
    fn step(self, to: Self) -> /* like SegmentChoices */ SmallVec<[Segment; 2]> {
        use Direction::*;

        // will need to move horizontally (cols) and vertically (rows)
        let diff = to.as_coord().as_signed() - self.as_coord().as_signed();
        let Coord { row: d_row, col: d_col } = diff;

        let mut opts = vec![];
        match (d_row, d_col) {
            // if either delta is zero, there's only 1 path to take:
            (0, c) => opts.push(vec![(0, c)]),
            (r, 0) => opts.push(vec![(r, 0)]),
            // otherwise there are potentially two options: horiz then vert or
            // vice versa
            (row, col) => {
                let (this, e) = (self.as_coord(), Some(Self::EMPTY_SPACE));
                // the thing that determines whether a path is viable is whether
                // we hit the empty space while traversing it
                if this.checked_add_signed(Coord { row, col: 0 }) != e {
                    // can do the row (vertical) movement first:
                    opts.push(vec![(row, 0), (0, col)]);
                }

                if this.checked_add_signed(Coord { row: 0, col }) != e {
                    // can do the col (horizontal) movement first:
                    opts.push(vec![(0, col), (row, 0)]);
                }

                debug_assert!(opts.len() >= 1);
            }
        }

        let paths = opts.into_iter().map(|dir_list| {
            dir_list
                .into_iter()
                .flat_map(|(r, c)| match (r, c) {
                    (0, c) if c >= 0 => repeat_n(East.into(), c.unsigned_abs()),
                    (0, c) if c < 0 => repeat_n(West.into(), c.unsigned_abs()),
                    (r, 0) if r >= 0 => repeat_n(South.into(), r.unsigned_abs()),
                    (r, 0) if r < 0 => repeat_n(North.into(), r.unsigned_abs()),
                    _ => unreachable!("{r}, {c}"),
                })
                .chain([Directional::Press])
                .collect::<Vec<_>>()
        });

        SmallVec::from_iter(paths)
    }
}

impl Steppable for Numeric {
    const EMPTY_SPACE: Coord = Coord { row: 3, col: 0 };

    fn as_coord(self) -> Coord {
        use Numeric::*;
        let r = match self {
            _7 | _8 | _9 => 0,
            _4 | _5 | _6 => 1,
            _1 | _2 | _3 => 2,
            /**/ _0 | _A => 3,
        };
        let c = match self {
            _9 | _6 | _3 | _A => 2,
            _8 | _5 | _2 | _0 => 1,
            _7 | _4 | _1 => 0,
        };
        Coord { row: r, col: c }
    }
}

impl Steppable for Directional {
    const EMPTY_SPACE: Coord = Coord { row: 0, col: 0 };

    fn as_coord(self) -> Coord {
        use {Direction as D, Directional::*};
        match self {
            Move(D::Up) => (0usize, 1),
            Press => (0, 2),
            Move(D::Left) => (1, 0),
            Move(D::Down) => (1, 1),
            Move(D::Right) => (1, 2),
        }
        .into()
    }
}

//------------------------------------------------------------------------------

// at the end of a segment everything below L0 is back at `A`..

pub struct NumericKeypad;
impl NumericKeypad {
    fn seq(nums: impl IntoIterator<Item = Numeric>) -> Path {
        let mut out: Path = vec![];
        let mut curr = Numeric::_A;
        for n in nums {
            let mut chain_choices = vec![];

            let segment_choices = curr.step(n).to_vec();
            // must be at least one choice; all choices must have same len
            // (i.e. cost 1-level up)
            debug_assert!(!segment_choices.is_empty());
            debug_assert!(segment_choices
                .iter()
                .all(|s| s.len() == segment_choices[0].len()));

            // each segment here is really... a 1-element chain
            for seg in segment_choices {
                chain_choices.push(vec![seg]);
            }

            out.push(chain_choices); // choices of single element chains
            curr = n;
        }

        out
    }
}

pub struct DirectionalKeypad;
impl DirectionalKeypad {
    // chain and score for all possible reorderings
    //
    // note: should be able to memoize
    // TODO: not sure if this is a win...
    fn seq(movements: &Segment) -> ChainChoices {
        use std::sync::Mutex;
        static CACHE: Mutex<Option<FxHashMap<Segment, ChainChoices>>> = Mutex::new(None);

        let mut cache = CACHE.lock().unwrap();
        if cache.is_none() {
            *cache = Some(FxHashMap::default());
        }
        let cache = cache.as_mut().unwrap();

        if cache.contains_key(movements) {
            return cache[movements].clone();
        }

        let res = Self::seq_(movements);
        cache.insert(movements.clone(), res.clone());
        res
    }

    fn seq_(movements: &Segment) -> ChainChoices {
        // todo: this is inefficient and bad...
        let mut curr = Directional::Press;
        let mut chain_choices: ChainChoices = vec![vec![]];

        for &m in movements {
            let segment_choices = curr.step(m);
            // flatten by doing the cartesian product:
            let curr_chain_choices = mem::replace(&mut chain_choices, vec![]);
            for curr_chain in curr_chain_choices {
                // each segment here is a 1-element chain
                for segment in &segment_choices {
                    let mut chain: Chain = curr_chain.clone();
                    chain.push(segment.clone());
                    chain_choices.push(chain);
                }
            }

            curr = m;
        }
        debug_assert_eq!(curr, Directional::Press);
        chain_choices
    }

    fn tiebreak_chain_choices(chain_choices: ChainChoices) -> Chain {
        use std::sync::Mutex;
        static CACHE: Mutex<Option<FxHashMap<ChainChoices, Chain>>> = Mutex::new(None);

        let mut cache = CACHE.lock().unwrap();
        if cache.is_none() {
            *cache = Some(FxHashMap::default());
        }
        let cache = cache.as_mut().unwrap();

        if cache.contains_key(&chain_choices) {
            return cache[&chain_choices].clone();
        }

        let res = Self::tiebreak_chain_choices_(chain_choices.clone());
        cache.insert(chain_choices, res.clone());
        res
    }

    fn tiebreak_chain_choices_(chain_choices: ChainChoices) -> Chain {
        let chain_choices_with_index = chain_choices
            .clone()
            .into_iter()
            .enumerate()
            .map(|(idx, chain)| (chain, idx))
            .collect();

        #[cfg(debug_assertions)]
        eprintln!("{} options", chain_choices.len());

        let idx = Self::tiebreak_chain_choices_inner(chain_choices_with_index, 0);
        chain_choices[idx].clone()
    }

    fn tiebreak_chain_choices_inner(
        chain_choices_with_index: Vec<(Chain, usize)>,
        depth: usize,
    ) -> usize {
        // todo: tune?
        if depth == 2 {
            // uhhhh
            //
            // todo: prune search space and continue search?
            // todo: be smarter about the recursive search? (subset, parts)
            //
            // for now: assume they're equally matched; return the first item
            return chain_choices_with_index[0].1;
        }

        let chain_score =
            |chain: &Chain| chain.iter().map(|seg: &Segment| seg.len()).sum::<usize>();

        #[cfg(debug_assertions)]
        {
            eprintln!("{} INNER[{depth}] options", chain_choices_with_index.len());
            let mut map: HashMap<usize, Vec<(usize, _)>> = HashMap::new();
            for (ch, idx) in &chain_choices_with_index {
                map.entry(*idx).or_default().push((chain_score(ch), ch));
            }

            for (idx, scores_and_chs) in map.iter() {
                // let scores_only = scores_and_chs.iter().map(|(sc, _)| *sc).collect_vec();
                eprintln!("[{idx}]: {}", scores_and_chs.len());
                #[cfg(debug_assertions)]
                for (_, ch) in scores_and_chs {
                    eprintln!("    + {}", chain_to_str(ch));
                }
            }
            eprintln!()
        }

        if chain_choices_with_index.len() == 1 {
            return chain_choices_with_index[0].1;
        }

        // drive each chain through `seq`, segment by segment, and produce chain
        // choices
        let chain_choices: Vec<(Chain, usize, _, usize)> = chain_choices_with_index
            .into_iter()
            .map(|(chain, index)| {
                let segs_in_chain: Vec<Vec<(Chain, usize)>> = chain
                    .iter()
                    .map(|seg| {
                        let chain_choices_and_score_for_seg: Vec<(Chain, usize)> =
                            DirectionalKeypad::seq(seg)
                                .into_iter()
                                .map(|chain| {
                                    let score = chain_score(&chain);
                                    (chain, score)
                                })
                                .collect();
                        chain_choices_and_score_for_seg
                    })
                    .collect();
                (chain, index, segs_in_chain)
            })
            .map(|(chain, index, segs_nexts)| {
                // for each chain choice, we can figure out it's next level
                // score by:
                // 1. for each segment in the chain, taking the corresponding
                //    next-level chain with the least score
                // 2. adding up all of ^
                let score = segs_nexts
                    .iter()
                    .map(|seg| seg.iter().map(|(_ch, score)| score).min().unwrap())
                    .sum();
                (chain, index, segs_nexts, score)
            })
            .collect();

        // find the best score:
        let best_score = chain_choices
            .iter()
            .map(|(_, _, _, score)| *score)
            .min()
            .unwrap();

        // winnow:
        let chain_choices = chain_choices
            .into_iter()
            .filter(|(_, _, _, score)| *score == best_score)
            .collect_vec();

        // if we've only got 1 remaining entry great; return it:
        if chain_choices.len() == 1 {
            return chain_choices[0].1;
        }

        // if we've only got 1 index remaining, great; we're done:
        //
        // (a note about what to pass our recursive call: technically we can
        // stop once all the "best" results correspond to a single choice up at
        // this level; we don't need to hit a singular best entry at the
        // depth...; that's what this index stuff is here to accommodate)
        if chain_choices
            .iter()
            .all(|(_, idx, _, _)| *idx == chain_choices[0].1)
        {
            return chain_choices[0].1;
        }

        // otherwise, we need to actually flatten out our remaining entries into
        // a list of chain choices and recurse one level deeper to try to
        // tiebreak...
        let chain_choices = chain_choices
            .into_iter()
            .flat_map(|(_, idx, segs_next, _)| {
                let mut next_chains = vec![vec![]];
                // for each seg in this chain, we have `ChainChoices` — several
                // chains to choose from to use in lieu of this segment for the
                // next level
                //
                // we essentially do a cartesian product of the choices
                // available for each segment
                for seg in segs_next {
                    let curr_next_chains = mem::replace(&mut next_chains, vec![]);
                    for existing_chain in curr_next_chains {
                        for (chain_for_seg, _score) in &seg {
                            let mut chain = existing_chain.clone();
                            chain.extend(chain_for_seg.clone());
                            next_chains.push(chain);
                        }
                    }
                }

                next_chains.into_iter().map(move |chain| (chain, idx))
            })
            .filter(|(chain, _idx)| chain_score(chain) == best_score)
            .collect();

        Self::tiebreak_chain_choices_inner(chain_choices, depth + 1)
    }
}

fn chain_to_str(chain: &Chain) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    for seg in chain {
        for m in seg {
            match m {
                Directional::Move(direction) => write!(&mut out, "{direction}"),
                Directional::Press => write!(&mut out, "A"),
            }
            .unwrap()
        }
        write!(&mut out, " ").unwrap();
    }
    out
}

// TODO: can't really effectively memo-ize?
//
// need flattening to score?
//
// recursion?

/* fn drive_one<const N: usize>(nums: impl IntoIterator<Item = Numeric> + Clone) -> usize {
    let mut cost = 0;
    let chain_score = |chain: &Chain| chain.iter().map(|seg: &Segment| seg.len()).sum::<usize>();

    // we can calculate cost for each number's resulting chain independently
    for mut chain_choices in /* path */ NumericKeypad::seq(nums.clone()) {
        let mut next_chain_choices = Vec::with_capacity(chain_choices.len() * 3 / 2);

        // lower each chain through the `N` levels, yielding new chains:
        for n in 0..N {
            dbg!(n, chain_choices.len());
            #[cfg(debug_assertions)]
            {
                for c in &chain_choices {
                    for seg in c {
                        for m in seg {
                            match m {
                                Directional::Move(direction) => eprint!("{direction}"),
                                Directional::Press => eprint!("A"),
                            }
                        }
                        eprint!(" ");
                    }
                    eprintln!();
                }
            }

            // each chain yields new chains of its own; tack these onto
            // `next_chain_choices`:
            for chain in chain_choices.drain(..) {
                let mut new_chains_from_chain: ChainChoices = vec![vec![]]; // todo: sizing

                // a chain is composed of segments...
                for seg in chain {
                    // when lowered, each produces `ChainChoices`:
                    let seg_lowered: ChainChoices = DirectionalKeypad::seq(&seg);

                    /*
                    // unfortunately in order to tack this onto the new
                    // `ChainChoices` we're forming we need to do: a cartesian
                    // product!
                    //
                    // todo: maybe in the future we can eagerly do the search
                    // one level deep right here to sus out which option is
                    // better?
                    //
                    // the actual arity of the thing we're doing the cartesian
                    // product on should be pretty small, hopefully..
                    eprintln!(
                        "cart product! {} x {}",
                        new_chains_from_chain.len(),
                        seg_lowered.len()
                    );

                    let curr_new_chains = mem::replace(&mut new_chains_from_chain, vec![]);
                    for choice in curr_new_chains {
                        for next_choice in &seg_lowered {
                            let mut choice = choice.clone();
                            choice.extend(next_choice.clone());
                            new_chains_from_chain.push(choice);
                        }
                    }

                    // prune `new_chains_from_chain`; only keep the chains that
                    // have the best length:
                    let best_length = new_chains_from_chain.iter().map(chain_score).min().unwrap();
                    new_chains_from_chain = new_chains_from_chain
                        .into_iter()
                        .filter(|chain| chain_score(chain) == best_length)
                        .collect();
                    // TODO: optimize clones/push down filtering
                    */

                    /*
                    // if there's only 1 chain choice, tack it onto all the
                    // existing chain choices (hopefully only 1):
                    if let [chain] = seg_lowered.as_slice() {
                        for existing in &mut new_chains_from_chain {
                            existing.extend(chain.clone());
                            // TODO: opt?
                        }

                        continue;
                    }
                    // if there are multiple choices, figure out which is best:
                    //
                    // to do so, drive each chain choice through `seq` once more
                    // and compare scores
                    //
                    // hopefully we don't have to search arbitrarily far forward
                    // to tie-break? (TODO)
                    let scored_chain_choices_for_seg = seg_lowered
                        .into_iter()
                        .map(|chain| {
                            // add up the scores for each segment
                            let score: usize = chain
                                .iter()
                                .map(|seg| {
                                    // take the best "score" of the chain choices that are
                                    // lowered for this segment
                                    DirectionalKeypad::seq(seg)
                                        .iter()
                                        .map(chain_score)
                                        .min()
                                        .unwrap()
                                })
                                .sum();

                            (dbg!(score), chain)
                        })
                        .collect_vec();
                    let orig_len = scored_chain_choices_for_seg.len();
                    let best_score = &scored_chain_choices_for_seg
                        .iter()
                        .map(|(sc, _)| *sc)
                        .min()
                        .unwrap();
                    let chain_choices_for_seg = scored_chain_choices_for_seg
                        .into_iter()
                        .filter(|(sc, _chain)| sc == best_score)
                        .map(|(_, chain)| chain)
                        .collect_vec();
                    // debug_assert_eq!(chain_choices_for_seg.len(), 1);
                    eprintln!("narrowed {orig_len} -> {}", chain_choices_for_seg.len());
                    // if chain_choices_for_seg.len() != 1 {
                    // }
                    // cartesian product onto the current existing chain choices:
                    let curr_new_chains = mem::replace(&mut new_chains_from_chain, vec![]);
                    for existing_chain in curr_new_chains {
                        for chain_choice_for_seq in &chain_choices_for_seg {
                            let mut chain = existing_chain.clone();
                            chain.extend(chain_choice_for_seq.clone());
                            new_chains_from_chain.push(chain);
                        }
                    }

                    // prune `new_chains_from_chain`; only keep the chains that
                    // have the best length: (TODO: unneccessary?)
                    let best_length = new_chains_from_chain.iter().map(chain_score).min().unwrap();
                    new_chains_from_chain = new_chains_from_chain
                        .into_iter()
                        .filter(|chain| chain_score(chain) == best_length)
                        .collect();
                    */

                    let next = DirectionalKeypad::tiebreak_chain_choices(seg_lowered);
                    for existing in &mut new_chains_from_chain {
                        existing.extend(next.clone());
                    }
                }

                // finally, tack on the chain choices from this chain to the
                // next level's chain choices:
                next_chain_choices.extend(new_chains_from_chain);
                // do the same pruning; only keep the best:
                let best_length = next_chain_choices.iter().map(chain_score).min().unwrap();
                next_chain_choices = next_chain_choices
                    .into_iter()
                    .filter(|chain| chain_score(chain) == best_length)
                    .collect();
                // TODO: optimize clones/push down filtering? maybe hold onto
                // scores (should be all same; maybe gate at insert time?)
            }

            mem::swap(&mut chain_choices, &mut next_chain_choices);
            next_chain_choices.clear(); // TODO: debug assert instead? should be empty
        }

        let best_length = chain_choices.iter().map(chain_score).min().unwrap();
        debug_assert!(best_length != 0);
        cost += best_length; // TODO:
    }

    /*     for mut seg_options in /* path */ NumericKeypad::seq(nums) {
        let mut next_level_seg_options = Vec::with_capacity(seg_options.len() * 3 / 2);
        let mut shortest_len = 0;

        // for each seg, for each level...
        for _ in 0..N {
            // drive the options for the segment through directional `seq`:
            next_level_seg_options.extend(seg_options.iter().flat_map(DirectionalKeypad::seq));

            // prune all that aren't of the shortest length:
            shortest_len = next_level_seg_options
                .iter()
                .map(|s| s.len())
                .min()
                .unwrap();
            seg_options.clear();
            for nls in &next_level_seg_options {
                seg_options.push(nls.clone());
            }
            next_level_seg_options.clear();
        }

        debug_assert!(shortest_len != 0);
        cost += shortest_len * 0; // TODO:
    } */

    let num = nums
        .into_iter()
        .filter_map(Numeric::as_num)
        .fold(0, |acc, n| acc * 10 + n);
    dbg!((cost, num));

    cost * num
} */

fn drive_one<const N: usize>(nums: impl IntoIterator<Item = Numeric> + Clone) -> usize {
    let mut cost = 0;
    let chain_score = |chain: &Chain| chain.iter().map(|seg: &Segment| seg.len()).sum::<usize>();

    type ChainChoicesAsCounts = Vec<
        FxHashMap<Segment, usize>, // order of segments doesn't matter...
    >;
    fn countify(chain_choices: ChainChoices) -> ChainChoicesAsCounts {
        chain_choices
            .into_iter()
            .map(|chain| {
                let mut counts = FxHashMap::default();
                for seg in chain {
                    *counts.entry(seg).or_default() += 1;
                }
                counts
            })
            .collect()
    }

    // we can calculate cost for each number's resulting chain independently
    for chain_choices in /* path */ NumericKeypad::seq(nums.clone()) {
        let mut chain_choices_counts = countify(chain_choices);
        let mut next_chain_choices_counts: ChainChoicesAsCounts =
            Vec::with_capacity(chain_choices_counts.len());

        // lower each chain through the `N` levels, yielding new chains:
        for n in 0..N {
            // dbg!(n, chain_choices_counts.len());

            // each chain yields new chains (as counts) of its own; tack these
            // onto `next_chain_choices_counts`:
            for chain in chain_choices_counts.drain(..) {
                // dbg!(chain.len());
                let mut counts = FxHashMap::default();

                // a chain is composed of segments...
                for (seg, count) in chain {
                    // when lowered, each produces `ChainChoices`:
                    let seg_lowered: ChainChoices = DirectionalKeypad::seq(&seg);
                    let next = DirectionalKeypad::tiebreak_chain_choices(seg_lowered);
                    for seg in next {
                        *counts.entry(seg).or_default() += count;
                    }
                }

                // finally, tack on the chain choices from this chain to the
                // next level's chain choices:
                next_chain_choices_counts.push(counts);
                // next_chain_choices.extend(new_chains_from_chain);
                // // do the same pruning; only keep the best:
                // let best_length = next_chain_choices.iter().map(chain_score).min().unwrap();
                // next_chain_choices = next_chain_choices
                //     .into_iter()
                //     .filter(|chain| chain_score(chain) == best_length)
                //     .collect();
            }

            mem::swap(&mut chain_choices_counts, &mut next_chain_choices_counts);
            // next_chain_choices.clear(); // TODO: debug assert instead? should be empty
        }

        let best_length = chain_choices_counts
            .iter()
            .map(|chain_choice_as_counts| {
                let score = chain_choice_as_counts
                    .iter()
                    .map(|(seg, count)| seg.len() * count)
                    .sum::<usize>();
                score
            })
            .min()
            .unwrap();
        debug_assert!(best_length != 0);
        cost += best_length; // TODO:
    }

    let num = nums
        .into_iter()
        .filter_map(Numeric::as_num)
        .fold(0, |acc, n| acc * 10 + n);
    dbg!((cost, num));

    cost * num
}

//------------------------------------------------------------------------------

// TODO: bake? for step directional and step numeric

// struct NumericKeypad;
// // TODO: yield iterator!
// impl NumericKeypad {
//     fn step(on: char, to: char) -> Vec<Directional> {
//         // todo: bake table of jumps?
//         // todo: be more clever about how we do the stuff?
//         let mut curr = Coord { row: 3, col: 2 };

//         todo!()
//     }
// }

// struct DirectionalKeypad;

fn main() {
    let mut aoc = AdventOfCode::new(2024, 21);
    let inp = aoc.get_input();
    let inp = inp.lines().map(|l| l.chars().map_to::<Numeric>());

    let p1 = inp.clone().map(drive_one::<2>).sum::<usize>();
    let _ = aoc.submit_p1(p1);

    let p2 = inp.clone().map(drive_one::<25>).sum::<usize>();
    let _ = aoc.submit_p2(p2);
}

// assumption: "best" input sequence (from a perspective of levels above) will
// be one with repeated key presses
//
// each move will require horizontal moves and/or vertical moves (or neither...)
//
// which you choose to do first doesn't matter I think? from a cost perspective
//   - this is because: at the end/start of a move, all above levels
//     (directional keypad) must be "on" `A`
//   - i.e. 7 -> 3; two down, two right
//     + doesn't matter if you do the down first or the right first
//     + for 2 levels above: doesn't matter if we do:
//       * A → v → > → A
//       * A → > → v → A
//     - `vv>>A` → `<vAA>AA^A` | `v<AA>AA^A`
//     - `>>vvA` → `vAA<AA>^A` | `vAA<AA^>A`
//     + considering 1 level further up:
//       * `<vAA>AA^A` → `v<<A >A >^A A vA ^A A <A >A`
//       * `v<AA>AA^A` → `v<A <A >>^A A vA ^A A <A >A`
//         - same... (`<v` vs. `v<`); but I suspect one more level up the
//           difference would be meaningful?
//       * `vAA<AA>^A` → `<vA ^>A A v<<A >>^A A vA <^A >A`
//       * ... rip. nevermind. need another heuristic? hmmmm idk
//
// seems like ordering choices matter not 1 level up but 2 levels up...
//
// okay, we're an optimizer
//
// at each level we don't have enough information to determine the best sequence
// of moves w.r.t. to upper levels
//
// so instead lets propagate "bundles" containing sequences of moves that upper
// levels are free to re-order
//
// i.e. to go from 7 to 3 we need: 2 `>`s and 2 `v`s and an `A`
//
// order doesn't matter for the `>`s and the `v`s; can do whatever, including
// interpersing them (though this is never optimal...)
//
// thing to watch out for: because of the empty spot sometimes you cannot
// reorder! i.e. 7 -> A; 3 `v`s, and 2 `>`s, can't put all the downs up front
// because we'll hit the empty space?
//   - todo: in this case do we just _know_ the good ordering? or is
//     interspersing (i.e. `>`, 3 x `v`, `>`) ever optimal? assuming no
//
// anyways... level above can then optimize? or just, propagate/punt the
// decision? i.e. `{ >(2), v(2) }, {A}`
//
// ```
// L[0]: `7 → 3`
// L[1]: `{ >(2), v(2) }, {A}`
// L[2]:
//        - `vv>>A` -> `<vAA>AA^A` | `v<AA>AA^A`
//          + `{ v, < }, A; A; >, A; A; ^, A;`
//        - `>>vvA` -> `vAA<AA>^A` | `vAA<AA^>A`
//          + `v, A; A; <, A; A; { >, ^ }, A;`
//        - note: I think we do have to just flatten the `L[1]`; idt
//          there's an easy way to propagate symbolically? maybe repr as
//          a choice between dir and anti-dir? idk; feels like you'd
//          need to propagate/track the position...
//
//        { ({ v, < }, A; A; >, A; A; ^, A)
//        , (v, A; A; <, A; A; { >, ^ }, A;)
//        }
// ```
//
// maybe don't bother with the symbolic repr; just flatten eagerly...
