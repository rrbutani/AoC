#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};
use std::collections::{HashMap, HashSet};
use std::ops::RangeInclusive;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Field /* <'a> */ {
    name: String,
    ranges: Vec<RangeInclusive<u16>>,
}

impl Field /* <'_> */ {
    fn is_valid(&self, val: &u16) -> bool {
        self.ranges.iter().any(|r| r.contains(val))
    }
}

impl FromStr for Field /* <'_> */ {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        let mut iter = s.split(':');
        let name = iter.next().unwrap().to_string();

        let ranges = iter
            .next()
            .unwrap()
            .trim()
            .split(" or ")
            .map(|s| {
                let mut iter = s.split('-');
                let start = iter.next().unwrap().parse().unwrap();
                let end = iter.next().unwrap().parse().unwrap();
                start..=end
            })
            .collect();

        Ok(Field { name, ranges })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Ticket {
    field_vals: Vec<u16>,
}

impl Ticket {
    fn num_fields(&self) -> usize {
        self.field_vals.len()
    }
}

impl FromStr for Ticket {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        Ok(Ticket {
            field_vals: s.split(',').map(|f| f.parse().unwrap()).collect(),
        })
    }
}

impl Ticket {}

fn main() {
    let mut aoc = AdventOfCode::new(2020, 16);
    //     let input = "class: 1-3 or 5-7
    // row: 6-11 or 33-44
    // seat: 13-40 or 45-50

    // your ticket:
    // 7,1,14

    // nearby tickets:
    // 7,3,47
    // 40,4,50
    // 55,2,20
    // 38,6,12";
    let input: String = aoc.get_input();
    let mut iter = input.split("\n\n");

    let fields: Vec<Field> = iter
        .next()
        .unwrap()
        .trim()
        .lines()
        .map(|l| l.parse().unwrap())
        .collect();
    let my_ticket: Ticket = iter
        .next()
        .unwrap()
        .split("your ticket:\n")
        .nth(1)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .parse()
        .unwrap();
    let num_fields = my_ticket.num_fields();
    let tickets: Vec<Ticket> = iter
        .next()
        .unwrap()
        .split("nearby tickets:\n")
        .nth(1)
        .unwrap()
        .lines()
        .map(|l| l.parse().unwrap())
        .inspect(|t: &Ticket| assert!(t.num_fields() == num_fields))
        .collect();

    assert_eq!(fields.len(), num_fields);
    assert_eq!(iter.next(), None);

    // dbg!(&fields);
    // dbg!(&my_ticket);
    // dbg!(&tickets);

    let scanning_error_rate: usize = tickets
        .iter()
        .map::<usize, _>(|t: &Ticket| {
            t.field_vals
                .iter()
                .filter(|v| fields.iter().all(|f| !f.is_valid(v)))
                .map(|v| *v as usize)
                .sum()
        })
        .sum();
    println!("{}", scanning_error_rate);
    let _ = aoc.submit_p1(scanning_error_rate);

    let valid_tickets = tickets.iter().filter(|t| {
        !t.field_vals
            .iter()
            .any(|v| fields.iter().all(|f| !f.is_valid(v)))
    });
    let mut field_map: HashMap<&Field, HashSet<usize>> = fields
        .iter()
        .map(|f| (f, (0..num_fields).collect()))
        .collect();

    for t in valid_tickets.clone() {
        for (idx, v) in t.field_vals.iter().enumerate() {
            for f in &fields {
                if !f.is_valid(v) {
                    // println!("{} can't be {}", idx, f.name);
                    field_map.get_mut(f).unwrap().remove(&idx);
                }
            }
        }
    }

    let mut fields_finalized: HashMap<&Field, usize> = HashMap::new();
    while fields_finalized.len() != num_fields {
        for (f, idxs) in field_map.iter() {
            if idxs.len() == 1 {
                let idx = idxs.iter().next().unwrap();
                fields_finalized.insert(f, *idx);
            }
        }

        for (&f, idx) in fields_finalized.iter() {
            field_map.remove(&f);
            for (_, idxs) in field_map.iter_mut() {
                idxs.remove(&idx);
            }
        }
    }

    let depature_fields = fields.iter().filter(|f| f.name.starts_with("departure "));

    let p2: usize = depature_fields
        .map(|f| fields_finalized[f])
        .map(|idx| my_ticket.field_vals[idx] as usize)
        .product();

    println!("{}", p2);
    let _ = aoc.submit_p2(p2);
}
