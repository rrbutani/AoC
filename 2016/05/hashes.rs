#!/usr/bin/env rustr

// 1:43PM
// _
// 3:02PM

use md5::Context;
use owo_colors::{
    colors::{xterm::CodGray as Grey, Green},
    OwoColorize,
};

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::{Duration, SystemTime};

fn hashes(base: &str) -> impl Iterator<Item = String> {
    let mut ctx = Context::new();
    ctx.consume(base);

    (0..)
        .map(move |d| {
            let mut ctx = ctx.clone();
            ctx.consume(format!("{}", d));

            format!("{:?}", ctx.compute())
        })
        .filter(|s| s.starts_with("00000"))
}

fn digits(base: &str) -> impl Iterator<Item = char> {
    hashes(base).map(|s| s.chars().skip(5).next().unwrap())
}

fn animate(state: &[Option<char>; 8], count: usize) {
    let s: String = state
        .iter()
        .enumerate()
        .map(|(idx, c)| {
            if let Some(c) = c {
                format!("{}", c.fg::<Green>())
            } else {
                format!(
                    "{}",
                    ((((idx * count) + idx) % 93 + 33) as u8 as char).fg::<Grey>()
                )
            }
        })
        .map(|s| format!("{}{}", s, ('\u{0332}').fg::<Grey>()))
        .collect();

    print!("  {}\r", s);
}

fn main() {
    let mut aoc = AdventOfCode::new(2016, 05);
    let input: String = aoc.get_input();

    let p1: String = digits(input.trim()).take(8).collect();
    let _ = aoc.submit_p1(p1);

    let mut p2 = [None; 8];
    let mut hashes = hashes(input.trim())
        .map(|s| {
            let mut ch = s.chars().skip(5);
            let pos = ch.next().unwrap();
            let c = ch.next().unwrap();

            (pos, c)
        })
        .filter(|(pos, _)| ('0'..'8').contains(pos))
        .map(|(pos, c)| (pos as u8 - b'0', c));

    let (tx, rx) = mpsc::channel::<Option<_>>();
    let animate_th = thread::spawn(move || {
        let mut state: [Option<char>; 8] = rx.recv().unwrap().unwrap();

        loop {
            match rx.recv_timeout(Duration::from_millis(10)) {
                Ok(Some(update)) => state = update,
                Ok(None) => {
                    println!("");
                    break;
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => panic!(),
            }

            animate(
                &state,
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_micros() as usize,
            );
        }
    });

    tx.send(Some(p2.clone())).unwrap();
    let mut remaining = 8;
    while remaining > 0 {
        let (pos, c) = hashes.next().unwrap();

        if let None = p2[pos as usize] {
            remaining -= 1;
            p2[pos as usize] = Some(c);

            tx.send(Some(p2.clone())).unwrap();
        }
    }

    tx.send(None).unwrap();
    animate_th.join().unwrap();

    let p2: String = p2.iter().filter_map(|c| *c).collect();
    let _ = aoc.submit_p2(p2);
}
