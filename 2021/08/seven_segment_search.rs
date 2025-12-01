use std::str::Split;

use aoc::*;
use derive_more::{BitAnd, BitOr, BitOrAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, BitAnd, BitOr, BitOrAssign)]
struct SignalPattern(u8);

impl SignalPattern {
    const A: Self = Self(1 << 0);
    const B: Self = Self(1 << 1);
    const C: Self = Self(1 << 2);
    const D: Self = Self(1 << 3);
    const E: Self = Self(1 << 4);
    const F: Self = Self(1 << 5);
    const G: Self = Self(1 << 6);
    const ALL: &[SignalPattern] = {
        use SignalPattern as S;
        &[S::A, S::B, S::C, S::D, S::E, S::F, S::G]
    };

    const fn of(mut segments: &[Self]) -> Self {
        let mut out = SignalPattern(0);

        while let Some((&SignalPattern(s), rest)) = segments.split_first() {
            segments = rest;
            out.0 = out.0 | s;
        }

        out
    }
}

impl FromStr for SignalPattern {
    type Err = char;

    fn from_str(inp: &str) -> Result<Self, Self::Err> {
        let mut s = SignalPattern(0);
        for c in inp.chars() {
            s |= match c {
                'a' => Self::A,
                'b' => Self::B,
                'c' => Self::C,
                'd' => Self::D,
                'e' => Self::E,
                'f' => Self::F,
                'g' => Self::G,
                other => return Err(other),
            };
        }

        Ok(s)
    }
}

macro_rules! segs {
    ($($s:ident)+) => {
        SignalPattern::of(&[$(SignalPattern::$s),+])
    };
}
impl SignalPattern {
    const _0: Self = segs![A B C   E F G];
    const _1: Self = segs![    C     F  ];
    const _2: Self = segs![A   C D E   G];
    const _3: Self = segs![A   C D   F G];
    const _4: Self = segs![  B C D   F  ];
    const _5: Self = segs![A B   D   F G];
    const _6: Self = segs![A B   D E F G];
    const _7: Self = segs![A   C     F  ];
    const _8: Self = segs![A B C D E F G];
    const _9: Self = segs![A B C D   F G];
    const ALL_NUMS: &[SignalPattern] = {
        use SignalPattern as S;
        &[S::_0, S::_1, S::_2, S::_3, S::_4, S::_5, S::_6, S::_7, S::_8, S::_9]
    };

    const SIGNAL_TO_NUM_MAP: [Option<u8>; 2usize.pow(Self::ALL.len() as _)] = {
        let mut out = [None; 2usize.pow(Self::ALL.len() as _)];
        let mut nums = Self::ALL_NUMS;
        let mut i = 0;
        while let Some((curr, rest)) = nums.split_first() {
            out[curr.0 as usize] = Some(i);
            nums = rest;
            i += 1;
        }

        out
    };
}

macro_rules! bucket {
    ($($s:ident)+) => {
        {
            bucket! { @for_each($($s)+) }

            &[$($s),+]
        }
    };
    (@for_each($curr:ident $($pending:ident)*) $($used:ident)*) => {
        bucket!(@of($curr $($used)*) $($used)* $curr $($pending)*);
        bucket!(@for_each($($pending)*) $($used)* $curr);
    };
    (@for_each() $($used:ident)*) => {};
    (@of($curr:ident $($rest:ident)*) $($all:ident)+) => {
        const $curr: &[u8] = {
            const COUNT: u8 = [
                SignalPattern::_0, $(SignalPattern::$rest),*
            ].len() as u8 - 1;

            const NUM_IN_BUCKET: usize = {
                let i = 0;
                $(
                    let i = if SignalPattern::$all.count() == COUNT {
                        i + 1
                    } else {
                        i
                    };
                )+
                i
            };

            #[allow(unused_mut)]
            const BUCKET: [u8; NUM_IN_BUCKET] = {
                let mut bucket = [0xFF; NUM_IN_BUCKET];
                let _n = 0;
                let i = 0;

                $(
                    let (i, _n) = if SignalPattern::$all.count() == COUNT {
                        // bucket[i] = (_n, SignalPattern::$all);
                        bucket[i] = _n;
                        (i + 1, _n + 1)
                    } else {
                        (i, _n + 1)
                    };
                )+

                assert!(i == NUM_IN_BUCKET);
                bucket
            };

            &BUCKET
        };
    }
}
impl SignalPattern {
    const fn count(&self) -> u8 {
        self.0.count_ones() as u8
    }

