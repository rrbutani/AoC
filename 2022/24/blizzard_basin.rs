use std::{
    cell::RefCell,
    collections::HashMap,
    convert::identity,
    iter::once,
    ops::{Index, IndexMut},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
    thread::LocalKey,
};

use aoc::*;
use owo_colors::OwoColorize;
use rayon::prelude::{ParallelBridge, ParallelIterator};

macro_rules! d {
    ($($tt:tt)*) => {
        if DBG {
            eprintln!($($tt)*);
        }
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Blizzard {
    Up = 0,
    Right,
    Down,
    Left = 3,
}

type Dir = Blizzard;

impl Display for Blizzard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Blizzard::*;
        write!(
            f,
            "{}",
            match self {
                Up => '^',
                Down => 'v',
                Left => '<',
                Right => '>',
            }
        )
    }
}

impl TryFrom<char> for Blizzard {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use Blizzard::*;
        Ok(match value {
            '^' => Up,
            'v' => Down,
            '<' => Left,
            '>' => Right,
            _ => return Err(()),
        })
    }
}

impl Blizzard {
    const ALL: [Blizzard; Blizzard::Left as usize + 1] = {
        use Blizzard::*;
        [Up, Right, Down, Left]
    };

    #[allow(unused)]
    fn flip(self) -> Self {
        Self::ALL[(self as usize + 2) % Self::ALL.len()]
    }
}

impl Blizzard {
    fn offs(self) -> (isize, isize) {
        use Blizzard::*;
        match self {
            // (dx, dy)
            Up => (0, -1),
            Right => (1, 0),
            Down => (0, 1),
            Left => (-1, 0),
        }
    }

