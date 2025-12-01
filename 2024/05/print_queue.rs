use std::collections::{BTreeSet, HashMap, HashSet};

use aoc::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct LaunchManual {
    // [key] after [val(s)]
    safety_protocols: HashMap<usize, BTreeSet<usize>>,
    page_lists: Vec<Vec<usize>>,
}

impl FromStr for LaunchManual {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (p, page_lists) = s.split_once("\n\n").ok_or(())?;

        let mut safety_protocols = HashMap::<_, BTreeSet<_>>::new();
        for a in p.lines().map(|l| l.split('|').map_parse().collect_tuple()) {
            let (before, after) = a.ok_or(())?;
            safety_protocols.entry(after).or_default().insert(before);
        }

        let page_lists = page_lists
            .lines()
            .map(|l| l.split(',').map_parse().collect_vec())
            .collect();

        Ok(LaunchManual { safety_protocols, page_lists })
    }
}

impl LaunchManual {
    fn check_list(&self, list: &[usize]) -> Result<(), BTreeSet<usize>> {
        let all = BTreeSet::from_iter(list.iter().copied());
        let mut seen = BTreeSet::new();
        for &n in list {
            if let Some(required_before) = self.safety_protocols.get(&n) {
                if !(required_before & &all).is_subset(&seen) {
                    return Err(all);
                }
            }

            seen.insert(n);
        }

        Ok(())
    }

    pub fn middle_pages_of_matching_lists(&self) -> usize {
        let mut sum = 0;
        for list in &self.page_lists {
            if self.check_list(list).is_ok() {
                assert!(list.len() % 2 == 1); // odd number of pages
                sum += list[list.len() / 2];
            }
        }

        sum
    }

    // note: a cute optimization is that since we only care about the midpoint
    // we can technically stop early... not going to bother though
    fn topo_sort(&self, all: &BTreeSet<usize>) -> Vec<usize> {
        let mut processed: HashSet<usize> = HashSet::new();
        let mut retired = Vec::new();

        // assumes no cycles...
        fn dfs(
            curr: usize,
            all: &BTreeSet<usize>,
            predecessors: &HashMap<usize, BTreeSet<usize>>,
            processed: &mut HashSet<usize>,
            retired: &mut Vec<usize>,
        ) {
            // was already inserted, stop traversal
            if !processed.insert(curr) {
                return;
            }

            // visit all predecessors so they're guaranteed to retire first
            if let Some(preds) = predecessors.get(&curr) {
                for &p in preds.intersection(all) {
                    dfs(p, all, predecessors, processed, retired);
                }
            }

            // retire
            retired.push(curr);
        }

        for &n in all.iter() {
            dfs(n, all, &self.safety_protocols, &mut processed, &mut retired);
        }

        retired
    }

    // note: this is equivalent to asking for a topological sort of a DAG
    pub fn middle_page_of_corrected_lists(&self) -> usize {
        let mut sum = 0;
        for list in self.page_lists.iter() {
            // consider only incorrectly-ordered lists, fix them up:
            if let Err(all) = self.check_list(list) {
                let list = self.topo_sort(&all);
                assert!(list.len() % 2 == 1); // odd number of pages
                sum += list[list.len() / 2];
            }
        }

        sum
    }
}

const EX: &str = "47|53
97|13
97|61
97|47
75|29
61|13
75|53
29|13
97|29
53|29
61|53
97|53
61|29
47|13
75|47
97|75
47|61
75|61
47|29
75|13
53|13

75,47,61,53,29
97,61,53,29,13
75,29,13
75,97,47,61,53
61,13,29
97,13,75,29,47";

fn main() {
    let mut aoc = AdventOfCode::new(2024, 5);
    let inp = aoc.get_input();
    let man: LaunchManual = inp.parse().unwrap();

    let p1 = man.middle_pages_of_matching_lists();
    _ = aoc.submit_p1(p1);

    let p2 = man.middle_page_of_corrected_lists();
    _ = aoc.submit_p2(p2);
}
