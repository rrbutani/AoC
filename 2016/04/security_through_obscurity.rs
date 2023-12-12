#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::collections::HashMap;

// 6:47PM
// 7:14PM
// 7:25PM

fn main() {
    let mut aoc = AdventOfCode::new(2016, 04);
    let input: String = aoc.get_input();
    // let input: String = "aaaaa-bbb-z-y-x-123[abxyz]".to_string();

    println!("{}", input);

    let mut p1 = 0;
    for room in input.lines() {
        let mut letters = HashMap::<char, usize>::new();

        let idx = room.rfind("-").unwrap();
        let (sector_id, chksum) =
            sf::scan_fmt!(&room[(idx + 1)..], "{}[{}]", usize, String).unwrap();
        let encrypted_name = &room[..idx];

        for c in encrypted_name.chars().filter(|c| *c != '-') {
            *letters.entry(c).or_insert(0) += 1;
        }

        let mut letters = letters
            .drain()
            .map(|(c, f)| (f, c))
            .collect::<Vec<(usize, char)>>();
        letters
            .sort_by(|(a1, a2), (b1, b2)| (b1, b'z' - (*b2 as u8)).cmp(&(a1, b'z' - (*a2 as u8))));

        let correct_checksum: String = letters.iter().take(5).map(|(_, c)| c).collect();

        if chksum == correct_checksum {
            p1 += sector_id;
        }

        println!("{:60} | {} ⇒ {}", room, chksum, correct_checksum);
    }

    let _ = aoc.submit_p1(p1);

    let mut p2 = None;
    for room in input.lines() {
        let idx = room.rfind("-").unwrap();
        let enc = &room[0..idx];
        let idx2 = room.rfind("[").unwrap();
        let sector_id: usize = (&room[(idx + 1)..idx2]).parse().unwrap();

        let dec = enc
            .chars()
            .map(|c| {
                if c == '-' {
                    ' '
                } else {
                    let c = (c as u8) - b'a';
                    let c = (c as usize + sector_id) % 26;
                    (c as u8 + b'a') as char
                }
            })
            .collect::<String>();
        println!("{:10} ⇒ {}", sector_id, dec);

        if dec == "northpole object storage" {
            p2 = Some(sector_id);
            break;
        }
    }

    let _ = aoc.submit_p2(p2.unwrap());
}
