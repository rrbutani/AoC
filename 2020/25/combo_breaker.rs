#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

fn transformN(mut value: usize, subject: usize, loop_count: usize) -> usize {
    for _ in 0..loop_count {
        value = (value * subject) % 2020_1227;
    }

    value
}

fn transform(value: usize, subject: usize) -> usize {
    (value * subject) % 2020_1227
}

// Essentially solving `7 ^ loop_size (mod 20201227) == pub_key` for
// `loop_size`.
fn find_loop_size(pub_key: usize, initial_subject: usize) -> usize {
    let mut it = 0..;
    // .inspect(|l| println!("Trying {}", l))
    it.try_fold(1, |val, _| {
        Some(transform(val, initial_subject)).filter(|v| *v != pub_key)
    });

    it.next().unwrap()
}

// (((7 ^ C) % N) ^ D) % N
// (((7 ^ D) % N) ^ C) % N
//
// Mod is distributive over exponentiation; ^ eq:
// (7 ^ (C * D)) % N
//
// Assuming C and D are close in value, we can find C - D (or D - C) cheaply.
//
// Given that (and 7 ^ C and 7 ^ D) can we find 7 ^ (C * D)?
//
// 7 ^ C * 7 ^ D = 7 ^ (C + D)
//
// (7 ^ C) ^ (7 ^ D) = 7 ^ (C * (7 ^ D))
//
// I don't think so :-(

fn main() {
    let mut aoc = AdventOfCode::new(2020, 25);
    let input: String = aoc.get_input();
    let mut input = input.lines().map(|l| l.parse().unwrap());

    let card_pub_key: usize = input.next().unwrap();
    let door_pub_key = input.next().unwrap();

    // let card_pub_key = 5764801;
    // let door_pub_key = 17807724;

    let card_priv = find_loop_size(card_pub_key, 7);
    let door_priv = find_loop_size(door_pub_key, 7);

    let enc_key = transformN(1, card_pub_key, door_priv);
    assert_eq!(transformN(1, door_pub_key, card_priv), enc_key);
    // the encryption key should be equivalent to 7 run through `transform`
    // `card_loop_count` * `door_loop_count` times:
    debug_assert_eq!(transformN(1, 7, card_priv * door_priv), enc_key);

    // // which is equivalent to running the card's public key through `transform`
    // // `door_loop_count` more times:
    // assert_eq!(transformN(1, card_pub_key, door_priv), enc_key);
    // // and also to running the door's public key through `transform`
    // // `card_loop_count` more times:
    // assert_eq!(transformN(1, door_pub_key, card_priv), enc_key);

    // // We'll instead try to find the number of iterations it takes to get from 1
    // // public key to the other to get the delta between the keys:
    // let mut

    // let enc_key = transform(1, card_pub_key, door_priv);
    // assert_eq!(transform(1, door_pub_key, card_priv), enc_key);

    let _ = aoc.submit_p1(enc_key);

    let p2 = 0;
    println!("{}", p2);
    let _ = aoc.submit_p2(p2);
}
