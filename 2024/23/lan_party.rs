use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use aoc::*;

type AdjacencyMap<'s> = HashMap<&'s str, HashSet<&'s str>>;

fn as_dot(map: &AdjacencyMap) -> String {
    todo!()
}

fn is_clique<'m>(map: &AdjacencyMap, set: impl Iterator<Item = &'m str> + Clone) -> bool {
    for a in set.clone() {
        for b in set.clone() {
            if a == b {
                continue;
            }
            if !map[a].contains(b) {
                return false;
            }
        }
    }
    true
}

// whatever, brute force; ugh

fn main() {
    let mut aoc = AdventOfCode::new(2024, 23);
    let inp = aoc.get_input();
    let adj_map = {
        let mut map: AdjacencyMap = HashMap::new();
        for (a, b) in inp.lines().map(|l| l.split_once('-').unwrap()) {
            map.entry(a).or_default().insert(b);
            map.entry(b).or_default().insert(a);
        }
        map
    };

    let p1 = {
        let mut trios = Vec::new();

        for three in adj_map.keys().copied().combinations(3) {
            if is_clique(&adj_map, three.iter().copied()) {
                trios.push(three);
            }
        }

        trios
            .iter()
            .filter(|trio| trio.iter().any(|m| m.starts_with('t')))
            .count()
    };
    _ = aoc.submit_p1(p1);

    // clique problem
    //
    // dumb brute force solution
    #[cfg(any())]
    let p2 = {
        let mut largest_strongly_connected = None;
        'k_loop: for k in (1..adj_map.len()).rev() {
            eprint!(".");
            for group in adj_map.keys().copied().combinations(k) {
                if is_clique(&adj_map, group.iter().copied()) {
                    largest_strongly_connected = Some(group);
                    break 'k_loop;
                }
            }
        }
        eprintln!();

        let Some(mut clique) = largest_strongly_connected else {
            panic!();
        };
        clique.sort();
        clique.join(",")
    };
    // still dumb and brute force; a little smarter
    #[cfg(any())]
    let p2 = {
        // in order to be a part of a clique of size `k` you need to have at
        // least `k` nodes in your adjacency list:
        let mut largest_strongly_connected = None;
        'k_loop: for k in (1..adj_map.len()).rev() {
            let candidates = adj_map
                .iter()
                .filter(|(_, v)| v.len() >= k)
                .map(|(m, _)| *m);

            eprintln!("k={k}; {} candidates", candidates.clone().count());
            for group in candidates.combinations(k) {
                if is_clique(&adj_map, group.iter().copied()) {
                    largest_strongly_connected = Some(group);
                    break 'k_loop;
                }
            }
        }
        // eprintln!();

        let Some(mut clique) = largest_strongly_connected else {
            panic!();
        };
        clique.sort();
        clique.join(",")
    };

    #[cfg(any())]
    let p2 = {
        // recursive; dfs
        //
        // not really good about being efficient though; lots of unnecessary
        // hashset construction/copying?
        fn biggest_clique<'m>(
            map: &'m AdjacencyMap<'m>,
            clique: HashSet<&'m str>,
        ) -> HashSet<&'m str> {
            eprintln!("k-{}+", clique.len());

            let initial = clique
                .iter()
                .next()
                .map(|c| map[c].clone())
                .unwrap_or_else(|| {
                    // base case; clique of size 0:
                    map.keys().copied().collect()
                });
            let mut common_connected = clique.iter().map(|&c| &map[c]).fold(initial, |a, b| &a & b);

            for c in &clique {
                let ret = common_connected.remove(c);
                debug_assert!(ret);
            }

            // try growing each next clique
            let mut largest = None;
            for next in common_connected {
                let mut clique = clique.clone();
                clique.insert(next);
                let ret = biggest_clique(map, clique);
                if largest
                    .as_ref()
                    .map(|c: &HashSet<_>| c.len())
                    .unwrap_or_default()
                    < ret.len()
                {
                    largest = Some(ret);
                }
            }

            match largest {
                Some(c) => c,
                None => clique,
            }
        }

        let clique = biggest_clique(&adj_map, HashSet::new());
        let mut clique = Vec::from_iter(clique);
        clique.sort();
        clique.join(",")
    };

    let p2 = {
        fn biggest_clique_inner(
            matrix: &Vec<Vec<bool>>,
            current_clique: &mut Vec<usize>,
            potential_next: usize,
            callback: &mut impl FnMut(&Vec<usize>),
        ) {
            // check if we can add `potential_next`
            if !current_clique.iter().all(|&n| matrix[n][potential_next]) {
                return;
            }

            // if so, great; try to expand the clique even further:
            current_clique.push(potential_next);
            callback(&current_clique);

            for next in (potential_next + 1)..matrix.len() {
                biggest_clique_inner(matrix, current_clique, next, callback);
            }

            current_clique.pop();
        }

        fn biggest_clique<'m>(map: &'m AdjacencyMap<'m>) -> Vec<&'m str> {
            let mut matrix = vec![vec![false; map.len()]; map.len()];
            let mut ids = map.keys().copied().collect_vec();
            ids.sort();
            let machine_to_id: HashMap<_, _> =
                ids.iter().enumerate().map(|(id, &m)| (m, id)).collect();
            let id_to_machine = ids;

            for (id, row) in matrix.iter_mut().enumerate() {
                for connected in &map[id_to_machine[id]] {
                    row[machine_to_id[connected]] = true;
                }
            }

            let mut stack = vec![];
            let mut curr_max = vec![];
            let mut callback = |stack: &Vec<usize>| {
                if stack.len() > curr_max.len() {
                    curr_max = stack.clone();
                    eprintln!("new biggest clique [{}]: {:?}", curr_max.len(), {
                        curr_max.iter().map(|&id| id_to_machine[id]).collect_vec()
                    });
                }
            };
            for i in 0..matrix.len() {
                biggest_clique_inner(&matrix, &mut stack, i, &mut callback);
            }

            curr_max.into_iter().map(|id| id_to_machine[id]).collect()
        }

        // should already be in sorted order..
        biggest_clique(&adj_map).join(",")
    };
    _ = aoc.submit_p2(p2);
}

/*

    // connected, not strongly connected

    let p2 = {
        let mut unvisited = HashSet::<&str>::from_iter(adj_map.keys().copied());
        let mut largest = HashSet::new();

        while let Some(&starting) = unvisited.iter().next() {
            unvisited.remove(starting);
            let mut queue = HashSet::<&str>::from_iter(adj_map[starting].iter().copied());
            let mut connected = HashSet::new();

            while let Some(&next) = queue.iter().next() {
                queue.remove(next);
                let new = connected.insert(next);
                debug_assert!(new);

                for nn in &adj_map[next] {
                    if !connected.contains(nn) {
                        queue.insert(nn);
                    }
                }
            }

            if connected.len() > largest.len() {
                largest = connected;
            }
        }

        let mut computers = largest.iter().copied().collect_vec();
        computers.sort();
        computers.join(",")
    };

*/