    const POTENTIAL_NUMS_BY_SEG_COUNT: &[&[u8]] = bucket! {
        _0 _1 _2 _3 _4 _5 _6 _7 _8 _9
    };
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Entry {
    observed: [SignalPattern; 10],
    values: [SignalPattern; 4],
}

impl FromStr for Entry {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (obs, val) = s.split_once(" | ").unwrap();
        let (mut observed, mut values) = (obs.split(' '), val.split(' '));
        let next = |it: &mut Split<'_, _>| it.next().unwrap().parse().unwrap();

        let ret = Entry {
            observed: [(); 10].map(|()| next(&mut observed)),
            values: [(); 4].map(|()| next(&mut values)),
        };
        assert_eq!(observed.next(), None);
        assert_eq!(values.next(), None);

        Ok(ret)
    }
}

type Mapping = [SignalPattern; SignalPattern::ALL.len()];

impl Entry {
    fn find_mapping(&self) -> Mapping {
        use SignalPattern as S;

        // todo: const-ify?
        let mut i = 0;
        let nums_with_seg: [u16; S::ALL.len()] = [(); S::ALL.len()].map(|()| {
            let s = S::ALL[i];
            i += 1;
            let mut out = 0;
            for (i, pat) in S::ALL_NUMS.iter().enumerate() {
                if pat.0 & s.0 != 0 {
                    out |= 1 << i;
                }
            }

            out
        });

        // todo: const-ify?
        let mut i = 0;
        let occurrence_counts_for_segs: [u8; S::ALL.len()] = [(); S::ALL.len()].map(|()| {
            let s = S::ALL[i];
            i += 1;
            let mut out = 0;
            for pat in S::ALL_NUMS {
                if pat.0 & s.0 != 0 {
                    out += 1;
                }
            }
            out
        });
        let mut i = 0;
        let potential_segs_for_occurrence_count: [S; S::ALL_NUMS.len()] = [(); S::ALL_NUMS.len()]
            .map(|()| {
                let count = i;
                i += 1;
                let mut segs = S::of(&[]);
                for (s, &c) in occurrence_counts_for_segs.iter().enumerate() {
                    if c == count {
                        segs |= S::ALL[s];
                    }
                }

                segs
            });

        ////////////////////////////////////////////////////////////////////////

        // start with all wires potentially being any segment that matches their
        // occurrence count:
        let mut mapping: Mapping = [S::of(S::ALL); S::ALL.len()];
        for (w, pot_segs) in mapping.iter_mut().enumerate() {
            let mut occ_count = 0;
            for o in self.observed {
                if o.0 & (1 << w) != 0 {
                    occ_count += 1;
                }
            }
            *pot_segs = potential_segs_for_occurrence_count[occ_count];
        }

        // and with the potential list of numbers for each observed value being
        // any number with the right number of active signals:
        let mut observed = self.observed.map(|s| {
            let nums = S::POTENTIAL_NUMS_BY_SEG_COUNT[s.count() as usize];
            let nums = nums.iter().map(|&i| 1u16 << i).fold(0, BitOr::bitor);

            (s, nums)
        });

        fn dump_state(mapping: &Mapping, observed: &[(S, u16); 10]) {
            const _DEBUG: bool = false;

            #[cfg(debug_assertions)]
            if _DEBUG {
                fn itoa(i: usize) -> char {
                    char::from(b'a' + i as u8)
                }

                eprintln!("-------------------------------------------------");
                for (w, pot_seg) in mapping.iter().enumerate() {
                    eprint!("wire {}:", itoa(w));
                    for s in 0..S::ALL.len() {
                        if pot_seg.0 & (1 << s) != 0 {
                            eprint!(" {}", itoa(s));
                        } else {
                            eprint!("  ");
                        }
                    }
                    eprintln!();
                }
                eprintln!();

                for (o, (wires, pot_nums)) in observed.iter().enumerate() {
                    eprint!("obsv {o}:");
                    for n in 0..S::ALL_NUMS.len() {
                        if pot_nums & (1 << n) != 0 {
                            eprint!(" {n}");
                        } else {
                            eprint!("  ");
                        }
                    }
                    eprint!(" {{ ");
                    for s in 0..S::ALL.len() {
                        if wires.0 & (1 << s) != 0 {
                            eprint!(" {}", itoa(s));
                        } else {
                            eprint!("  ");
                        }
                    }
                    eprint!(" }} ");
                    eprintln!();
                }
                eprintln!("-------------------------------------------------");
            }

            _ = (mapping, observed);
        }

        #[cfg(debug_assertions)]
        let (mut old_mapping, mut old_observed) = (mapping, observed);

        dump_state(&mapping, &observed);
        loop {
            /* constrain potential segments using potential nums */
            // if a wire `W` is set for a number containing a set of segments
            // `S`, `W` must correspond to a segment in `S`
            for (obs, pot_nums) in observed {
                let mut poss_segs_for_poss_nums = S::of(&[]);
                for i in 0..S::ALL_NUMS.len() {
                    if pot_nums & (1 << i) != 0 {
                        poss_segs_for_poss_nums |= S::ALL_NUMS[i];
                    }
                }

                // constrain each wire that was set for this observation
                // accordingly
                for s in 0..mapping.len() {
                    if obs.0 & (1 << s) != 0 {
                        mapping[s] = mapping[s] & poss_segs_for_poss_nums;
                    }
                }
            }

            /* constrain potential segments using potential segments */
            // any wire that maps to a single segment has been "solved":
            let mut all_wires_solved = true;
            for w in 0..mapping.len() {
                debug_assert!(mapping[w].count() != 0);
                // if a wire has been solved:
                if mapping[w].count() == 1 {
                    // the segment it corresponds to cannot be the segment any
                    // other wire maps to:
                    for w_other in 0..mapping.len() {
                        if w == w_other {
                            continue;
                        }
                        mapping[w_other].0 &= !mapping[w].0;
                    }
                } else {
                    all_wires_solved = false;
                }
            }

            /* constrain potential nums using potential segments */
            // for each wire..
            for w in 0..mapping.len() {
                let potential_segments = mapping[w];
                let mut nums_with_any_of_pot_segs = 0;
                for s in 0..S::ALL.len() {
                    if potential_segments.0 & (1 << s) != 0 {
                        nums_with_any_of_pot_segs |= nums_with_seg[s];
                    }
                }

                // any observation containing this wire..
                for (obs, pot_nums) in &mut observed {
                    if obs.0 & (1 << w) != 0 {
                        // can only be a number that contains any of the
                        // segments this wire may be
                        *pot_nums &= nums_with_any_of_pot_segs;
                    }
                }
            }

            /* constrain potential nums (+segs) using potential nums */
            // any observation that maps to a single number has been "solved":
            let mut all_observations_solved = true;
            for o in 0..observed.len() {
                let (_obs, pot_nums) = observed[o];
                debug_assert!(pot_nums.count_ones() != 0);
                if pot_nums.count_ones() == 1 {
                    // that means that no other observation can be this number:
                    for other_o in 0..observed.len() {
                        if o == other_o {
                            continue;
                        }

                        observed[other_o].1 &= !pot_nums;
                    }
                } else {
                    all_observations_solved = false;
                }
            }

            if all_observations_solved && all_wires_solved {
                break;
            }

            #[cfg(debug_assertions)]
            if old_mapping == mapping && old_observed == observed {
                panic!("stuck");
            } else {
                (old_mapping, old_observed) = (mapping, observed);
            }

            dump_state(&mapping, &observed);

            // TODO: event based?
        }

        dump_state(&mapping, &observed);
        mapping
    }

