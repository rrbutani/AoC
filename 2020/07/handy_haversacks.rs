#!/usr/bin/env rustr


#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};
use std::{collections::HashMap, str::FromStr};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct BagColor(String);

impl<T: ToString> From<T> for BagColor {
    fn from(t: T) -> Self {
        BagColor(t.to_string())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Rule {
    allowed: HashMap<BagColor, u32>,
}

impl FromStr for Rule {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        let rule = s.strip_prefix("contain ").ok_or(())?;

        if rule == "no other bags." {
            Ok(Rule::default())
        } else {
            let allowed = rule
                .split(", ")
                .map(|s| {
                    let (q, c1, c2) = scan_fmt!(s, "{} {} {} bag", u32, String, String);
                    (
                        BagColor(format!("{} {}", c1.unwrap(), c2.unwrap())),
                        q.unwrap(),
                    )
                })
                .collect();

            Ok(Rule { allowed })
        }
    }
}

impl Rule {
    #[allow(unused)]
    fn matches(&self, (color, quantity): (&BagColor, u32)) -> bool {
        if let Some(q) = self.allowed.get(color) {
            *q >= quantity
        } else {
            false
        }
    }

    // We _should_ memoize this but we do not _need_ to.
    fn recursive_contents_inner<'a>(
        &'a self,
        rule_set: &'a RuleSet,
        contents: &mut HashMap<&'a BagColor, u32>,
        multiplier: u32,
    ) {
        for (c, q) in self.allowed.iter() {
            *contents.entry(&c).or_insert(0) += q * multiplier;

            rule_set.rules.get(&c).unwrap().recursive_contents_inner(
                rule_set,
                contents,
                q * multiplier,
            );
        }
    }

    fn recursive_contents<'a>(&'a self, rule_set: &'a RuleSet) -> HashMap<&'a BagColor, u32> {
        let mut contents = HashMap::new();
        self.recursive_contents_inner(rule_set, &mut contents, 1);

        contents
    }
}

#[derive(Debug, Default)]
struct RuleSet {
    rules: HashMap<BagColor, Rule>,
}

impl RuleSet {
    fn insert(&mut self, bag: BagColor, rule: Rule) -> Result<(), ()> {
        if let Some(_) = self.rules.insert(bag, rule) {
            Err(())
        } else {
            Ok(())
        }
    }

    fn can_hold<'a>(
        &'a self,
        (color, quantity): (&'a BagColor, u32),
    ) -> impl Iterator<Item = BagColor> + 'a {
        self.rules
            .iter()
            .filter(move |(c, _r)| *c != color)
            .filter(move |(_c, rule)| {
                rule.recursive_contents(self)
                    .get(color)
                    .filter(|q| **q >= quantity)
                    .map(|_| true)
                    .unwrap_or(false)
            })
            .map(|(c, _)| c.clone())
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2020, 07);
    let input: String = aoc.get_input();

    let rule_set = input
        .lines()
        .map(|s| {
            // s.split_once("bags ")
            let idx = s.find("contain").unwrap();
            let color = s[..idx].split(" bags").next().unwrap().into();
            let rule = s[idx..].parse().unwrap();

            (color, rule)
        })
        .fold(RuleSet::default(), |mut rs, (c, r)| {
            rs.insert(c, r).unwrap();
            rs
        });

    let sg = "shiny_gold".into();

    let p1 = rule_set.can_hold((&sg, 1)).count();
    let _ = aoc.submit_p1(p1);

    let p2: u32 = rule_set
        .rules
        .get(&sg)
        .unwrap()
        .recursive_contents(&rule_set)
        .iter()
        .map(|(_, q)| *q)
        .sum();
    let _ = aoc.submit_p2(p2);
}
