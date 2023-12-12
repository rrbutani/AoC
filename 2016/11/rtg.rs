#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{
    friends::*,
    object_store::{ObjectStore, Ref},
    AdventOfCode,
};

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Display},
};
use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    // marker::PhantomData,
};

// In the input there are 5 generator-chip pairs meaning we've got 11 items in
// total.
//
// Since each item can be on one of 4 floors, we've got 4 ^ 11 states or
// ~4 million states.
//
// It doesn't seem too insane to try to explore all of these...

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, Hash)]
enum Item<'a> {
    Generator(&'a str),
    Microchip(&'a str),
}

impl<'a> PartialOrd for Item<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let extract = |i: &Self| match i {
            Item::Generator(g) => (*g, 0),
            Item::Microchip(m) => (*m, 1),
        };

        let (s1, s2) = extract(self);
        let (o1, o2) = extract(other);

        Some(s1.cmp(o1).then(s2.cmp(&o2)))
    }
}

impl<'a> Display for Item<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (c, kind) = match self {
            Item::Generator(g) => (g, 'G'),
            Item::Microchip(m) => (m, 'M'),
        };

        let c = c.chars().next().unwrap().to_uppercase().next().unwrap();
        write!(fmt, "{}{}", c, kind)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Level {
    One,
    Two,
    Three,
    Four,
}

impl Display for Level {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "F{}", (*self as u8) + 1)
    }
}

impl Level {
    const fn all() -> [Level; 4] {
        use Level::*;
        [One, Two, Three, Four]
    }
}

const NUM_LEVELS: usize = Level::all().len();

// we could/probably should do better than an array of hashsets here; maybe an
// array of bitvecs of fixed size where the bits correspond to the items we have
// which are iterned _properly_ for the entire state store

// Note: we use a BTreeSet because it maintains an order and thus can impl Hash
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct State<'n> {
    elevator: Level,
    // levels: [HashSet<Item<'n>>; NUM_LEVELS], // Not `Hash`!
    // levels: [Vec<Item<'n>>; NUM_LEVELS],
    // levels: [HashSet<Item<'n>>; NUM_LEVELS],
    levels: [BTreeSet<Item<'n>>; NUM_LEVELS],
}

// // We have to impl this manually because HashSet is not Hash..
// impl<'n> Hash for State<'h> {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.elevator.hash(state);
//     }
// }

impl<'n> State<'n> {
    // const NUM_LEVELS: usize = Level::all().len();

    fn from_input(inp: &'n str) -> Result<Self, ()> {
        if inp.lines().count() != /* Self:: */NUM_LEVELS {
            return Err(());
        }

        let mut s = State {
            elevator: Level::One,
            // levels: [HashSet::new(); /* Self:: */NUM_LEVELS],
            // levels: [vec![]; /* Self:: */NUM_LEVELS],
            // levels: [vec![], vec![], vec![], vec![]],
            // levels: [
            //     HashSet::new(),
            //     HashSet::new(),
            //     HashSet::new(),
            //     HashSet::new(),
            // ],
            levels: [
                BTreeSet::new(),
                BTreeSet::new(),
                BTreeSet::new(),
                BTreeSet::new(),
            ],
        };

        for (level, line) in inp.lines().enumerate() {
            if line.ends_with("nothing relevant.") {
                continue;
            }

            let contents = line.split("contains").skip(1).next().ok_or(())?;
            for item in contents.split(" a ").skip(1) {
                let item = item
                    .trim_end_matches("and")
                    .trim_end_matches(", ")
                    .trim_end_matches(",")
                    .trim_end_matches(".");

                let mut iter = item.split(" ");
                let element = iter.next().unwrap().trim_end_matches("-compatible");

                let item = match iter.next().unwrap() {
                    "generator" => Item::Generator(element),
                    "microchip" => Item::Microchip(element),
                    unexpected => panic!("Got: {:?}.", unexpected),
                };

                // TODO: check for duplicates; check for generator-microchip pairs..

                // assert!(s.levels[level].insert(item));
                // s.levels[level].push(item);
                assert!(s.levels[level].insert(item));
            }
        }

        Ok(s)
    }
}