    fn decode(&self, mapping: Mapping) -> [u8; 4] {
        fn decode_one(mapping: Mapping, signals: SignalPattern) -> u8 {
            let mut remapped = SignalPattern::of(&[]);
            for w in 0..SignalPattern::ALL.len() {
                if signals.0 & (1 << w) != 0 {
                    remapped |= mapping[w];
                }
            }

            SignalPattern::SIGNAL_TO_NUM_MAP[remapped.0 as usize].unwrap()
        }

        #[cfg(debug_assertions)]
        {
            for o in self.observed {
                eprint!("{} ", decode_one(mapping, o));
            }
            eprint!("|");
            for v in self.values {
                eprint!(" {}", decode_one(mapping, v));
            }
            eprintln!();
        }

        self.values.map(|s| decode_one(mapping, s))
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2021, 8);
    let inp = aoc.get_input();
    let entries = inp.lines().map_parse::<Entry>();

    let mut p1 = 0;
    let mut p2 = 0;
    for e in entries {
        let m = e.find_mapping();
        let mut n: usize = 0;
        for i in e.decode(m) {
            match i {
                1 | 4 | 7 | 8 => p1 += 1,
                _ => {}
            }

            n = n * 10 + (i as usize);
        }

        p2 += n;
    }

    _ = aoc.submit_p1(p1);
    _ = aoc.submit_p2(p2);
}

// TODO: z3 based solution...
