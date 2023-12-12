#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::iter;
use std::ops::Index;
use std::pin::Pin;

use dyn_clone::DynClone;
use genawaiter::{rc::Gen, GeneratorState};

trait SeqIterator: Iterator<Item = char> + DynClone {}
dyn_clone::clone_trait_object!(SeqIterator);
impl<T: Clone + Iterator<Item = char>> SeqIterator for T {}

trait SeqOfSeqIterator: Iterator<Item = Box<dyn SeqIterator>> + DynClone {}
dyn_clone::clone_trait_object!(SeqOfSeqIterator);
impl<T: Clone + Iterator<Item = Box<dyn SeqIterator>>> SeqOfSeqIterator for T {}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Rule {
    Literal(char),
    PossibleSequences { options: Vec<Vec<usize>> },
}

impl Rule {
    fn matches_inner<'s, 'a>(&'s self, s: &'a str, rs: &'s RuleSet) -> Option<&'a str> {
        let res = match self {
            Rule::Literal(c) => {
                if s.chars().next()? == *c {
                    Some(s.split_at(1).1)
                } else {
                    None
                }
            }
            // We're making an important assumption here: that all variants in
            // rule consume the same nunmber of characters.
            //
            // That way if something matches, it matches; there's no "better"
            // match and no possibility that we need to backtrack because
            // something matched in a way that makes matching the rest of the
            // string impossible.
            Rule::PossibleSequences { options } => options
                .iter()
                .filter_map(|seq| {
                    seq.iter()
                        .try_fold(s, |s, r| rs.rules.get(r).unwrap().matches_inner(s, rs))
                })
                .next(),
        };

        // println!("M `{:30}` against {:?}:\n\t  - {:?}", s, self, res);
        res
    }

    fn matches(&self, s: &str, rs: &RuleSet) -> bool {
        // println!("Matching {}:\n", s);
        matches!(self.matches_inner(s, rs), Some(s) if s.is_empty())
    }

    fn print_dfs_inner<'s>(
        &'s self,
        depth: usize,
        rs: &'s RuleSet,
    ) -> Box<dyn SeqOfSeqIterator + 's> {
        if depth == 0 {
            return Box::new(iter::empty());
        }
        match self {
            Rule::Literal(c) => Box::new(iter::once(Box::new(iter::once(*c)) as _)),
            Rule::PossibleSequences { options } => {
                // 92 5 | 5 10 => flat_map
                // 8 11 => cartesian_product
                // normally we'd filter here instead of doing the full cartesian
                // product..
                Box::new(options.iter().flat_map(move |seq| {
                    seq.iter()
                        .map(|r| rs[*r].print_dfs_inner(depth - 1, rs))
                        .multi_cartesian_product()
                        // .map(|v| v.fold(Box::new(iter::empty()), |acc, s| Box::new(acc.chain(s)))
                        .map(|v| {
                            // This is bad!
                            let v = Box::leak(Box::new(v));

                            Box::new(v.iter().flat_map(move |v| Box::<dyn SeqIterator>::clone(v)))
                                as Box<dyn SeqIterator>
                        })
                })) as _
            }
        }
    }

    fn print_dfs<'s>(&'s self, depth: usize, rs: &'s RuleSet) -> impl Iterator<Item = String> + 's {
        self.print_dfs_inner(depth, rs).map(|i| i.collect())
    }

    // Difference between this and matches is that when we have a sequence
    // (a | b) we'll backtrack and try b if continuing down the a path fails
    // but not immediately.
    fn matches_dfs_(&self, s: &str, rs: &RuleSet) -> Option<&str> {
        // i.e.
        // 0: 8 11
        // 8: 92 5 | 5 10
        //
        // 92 5 11
        // 5 10 11
        //
        // it's possible that we match 8 92 5
        // but ultimately need to match against 8 5 10 11
        // we'd want to explore the 92 5 route and then, when it ultimately
        // doesn't pan out, we'd want to bail and then try the 5 10 route
        //
        // tricky bit is that we want to do the exploration for something like
        // 8: (92 | 5) | (2 | 3)
        // i.e. we'd want to try 92, then 2, then 3, then pop up and try 5 with
        // 2 and 3
        // match self {
        //     Rule::Literal()
        // }

        todo!()
    }

    fn matches_dfs_sugar(&self, s: &str, rs: &RuleSet, limit: usize) -> bool {
        self.matches_dfs(s, rs, limit)
            .into_iter()
            // .inspect(|m| println!("{:?}", m))
            .any(|remaining| remaining.is_empty())
    }

    // ) -> Gen<&'s str, (), impl Future<Output = ()> + 's> {
    fn matches_dfs<'s>(
        &'s self,
        s: &'s str,
        rs: &'s RuleSet,
        limit: usize,
    ) -> Gen<&'s str, (), Pin<Box<dyn Future<Output = ()> + 's>>> {
        // println!("trying to match {}", s);

        Gen::new(|co| {
            let fut = Box::new(async move {
                if limit == 0 {
                    return;
                }

                match self {
                    Rule::Literal(c) => {
                        if let Some(s) = s.strip_prefix(*c) {
                            co.yield_(s).await
                        }
                    }
                    Rule::PossibleSequences { options } => {
                        // Variants (i.e. the outer Vec) contain multiple
                        // sequences; these sequences are the variants and get
                        // tried one after another.
                        for sequence in options.iter() {
                            // Sequences (i.e. the inner Vec) get chained together
                            // in a stack like way, yielding when we satisfy all
                            // the rules in the sequence.
                            if sequence.is_empty() {
                                continue;
                            }

                            // println!("{:?}", sequence);

                            let mut stack = Vec::with_capacity(sequence.len());
                            stack.push(rs[sequence[0]].matches_dfs(s, rs, limit - 1));

                            while !stack.is_empty() {
                                // Clear finished generators off of the stack:
                                let next = loop {
                                    let res = stack.last_mut().unwrap().resume();

                                    match res {
                                        GeneratorState::Complete(()) => {
                                            stack.pop();
                                        }
                                        GeneratorState::Yielded(next) => break Some(next),
                                    }

                                    if stack.is_empty() {
                                        break None;
                                    }
                                };

                                // If we've run out of generators, we're done:
                                let mut next = if let Some(next) = next {
                                    next
                                } else {
                                    break;
                                };

                                // Next, thread the output of the last generator
                                // through the remaining rules we've got until we
                                // hit an error or get through all the rules:
                                while stack.len() != sequence.len() {
                                    stack.push(rs[sequence[stack.len()]].matches_dfs(
                                        next,
                                        rs,
                                        limit - 1,
                                    ));
                                    next = match stack.last_mut().unwrap().resume() {
                                        GeneratorState::Complete(()) => {
                                            // If we hit an error, bail:
                                            stack.pop();
                                            break;
                                        }
                                        GeneratorState::Yielded(next) => next,
                                    };
                                }

                                // If we do manage to fill up the stack, yield what
                                // we got:
                                if stack.len() == sequence.len() {
                                    co.yield_(next).await;
                                }
                            }
                        }
                    }
                }
            }) as Box<_>;

            Into::<Pin<Box<_>>>::into(fut)
        })
    }
}

