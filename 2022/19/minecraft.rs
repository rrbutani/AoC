use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::HashMap,
    iter::once,
    ops::{Index, IndexMut, Mul},
};

use aoc::*;

use derive_more as d;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
enum Kind {
    Ore = 0,
    Clay = 1,
    Obsidian = 2,
    Geode = 3,
}
use Kind::*;

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, d::Add, d::AddAssign, d::Sub, Default,
)]
struct Resources {
    ore: usize,
    clay: usize,
    obsidian: usize,
    geode: usize,
}

impl Resources {
    fn count(&self) -> usize {
        self.ore + self.clay + self.obsidian + self.geode
    }

    fn iter(&self) -> impl Iterator<Item = (Kind, usize)> {
        [
            (Ore, self.ore),
            (Clay, self.clay),
            (Obsidian, self.obsidian),
            (Geode, self.geode),
        ]
        .into_iter()
    }
}

impl Resources {
    fn afford(&self, other: &Self) -> bool {
        self.ore >= other.ore
            && self.clay >= other.clay
            && self.obsidian >= other.obsidian
            && self.geode >= other.geode
    }
}

impl Mul<usize> for Resources {
    type Output = Self;

    fn mul(self, rhs: usize) -> Self::Output {
        Self {
            ore: self.ore * rhs,
            clay: self.clay * rhs,
            obsidian: self.obsidian * rhs,
            geode: self.geode * rhs,
        }
    }
}

impl Mul<usize> for &Resources {
    type Output = Resources;

    fn mul(self, rhs: usize) -> Self::Output {
        (*self) * rhs
    }
}

impl Index<Kind> for Resources {
    type Output = usize;

    fn index(&self, index: Kind) -> &Self::Output {
        match index {
            Ore => &self.ore,
            Clay => &self.clay,
            Obsidian => &self.obsidian,
            Geode => &self.geode,
        }
    }
}

impl IndexMut<Kind> for Resources {
    fn index_mut(&mut self, index: Kind) -> &mut Self::Output {
        match index {
            Ore => &mut self.ore,
            Clay => &mut self.clay,
            Obsidian => &mut self.obsidian,
            Geode => &mut self.geode,
        }
    }
}

impl FromStr for Resources {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut out = Resources::default();
        for c in s.split("and ") {
            let (n, kind) = c
                .trim()
                .split_once(' ')
                .map(|(n, k)| (n.parse::<usize>().unwrap(), k))
                .unwrap();
            let kind = match kind {
                "ore" => &mut out.ore,
                "clay" => &mut out.clay,
                "obsidian" => &mut out.obsidian,
                _ => panic!("{kind}"),
            };
            *kind += n;
        }

        Ok(out)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
struct Blueprint {
    costs: [Resources; Kind::Geode as usize + 1],
    max_cost_by_kind: [usize; Kind::Geode as usize + 1],
}

impl Blueprint {
    fn max_cost_for_kind(&self, kind: Kind) -> usize {
        self.max_cost_by_kind[kind as usize]
    }
}

impl Index<Kind> for Blueprint {
    type Output = Resources;

    fn index(&self, index: Kind) -> &Self::Output {
        &self.costs[index as usize]
    }
}

impl FromStr for Blueprint {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let it = s
            .split_once(": ")
            .unwrap()
            .1
            .split('.')
            .filter(|s| !s.is_empty());

        let costs = it
            .map(|s| {
                s.split_once("costs ")
                    .unwrap()
                    .1
                    .parse::<Resources>()
                    .unwrap()
            })
            .arr();

        let mut max_cost_by_kind = [0; Kind::Geode as usize + 1];
        for cost in costs {
            for (kind, cost) in cost.iter() {
                let m = &mut max_cost_by_kind[kind as usize];
                *m = (*m).max(cost);
            }
        }

