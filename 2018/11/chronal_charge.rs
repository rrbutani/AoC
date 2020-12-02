#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};

fn power_level(serial_number: i32, x: u16, y: u16) -> i32 {
    let rack_id = x as i32 + 10;
    let pl: i32 = y as i32 * rack_id;

    let pl = pl + serial_number;
    let pl = pl * rack_id;

    let pl = (pl / 100) % 10;
    pl as i32 - 5
}

fn max_in_grid_of_size(serial_number: i32, square_size: u16) -> ((u16, u16), i32) {
    (1..=(300 - square_size + 1)).map(|x| {
        (1..=(300 - square_size + 1)).map(move |y| {
            let s: i32 = (x..=(x + square_size - 1)).map(|x| {
                (y..=(y + square_size - 1)).map(|y| {
                    power_level(serial_number, x, y)
                }).sum::<i32>()
            }).sum::<i32>();

            ((x, y), s)
        }).max_by_key(|(_, s)| s.clone()).unwrap()
    }).max_by_key(|(_, s)| s.clone()).unwrap()
}

fn max_in_grid(serial_number: i32) -> ((u16, u16, u16), i32) {
    (1..=300).map(|s| {
        println!("{}", s);
        let ((x, y), pl) = max_in_grid_of_size(serial_number, s);
        ((x, y, s), pl)
    }).max_by_key(|(_, s)| s.clone()).unwrap()
}

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new(2018, 11);
    let input: String = aoc.get_input();

    let serial_number = input.lines().next().unwrap().parse::<i32>().unwrap();

    let coordinate = Some(max_in_grid_of_size(serial_number, 3))
        .map(|((x, y), _)| format!("{},{}", x, y))
        .unwrap();

    aoc.submit_p1(coordinate);

    let coordinate = Some(max_in_grid(serial_number))
        .map(|((x, y, s), _)| format!("{},{},{}", x, y, s))
        .unwrap();

    aoc.submit_p2(coordinate);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_level_tests() {
        assert_eq!(power_level(8, 3, 5), 4);
        assert_eq!(power_level(57, 122, 79), -5);
        assert_eq!(power_level(39, 217, 196), 0);
        assert_eq!(power_level(71, 101, 153), 4);
    }

    #[test]
    fn max_in_grid_three_tests() {
        assert_eq!(max_in_grid_of_size(18, 3), ((33, 45), 29));
        assert_eq!(max_in_grid_of_size(42, 3), ((21, 61), 30));
    }

    #[test]
    fn max_in_grid_tests() {
        assert_eq!(max_in_grid(18), ((90, 269, 16), 113));
        assert_eq!(max_in_grid(42), ((232, 251, 12), 119));
    }
}