/*
    // ) -> Gen<&'s str, (), impl Future<Output = ()> + 's> {
    fn matches_dfs<'s>(
        &'s self,
        s: &'s str,
        rs: &'s RuleSet,
        limit: usize,
    ) -> Gen<&'s str, (), Pin<Box<dyn Future<Output = ()> + 's>>> {
        // println!("trying to match {}", s);

        Gen::new(|co| {
            let fut = Box::new(async move {
                if limit == 0 {
                    return;
                }

                match self {
                    Rule::Literal(c) => {
                        if let Some(s) = s.strip_prefix(*c) {
                            co.yield_(s).await
                        }
                    }
                    Rule::PossibleSequences { options } => {
                        // Variants (i.e. the outer Vec) contain multiple
                        // sequences; these sequences are the variants and get
                        // tried one after another.
                        for sequence in options.iter() {
                            // Sequences (i.e. the inner Vec) get chained
                            // together in a stack like way, yielding when we
                            // satisfy all the rules in the sequence.
                            if sequence.is_empty() {
                                continue;
                            }

                            let mut stack = Vec::with_capacity(sequence.len());

                            // We push the first generator to begin and then
                            // continue until the stack is empty (i.e. until it
                            // and all the generators after it are exhausted).
                            stack.push(rs[sequence[0]].matches_dfs(s, rs, limit - 1));

                            'stack: while !stack.is_empty() {
                                // Clear finished generators off of the stack
                                // and get what's left of the string after the
                                // rules are applied:
                                let mut next = loop {
                                    match stack.last_mut().map(|g| g.resume()) {
                                        Some(GeneratorState::Complete(())) => {
                                            stack.pop();
                                        }
                                        Some(GeneratorState::Yielded(next)) => break next,
                                        // If we've run out of generators, we're
                                        // done:
                                        None => break 'stack,
                                    }
                                };

                                // Next, thread the output of the last generator
                                // through the remaining rules we've got until we
                                // hit an error or get through all the rules:
                                for r in &sequence[stack.len()..] {
                                    next = match stack
                                        .put(rs[*r].matches_dfs(next, rs, limit - 1))
                                        .resume()
                                    {
                                        // If the generator doesn't produce any
                                        // matches, we can't use it; bail:
                                        GeneratorState::Complete(()) => {
                                            stack.pop();
                                            break;
                                        }
                                        GeneratorState::Yielded(next) => next,
                                    }
                                }

                                // If we do manage to fill up the stack, yield
                                // what we got:
                                if stack.len() == sequence.len() {
                                    co.yield_(next).await;
                                }
                            }
                        }
                    }
                }
            }) as Box<_>;

            Into::<Pin<Box<_>>>::into(fut)
        })
    }
*/