        Ok(Self {
            costs,
            max_cost_by_kind,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Blueprints {
    inner: Vec<Blueprint>,
}

impl FromStr for Blueprints {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: s.lines().map(|l| l.parse().unwrap()).collect(),
        })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
struct State<'bp> {
    blueprint: &'bp Blueprint,
    resources: Resources,
    robots: Resources,
}

impl State<'static> {
    fn new(blueprint: &'static Blueprint) -> Self {
        Self {
            blueprint,
            resources: Resources::default(),
            robots: {
                let mut res = Resources::default();
                res[Kind::Ore] = 1;
                res
            },
        }
    }

    // (robots purchased, cost)
    fn purchase_possibilities(
        resources: Resources,
        robots: Resources,
        blueprint: &Blueprint,
    ) -> impl Iterator<Item = (Resources, Resources)> + '_ {
        // (robots bought, remaining resources)
        fn possibilites_for_kind(
            bought: Resources,
            resources: Resources,
            robots: Resources,
            blueprint: &Blueprint,
            kind: Kind,
        ) -> impl Iterator<Item = (Resources, Resources)> + '_ {
            (0..=1) // can buy up to 1 robot per minute
                /* only if we haven't already bought a robot! */
                .filter(move |num_to_buy| (bought.count() + num_to_buy) <= 1)
                /* if we've already got enough robots of this kind to meet our demand, don't bother */
                /* except for Geodes! can't get enough of those */
                .filter(move |&num_to_buy| {
                    let demand = blueprint.max_cost_for_kind(kind);
                    let got = robots[kind];
                    let skip = kind != Geode && num_to_buy > 0 && got >= demand;

                    !skip
                })
                // .inspect(move |x| eprintln!("buying {x} {kind:?}"))
                .map(move |num_to_buy| (num_to_buy, blueprint[kind] * num_to_buy))
                .take_while(move |(_num, cost)| resources.afford(cost))
                .map(move |(num, cost)| {
                    (
                        {
                            let mut robots = bought;
                            robots[kind] += num;
                            // eprintln!("buyin: {robots:?}");
                            robots
                        },
                        resources - cost,
                    )
                })
        }

        #[allow(unused)]
        fn greedy<T>(x: impl Iterator<Item = T>) -> impl Iterator<Item = T> {
            once(x.last().unwrap())
        }

        let (res, rbt, bp) = (resources, robots, blueprint);
        // I missed that we can only build 1 robot a cycle! This changes the
        // problem significantly.
        //
        // some optimiziations fall out of this:
        //   - stop building robots of a particular kind once you've exceeded
        //     the highest demanding bot's need for that kind
        //     + more generally, if you're on track for exceeding the supply
        //       that building 1 of that bot on every remaining turn would
        //       require... (different than ^ as this accounts for somehow
        //       having a surplus in the beginning)
        //   -
        //
        // and, as a heuristic:
        //   - if we can buy a geode robot, always buy it
        //     + i don't think this is _necessarily_ better (i think it's
        //       plausible that investing in a clay robot or w/e can ultimately
        //       lead you to be able to build more geode bots?) but it seems to
        //       hold for our inputs...
        greedy(possibilites_for_kind(
            Default::default(),
            res,
            rbt,
            bp,
            Geode,
        )) // always build Geodes if we can..
        .flat_map(move |(b, r)| possibilites_for_kind(b, r, rbt, bp, Obsidian))
        .flat_map(move |(b, r)| possibilites_for_kind(b, r, rbt, bp, Clay))
        // .flat_map(|(bought, resources)| possibilites_for_kind(bought, resources, blueprint, Ore))
        .flat_map(move |(b, r)| possibilites_for_kind(b, r, rbt, bp, Ore))

