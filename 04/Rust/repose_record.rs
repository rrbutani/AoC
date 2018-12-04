#!/usr/bin/env rustr
#![feature(nll)]
extern crate aoc;
#[macro_use(scan_fmt)] extern crate scan_fmt;

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
    /// Number of times asleep at each minute
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
            _ => { },
        };
    }

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

    let p2: usize = guards.iter().map(|(id, g)| {
        // Let's turn every guard into their sleepiest minute + count for that minute:
        g.sleep_record.iter()
            .enumerate()
            .max_by(|(_, t1),(_, t2)| t1.cmp(t2))
            .map(|(m, t)| (t, m, id))
            .unwrap()
    }).max() // Get the guard who slept the most on their minute
        .map(|(_, m, id)| *id as usize * m).unwrap(); // their minute * their ID

    aoc.submit_p2(p2);
}
