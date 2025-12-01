use std::collections::{HashMap, HashSet};

use aoc::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Cave<'s> {
    Big(&'s str),
    Small(&'s str),
    Start,
    End,
}

impl<'s> TryFrom<&'s str> for Cave<'s> {
    type Error = String;

    fn try_from(s: &'s str) -> Result<Self, Self::Error> {
        use Cave::*;
        Ok(match s.as_bytes() {
            b"start" => Start,
            b"end" => End,
            l if l.iter().all(|b| b.is_ascii_lowercase()) => Small(s),
            u if u.iter().all(|b| b.is_ascii_uppercase()) => Big(s),
            _ => return Err(s.to_string()),
        })
    }
}

fn paths<const ALLOW_REVISITING_SMALL_CAVE_ONCE: bool>(
    edges: &HashMap<Cave<'_>, HashSet<Cave>>,
) -> usize {
    fn visit<'c, const REVISIT_SMALL_ONCE: bool>(
        edges: &HashMap<Cave<'c>, HashSet<Cave<'c>>>,
        curr: Cave<'c>,
        visited: &mut HashMap<Cave<'c>, usize>,
        revisited_small: bool,
    ) -> usize {
        use Cave::*;
        if curr == End {
            return 1;
        }

        *visited.entry(curr).or_default() += 1;

        let mut count = 0;
        for &next in &edges[&curr] {
            let mut is_revisit = revisited_small;
            match next {
                Start => continue,
                Small(_) => {
                    if let Some(n) = visited.get(&next) {
                        match n {
                            0 => {}
                            1 => {
                                if REVISIT_SMALL_ONCE && !revisited_small {
                                    is_revisit = true;
                                } else {
                                    continue;
                                }
                            }
                            2 => {
                                debug_assert!(REVISIT_SMALL_ONCE);
                                debug_assert!(revisited_small);
                                continue;
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                _ => {}
            }

            count += visit::<REVISIT_SMALL_ONCE>(edges, next, visited, is_revisit);
        }

        *visited.get_mut(&curr).unwrap() -= 1;
        count
    }

    let mut v = HashMap::new();
    visit::<ALLOW_REVISITING_SMALL_CAVE_ONCE>(edges, Cave::Start, &mut v, false)
}

fn main() {
    let mut aoc = AdventOfCode::new(2021, 12);
    let inp = aoc.get_input();

    let mut edges: HashMap<_, HashSet<_>> = HashMap::new();
    inp.lines()
        .map(|l| l.split('-').map_to::<Cave>().collect_tuple().unwrap())
        .for_each(|(a, b)| {
            edges.entry(a).or_default().insert(b);
            edges.entry(b).or_default().insert(a);
        });

    let p1 = paths::<false>(&edges);
    _ = aoc.submit_p1(p1);

    let p2 = paths::<true>(&edges);
    _ = aoc.submit_p2(p2);
}
