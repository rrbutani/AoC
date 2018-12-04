#!/usr/bin/env rustr
#![feature(nll)] // Yes this is bad but you know what? Shut up.
extern crate aoc;
#[macro_use(scan_fmt)] extern crate scan_fmt;
extern crate bit_vec;

#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};
use std::str::FromStr;
use std::collections::HashMap;
use std::u16;

#[derive(Debug, Clone, Ord, Eq, PartialEq, PartialOrd)]
enum Event {
    Wakes,
    Sleeps,
    New(u16),
    Finish,
}

#[derive(Debug, Clone, Ord, Eq, PartialEq, PartialOrd)]
struct Item {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    event: Event
}

#[derive(Clone)]
struct GuardRecord {
    id: u16,
    /// true when asleep
    sleep_record: [u8; 60],
}

impl Default for GuardRecord {
    fn default() -> Self {
        Self { id: 0, sleep_record: [0u8; 60] }
    }
}

impl FromStr for Item {
    type Err = &'static str;

    fn from_str(inp: &str) -> Result<Self, Self::Err> {
        let mut v = inp.split("]");
        let (time, event) = (v.next().unwrap(), v.next().unwrap());

        // Ex: [1518-04-05 00:03] falls asleep | wakes up | Guard #1301
        let time = scan_fmt!(time, "[{d}-{d}-{d} {d}:{d}", u16, u8, u8, u8, u8);
        let event = match event {
            e if e.contains("falls asleep") => Event::Sleeps,
            e if e.contains("wakes up") => Event::Wakes,
            e if e.contains("Guard") => {
                let g = scan_fmt!(e, "Guard #{d} begins shift", u16);
                Event::New(g.unwrap())
            },
            _ => return Err("Bad input!!")
        };

        Ok(Item {
            year: time.0.ok_or("Bad year")?,
            month: time.1.ok_or("Missing month")?,
            day: time.2.ok_or("Where'd the day go?")?,
            hour: time.3.ok_or("Need an hour")?,
            minute: time.4.ok_or("No minute!")?,
            event
        })
    }
}

fn print_records(guards: &HashMap<u16, GuardRecord>) {
    println!("       000000000011111111112222222222333333333344444444445555555555");
    println!("       012345678901234567890123456789012345678901234567890123456789");
    guards.iter().for_each(|(id, g)|{
        print!("#{:04}: ", id);
        g.sleep_record.iter().for_each(|i| match i {
            0 => print!("."),
            1 => print!("-"),
            2 | 3 | 4 | 5 => print!("+"),
            6 => print!("6"),
            7 => print!("7"),
            8 => print!("8"),
            9 => print!("9"),
            _ => print!("&"),
        });
        println!("");
    });
}

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new_with_year(2018, 04);
    let input = aoc.get_input();
    let input = input.lines().map(|l| l.parse().unwrap());

    let mut guards: HashMap<u16, GuardRecord> = HashMap::new();
    let mut event_stream: Vec<Item> = input.clone().collect();

    // So that we properly finish off the last real guard:
    event_stream.push(Item {year: u16::MAX, month: 0, day: 0, hour: 0, minute: 60, event: Event::Finish});
    event_stream.sort();

    let mut guard = &mut GuardRecord::default();

    for v in event_stream.windows(2) {
        let (c, n) = (v[0].clone(), v[1].clone());

        match (c.event, n.event) {
            (Event::New(id), _) => {
                // We have a new guard! Let's set them up:
                guard = guards.entry(id).or_insert(GuardRecord { id, sleep_record: [0u8; 60] });
            },
            (Event::Sleeps, Event::Wakes) => {
                // Record the guard's nap!
                (c.minute..n.minute).for_each(|i| guard.sleep_record[i as usize] += 1)
            },
            (Event::Sleeps, Event::New(_)) | (Event::Sleeps, Event::Finish) => {
                (c.minute..60).for_each(|i| guard.sleep_record[i as usize] += 1)  
            },
            (Event::Sleeps, Event::Sleeps) => { /* bad input */ },
            (Event::Wakes, Event::Sleeps) | (Event::Wakes, Event::New(_)) | (Event::Wakes, Event::Finish)=> { /* no need */ },
            (Event::Wakes, Event::Wakes) => { /* bad input */ },
            (Event::Finish, _) => { unreachable!() },
        };
    }

    // print_records(&guards);

    let p1: usize = guards.iter()
        .max_by_key(|(_, g)| g.sleep_record.iter().fold(0u16, |acc, i| acc + *i as u16))
        .map(|(i, g)|
            g.sleep_record.iter()
                .enumerate()
                .max_by(|(_, t1),(_, t2)| t1.cmp(t2))
                .map(|(i, _)| i)
                .unwrap() * *i as usize
    ).unwrap();

    aoc.submit_p1(p1);

    // print_records(&guards);

    let p2: usize = guards.iter().map(|(id, g)| {
        // Let's turn every guard into their sleepiest minute + count for that minute:
        g.sleep_record.iter()
            .enumerate()
            .max_by(|(_, t1),(_, t2)| t1.cmp(t2))
            .map(|(m, t)| (t, m, id))
            .unwrap()
    })
    // Get the guard who slept the most on their minute
    .max()
        // And now multiply their minute by their ID.
        .map(|(t, m, id)| { println!("Guard {} slept {} times at minute {}", id, t, m); *id as usize * m }).unwrap();

    aoc.submit_p2(p2);
}

// A nice minimal lifetimes puzzle:
// let v = vec![(0, 1), (1, 2), (2, 3)];
// v.iter().map(|(_, _)| (8, 9)).max_by_key(|(_, t)| t);