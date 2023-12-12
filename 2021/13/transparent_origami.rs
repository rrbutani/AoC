#!/usr/bin/env rustr

use std::{
    collections::HashSet,
    io::{self, BufRead},
};

use aoc::*;

fn draw(points: impl Iterator<Item = (usize, usize)>) -> String {
    let points = points.collect_vec();
    let (&max_x, &max_y) = (
        points.iter().map(|(x, _)| x).max().unwrap(),
        points.iter().map(|(_, y)| y).max().unwrap(),
    );

    let mut grid = (0..=max_y)
        .map(|_| (0..=max_x).map(|_| false).collect_vec())
        .collect_vec();

    for (x, y) in points {
        grid[y][x] = true;
    }

    grid.iter()
        .map(|line| {
            line.iter()
                .map(|&c| if c { '#' } else { '.' })
                .collect::<String>()
        })
        .join("\n")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Fold {
    X(usize),
    Y(usize),
}

fn apply_fold(
    points: impl Iterator<Item = (usize, usize)>,
    fold: Fold,
) -> impl Iterator<Item = (usize, usize)> {
    points.map(move |(x, y)| match fold {
        Fold::X(xf) => (if x > xf { xf - (x - xf) } else { x }, y),
        Fold::Y(yf) => (x, if y > yf { yf - (y - yf) } else { y }),
    })
}

fn main() {
    let mut aoc = AdventOfCode::new(2021, 13);
    let inp = aoc.get_input();
    let (points, folds) = inp.split_once("\n\n").unwrap();

    let points = points
        .lines()
        .map(|l| l.split(',').map_parse::<usize>().tuple::<2>());
    let folds = folds
        .lines()
        .map(|l| l.split(' ').nth(2).unwrap().split_once('=').unwrap())
        .map(|(axis, num)| {
            let num = num.parse().unwrap();
            match axis {
                "x" => Fold::X(num),
                "y" => Fold::Y(num),
                other => panic!("not an axis! `{}`", other),
            }
        });

    let p1 = apply_fold(points.clone(), folds.clone().next().unwrap())
        .collect::<HashSet<_>>()
        .len();
    aoc.submit_p1(p1).unwrap();

    let p2 = folds.fold(points.collect::<HashSet<_>>(), |points, fold| {
        apply_fold(points.into_iter(), fold).collect()
    });
    println!("{}\n\nWhat do you see?", draw(p2.into_iter()));

    let io = io::stdin();
    let mut input = String::new();
    io.lock().read_line(&mut input).ok().unwrap();
    aoc.submit_p2(input.trim()).unwrap();
}
