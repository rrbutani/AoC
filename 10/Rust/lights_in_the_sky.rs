#!/usr/bin/env rustr
extern crate aoc;
#[macro_use(scan_fmt)] extern crate scan_fmt;

#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};
use std::fmt::{self, Display};
use std::i32;
use std::io::{self, BufRead, Write};

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
    vx: i32,
    vy: i32,
}

impl Point {
    pub fn step_forward(&mut self) {
        self.x += self.vx;
        self.y += self.vy;
    }

    pub fn step_forward_count(&mut self, count: usize) {
        self.step(count, |a, b| a + b)
    }

    pub fn step_backward(&mut self) {
        self.x -= self.vx;
        self.y -= self.vy;
    }

    pub fn step_backward_count(&mut self, count: usize) {
        self.step(count, |a, b| a - b)
    }

    fn step(&mut self, count: usize, op: impl Fn(i32, i32) -> i32) {
        self.x = op(self.x, self.vx * count as i32);
        self.y = op(self.y, self.vy * count as i32);
    }

    pub fn get_with_offset(&self, min_x: i32, min_y: i32) -> (usize, usize) {
        ((self.x - min_x) as usize, (self.y - min_y) as usize)
    }

    pub fn min_max(&self, (min_x, max_x, min_y, max_y): (i32, i32, i32, i32)) -> (i32, i32, i32, i32) {
        (self.x.min(min_x), self.x.max(max_x), self.y.min(min_y), self.y.max(max_y))
    }
}

enum Command {
    Forward(usize),
    Backward(usize),
    Search,
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Command::Forward(c) | Command::Backward(c) => {
                write!(f, "Going {} {} {}.",
                    if let Command::Forward(_) = *self { "forward" } else { "backward" },
                    c,
                    if *c == 1 { "step" } else { "steps" }
                )
            },
            Command::Search => write!(f, "Searching for smallest drawing by area.")
        }
    }
}

impl Command {
    fn new(mode: char, num: usize) -> Self {
        match mode {
            'f' | 'F' => Command::Forward(num),
            'b' | 'B' => Command::Backward(num),
            's' | 'S' => Command::Search,
            _ => Command::Forward(num),
        }
    }
}

fn command() -> Command {
    print!("\nCommand? [f|b|s] [num]: "); io::stdout().flush().unwrap();

    let io = io::stdin();
    let mut input = String::new();
    io.lock().read_line(&mut input).expect("Couldn't read input.");

    let (dir, count) = scan_fmt!(&mut input, "{[fbs]} {d}", char, usize);

    Command::new(dir.unwrap_or('_'), count.unwrap_or(1))
}

fn message() -> Option<String> {
    print!("What is it? "); io::stdout().flush().unwrap();

    let io = io::stdin();
    let mut input = String::new();
    io.lock().read_line(&mut input).ok()?;

    Some(input)
}

fn prompt(s: &str) -> bool {
    print!("{} (y/n): ", s); io::stdout().flush().unwrap();

    let io = io::stdin();
    let mut input = String::new();
    io.lock().read_line(&mut input).ok();

    if let Some('y') = input.trim().to_lowercase().chars().next() {
        println!("Proceeding.\n");
        true
    } else {
        println!("Not Proceeding.\n");
        false
    }
}

fn search(v: &mut Vec<Point>) -> usize {
    let size = |v: &Vec<Point>| {
        let (mix, max, miy, may) = v.iter().fold((i32::MAX, i32::MIN, i32::MAX, i32::MIN), |acc, p| p.min_max(acc));
        (max - mix) as usize * (may - miy) as usize
    };

    let mut current = 0;
    let mut min_size = size(v);

    let forward = |p: &mut Point| p.step_forward();
    let backward = |p: &mut Point| p.step_backward();

    // We're assuming a hyperbola:
    // By jumping and checking the direction of growth we can do a hokey sort
    // of binary search, but time. (TODO)

    loop {
        v.iter_mut().for_each(forward);

        let size = size(&v);
        if size > min_size {
            v.iter_mut().for_each(backward);
            return current
        } else {
            min_size = size;
            current += 1;
        }
    }
}

fn draw(v: &Vec<Point>) -> Option<String> {
    let (min_x, max_x, min_y, max_y) = v.iter().fold((i32::MAX, i32::MIN, i32::MAX, i32::MIN), |acc, p| p.min_max(acc));

    println!("{} to {}; {} to {}", min_x, max_x, min_y, max_y);
    if ! prompt("Proceed to draw?") { return None; }

    let mut grid = Vec::<Vec<bool>>::with_capacity((max_y - min_y + 1) as usize); // (y, x) indexed
    let mut row = Vec::<bool>::with_capacity((max_x - min_x + 1) as usize); // num of cols

    (0..(max_x - min_x + 1)).for_each(|_| row.push(false)); // Fill out a single row
    (0..(max_y - min_y + 1)).for_each(|_| grid.push(row.clone())); // And add clones of them to make our grid

    v.iter().for_each(|p| {
        let (x, y) = p.get_with_offset(min_x, min_y);

        grid[y][x] |= true;
    });

    grid.iter().for_each(|r| {
        r.iter().for_each(|p| print!("{}", if *p { '#' } else { '.' }));
        println!("");
    });

    if prompt("Do you see it?") { message() } else { None }
}

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new_with_year(2018, 10);
    let input: String = aoc.get_input();

    let mut points = input.lines().filter_map(|l| {
        let (x, y, v, w) = scan_fmt!(l, "position=<{d},{d}> velocity=<{d},{d}>", i32, i32, i32, i32);

        Some((x?, y?, v?, w?))
    }).map(|(x, y, vx, vy)| Point { x, y, vx, vy })
    .collect::<Vec<Point>>();

    let forward = |p: &mut Point| p.step_forward();
    let backward = |p: &mut Point| p.step_backward();

    let mut total_count = 0;

    loop {
        let c = command();

        println!("{}", c);

        match c {
            Command::Forward(count) => {
                total_count += count;
                (0..count).for_each(|_| points.iter_mut().for_each(forward))
            }
            Command::Backward(count) => {
                total_count -= count;
                (0..count).for_each(|_| points.iter_mut().for_each(backward))
            }
            Command::Search => {
                total_count += search(&mut points);
                println!("Jumped {} steps.", total_count)
            }
        }

        if let Some(s) = draw(&points) {
            aoc.submit_p1(s);//.and_then(|_| aoc.submit_p2(total_count));
            if prompt("Right?") { aoc.submit_p2(total_count); }
            if prompt("Exit?") { break; }
        }
    }
}
