#!/usr/bin/env rustr
extern crate aoc;

#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};
use std::u32::{self};
// use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone)]
struct Coord {
    x: u16,
    y: u16,
    count: u32,
    infinite: bool,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone)]
struct Cell {
    owner: Option<u16>,
    matched: Vec<u16>,
    dist: u32,
}

fn manhat(x: usize, y: usize, coord: &Coord) -> u32 {
    ((coord.x as i16 - x as i16).abs() + (coord.y as i16 - y as i16).abs()) as u32
}

fn flood(grid: &mut Vec<Vec<Cell>>, idx: u16, coord: &Coord) {

    for x in 0..grid.len() {
        for y in 0..grid[x].len() {
            let h = &mut grid[x][y];
            let d = manhat(x, y, coord);

            h.owner = if h.dist > d {
                h.dist = d;
                h.matched.drain(..);
                h.matched.push(idx);
                Some(idx)
            } else if h.dist == d {
                h.matched.push(idx);
                None
            } else {
                h.owner
            };
        }
    }
}

fn fill(grid: &mut Vec<Vec<Cell>>, coord: &Coord) {

    for x in 0..grid.len() {
        for y in 0..grid[x].len() {
            let h = &mut grid[x][y];
            let d = manhat(x, y, coord);

            h.dist += d;
        }
    }
}

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new_with_year(2018, 06);
    let input: String = aoc.get_input();

    let input = input.lines().map(|s| {
        let mut v = s.split(',');
        // println!("{}", v.next().unwrap());
        // println!("{}", v.next().unwrap());
        let t: (u16, u16) = (v.next().unwrap().parse().unwrap(),
            v.next().unwrap().trim().parse().unwrap());
        
        Coord { x: t.0, y: t.1, count: 0, infinite: false }
    });

    // Need to start by establishing bounds
    let coords = input.clone().collect::<Vec<Coord>>();

    // Alright, let's be dumb and naive:
    let (min_x, max_x) = coords.iter().minmax_by_key(|c| c.x).into_option().unwrap();
    let (min_y, max_y) = coords.iter().minmax_by_key(|c| c.y).into_option().unwrap();
    // let max_x = coords.iter().max_by_cmp(|(ax, ay), (bx, by)| ax.cmp(bx))
    // let br = coords.iter().max().unwrap();
    let (min_x, max_x, min_y, max_y) = (min_x.x, max_x.x, min_y.y, max_y.y);

    let mut coords = coords.iter().map(|c| Coord { x: c.x - min_x, y: c.y - min_y, count: 0, infinite: false } ).collect::<Vec<Coord>>();

    let mut grid: Vec<Vec<Cell>> = Vec::with_capacity((max_x - min_x) as usize + 1);

    for _x in 0..(max_x - min_x + 1) {
        let mut v = Vec::with_capacity((max_y - min_y) as usize + 1);
        for _y in 0..(max_y - min_y + 1) {
            v.push(Cell { owner: None, matched: Vec::new(), dist: u32::MAX });
        }
        grid.push(v);
    }

    coords.iter().enumerate().for_each(|(i, c)| {
        flood(&mut grid, i as u16, c)
    });


    for x in 0..grid.len() {
        for y in 0..grid[x].len() {
            let c = &grid[x][y];

            // println!("{}, {}", x, y);

            if x == 0 || y == 0 || x as u16 == max_x || y as u16 == max_y {
                if let Some(idx) = c.owner {
                    coords[idx as usize].infinite = true;
                } else {
                    for v in c.matched.iter() {
                        coords[*v as usize].infinite = true;
                    }
                } // I hope I don't need to handle this edge case!!
            }

            if let Some(idx) = c.owner {
                coords[idx as usize].count += 1;
                // println!("{} ({}, {}): {}", idx, coords[idx as usize].x, coords[idx as usize].y, coords[idx as usize].count);
            }

        }
    }

    let p1 = coords.iter().filter(|c| ! c.infinite).map(|c| { println!("{}", c.count); c.count }).max().unwrap();

    println!("{}", p1);

    // aoc.submit_p1(p1);

    let mut grid: Vec<Vec<Cell>> = Vec::with_capacity((max_x - min_x) as usize + 1);

    for _x in 0..(max_x - min_x + 1) {
        let mut v = Vec::with_capacity((max_y - min_y) as usize + 1);
        for _y in 0..(max_y - min_y + 1) {
            v.push(Cell { owner: None, matched: Vec::new(), dist: 0 });
        }
        grid.push(v);
    }

    coords.iter().enumerate().for_each(|(_i, c)| {
        fill(&mut grid, c)
    });

    let p2 = grid.iter().flat_map(|y| y.iter()).filter(|c| c.dist < 10_000).count();
    println!("{}", p2);
    aoc.submit_p2(p2);
}