        /* // most expensive first
        possibilites_for_kind(Default::default(), resources, blueprint, Geode)
            .flat_map(|(bought, resources)| {
                possibilites_for_kind(bought, resources, blueprint, Obsidian)
            })
            .flat_map(|(bought, resources)| {
                possibilites_for_kind(bought, resources, blueprint, Clay)
            })
            // .flat_map(|(bought, resources)| possibilites_for_kind(bought, resources, blueprint, Ore))
            .flat_map(|(bought, resources)| {
                possibilites_for_kind(bought, resources, blueprint, Ore)
            }) */
        // greedy:
        //
        // doesn't work; this is not always the optimal choice..
        /*
        greedy(possibilites_for_kind(
            Default::default(),
            resources,
            blueprint,
            Geode,
        ))
        .flat_map(|(bought, resources)| {
            greedy(possibilites_for_kind(
                bought, resources, blueprint, Obsidian,
            ))
        })
        .flat_map(|(bought, resources)| {
            greedy(possibilites_for_kind(bought, resources, blueprint, Clay))
        })
        .flat_map(|(bought, resources)| {
            greedy(possibilites_for_kind(bought, resources, blueprint, Ore))
        }) */
    }

    /*     // ((bought, have), resources, robots)
    fn search(&self, time: usize) -> (Vec<(Resources, Resources)>, Resources, Resources) {
        thread_local! {
            static CACHE: RefCell<HashMap<(State<'static>, usize), (Vec<(Resources, Resources)>, Resources, Resources)>>
                = RefCell::new(HashMap::new());
        }

        if let Some(cached) = CACHE.with(|c| c.borrow().get(&(*self, time)).cloned()) {
            return cached;
        }

        // eprintln!(
        //     "\ntime = {time} | have: resource({:?}) robots({:?})",
        //     self.resources, self.robots
        // );
        if time == 0 {
            return (vec![], self.resources, self.robots);
        }

        // produce
        let produced = self.robots;

        let mut max_minimum_geode_count = 0;

        // consider all the different sets of things we could buy:
        let ret = Self::purchase_possibilities(self.resources, self.robots, self.blueprint)
            .inspect(|_| {
                // count += 1;
                // if time >= 5 {
                //     eprintln!("[{time:02}] count: {count}");
                // }
            })
            // for each, calculate the amount of resources and robots available
            // by the end of the turn:
            .map(|(purchased, remaining)| {
                let robots = self.robots + purchased;
                let resources = remaining + produced;
                // eprintln!(
                //     "had: {:?} | produced: {produced:?} => resources: {resources:?}",
                //     self.resources
                // );
                (
                    (purchased, resources),
                    Self {
                        robots,
                        resources,
                        blueprint: self.blueprint,
                    },
                )
            })
            /******************************************************************/
            /*
            // eagerly pick the max! this turns our DFS into a weird kind of
            // pseudo BFS search
            .max_by(|(_, a), (_, b)| {
                use std::cmp::Ordering::*;
                // we don't actually _know_ what's better so we're using a dumb
                // heuristic
                // pick whichever outcome has the most geode robots
                match a.robots.geode.cmp(&b.robots.geode) {
                    // with individual geodes as the tie-breaker:
                    Equal => match a.resources.geode.cmp(&b.resources.geode) {
                        Equal => {}
                        x => return x,
                    },
                    x => return x,
                }
                // next up we'll prioritize
                todo!()
            })
            .into_iter() */
            /******************************************************************/
            // we _can_ prune out possibilities that we can prove are inferior
            // here (i.e. have a max(min_projected_throughput) and filter out
            // any possibilities that won't hit it even if they build a geode
            // bot every remaining minute)
            .inspect(|(_, state)| {
                // assuming we produce no new geode bots, how many geodes will
                // we produce?
                let minimum_final_geode_count =
                    state.robots[Geode] * (time - 1) + state.resources[Geode];

                max_minimum_geode_count = max_minimum_geode_count.max(minimum_final_geode_count);
            })
            .collect_vec()
            .into_iter()
            .filter(|(_, state)| {
                // now, assuming we produce new geodes on *every* remaining
                // minute, how many geodes will we produce?
                let min_final_count = state.robots[Geode] * (time - 1) + state.resources[Geode];

                // producing 1 new geode _bot_ for every remaining minute means
                // that
                //   - next minute we'll make: 0 additional geodes but 1 bot
                //   - +2 minutes we'll make:  1 addtional geode and 1 bot
                //   - +3:                     2, 1 bot
                //   - +4:                     3, 1 bot
                // ...
                //
                // this is `(0..(time - 1)).sum()` which in closed form is
                // (time - 1) * (time) / 2
                let time = time - 1;
                let max_final_count = min_final_count + (((time - 1) * time) / 2);

                // if we can't beat the minimum we're definitely not the right
                // answer
                max_final_count >= max_minimum_geode_count
            })
            /******************************************************************/
            // continue the search for each of these possiblities:
            .map(|((purchased, have), state)| ((purchased, have), state.search(time - 1)))
            // and finally take the best possibility:
            .max_by_key(|(_state_tracking, (_moves, resources, _robots))| resources[Geode])
            .map(|((purchased, have), (mut moves, resources, robots))| {
                moves.push((purchased, have));
                (moves, resources, robots)
            })
            .unwrap();

        CACHE.with(|c| c.borrow_mut().insert((*self, time), ret.clone()));
        ret
    } */

    // ((bought, have), resources, robots)
    fn search(&self, time: usize) -> (/* Vec<(Resources, Resources)>, */ Resources, Resources) {
        thread_local! {
            static CACHE: RefCell<HashMap<(State<'static>, usize), (/* Vec<(Resources, Resources)>, */Resources, Resources)>>
                = RefCell::new(HashMap::new());
        }

        if let Some(cached) = CACHE.with(|c| c.borrow().get(&(*self, time)).cloned()) {
            return cached;
        }

        // eprintln!(
        //     "\ntime = {time} | have: resource({:?}) robots({:?})",
        //     self.resources, self.robots
        // );
        if time == 0 {
            return (/* vec![],  */ self.resources, self.robots);
        }

        // produce
        let produced = self.robots;

        let mut max_minimum_geode_count = 0;

        // consider all the different sets of things we could buy:
        let ret = Self::purchase_possibilities(self.resources, self.robots, self.blueprint)
            .inspect(|_| {
                // count += 1;
                // if time >= 5 {
                //     eprintln!("[{time:02}] count: {count}");
                // }
            })
            // for each, calculate the amount of resources and robots available
            // by the end of the turn:
            .map(|(purchased, remaining)| {
                let robots = self.robots + purchased;
                let resources = remaining + produced;
                // eprintln!(
                //     "had: {:?} | produced: {produced:?} => resources: {resources:?}",
                //     self.resources
                // );
                (
                    (purchased, resources),
                    Self {
                        robots,
                        resources,
                        blueprint: self.blueprint,
                    },
                )
            })
            /******************************************************************/
            /*
            // eagerly pick the max! this turns our DFS into a weird kind of
            // pseudo BFS search
            .max_by(|(_, a), (_, b)| {
                use std::cmp::Ordering::*;
                // we don't actually _know_ what's better so we're using a dumb
                // heuristic
                // pick whichever outcome has the most geode robots
                match a.robots.geode.cmp(&b.robots.geode) {
                    // with individual geodes as the tie-breaker:
                    Equal => match a.resources.geode.cmp(&b.resources.geode) {
                        Equal => {}
                        x => return x,
                    },
                    x => return x,
                }
                // next up we'll prioritize
                todo!()
            })
            .into_iter() */
            /******************************************************************/
            // we _can_ prune out possibilities that we can prove are inferior
            // here (i.e. have a max(min_projected_throughput) and filter out
            // any possibilities that won't hit it even if they build a geode
            // bot every remaining minute)
            .inspect(|(_, state)| {
                // assuming we produce no new geode bots, how many geodes will
                // we produce?
                let minimum_final_geode_count =
                    state.robots[Geode] * (time - 1) + state.resources[Geode];

                max_minimum_geode_count = max_minimum_geode_count.max(minimum_final_geode_count);
            })
            .collect_vec()
            .into_iter()
            .filter(|(_, state)| {
                // now, assuming we produce new geodes on *every* remaining
                // minute, how many geodes will we produce?
                let min_final_count = state.robots[Geode] * (time - 1) + state.resources[Geode];

                // producing 1 new geode _bot_ for every remaining minute means
                // that
                //   - next minute we'll make: 0 additional geodes but 1 bot
                //   - +2 minutes we'll make:  1 addtional geode and 1 bot
                //   - +3:                     2, 1 bot
                //   - +4:                     3, 1 bot
                // ...
                //
                // this is `(0..(time - 1)).sum()` which in closed form is
                // (time - 1) * (time) / 2
                let time = time - 1;
                let max_final_count = min_final_count + (((time - 1) * time) / 2);

                // if we can't beat the minimum we're definitely not the right
                // answer
                max_final_count >= max_minimum_geode_count
            })
            /******************************************************************/
            // continue the search for each of these possiblities:
            .map(|((purchased, have), state)| ((purchased, have), state.search(time - 1)))
            // and finally take the best possibility:
            .max_by_key(|(_state_tracking, (/* _moves,  */ resources, _robots))| resources[Geode])
            .map(
                |((purchased, have), (/* mut moves,  */ resources, robots))| {
                    // moves.push((purchased, have));
                    (/* moves,  */ resources, robots)
                },
            )
            .unwrap();

        CACHE.with(|c| c.borrow_mut().insert((*self, time), ret.clone()));
        ret
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2022, 19);
    let inp = aoc.get_input();
    // let inp = include_str!("ex");

    let blueprints: Blueprints = inp.parse().unwrap();
    let blueprints = blueprints.inner.leak();
    // dbg!(&blueprints);

    let p1: usize = blueprints
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let s = State::new(b);
            let /* mut  */res = s.search(24);
            // res.0.reverse();
            // let moves = res.0;
            // dbg!(moves);
            // let res = res.1[Geode];
            let res = res.0[Geode];
            (i, dbg!(res))
        })
        .map(|(i, count)| (i + 1) * count)
        .sum();
    dbg!(p1);

    // let p1 = p1(&inp);
    // aoc.submit_p1(dbg!(p1)).unwrap();

    let p2: usize = blueprints
        .iter()
        .take(3)
        .map(|b| {
            let s = State::new(b);
            let /* mut  */res = s.search(32);
            // res.0.reverse();
            // let moves = res.0;
            // dbg!(moves);
            // let res = res.1[Geode];
            let res = res.0[Geode];
            dbg!(res)
        })
        .product();

    dbg!(p2);
    // aoc.submit_p2(dbg!(p2)).unwrap();

    // let p2 = p2(&inp);
    // aoc.submit_p2(dbg!(p2)).unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn p1_small() {
        const EX_SMALL: &str = "1,1,1\n2,1,1\n";
        assert_eq!(super::p1(EX_SMALL), 10);
    }

    #[test]
    fn p1_ex() {
        assert_eq!(super::p1(include_str!("ex")), 64);
    }

    #[test]
    fn p2_ex() {
        assert_eq!(super::p2(include_str!("ex")), 58);
    }
}

/*

ore  -> O
clay -> C
obsi -> D
geod -> G

G = 3 * O + 12 * D
D = 3 * D +  8 * C
C = 3 * O
O = 2 * O


0 - 0 | 1 ->  1
1 - 0 | 1 ->  1
2 - 2 | 1 ->  1  | 1
1 - 0 | 2 ->  2  | 0
3 - 2 | 2 ->  2  | 1
3 - 2 | 3 ->  3  | 1
4 - 4 | 4 ->  4  | 2
4 - 4 | 6 ->  6  | 2
6 - 6 | 8 ->  8  | 3
8 - 8 | 11 -> 11 | 4


4 - 4 | 2 ->  2  | 2
2 - 2 | 4 ->  4  | 1
4 - 4 | 4 ->  4  | 2
4 - 4 | 6 ->  6  | 2
6 - 6 |        | 9 -> 18


0: 0o
1: 1o
2: 2o
3: 3o
4: 4o
*/
