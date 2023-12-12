use aoc::*;

use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap, HashSet, LinkedList},
    fmt::Write,
    iter::{once, repeat},
    mem,
    sync::Mutex,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Valve<'s>(&'s str);

#[derive(Debug, Clone, PartialEq, Eq)]
struct Link<'s> {
    flow_rate: usize,
    leads_to: HashMap<Valve<'s>, Vec<Action<'s>>>, // (valve => steps to get there)
}

#[derive(Debug, Clone)]
struct Report<'s> {
    map: HashMap<Valve<'s>, Link<'s>>,
    shortest_paths: HashMap<(Valve<'s>, Valve<'s>), Option<usize>>,
}

impl Report<'_> {
    fn dot(&self, directed: bool) -> String {
        let mut seen = HashSet::new();

        let mut buf = String::new();
        writeln!(
            buf,
            "{} tunnels {{",
            if directed { "digraph" } else { "graph " }
        )
        .unwrap();
        buf.write_str("  forcelabels=true\n").unwrap();

        for (src, links) in self.map.iter() {
            writeln!(buf, "  {} [xlabel =\"{}\"] ", src.0, links.flow_rate).unwrap();
            for (to, path) in &links.leads_to {
                if !directed && seen.contains(&(*to, *src)) {
                    continue;
                }

                if !directed {
                    seen.insert((*src, *to));
                }

                writeln!(
                    buf,
                    "  {} -- {} [label = \"{} ({})\"];",
                    src.0,
                    to.0,
                    path.len(),
                    path.iter()
                        .map(|a| match a {
                            Action::Travel(to) => to.0,
                            _ => unreachable!(),
                        })
                        .join(", ")
                )
                .unwrap();
            }
        }

        buf.write_str("}").unwrap();
        buf
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Action<'s> {
    Travel(Valve<'s>),
    Open(Valve<'s>),
    Nothing,
}

impl<'s> Report<'s> {
    pub fn new(inp: &'s str, prune: bool) -> Report<'s> {
        let mut map = HashMap::new();
        for l in inp.lines() {
            let l = l.strip_prefix("Valve ").unwrap();
            let (source, rest) = l.split_once(' ').unwrap();
            let l = rest.strip_prefix("has flow rate=").unwrap();
            let (flow_rate, rest) = l.split_once(';').unwrap();

            let leads_to = rest
                .split(' ')
                .skip(5)
                .map(|v| v.strip_suffix(',').unwrap_or(v))
                .map(Valve)
                .map(|v| (v, Vec::from([Action::Travel(v)])))
                .collect();

            let res = map.insert(
                Valve(source),
                Link {
                    flow_rate: flow_rate.parse().unwrap(),
                    leads_to,
                },
            );
            assert_eq!(res, None);
        }

        let mut out = Report {
            map,
            shortest_paths: HashMap::new(),
        };
        if prune {
            out.prune();
        } else {
            out.shortest_paths = Self::shortest_paths(&out.map);
        }

        out
    }

    fn shortest_paths(
        map: &HashMap<Valve<'s>, Link<'s>>,
    ) -> HashMap<(Valve<'s>, Valve<'s>), Option<usize>> {
        let mut paths = HashMap::with_capacity(map.len() * map.len());
        let node_list = map.keys().copied().collect_vec();
        for &from in &node_list {
            for &to in &node_list {
                paths.insert((from, to), if from == to { Some(0) } else { None });
            }
        }

        for (&from, link) in map.iter() {
            for (&to, steps) in &link.leads_to {
                paths.insert((from, to), Some(steps.len()));
            }
        }

        // floyd-warshall
        for &k in &node_list {
            for &i in &node_list {
                for &j in &node_list {
                    // shortest path from `i → j` is either the current path
                    // from `i → j` (not involving `k`) or is the path from
                    // `i → k` plus the path from `k → j`
                    if let Some((l, r)) = paths[&(i, k)].zip(paths[&(k, j)]) {
                        let dist = l + r;
                        let shorter = match (paths[&(i, j)], dist) {
                            (Some(curr), new) if new < curr => true,
                            (None, _) => true,
                            _ => false,
                        };

                        if shorter {
                            paths.insert((i, j), Some(dist));
                        }
                    }
                }
            }
        }

        paths
    }

    // eliminates corridors
    //
    // the graph is actually undirected; for every edge there's a corresponding
    // edge going the other way
    pub fn prune(&mut self) {
        let mut to_map = mem::take(&mut self.map); // src -> dest
        let mut from_map: HashMap<Valve<'s>, HashSet<Valve<'s>>> =
            HashMap::with_capacity(to_map.len()); // dest <- src

        for (&src, dests) in to_map.iter() {
            for dest in dests.leads_to.keys() {
                from_map.entry(*dest).or_default().insert(src);
            }
        }
        // dbg!(&to_map);
        // dbg!(&from_map);

        // for all the nodes, if a node only has one successor and one
        // predecessor and a flow rate that's 0 (i.e. if it is a corridor) fold
        // it away
        let nodes = to_map.keys().copied().collect_vec();
        for n in nodes {
            let node = &to_map[&n];
            let to = &node.leads_to;
            let from = &from_map[&n];

            // because we're modeling this as a directed graph despite every
            // edge being reflexive, we have to do this `to == from`, len == 2
            // check
            if node.flow_rate == 0
                && to.len() == 2
                && from.len() == 2
                && &HashSet::from_iter(to.keys().copied()) == from
            {
                // we can fold this node away! this means:
                //  - deleting it from `to_map` and `from_map`
                //  - deleting it from `from[succ]` and `from[pred]`
                //  - rewriting the successor to point to the predecessor in
                //    `to_map` and `from_map`
                //  - rewriting the predecessor to point to the successor in
                //    `to_map` and `from_map`
                //
                // since we're not directed we can just pick one node to call
                // the predecessor and one to call the successor:
                let linked = to_map.remove(&n).unwrap().leads_to;
                let ((pred, to_pred_steps), (succ, to_succ_steps)) =
                    linked.iter().collect_tuple().unwrap();
                from_map.remove(&n);

                assert!(from_map.get_mut(pred).unwrap().remove(&n));
                assert!(from_map.get_mut(succ).unwrap().remove(&n));

                // the paths above (`to_pred_steps`, `to_succ_steps`) are from
                // `n` to `pred` and `n` to `succ` respectively.
                // ```
                //   pred <---- n ----> succ
                //          |       \----> to_succ_steps
                //          \--> to_pred_steps
                // ```
                // to form the paths from `pred` to `succ` and `succ` to `pred`
                // we want:
                // ```
                //   pred <---- – <---- succ
                //   pred ----> – ----> succ
                // ```
                //
                // which means `succ -> pred` = `to_succ_steps.rev() + to_pred_steps`
                // and `pred -> succ` = `to_pred_steps.rev() + to_succ_steps`

                // update pred:
                let path = to_pred_steps
                    .iter()
                    .copied()
                    .rev()
                    .chain(to_succ_steps.iter().copied())
                    .collect();
                let pred_next = &mut to_map.get_mut(pred).unwrap().leads_to;
                pred_next.remove(&n);
                pred_next.insert(*succ, path);
                from_map.get_mut(pred).unwrap().insert(*succ);

                // update succ:
                let path = to_succ_steps
                    .iter()
                    .copied()
                    .rev()
                    .chain(to_pred_steps.iter().copied())
                    .collect();
                let succ_next = &mut to_map.get_mut(succ).unwrap().leads_to;
                succ_next.remove(&n);
                succ_next.insert(*pred, path);
                from_map.get_mut(succ).unwrap().insert(*pred);
            }
        }

        self.map = to_map;
        self.shortest_paths = Self::shortest_paths(&self.map);
    }

    pub fn max_path<const TIME: usize>(
        &self,
        starting_at: Valve<'s>,
    ) -> ([Action<'s>; TIME], usize) {
        let mut visited = BTreeSet::new();
        let mut table = HashMap::with_capacity(TIME * self.map.len());
        let all_nonzero = self
            .map
            .keys()
            .copied()
            .filter(|v| self.map[v].flow_rate != 0)
            .collect();

        let (path, score) =
            self.max_path_inner(starting_at, &mut visited, TIME, &mut table, &all_nonzero);

        assert_eq!(path.len(), TIME);
        let mut out = [Action::Nothing; TIME];
        for (idx, n) in path.into_iter().enumerate() {
            out[idx] = n;
        }

        (out, score)
    }

    fn max_path_inner(
        &self,
        next: Valve<'s>,
        opened: &mut BTreeSet<Valve<'s>>,
        time_remaining: usize,
        table: &mut HashMap<
            (Valve<'s>, BTreeSet<Valve<'s>>, usize),
            (LinkedList<Action<'s>>, usize),
        >,
        all_nonzero: &BTreeSet<Valve<'s>>,
    ) -> (LinkedList<Action<'s>>, usize) {
        // eprintln!("next: {next:?} @ {time_remaining} | visited: {opened:?}");
        if table.len() % 100_000 == 0 {
            eprintln!(
                "time_remaining: {time_remaining}; table len: {}",
                table.len()
            );
        }
        if time_remaining == 0 {
            return (LinkedList::new(), 0);
        }

        let key = (next, opened.clone(), time_remaining);
        if let Some(res) = table.get(&key) {
            return res.clone();
        }

        // No use continuing the search..
        //
        // (see the comment below)
        if opened.is_superset(all_nonzero) {
            // in practice (for the real input data) we never actually get here?
            //
            // perhaps we ought to have `all_nonzero` be the set of things that
            // are reachable from the starting node..
            //
            // update: everything is reachable..
            return (repeat(Action::Nothing).take(time_remaining).collect(), 0);
        } else {
            // let diff = all_nonzero.difference(opened);
            // eprintln!("missing: {:?} of {:?}", diff.count(), all_nonzero.len());
            // opened.sub
        }

        ///////////////////////////////////////////////////////////////////////

        let node = &self.map[&next];

        // We have three options:
        //  - spend this minute moving through a tunnel to another valve
        //  - open the current valve (if not open)
        //  - do nothing
        //
        // We fold the last two options since it's strictly preferable to open
        // the current valve if possible than to do nothing.
        //
        // We also introduce two heuristics to reduce the search space:
        //  - the majority of our valves have a flow rate of 0. it never makes
        //    sense to open these and so, we gate `option1` on the flow rate
        //    being non-zero. we always want to move; that is always preferable
        //    + this matters less now that we've eliminated corridors but it's
        //      a cheap check so why not
        //  - once we've opened all the non-zero valves we can stop the search
        //    (see the check above -- not memoized but w/e)

        let option1 = if !opened.contains(&next) && node.flow_rate != 0 {
            // If we haven't already opened this valve we can spend this minute
            // opening it.
            opened.insert(next);
            let time_remaining = time_remaining - 1;
            let score = time_remaining * node.flow_rate;

            let (steps, next_score) =
                self.max_path_inner(next, opened, time_remaining, table, all_nonzero);
            opened.remove(&next);

            (
                LinkedList::from([Action::Open(next)]),
                steps,
                score + next_score,
            )
        } else {
            // // Alternatively we can just do nothing this minute.
            // let (steps, next_score) =
            //     self.max_path_inner(next, opened, time_remaining - 1, table, all_nonzero);
            // (Action::Nothing, steps, next_score)

            // This should never be taken; it's always preferable to move than
            // to do nothing (and we handle the case where we're starved for
            // inputs with the `all_nonzero` check above).

            // Somehow this _does_ actually get taken (an artifact of
            // `max_by_key` picking later results perhaps?) so we have to at
            // least make sure the length of the linked list is right
            //
            // This also gets taken when there are no places (non-corridor
            // places) that we can travel in the time remaining.

            (
                LinkedList::from([]),
                repeat(Action::Nothing).take(time_remaining).collect(),
                0,
            )
        };

        // The other option is to spend this minute moving:
        let move_options = node
            .leads_to
            .iter()
            .filter(|(_, path)| path.len() <= time_remaining)
            .map(|(n, steps)| {
                // we want to visit nodes we've already visited in case they
                // lead us to new paths so we don't gate this on `opened`..
                let (steps_next, next_score) = self.max_path_inner(
                    *n,
                    opened,
                    time_remaining - steps.len(),
                    table,
                    all_nonzero,
                );
                (
                    LinkedList::from_iter(steps.iter().copied()),
                    steps_next,
                    next_score,
                )
            });

        // We want to pick whatever option gets us the best score:
        let (mut steps, steps_next, score) = move_options
            .chain(once(option1))
            .max_by_key(|(_, _, s)| *s)
            .unwrap();

        // Construct our result (the full path):
        steps.extend(steps_next);

        // insert into the table (memoize)
        table.insert(key, (steps.clone(), score));
        (steps, score)
    }

    /*     pub fn max_path<const TIME: usize>(
        &self,
        starting_at: Valve<'s>,
    ) -> ([Action<'s>; TIME], usize) {
        let mut visited = BTreeSet::new();
        let mut table = HashMap::with_capacity(TIME * self.map.len());
        let all_nonzero = self
            .map
            .keys()
            .copied()
            .filter(|v| self.map[v].flow_rate != 0)
            .collect();

        let (path, score) =
            self.max_path_inner(starting_at, &mut visited, TIME, &mut table, &all_nonzero);

        dbg!(all_nonzero);
        dbg!(&path);
        assert_eq!(path.len(), TIME);
        let mut out = [Action::Nothing; TIME];
        for (idx, n) in path.into_iter().enumerate() {
            out[idx] = n;
        }

        (out, score)
    }

    fn max_path_inner(
        &self,
        next: Valve<'s>,
        opened: &mut BTreeSet<Valve<'s>>,
        time_remaining: usize,
        table: &mut HashMap<
            (Valve<'s>, BTreeSet<Valve<'s>>, usize), // 30 * (2 ^ )
            (LinkedList<Action<'s>>, usize),
        >,
        all_nonzero: &BTreeSet<Valve<'s>>,
    ) -> (LinkedList<Action<'s>>, usize) {
        // eprintln!("next: {next:?} @ {time_remaining} | visited: {opened:?}");
        if table.len() % 100_000 == 0 {
            eprintln!(
                "time_remaining: {time_remaining}; table len: {}",
                table.len()
            );
        }
        if time_remaining == 0 {
            return (LinkedList::new(), 0);
        }

        let key = (next, opened.clone(), time_remaining);
        if let Some(res) = table.get(&key) {
            return res.clone();
        }

        // No use continuing the search..
        //
        // (see the comment below)
        if opened.is_superset(all_nonzero) {
            // in practice (for the real input data) we never actually get here?
            //
            // perhaps we out to have `all_nonzero` be the set of things that
            // are reachable from the starting node..
            return (repeat(Action::Nothing).take(time_remaining).collect(), 0);
        } else {
            // let diff = all_nonzero.difference(opened);
            // eprintln!("missing: {:?} of {:?}", diff.count(), all_nonzero.len());
            // opened.sub
        }

        ///////////////////////////////////////////////////////////////////////

        let node = &self.map[&next];

        // We have three options:
        //  - spend this minute moving through a tunnel to another valve
        //  - open the current valve (if not open)
        //  - do nothing
        //
        // We fold the last two options since it's strictly preferable to open
        // the current valve if possible than to do nothing.
        //
        // We also introduce two heuristics to reduce the search space:
        //  - the majority of our valves have a flow rate of 0. it never makes
        //    sense to open these and so, we gate `option1` on the flow rate
        //    being non-zero. we always want to move; that is always preferable
        //  - once we've opened all the non-zero valves we can stop the search
        //    (see the check above -- not memoized but w/e)

        let option1 = if !opened.contains(&next) && node.flow_rate != 0 {
            // If we haven't already opened this valve we can spend this minute
            // opening it.
            opened.insert(next);
            let time_remaining = time_remaining - 1;
            let score = time_remaining * node.flow_rate;

            let (steps, next_score) =
                self.max_path_inner(next, opened, time_remaining, table, all_nonzero);
            opened.remove(&next);

            (Action::Open(next), steps, score + next_score)
        } else {
            // // Alternatively we can just do nothing this minute.
            // let (steps, next_score) =
            //     self.max_path_inner(next, opened, time_remaining - 1, table, all_nonzero);
            // (Action::Nothing, steps, next_score)

            // This should never be taken; it's always preferable to move than
            // to do nothing (and we handle the case where we're starved for
            // inputs with the `all_nonzero` check above).

            // Somehow this _does_ actually get taken (an artifact of
            // `max_by_key` picking later results perhaps?) so we have to at
            // least make sure the length of the linked list is right

            (
                Action::Nothing,
                repeat(Action::Nothing).take(time_remaining - 1).collect(),
                0,
            )
        };

        // The other option is to spend this minute moving:
        let move_options = node.leads_to.iter().map(|n| {
            // we want to visit nodes we've already visited in case they
            // lead us to new paths..
            let (steps, next_score) =
                self.max_path_inner(*n, opened, time_remaining - 1, table, all_nonzero);
            (Action::Travel(*n), steps, next_score)
        });

        // We want to pick whatever option gets us the best score:
        let (action, mut steps, score) = move_options
            .chain(once(option1).filter(|_| node.flow_rate != 0))
            .max_by_key(|(_, _, s)| *s)
            .unwrap();

        // Construct our result (the list of )
        steps.push_front(action);

        // insert into the table (memoize)
        table.insert(key, (steps.clone(), score));
        (steps, score)
    } */
}

// Duplicate of the above logic (😬) but with a pair of actors instead of just
// one:
/* impl<'s> Report<'s> {
    pub fn max_path_duo<const TIME: usize>(
        &self,
        starting_at: Valve<'s>,
    ) -> ([(Action<'s>, Action<'s>); TIME], usize) {
        let mut opened = BTreeSet::new();
        let mut table = HashMap::with_capacity(TIME * self.map.len() * self.map.len());
        let all_nonzero = self
            .map
            .keys()
            .copied()
            .filter(|v| self.map[v].flow_rate != 0)
            .collect();

        let (path, score) = self.max_path_duo_inner(
            (starting_at, starting_at),
            &mut opened,
            TIME,
            &mut table,
            &all_nonzero,
        );

        assert_eq!(path.len(), TIME);
        let mut out = [(Action::Nothing, Action::Nothing); TIME];
        for (idx, n) in path.into_iter().enumerate() {
            out[idx] = n;
        }

        (out, score)
    }

    fn max_path_duo_inner(
        &self,
        next @ (next_a, next_b): (Valve<'s>, Valve<'s>),
        opened: &mut BTreeSet<Valve<'s>>,
        time_remaining: usize,
        table: &mut HashMap<
            ((Valve<'s>, Valve<'s>), BTreeSet<Valve<'s>>, usize),
            (LinkedList<(Action<'s>, Action<'s>)>, usize),
        >,
        all_nonzero: &BTreeSet<Valve<'s>>,
    ) -> (LinkedList<(Action<'s>, Action<'s>)>, usize) {
        if table.len() % 100_000 == 0 {
            eprintln!(
                "time_remaining: {time_remaining}; table len: {}",
                table.len()
            );
        }
        if time_remaining == 0 {
            return (LinkedList::new(), 0);
        }

        let key = (next, opened.clone(), time_remaining);
        if let Some(res) = table.get(&key) {
            return res.clone();
        }

        // No use continuing the search..
        if opened.is_superset(all_nonzero) {
            return (
                repeat((Action::Nothing, Action::Nothing))
                    .take(time_remaining)
                    .collect(),
                0,
            );
        }

        ///////////////////////////////////////////////////////////////////////

        let node_a = &self.map[&next_a];
        let node_b = &self.map[&next_b];

        // Each of a and b has three options:
        //  - move to another tunnel
        //  - open the current value if closed
        //  - do nothing
        //
        // Doing nothing is never optimal so we will ignore it.
        //
        // We have to evaluate the score of a and b's actions as a pair which
        // means a cartesian product.
        //
        // As in the single action version of this function, we elide the "open
        // if closed" option if the current node has a flow rate of 0.

        let gen_options = |starting_valve| {
            let node = &self.map[&starting_valve];
            let opened = opened.contains(&starting_valve);

            // If we haven't already opened this valve and it has a flow
            // rate greater than 0 it makes sense for us to explore what
            // happens if we open it.
            let open_current_valve = once(())
                .filter(|()| node.flow_rate != 0)
                .filter(move |()| !opened)
                .map(move |()| Action::Open(starting_valve));

            // Our other option is to move to other tunnels:
            let move_options = node.leads_to.iter().map(|&v| Action::Travel(v));

            open_current_valve.chain(move_options)
        };

        let a_options = gen_options(next_a);
        let b_options = gen_options(next_b);

        // Test the cartesian product, get scores, and then find the highest:
        let mut seen = HashSet::<(Action, Action)>::with_capacity(
            (node_a.leads_to.len() + 1) * (node_b.leads_to.len() + 1),
        );
        let options = a_options
            .cartesian_product(b_options)
            // We can't open a valve twice in the same timestep (this will lead
            // to double counting for the score).
            .filter(|pair| !matches!(pair, (Action::Open(a), Action::Open(b)) if a == b))
            // (a, b) is the same as exploring (b, a) so we should dedupe:
            .filter(|pair @ (a, b)| {
                if seen.contains(&(*b, *a)) {
                    false
                } else {
                    seen.insert(*pair);
                    true
                }
            })
            .map(
                |(a, b)| -> (
                    (Action, Action),             /* next */
                    LinkedList<(Action, Action)>, /* path */
                    usize,                        /* score */
                ) {
                    use Action::*;

                    let mut step = |action| -> (
                        Valve,         /* next */
                        Option<Valve>, /* removes */
                        usize,         /* score add */
                    ) {
                        match action {
                            Nothing => unreachable!(),
                            Travel(to) => (to, None, 0),
                            Open(valve) => {
                                opened.insert(valve);

                                let score = (time_remaining - 1) * self.map[&valve].flow_rate;

                                (valve, Some(valve), score)
                            }
                        }
                    };

                    let (start_a, rem_a, a_score) = step(a);
                    let (start_b, rem_b, b_score) = step(b);

                    // Get results:
                    let (steps, score) = self.max_path_duo_inner(
                        (start_a, start_b),
                        opened,
                        time_remaining - 1,
                        table,
                        all_nonzero,
                    );

                    // Unwind the changes to the hashmap:
                    let mut rem = |v| {
                        if let Some(v) = v {
                            opened.remove(&v);
                        }
                    };
                    rem(rem_a);
                    rem(rem_b);

                    let step = (a, b);
                    let score = a_score + b_score + score;

                    (step, steps, score)
                },
            );

        // Pick the best path:
        let (step, mut steps, score) = options.max_by_key(|(_, _, s)| *s).unwrap();

        // Construct our result:
        steps.push_front(step);

        // insert into the table (memoize)
        table.insert(key, (steps.clone(), score));
        (steps, score)
    }
} */

// impl<'s> Report<'s> {
//     pub fn max_path_duo_counts<const TIME: usize>(&self, starting_at: Valve<'s>) -> usize {
//         let mut opened = BTreeSet::new();
//         let mut table = HashSet::with_capacity(TIME * self.map.len() * self.map.len() * 100000);
//         let all_nonzero = self
//             .map
//             .keys()
//             .copied()
//             .filter(|v| self.map[v].flow_rate != 0)
//             .collect();

//         self.max_path_duo_inner_count(
//             (starting_at, starting_at),
//             &mut opened,
//             TIME,
//             &mut table,
//             &all_nonzero,
//         )
//     }

//     fn max_path_duo_inner_count(
//         &self,
//         next @ (next_a, next_b): (Valve<'s>, Valve<'s>),
//         opened: &mut BTreeSet<Valve<'s>>,
//         time_remaining: usize,
//         table: &mut HashSet<((Valve<'s>, Valve<'s>), BTreeSet<Valve<'s>>, usize)>,
//         all_nonzero: &BTreeSet<Valve<'s>>,
//     ) -> usize {
//         if table.len() % 100_000 == 0 {
//             eprintln!(
//                 "time_remaining: {time_remaining}; table len: {}",
//                 table.len()
//             );
//         }
//         if time_remaining == 0 {
//             return 1;
//         }

//         let key = (next, opened.clone(), time_remaining);
//         if let Some(res) = table.get(&key) {
//             return 1;
//         }

//         // No use continuing the search..
//         if opened.is_superset(all_nonzero) {
//             return 1;
//         }

//         ///////////////////////////////////////////////////////////////////////

//         let node_a = &self.map[&next_a];
//         let node_b = &self.map[&next_b];

//         // Each of a and b has three options:
//         //  - move to another tunnel
//         //  - open the current value if closed
//         //  - do nothing
//         //
//         // Doing nothing is never optimal so we will ignore it.
//         //
//         // We have to evaluate the score of a and b's actions as a pair which
//         // means a cartesian product.
//         //
//         // As in the single action version of this function, we elide the "open
//         // if closed" option if the current node has a flow rate of 0.

//         let gen_options = |starting_valve| {
//             let node = &self.map[&starting_valve];
//             let opened = opened.contains(&starting_valve);

//             // If we haven't already opened this valve and it has a flow
//             // rate greater than 0 it makes sense for us to explore what
//             // happens if we open it.
//             let open_current_valve = once(())
//                 .filter(|()| node.flow_rate != 0)
//                 .filter(move |()| !opened)
//                 .map(move |()| Action::Open(starting_valve));

//             // Our other option is to move to other tunnels:
//             let move_options = node.leads_to.iter().map(|&v| Action::Travel(v));

//             open_current_valve.chain(move_options)
//         };

//         let a_options = gen_options(next_a);
//         let b_options = gen_options(next_b);

//         // Test the cartesian product, get scores, and then find the highest:
//         let mut seen = HashSet::<(Action, Action)>::with_capacity(
//             (node_a.leads_to.len() + 1) * (node_b.leads_to.len() + 1),
//         );
//         let options = a_options
//             .cartesian_product(b_options)
//             // We can't open a valve twice in the same timestep (this will lead
//             // to double counting for the score).
//             .filter(|pair| !matches!(pair, (Action::Open(a), Action::Open(b)) if a == b))
//             // (a, b) is the same as exploring (b, a) so we should dedupe:
//             .filter(|pair @ (a, b)| {
//                 if seen.contains(&(*b, *a)) {
//                     false
//                 } else {
//                     seen.insert(*pair);
//                     true
//                 }
//             })
//             .inspect(|pair| {
//                 // eprintln!("{:?}", pair);
//             })
//             .map(|(a, b)| -> usize /* count */ {
//                 use Action::*;

//                 let mut step = |action| -> (Valve /* next */, Option<Valve> /* removes */) {
//                     match action {
//                         Nothing => unreachable!(),
//                         Travel(to) => (to, None),
//                         Open(valve) => {
//                             opened.insert(valve);
//                             (valve, Some(valve))
//                         }
//                     }
//                 };

//                 let (start_a, rem_a) = step(a);
//                 let (start_b, rem_b) = step(b);

//                 // Get results:
//                 let count = self.max_path_duo_inner_count(
//                     (start_a, start_b),
//                     opened,
//                     time_remaining - 1,
//                     table,
//                     all_nonzero,
//                 );

//                 // Unwind the changes to the hashset:
//                 let mut rem = |v| {
//                     if let Some(v) = v {
//                         opened.remove(&v);
//                     }
//                 };
//                 rem(rem_a);
//                 rem(rem_b);

//                 count
//             });

//         // Pick the best path:
//         let exp_count = options.sum();

//         // insert into the table (memoize)
//         table.insert(key);

//         exp_count
//     }
// }

// Duplicate of the above logic (😬) but with a pair of actors instead of just
// one:
impl<'s> Report<'s> {
    pub fn max_path_duo<const TIME: usize>(
        &self,
        starting_at: Valve<'s>,
    ) -> ([(Action<'s>, Action<'s>); TIME], usize) {
        let mut opened = BTreeSet::new();
        let mut table = HashMap::with_capacity(TIME * self.map.len() * self.map.len());
        let all_nonzero = self
            .map
            .keys()
            .copied()
            .filter(|v| self.map[v].flow_rate != 0)
            .collect();

        let ((path_a, path_b), score) = self.max_path_duo_inner(
            (starting_at, starting_at),
            &mut opened,
            (TIME, TIME),
            &mut table,
            &all_nonzero,
        );

        assert_eq!(path_a.len(), TIME);
        assert_eq!(path_b.len(), TIME);
        let mut out = [(Action::Nothing, Action::Nothing); TIME];
        for (idx, (a, b)) in path_a.into_iter().zip(path_b.into_iter()).enumerate() {
            out[idx] = (a, b);
        }

        (out, score)
    }

    fn max_path_duo_inner(
        &self,
        next @ (next_a, next_b): (Valve<'s>, Valve<'s>),
        opened: &mut BTreeSet<Valve<'s>>,
        time_remaining @ (time_remaining_a, time_remaining_b): (usize, usize),
        table: &mut HashMap<
            ((Valve<'s>, Valve<'s>), BTreeSet<Valve<'s>>, (usize, usize)),
            ((Vec<Action<'s>>, Vec<Action<'s>>), usize),
        >,
        all_nonzero: &BTreeSet<Valve<'s>>,
    ) -> ((Vec<Action<'s>>, Vec<Action<'s>>), usize) {
        let dbg = false;
        if dbg {
            eprintln!(
                "\nAt ({} [{time_remaining_a}], {} [{time_remaining_b}]).",
                next_a.0, next_b.0
            );
        }

        if table.len() % 1_000_000 == 0 {
            eprintln!(
                "time_remaining: {time_remaining:?}; table len: {}",
                table.len()
            );
        }
        if time_remaining == (0, 0) {
            return ((Vec::new(), Vec::new()), 0);
        }

        let key = (next, opened.clone(), time_remaining);
        if let Some(res) = table.get(&key) {
            return res.clone();
        }

        // No use continuing the search..
        if opened.is_superset(all_nonzero) {
            return (
                (
                    repeat(Action::Nothing).take(time_remaining_a).collect(),
                    repeat(Action::Nothing).take(time_remaining_b).collect(),
                ),
                0,
            );
        }

        ///////////////////////////////////////////////////////////////////////

        let node_a = &self.map[&next_a];
        let node_b = &self.map[&next_b];

        // Each of a and b has three options:
        //  - move to another tunnel
        //  - open the current value if closed
        //  - do nothing
        //
        // Doing nothing is never optimal so we will ignore it.
        //
        // We have to evaluate the score of a and b's actions as a pair which
        // means a cartesian product.
        //
        // As in the single action version of this function, we elide the "open
        // if closed" option if the current node has a flow rate of 0.
        //   - this matters less now that we've eliminated corridors but it's a
        //     cheap check so why not

        let gen_options = |starting_valve, time_remaining| {
            let node = &self.map[&starting_valve];
            let opened = opened.contains(&starting_valve);

            // If we haven't already opened this valve and it has a flow
            // rate greater than 0 it makes sense for us to explore what
            // happens if we open it.
            let open_current_valve = once(())
                .filter(move |()| time_remaining != 0)
                .filter(|()| node.flow_rate != 0)
                .filter(move |()| !opened)
                .map(move |()| vec![Action::Open(starting_valve)]);

            // Our other option is to move to other tunnels:
            //
            // We only bother including tunnels that we can get to in the time
            // remaining.
            let move_options = node
                .leads_to
                .values()
                .filter(move |path| path.len() <= time_remaining)
                .cloned();
            // .map(|path| path.clone());

            // Because we're not longer guaranteed a move option we need to
            // include this default (no nothing and run out the clock) option:
            let default_option =
                once(()).map(move |()| repeat(Action::Nothing).take(time_remaining).collect_vec());

            open_current_valve.chain(move_options).chain(default_option)
        };

        let a_options = gen_options(next_a, time_remaining_a);
        let b_options = gen_options(next_b, time_remaining_b);

        // TODO: if one gets way ahead of the other, we should pause until
        // they're kind of caught up so that the search space isn't
        // unnecessarily massive?
        //
        // also `opened` won't exactly be accurate for the actor that's behind
        // but I don't think it matters -- we've already committed to using the
        // other actor to flip a particular valve at that point so it makes
        // sense that we should avoid it *and* we'll still explore the
        // possibility that we flip the valve first as part of doing the search

        // Test the cartesian product, get scores, and then find the highest:
        let mut _seen = HashSet::<(Action, Action)>::with_capacity(
            (node_a.leads_to.len() + 1) * (node_b.leads_to.len() + 1),
        );
        let options = a_options
            .cartesian_product(b_options)
            // We can't open a single valve twice in the same timestep (this
            // will lead to double counting for the score).
            //
            // Note that `last()` can return `None` here because one actor can
            // run out of time before the other.
            .filter(|(a, b)| {
                !matches!(
                    (a.last(), b.last()),
                    (Some(Action::Open(a)), Some(Action::Open(b))) if a == b
                )
            })
            // (a, b) is the same as exploring (b, a) so we should dedupe:
            //
            // update: this isn't the same anymore now that the timesteps are
            // potentially different...
            /*             .filter(|(a, b)| {
                let (&a, &b) = (a.last().unwrap(), b.last().unwrap());
                if seen.contains(&(b, a)) {
                    false
                } else {
                    seen.insert((a, b));
                    true
                }
            }) */
            .inspect(|(a, b)| {
                if dbg {
                    eprintln!("  Trying: ({a:?}, {b:?}");
                }
            })
            .map(
                |(a, b)| -> (
                    (Vec<Action>, Vec<Action>), /* next */
                    (Vec<Action>, Vec<Action>), /* paths */
                    usize,                      /* score */
                ) {
                    use Action::*;

                    let mut step = |cur: Valve<'s>,
                                    actions: &Vec<Action<'s>>,
                                    time_remaining|
                     -> (
                        Valve,         /* next */
                        Option<Valve>, /* removes */
                        usize,         /* score add */
                        usize,         /* time remaining */
                    ) {
                        match actions.last().unwrap_or(&Nothing) {
                            Nothing => (cur, None, 0, time_remaining - actions.len()),
                            Travel(to) => (*to, None, 0, time_remaining - actions.len()),
                            Open(valve) => {
                                opened.insert(*valve);
                                assert_eq!(actions.len(), 1);

                                let score = (time_remaining - 1) * self.map[valve].flow_rate;

                                (*valve, Some(*valve), score, time_remaining - 1)
                            }
                        }
                    };

                    let (start_a, rem_a, a_score, a_remaining) = step(next_a, &a, time_remaining_a);
                    let (start_b, rem_b, b_score, b_remaining) = step(next_b, &b, time_remaining_b);

                    // Get results:
                    let (next_steps, score) = self.max_path_duo_inner(
                        (start_a, start_b),
                        opened,
                        (a_remaining, b_remaining),
                        table,
                        all_nonzero,
                    );

                    // Unwind the changes to the hashmap:
                    let mut rem = |v| {
                        if let Some(v) = v {
                            opened.remove(&v);
                        }
                    };
                    rem(rem_a);
                    rem(rem_b);

                    let steps = (a, b);
                    let score = a_score + b_score + score;

                    (steps, next_steps, score)
                },
            );

        // Pick the best path:
        let ((mut steps_a, mut steps_b), (next_steps_a, next_steps_b), score) =
            options.max_by_key(|(_, _, s)| *s).unwrap();

        // Construct our result:
        steps_a.extend(next_steps_a);
        steps_b.extend(next_steps_b);
        let steps = (steps_a, steps_b);

        // insert into the table (memoize)
        table.insert(key, (steps.clone(), score));
        (steps, score)
    }
}

// the above but faster (hopefully -- fewer allocations is the goal):

// This:
//  - precomputes the paths from every node to every other node
//    + [from][to] -> cost
//  - just goes and tries:
//    + select each of the remaining unopened nodes and for each: recurse (i.e.
//      dfs)
//      * for part 1 (single actor)
//  - for part two it does the same but at every level of the dfs for single it
//    does another dfs on the remaining nodes (starting at AA again, with the
//    time remaining reset)
//    + the key insight here is that because actual path traversal is separated
//      from picking nodes (i.e. we don't make decisions about which tunnels to
//      go to, just which valves to jump to to open) we can separate the search
//      for the two actors cleanly. in the attempts above, mixing these two
//      things (actually I think it's more that we were trying to use one "list"
//      of opened valves that we mutated as we went up and down in the search)
//      made it hard to separate and made the logic more messy
//    + doing the second DFS *at every level* is essentially our way of testing
//      the different partitioning of valves between the actors; because we try
//      all the orderings (i.e. 2 ^ number of valves options, order sensitive)
//      of valves as part of `single`, we're sure to try all the possible
//      partitionings (by feeding the remnants to a second dfs at every step) of
//      the way in `single`
//
// As an aside, we don't bother with LinkedLists for the unopened node list
// becase it doesn't actually save us allocations if you think about it (to get
// the first N elements of an existing list you need to make a copy of those
// elements; you can put individual elements behind some CoW or ref-counted
// structure to elide _those_ allocations but you can do that with a plain old
// vector too. also the elements – `Valve`s – are tiny and the linkedlist copy/
// traversal cost is almost certainly not worth it for us. we would be able to
// preserve tail linked lists but these lists are going to be tiny anyways)
impl Report<'static> {
    pub fn max_single_flow(&self, starting_at: Valve<'static>, time_remaining: usize) -> usize {
        self.single(
            starting_at,
            self.map.keys().copied().collect(),
            time_remaining,
            // zero if there were no options
            |_| 0,
            // cache tag
            1,
        )
    }

    fn single(
        &self,
        starting_at: Valve<'static>,
        // could do an `impl Iterator` that we keep adding filters to but this
        // won't interact well with the recursion and would probably perform
        // worse (though it would let us elide the allocations)
        rest: Vec<Valve<'static>>,
        time_remaining: usize,
        base_case: impl Clone + Fn(&Vec<Valve<'static>>) -> usize,
        cache_tag: u8,
    ) -> usize {
        thread_local! {
            static CACHE: RefCell<HashMap<
                ((Valve<'static>, Vec<Valve<'static>>, usize), u8),
                usize
            >> = RefCell::new(HashMap::new());
        }

        CACHE.with(|hm| {
            let m = hm.borrow();
            if let Some(res) = m.get(&((starting_at, rest.clone(), time_remaining), cache_tag)) {
                *res
            } else {
                drop(m);

                let res = self.single_inner(
                    starting_at,
                    rest.clone(),
                    time_remaining,
                    base_case,
                    cache_tag,
                );
                let mut m = hm.borrow_mut();
                m.insert(((starting_at, rest, time_remaining), cache_tag), res);
                res
            }
        })
    }

    fn single_inner(
        &self,
        starting_at: Valve<'static>,
        rest: Vec<Valve<'static>>,
        time_remaining: usize,
        base_case: impl Clone + Fn(&Vec<Valve<'static>>) -> usize,
        cache_tag: u8,
    ) -> usize {
        // Pick one valve to visit and open:
        rest.iter()
            .enumerate()
            // only valves we can get to + can get to _before time runs out_
            //
            // for each of these valves, compute the list of remaining valves,
            // were we to visit and open this valve
            //
            // and then recurse
            .filter_map(|(idx, &v)| {
                // we add 1 to the travel time to account for us opening `v`
                if let Some(time_remaining) =
                    time_remaining.checked_sub(self.shortest_paths[&(starting_at, v)]? + 1)
                {
                    let mut remaining = rest.clone();
                    remaining.swap_remove(idx);

                    let score = time_remaining * self.map[&v].flow_rate;

                    Some(
                        score
                            + self.single(
                                v,
                                remaining,
                                time_remaining,
                                base_case.clone(),
                                cache_tag,
                            ),
                    )
                } else {
                    None
                }
            })
            // base case/other option
            .chain(once(base_case(&rest)))
            // pick the best option
            .max()
            .unwrap()
    }

    pub fn max_duo_flow(&self, starting_at: Valve<'static>, time_remaining: usize) -> usize {
        // same as `single` but instead of picking 0 as the other option we
        // run a _second_ DFS on whatever is remaining at that point
        self.single(
            starting_at,
            self.map.keys().copied().collect(),
            time_remaining,
            |rem| {
                // starting from the top, with a full set of time
                self.single(starting_at, rem.clone(), time_remaining, |_| 0, 1)
            },
            // note the different cache tag..
            2,
        )
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2022, 16);
    let inp = aoc.get_input();
    let inp = {
        // since `String::leak` isn't stable..
        let inp = Vec::from(inp.as_bytes()).leak();
        std::str::from_utf8(inp).unwrap()
    };
    // let inp = include_str!("ex");

    // corridors: nodes that only have edges to/from 1 node
    // and have a flow rate of 0
    //
    // 43 of the 59 nodes are corridors

    let rep = Report::<'static>::new(inp, true);
    // rep.prune();
    // println!("{}", rep.dot(false));
    // panic!();
    // let (path, p1) = rep.max_path::<30>(Valve("AA"));
    // dbg!(path);
    // aoc.submit_p1(dbg!(p1)).unwrap();

    // dbg!(rep.max_path_duo_counts::<13>(Valve("AA")));
    // 15 55 246 1252 4453 14100 43141 124484 319673 751401 1673107 3511112 7100127
    // seems to scale as `3 ^ x`, `2 ^ x` ...
    //
    // too high a branching factor for 26 steps? 58,164,240,384 entries (7100127 * 2 ^ 13)
    //
    // need to be able to prune the search space better..
    // panic!();

    // let (path, p2) = rep.max_path_duo::<17>(Valve("AA"));
    // 13:    400_000
    // 15:  1_900_000
    // 17:  6_300_000
    // 19: 18_200_000 -> 9GB
    // ...
    // 26: -> ~240GB???
    // dbg!(path);
    // dbg!(p2);

    // let p1 = p1(reports.iter(), 2000000);

    let p1 = rep.max_single_flow(Valve("AA"), 30);
    aoc.submit_p1(dbg!(p1)).unwrap();

    let p2 = rep.max_duo_flow(Valve("AA"), 26);
    aoc.submit_p2(dbg!(p2)).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reports() -> impl Iterator<Item = Report> {
        include_str!("ex")
            .lines()
            .map(|l| l.parse::<Report>().unwrap())
    }

    #[test]
    fn p1() {
        let r = reports().collect_vec();
        assert_eq!(super::p1(r.iter(), 10), 26);
    }

    #[test]
    fn p2() {
        let r = reports().collect_vec();
        let pos = super::p2(r.iter(), 0..=20).unwrap();
        let score = dbg!(pos).tuning_frequency();
        assert_eq!(score, 56000011);
    }
}