// 0: (9 b | 10 a) (9 "b" | 10 "a") ("b" 17 | "a" ("b" (5 "b" | 16 1) | "a" (24 "b" | 19 1)))
// 42: 9 14 | 10 1
// 9: (b (a (bb | ab)) | (b ((a | b) (a | b)))) | (a 26)
// 10: 23 14 | 28 1
// 1: "a"
// 11: 42 31
// 5: 1 14 | 15 1
// 19: 14 1 | 14 14
// 12: 24 14 | 19 1
// 16: 15 1 | 14 14
// 31: 14 17 | 1 13
// 6: "b" "b" | "a" "b"
// 2: 1 24 | 14 4
// 13: 14 3 | 1 12
// 15: "a" | "b"
// 17: 14 2 | 1 7
// 23: 25 1 | 22 14
// 28: 16 1
// 4: 1 1
// 20: 14 14 | 1 15
// 3: 5 14 | 16 1
// 27: ("a" ("b" "b" | "a" "b")) | ("b" (("a" | "b") ("a" | "b")))
// 14: "b"
// 21: 14 1 | 1 14
// 25: 1 1 | 1 14
// 22: 14 14
// 8: 42
// 26: 14 22 | 1 20
// 18: ("a" | "b") ("a" | "b")
// 7: 14 5 | 1 21
// 24: 14 1

#[derive(Clone, Debug, PartialEq, Eq)]
struct RuleSet {
    rules: HashMap<usize, Rule>,
}

impl Index<usize> for RuleSet {
    type Output = Rule;

    fn index(&self, idx: usize) -> &Rule {
        self.rules.get(&idx).unwrap()
    }
}

impl FromStr for RuleSet {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        let rules = s
            .lines()
            .map(|l| {
                let mut iter = l.split(":");
                let idx = iter.next().unwrap().parse().unwrap();
                let rest = iter.next().unwrap().trim();

                let rule = if rest.contains('"') {
                    assert_eq!(rest.len(), 3);
                    Rule::Literal(rest.chars().nth(1).unwrap())
                } else {
                    Rule::PossibleSequences {
                        options: rest
                            .split("|")
                            .map(|seq| {
                                seq.trim()
                                    .split(" ")
                                    .map(|idx| idx.parse().unwrap())
                                    .collect()
                            })
                            .collect(),
                    }
                };

                (idx, rule)
            })
            .collect();

        Ok(RuleSet { rules })
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2020, 19);
    let input: String = aoc.get_input();

    let input2 = r#"42: 9 14 | 10 1