    fn add_to_coord(self, (x, y): (usize, usize)) -> Option<(usize, usize)> {
        let (dx, dy) = self.offs();
        Some((x.checked_add_signed(dx)?, y.checked_add_signed(dy)?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
enum Cell {
    Wall,
    #[default]
    Empty,
    Blizzards(Vec<Blizzard>),
}

impl TryFrom<char> for Cell {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use Cell::*;

        Ok(match value {
            '#' => Wall,
            '.' => Empty,
            _ => value.try_into().map(|b: Blizzard| vec![b]).map(Blizzards)?,
        })
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Cell::*;
        match self {
            Wall => write!(f, "{}", '#'.bold()),
            Empty => write!(f, "{}", '.'.dimmed()),
            Blizzards(b) => match &**b {
                [single] => Display::fmt(single, f),
                many @ [..] => match many.len() {
                    0 => unreachable!(),
                    n @ 0..=9 => write!(f, "{}", n.underline()),
                    n @ 10..=35 => write!(
                        f,
                        "{}",
                        ((n as u8 - 10 + b'A') as char).yellow().underline()
                    ),
                    _ => write!(f, "{}", '∞'.red().bold()),
                },
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Map {
    inner: Vec<Vec<Cell>>, // (row, col)
    curr: (usize, usize),  // col, row
    end: (usize, usize),
    dim: (usize, usize), // (cols, rows)
}

impl FromStr for Map {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let grid = s
            .lines()
            .map(|l| l.chars().map(|c| c.try_into().unwrap()).collect_vec())
            .collect_vec();
        let rows = grid.len();
        let cols = grid[0].len();
        debug_assert!(grid
            .windows(2)
            .map(|s| s[0].len() == s[1].len())
            .all(identity));

        let first_free_in_row = |row: &[Cell]| {
            row.iter()
                .enumerate()
                .find(|(_, c)| *c == &Cell::Empty)
                .map(|(col, _)| col)
                .unwrap()
        };

        let start = grid.first().map(|row| (first_free_in_row(row), 0)).unwrap();
        let end = grid
            .iter()
            .enumerate()
            .last()
            .map(|(r, row)| (first_free_in_row(row), r))
            .unwrap();

        Ok(Self {
            inner: grid,
            curr: start,
            end,
            dim: (cols, rows),
        })
    }
}

impl Index<(usize, usize)> for Map {
    type Output = Cell;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.inner[y][x]
    }
}

impl IndexMut<(usize, usize)> for Map {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        &mut self.inner[y][x]
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (r, row) in self.inner.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                let pos = (c, r);

                if cell == &Cell::Empty && pos == self.curr {
                    write!(f, "{}", 'E'.green())
                } else if cell == &Cell::Empty && pos == self.end {
                    write!(f, "{}", '$'.blue())
                } else {
                    Display::fmt(cell, f)
                }?
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

impl Map {
    #[allow(unused)]
    fn step_in_place(&mut self, movement: Option<Dir>) -> Result<(), ()> {
        *self = self.step(movement)?;

        Ok(())
    }

    fn step(&self, movement: Option<Dir>) -> Result<Self, ()> {
        use Cell::*;

        // inefficient but whatever
        let mut out = self.clone();

        for row in out.inner.iter_mut() {
            for cell in row.iter_mut() {
                if *cell != Cell::Wall {
                    *cell = Cell::Empty;
                }
            }
        }

        // move the blizzards:
        for (r, row) in self.inner.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                if let Blizzards(b) = cell {
                    let pos = (c, r);
                    for bliz in b {
                        let inc = |(mut x, mut y)| {
                            if x == 0 && bliz.offs().0 == -1 {
                                x = self.dim.0;
                            }
                            if y == 0 && bliz.offs().1 == -1 {
                                y = self.dim.1;
                            }
                            let (x, y) = bliz.add_to_coord((x, y)).unwrap();
                            (x % self.dim.0, y % self.dim.1)
                        };
                        let mut new_pos = inc(pos);

                        // to wrap:
                        while out[new_pos] == Wall {
                            new_pos = inc(new_pos);
                        }

                        match &mut out[new_pos] {
                            c @ Empty => *c = Blizzards(vec![*bliz]),
                            Wall => unreachable!(),
                            Blizzards(b) => b.push(*bliz),
                        }
                    }
                }
            }
        }

        // move ourself:
        if let Some(movement) = movement {
            out.curr = movement.add_to_coord(self.curr).unwrap();
        }
        if out[out.curr] != Cell::Empty {
            return Err(());
        }

        Ok(out)
    }
}

impl Map {
    fn search_step(
        &self,
        minute: usize,
        current_best: &AtomicUsize,
        limit: usize,
    ) -> Option<usize> {
        // dbg!(minute);
        // if we're at the end our search is finished!
        if self.curr == self.end {
            let old_best = current_best.fetch_min(minute, Ordering::SeqCst);
            if old_best > minute {
                if PROGRESS {
                    println!("new best: {minute}; old best: {old_best}");
                }
            }

            return Some(minute);
        }

        // perils of DFS..
        //
        // until we find _a_ solution our search is unbounded and we'll keep
        // going down a path – even if it's doomed – until we blow the stack
        if minute > limit {
            return None;
        }

        // if there's no hope we're going to beat the current min solution,
        // bail:
        let min_time_to_get_to_end = {
            let (ex, ey) = self.end;
            let (x, y) = self.curr;
            ex.abs_diff(x) + ey.abs_diff(y)
        };
        if (minute + min_time_to_get_to_end) >= current_best.load(Ordering::SeqCst) {
            return None;
        }

        // can't move into walls:
        let movements = Dir::ALL
            .iter()
            .filter(|d| {
                d.add_to_coord(self.curr)
                    .filter(|&p| {
                        d!("    ([m={minute}] considering pos: {p:?}, {d:?})");
                        self[p] != Cell::Wall
                    })
                    .is_some()
            })
            .inspect(|d| d!("  - [m={minute}] trying movement {d:?}"))
            .map(Some);

        // another option is to just wait:
        let movements = movements.chain(once(None));

        // play out each potential movement (this is DFS):
        //
        // the `AtomicUsize` means that we can parallelize here..
        //
        // TODO: it would be better to just run the blizzard logic for a bunch
        // of steps and then do path search on those maps since the blizzard
        // movements aren't affected by our movements...
        let movements = movements.filter_map(|d| {
            self.step(d.copied())
                .ok()?
                .search_step(minute + 1, current_best, limit)
        });

        // pick the best option, propagate upwards:
        let res = movements.min();
        d!("\n");
        res
    }

    fn search_step_with_store(
        starting_state: &Self,
        pos: (usize, usize),
        end: (usize, usize),
        minute: usize,
        counters @ (current_best, deepest, closest): (&AtomicUsize, &AtomicUsize, &AtomicUsize),
        limit: usize,
        // the blizzard state at timesteps
        store: &RwLock<Vec<Arc<Self>>>,
        // needs to match the starting state, starting pos, and end..
        cache: &'static LocalKey<
            RefCell<
                HashMap<
                    ((usize, usize), usize),           // (pos, minute)
                    Option<(usize, Vec<Option<Dir>>)>, // (res, path)
                >,
            >,
        >,
    ) -> Option<(usize, Vec<Option<Dir>>)> {
        let manhattan = |src: (usize, usize), dest: (usize, usize)| {
            let (ex, ey) = dest;
            let (sx, sy) = src;
            ex.abs_diff(sx) + ey.abs_diff(sy)
        };

        /*
        thread_local! {
            static CACHE: RefCell<HashMap<
                Vec<Vec<Cell>>, // starting state
                HashMap<
                    ((usize, usize), usize), // (pos, minute)
                    Option<(usize, Vec<Option<Dir>>)> // (res, path)
                >
            >> = RefCell::new(
                HashMap::new()
            );
        }
        {
            if let Some(res) = CACHE.with(|cache| {
                let mut cache = cache.borrow_mut();
                let cache = if let Some(c) = cache.get(&starting_state.inner) {
                    c
                } else {
                    cache.entry(starting_state.inner.clone()).or_default()
                };
                cache.get(&(pos, minute)).cloned()
            }) {
                return res;
            }
        } */
        {
            if let Some(res) = cache.with(|cache| cache.borrow().get(&(pos, minute)).cloned()) {
                return res;
            }
        }

        // set up the store, fill it in with some entries
        if minute == 0 {
            println!("filling store...");
            let mut store = store.write().unwrap();
            assert!(store.is_empty());

            // let min_dist = manhattan(starting_state.curr, end);
            //
            // // fudge factor; just a guess to save us a little time later
            // let min_dist = dbg!(min_dist * 5);

            // nevermind:
            let min_dist = limit;

            store.reserve(min_dist);

            let mut state = starting_state.clone();
            store.push(Arc::new(state.clone()));

            for _ in 0..min_dist {
                state.step_in_place(None).unwrap();
                store.push(Arc::new(state.clone()));
            }
            println!("done filling store.");
        }

        // helper to fetch from the state store/keep it up to date
        fn get_state(minute: usize, store: &RwLock<Vec<Arc<Map>>>) -> Arc<Map> {
            let store_read = store.read().unwrap();
            if let Some(state) = store_read.get(minute) {
                state.clone()
            } else {
                drop(store_read);

                let prev = get_state(minute.checked_sub(1).unwrap(), store);
                let state = (*prev).step(None).map_err(|()| minute).unwrap();

                let state = Arc::new(state);
                let mut store = store.write().unwrap();
                store.push(state.clone());
                state
            }
        }

        // dbg!(minute);
        // if we're at the end our search is finished!
        if pos == end {
            let old_best = current_best.fetch_min(minute, Ordering::SeqCst);
            if old_best > minute && PROGRESS {
                println!("new best: {minute}; old best: {old_best}");
            }

            return Some((minute, vec![]));
        }

        // perils of DFS..
        //
        // until we find _a_ solution our search is unbounded and we'll keep
        // going down a path – even if it's doomed – until we blow the stack
        if minute > limit {
            return None;
        }

        // if there's no hope we're going to beat the current min solution,
        // bail:
        if (minute + manhattan(pos, end)) >= current_best.load(Ordering::SeqCst).min(limit) {
            return None;
        }

        // progress:
        let dist = manhattan(pos, end);
        if dist < closest.fetch_min(dist, Ordering::SeqCst) {
            if PROGRESS {
                eprintln!("new closest: {dist}");
            } else {
                // hush clippy.
            }
        }

        // we have some level of ~omniscience~; we can see into the future to
        // find out where the wind is _going_ to go and we can filter our
        // moves accordingly:
        let curr_state = get_state(minute, store);
        let next_state = get_state(minute + 1, store);

        let movements = Dir::ALL
            .iter()
            .filter_map(|d| d.add_to_coord(pos).map(|pos| (Some(*d), pos)))
            // can't move out of bounds (matters for when we try to go backwards)
            .filter(|&(_, (x, y))| x < starting_state.dim.0 && y < starting_state.dim.1)
            // can't move into walls:
            .filter(|&(_, pos)| curr_state[pos] != Cell::Wall)
            // another option is to just wait:
            .chain(once((None, pos)))
            // don't move into (or stay in spots that will be hit by) blizzards:
            .filter(|&(_, pos)| next_state[pos] == Cell::Empty)
            .inspect(|(d, pos)| d!("  - [m={minute}] trying movement {d:?} to {pos:?}"));

        // play out each potential movement (this is DFS):
        //
        // the `AtomicUsize` means that we can parallelize here..
        let movements = movements.filter_map(|(d, pos)| {
            Self::search_step_with_store(
                starting_state,
                pos,
                end,
                minute + 1,
                counters,
                limit,
                store,
                cache,
            )
            .map(|(time, mut steps)| {
                steps.push(d);
                (time, steps)
            })
        });

        // pick the best option, propagate upwards:
        // let res = movements.min();
        // let res = movements.par_bridge().min();
        let prev_deepest = deepest.fetch_max(minute, Ordering::SeqCst);
        /* limit - minute == 5 */
        let par = if minute > prev_deepest {
            if PROGRESS {
                eprintln!("new deepest: {minute}; dist: {}", manhattan(pos, end));
            }
            limit - minute == 5
        } else {
            false
        };

        let res = if par {
            if PROGRESS {
                eprintln!("par: {minute}; dist: {}", manhattan(pos, end));
            }
            movements.par_bridge().min()
        } else {
            let res = movements.min();

            cache.with(|cache| {
                // let mut cache = cache.borrow_mut();
                // let cache = if let Some(cache) = cache.get_mut(&starting_state.inner) {
                //     cache
                // } else {
                //     cache.entry(starting_state.inner.clone()).or_default()
                // };
                // cache.insert((pos, minute), res.clone());
                let mut cache = cache.borrow_mut();
                cache.insert((pos, minute), res.clone());
            });

            res
        };

        d!("\n");
        res
    }
}

const DBG: bool = false;
const PROGRESS: bool = false;

fn main() {
    let mut aoc = AdventOfCode::new(2022, 24);
    let inp = aoc.get_input();
    // let inp = include_str!("ex");
    // let inp = include_str!("ex_small");

    let map: Map = inp.parse().unwrap();

    // d!("{map}");
    // for _ in 0..5 {
    //     map.step_in_place(Dir::Down).unwrap();
    //     d!("{map}");
    // }
    let store = RwLock::new(vec![]);

    let search = |starting_minute, state, start, end, limit, store, cache| {
        let best = AtomicUsize::new(usize::MAX);
        let closest = AtomicUsize::new(usize::MAX);
        let deepest = AtomicUsize::new(0);

        let (score, mut path) = Map::search_step_with_store(
            state,
            start,
            end,
            starting_minute,
            (&best, &deepest, &closest),
            limit,
            store,
            cache,
        )
        .unwrap();

        path.reverse();
        let path = path
            .iter()
            .enumerate()
            .map(|(i, m)| (i + 1, *m))
            .collect_vec();
        (score, path)
    };

    let search_limit = 500;

    let p1 = {
        // let best = AtomicUsize::new(usize::MAX);
        // let closest = AtomicUsize::new(usize::MAX);
        // // map.search_step(0, &best, 500).unwrap()
        // let deepest = AtomicUsize::new(0);
        // let (score, mut path) = Map::search_step_with_store(
        //     &map,
        //     map.curr,
        //     map.end,
        //     0,
        //     (&best, &deepest, &closest),
        //     500,
        //     &store,
        // )
        // .unwrap();

        // if DBG {
        //     path.reverse();
        //     let path = path
        //         .iter()
        //         .enumerate()
        //         .map(|(i, m)| (i + 1, m))
        //         .collect_vec();
        //     dbg!(path);
        // }

        // TODO: would memoization/cycle-detection help?
        // update: yes, memoization
        thread_local! {
            static CACHE: RefCell<HashMap<
                ((usize, usize), usize), // (pos, minute)
                Option<(usize, Vec<Option<Dir>>)> // (res, path)
            >> = RefCell::new(
                HashMap::new()
            );
        }

        let (score, path) = search(0, &map, map.curr, map.end, search_limit, &store, &CACHE);
        d!("{path:#?}");
        score
    };
    dbg!(p1);
    aoc.submit_p1(p1).unwrap();

    let p2 = {
        let get_state_at_minute = |store: &RwLock<Vec<Arc<Map>>>, minute| -> Arc<Map> {
            let store = store.read().unwrap();
            let map: &Arc<Map> = &store[minute];
            map.clone()
        };

        thread_local! {
            static CACHE2: RefCell<HashMap<
                ((usize, usize), usize),
                Option<(usize, Vec<Option<Dir>>)>
            >> = RefCell::new(HashMap::new());

            static CACHE3: RefCell<HashMap<
                ((usize, usize), usize),
                Option<(usize, Vec<Option<Dir>>)>
            >> = RefCell::new(HashMap::new());
        }

        let go_for_snacks = p1;
        dbg!(go_for_snacks);
        let state = get_state_at_minute(&store, go_for_snacks);
        let return_for_snacks = search(
            go_for_snacks,
            &state,
            map.end,
            map.curr,
            go_for_snacks + search_limit,
            &store,
            &CACHE2,
        )
        .0;

        dbg!(return_for_snacks);
        let state = get_state_at_minute(&store, return_for_snacks);
        let go_for_real = search(
            return_for_snacks,
            &state,
            map.curr,
            map.end,
            return_for_snacks + search_limit,
            &store,
            &CACHE3,
        );
        go_for_real.0
    };
    dbg!(p2);
    aoc.submit_p2(p2).unwrap();
}