impl<'n> Display for State<'n> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let all_items: HashSet<_> = self.levels.iter().flat_map(|l| l.iter()).collect();
        let num_items = all_items.len();

        // This is so that we have a stable order so that prints don't vary
        // randomly.
        let mut all_items_sorted: Vec<_> = all_items.into_iter().collect();
        all_items_sorted.sort();

        assert_eq!(num_items, all_items_sorted.len());

        let col_map: HashMap<&Item<'n>, usize> = all_items_sorted
            .iter()
            .enumerate()
            .map(|(idx, item)| (*item, idx))
            .collect();

        for (idx, level) in self.levels.iter().enumerate().rev() {
            write!(fmt, "F{} ", idx + 1)?;

            if self.elevator as usize == idx {
                write!(fmt, "E  ")?;
            } else {
                write!(fmt, ".  ")?;
            }

            let mut row = vec![None; col_map.len()];

            for item in level {
                let idx = col_map.get(item).unwrap();

                assert_eq!(row[*idx], None);
                row[*idx] = Some(item);
            }

            for cell in row {
                if let Some(item) = cell {
                    write!(fmt, "{} ", item)?;
                } else {
                    write!(fmt, ".  ")?;
                }
            }

            write!(fmt, "\n")?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Direction {
    Up,
    Down,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Step<'n> {
    dir: Direction,
    with: HashSet<Item<'n>>,
}

// #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// struct StateRef<'s>(usize, PhantomData<&'s ()>);

type StateRef<'s, 'n, U = ()> = Ref<'s, State<'n>, U>;

#[derive(Clone, Debug, PartialEq, Eq)]
enum StateStatus<'store, 'names, U = ()> {
    Origin {
        next: Vec<StateRef<'store, 'names, U>>,
    },
    Explored {
        steps: usize,
        prev: (StateRef<'store, 'names, U>, Step<'names>),
        next: Vec<StateRef<'store, 'names, U>>,
    },
    Illegal,
}

// // /// Ensures that there is only ever 1 of a state in the store and also provides
// // /// [`StateRef`]s to the [`State`]s stored within.
// // struct StateStore<'store, 'n> {
// //     // This is not great but we use a HashSet to ensure that things aren't
// //     // stored twice and to provide O(1) lookup.
// //     store: &'store mut Vec<State<'n>>,
// // }

// // impl<'store, 'n> StateStore<'store> {
// //     fn new(&'store mut Vec<State<'names>)
// // }

// // struct Ref<'store, T>(usize, PhantomData<(&'store mut (), T)>);

// /// A type that essentially wraps a [`Vec`] and ensures that instances of type
// /// `T` that are placed within it live for `'store`.
// ///
// /// This also ensures that there are no duplicate instances within the store.
// ///
// /// Finally, this provides [`Ref`]s to the `T` instances stored within which
// /// can then be turned into regular immutable or mutable references by
// /// providing a reference to the object store instance the reference came from.
// ///
// /// Because this type is append only (i.e. you cannot remove items; you can only
// /// add them), the provided [`Ref`]s can outlive
// //
// // TODO: can we get a Ref and then use it with an ObjectStore that it did not
// // originate from? or does the invariant nature of lifetimes on mutable
// // references effectively prevent this?
// //
// // TODO: does Ref need to store a "lifetime reference" corresponding to the
// // parent object store so that it does not get dropped and then recreated while
// // a Ref is out? For now to be cautious we do this. Actually we only reference
// // the parent since then the other bound is implied.
// struct ObjectStore<'store, T: Hash> {
//     // The `Vec` that the instances are actually stored in.
//     store: &'store mut Vec<T>,

//     // We want to have a `HashMap` somewhere so that we can quickly lookup new
//     // entries to make sure we don't have duplicates.
//     //
//     // At the same time we don't have to have this `HashMap` actually _store_
//     // instances of `T` since that'd be wasteful; we're already putting the
//     // instances inside of `store`.
//     //
//     // But, we do want to be able to map from values of type `T` to references
//     // to instances of `T`. What to do?
//     //
//     // `HashSet` (and `HashMap`) actually accept not just `T` but `Q: Borrow<T>`
//     // when looking up something. We can use this to make it so that we can
//     // look up a value of type `T` on a `HashSet` of `&T`s: when there are
//     // duplicates this lookup will give us the existing pointer for `T` within
//     // the store even though the `HashSet` does not store any instances of `T`.
//     //
//     // The reason this works is because the implementation of `PartialEq` for
//     // references will dereference (i.e. "see through") the pointer effectively
//     // meaning that the act of looking up a value in the `HashSet` will access
//     // data in the store which is exactly what we wanted.
//     map: HashSet<&'store
// }

// impl<'s, T: Hash> ObjectStore<'s, T> {
//     /// Returns [`Err`] if the provided [`Vec`] is not empty.
//     fn new(store: &'s mut Vec<T>) -> Result<Self, ()> {
//         if !store.is_empty() {
//             Err(())
//         } else {
//             Ok(Self { store })
//         }
//     }
// }

struct StateSpace<'store, 'names, U = ()> {
    // &'store mut Vec<State<'names>>,
    state_store: ObjectStore<'store, State<'names>, U>,
    graph: HashMap<StateRef<'store, 'names, U>, StateStatus<'store, 'names, U>>,
}

impl<'s, 'n, U> StateSpace<'s, 'n, U> {
    fn new(input: &'n str, store: &'s mut Vec<State<'n>>) -> Result<Self, ()> {
        let mut s = Self {
            state_store: ObjectStore::<_, U>::new(store)?,
            graph: HashMap::new(),
        };

        let initial_state = State::from_input(input)?;
        let initial_state = s.state_store.insert(initial_state).ok().unwrap();

        println!("{}", s.state_store.get(initial_state));

        s.graph
            .insert(initial_state, StateStatus::Origin { next: vec![] });

        Ok(s)
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2016, 11);
    let input: String = aoc.get_input();
    let input = "The first floor contains a hydrogen-compatible microchip and a lithium-compatible microchip.
    The second floor contains a hydrogen generator.
    The third floor contains a lithium generator.
    The fourth floor contains nothing relevant.
";

    let mut store = vec![];
    let mut states = StateSpace::<()>::new(&input, &mut store).unwrap();
}