9: 14 27 | 1 26
10: 23 14 | 28 1
1: "a"
11: 42 31
5: 1 14 | 15 1
19: 14 1 | 14 14
12: 24 14 | 19 1
16: 15 1 | 14 14
31: 14 17 | 1 13
6: 14 14 | 1 14
2: 1 24 | 14 4
0: 8 11
13: 14 3 | 1 12
15: 1 | 14
17: 14 2 | 1 7
23: 25 1 | 22 14
28: 16 1
4: 1 1
20: 14 14 | 1 15
3: 5 14 | 16 1
27: 1 6 | 14 18
14: "b"
21: 14 1 | 1 14
25: 1 1 | 1 14
22: 14 14
8: 42
26: 14 22 | 1 20
18: 15 15
7: 14 5 | 1 21
24: 14 1

abbbbbabbbaaaababbaabbbbabababbbabbbbbbabaaaa
bbabbbbaabaabba
babbbbaabbbbbabbbbbbaabaaabaaa
aaabbbbbbaaaabaababaabababbabaaabbababababaaa
bbbbbbbaaaabbbbaaabbabaaa
bbbababbbbaaaaaaaabbababaaababaabab
ababaaaaaabaaab
ababaaaaabbbaba
baabbaaaabbaaaababbaababb
abbbbabbbbaaaababbbbbbaaaababb
aaaaabbaabaaaaababaa
aaaabbaaaabbaaa
aaaabbaabbaaaaaaabbbabbbaaabbaabaaa
babaaabbbaaabaababbaabababaaab
aabbbbbaabbbaaaaaabbbbbababaaaaabbaaabba"#;

    let input3 = r#"0: 1 2 3
1: "a"
3: "b"
2: 1 | 3 2

asf
"#;

    let mut input = input.split("\n\n");

    let mut rules: RuleSet = input.next().unwrap().parse().unwrap();
    let messages = input.next().unwrap().lines();

    let r0 = &rules[0];
    let p1 = messages.clone().filter(|m| r0.matches(m, &rules)).count();
    // let _ = aoc.submit_p1(p1);

    let r8 = Rule::PossibleSequences {
        options: vec![vec![42], vec![42, 8]],
    };
    let r11 = Rule::PossibleSequences {
        options: vec![vec![42, 31], vec![42, 11, 31]],
    };
    rules.rules.insert(8, r8);
    rules.rules.insert(11, r11);

    let r0 = &rules[0];

    // let ss: HashSet<_> = r0.print_dfs(20, &rules).collect();
    // for s in ss {
    //     println!("{}", s);
    // }

    // yuck, we have a cycle:
    // [14, 14]
    // [28, 1]
    // [16, 1]
    // [15, 1]
    // [1]
    // [42, 31]
    // [9, 14]
    // [14, 27]
    // [1, 26]
    // [14, 22]
    // [1, 20]
    // [14, 14]
    // [1, 15]
    // [1]
    // [14]
    // [10, 1]
    // [23, 14]
    // [25, 1]
    // [1, 1]
    // [1, 14]
    // [22, 14]
    // [14, 14]

    // Limit values of 14 and up get us the right answer but we'll just use 500
    // to be safe (no appreciable difference in runtime).
    let p2 = messages
        .clone()
        .filter(|m| r0.matches_dfs_sugar(m, &rules, 500))
        .count();
    println!("{}", p2);
    let _ = aoc.submit_p2(p2);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p1_ex1() {
        let rules = r#"0: 4 1 5
1: 2 3 | 3 2
2: 4 4 | 5 5
3: 4 5 | 5 4
4: "a"
5: "b""#;

        let rs: RuleSet = rules.parse().unwrap();
        let r = &rs[0];

        assert!(r.matches("aaaabb", &rs));
        assert!(r.matches("aaabab", &rs));
        assert!(r.matches("abbabb", &rs));
        assert!(r.matches("abbbab", &rs));
        assert!(r.matches("aabaab", &rs));
        assert!(r.matches("aabbbb", &rs));
        assert!(r.matches("abaaab", &rs));
        assert!(r.matches("ababbb", &rs));
    }
}
