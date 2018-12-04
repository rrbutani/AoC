#!/usr/bin/env rustr
#![feature(nll)] // Yes this is bad but you know what? Shut up.
extern crate aoc;
#[macro_use(scan_fmt)] extern crate scan_fmt;
extern crate bit_vec;

#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};
use std::collections::HashMap;

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new_with_year(2018, 04);
    let input: String = aoc.get_input();

    #[derive(Debug, Clone)]
    enum Event {
        Wakes,
        Sleeps,
        New(u16),
    }
    
    #[derive(Debug, Clone)]
    struct Item {
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        event: Event
    }

    #[derive(Debug, Clone)]
    struct GuardRecord {
        id: u16,
        /// true when asleep
        sleep_records: Vec<Vec<bool>>,
    };

    // [1518-04-05 00:03] falls asleep | wakes up | Guard #1301
    let input = input.lines().map(|l| {
        let mut v = l.split("]");
        let (time, event) = (v.next().unwrap(), v.next().unwrap());

        let time = scan_fmt!(time, "[{d}-{d}-{d} {d}:{d}", u16, u8, u8, u8, u8);
        let event = match event {
            e if e.contains("falls asleep") => Event::Sleeps,
            e if e.contains("wakes up") => Event::Wakes,
            e if e.contains("Guard") => {
                let g = scan_fmt!(e, "Guard #{d} begins shift", u16);
                Event::New(g.unwrap())
            },
            _ => panic!("Bad input!! {}", event)
        };

        Item {
            year: time.0.unwrap(),
            month: time.1.unwrap(),
            day: time.2.unwrap(),
            hour: time.3.unwrap(),
            minute: time.4.unwrap(),
            event
        }
    });

    let mut hm: HashMap<u16, GuardRecord> = HashMap::new();
    let mut unprocessed: Vec<Item> = input.clone().collect();

    unprocessed.sort_by_key(|i| (i.year, i.month, i.day, i.hour, i.minute));

    let guard_id: u16 = if let Event::New(g) = unprocessed[0].event { g } else {
        panic!("No guard switch as the first thing!")
    };

    let mut guard = GuardRecord { id: guard_id, sleep_records: Vec::<Vec<bool>>::new() };
    guard.sleep_records.push(Vec::<bool>::with_capacity(60));

    hm.insert(guard_id, guard);
    let mut guard: &mut GuardRecord = hm.get_mut(&guard_id).unwrap();
    let mut last_state: &Event = &unprocessed[0].event;

    let mut unprocessed_iter = unprocessed.iter(); unprocessed_iter.next();
    for i in unprocessed_iter {
        match i.event {
            Event::Wakes | Event::Sleeps => {
                // If the previous state was Sleeps (and it really should be), mark
                // until now as asleep:
                let asleep = if let Event::Sleeps = last_state { true } else { false };

                (guard.sleep_records[0].len()..(i.minute as usize)).for_each(|_| {guard.sleep_records[0].push(asleep);});
            },
            // Event::Sleeps => {
            //     // If the previous state was Wakes or New (as it should be), mark
            //     // until now as awake. Otherwise (if we went to sleep *after* going to
            //     // sleep) just mark until now as asleep, I guess..
            //     let asleep = if let Event::Sleeps = last_state { true } else { false };

            //     (guard.sleep_records[0].len()..i.minute).for_each(|_| {guard.sleep_records[0].push(asleep);});
            // },
            Event::New(g) => {
                // Top off current record to 60:
                let asleep = if let Event::Sleeps = last_state { true } else { false };
                (guard.sleep_records[0].len()..60).for_each(|_| {guard.sleep_records[0].push(asleep);});
                // No need to stick it in the HashMap; it should already live there.

                // Check if we've got a record for our current guard:
                guard = if hm.contains_key(&g) { hm.get_mut(&g).unwrap() } else {
                    // And if we don't make one and stick it in:
                    hm.insert(g, GuardRecord { id: g, sleep_records: Vec::with_capacity(60) });
                    hm.get_mut(&g).unwrap()
                };

                // Add a new sleep record to the guard:
                guard.sleep_records.insert(0, Vec::<bool>::new());

                // And mark the guard as being not asleep until they started their
                // shift:
                if i.hour == 0 {
                    (0..=i.minute).for_each(|_| {guard.sleep_records[0].push(false);});
                }
            },
        };

        last_state = &i.event;
    }

    let asleep = if let Event::Sleeps = last_state { true } else { false };
    (guard.sleep_records[0].len()..60).for_each(|_| {guard.sleep_records[0].push(asleep);});

    let p1: usize = hm.iter().max_by_key(|(_, g)| {
        g.sleep_records.iter().map(|v|{
            v.iter().filter(|b| **b).count()
        }).sum::<usize>()
    }).map(|(i, g)| {
        let minute = (0..60).max_by_key(|i| {
            g.sleep_records
                .iter()
                .map(|v| v[*i] as u16)
                .sum::<u16>()
        }).unwrap();
        // let minute = minute.max_by_key(|(_, t)| t).unwrap();
            // .map(|(m, _)| m)
            // .unwrap();

        // A nice minimal lifetimes puzzle:
        // let v = vec![(0, 1), (1, 2), (2, 3)];
        // v.iter().map(|(_, _)| (8, 9)).max_by_key(|(_, t)| t);

        minute * *i as usize
    }).unwrap();

    println!("{}", p1);

    // aoc.submit_p1(p1);

    // hm.iter().for_each(|(id, g)| {
    //     g.sleep_records.iter().for_each(|v| {
    //         if v.len() != 60 {
    //             println!("{} has a vec of len {}", id, v.len());
    //         } else { println!("yo"); }
    //     })
    // });

    let p2: usize = hm.iter().map(|(id, g)| {
        // Let's turn every guard into their sleepiest minute + how many times
        // they've slept at that minute:
        let (min, time) = (0..60).map(|i| {
            g.sleep_records
                .iter()
                .map(|v| v[i] as u16)
                .sum::<u16>()
        }).enumerate().max_by(|(_, t1), (_, t2)| t1.cmp(t2)).unwrap();

        (id, min, time)
    }).max_by(|(.., t1), (.., t2)| t1.cmp(t2))
        .map(|(id, m, _)| *id as usize * m).unwrap();

    // .max_by_key(|(_, g)| {
    //     g.sleep_records.iter().map(|v|{
    //         v.iter().filter(|b| **b).count()
    //     }).sum::<usize>()
    // }).map(|(i, g)| {
    //     let minute = (0..60).max_by_key(|i| {
    //         g.sleep_records
    //             .iter()
    //             .map(|v| v[*i] as u16)
    //             .sum::<u16>()
    //     }).unwrap();
    //     // let minute = minute.max_by_key(|(_, t)| t).unwrap();
    //         // .map(|(m, _)| m)
    //         // .unwrap();

    //     // A nice minimal lifetimes puzzle:
    //     let v = vec![(0, 1), (1, 2), (2, 3)];
    //     // v.iter().map(|(_, _)| (8, 9)).max_by_key(|(_, t)| t);

    //     minute * *i as usize
    // }).unwrap();
    aoc.submit_p2(p2);
}
