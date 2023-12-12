#!/usr/bin/env rustr
#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

#[derive(Clone, Copy)]
struct Position {
    x: isize,
    y: isize,
    z: isize,
}

impl Position {
    fn dist(&self, other: &Position) -> usize {
        (self.x - other.x).abs() as usize
            + (self.y - other.y).abs() as usize
            + (self.z - other.z).abs() as usize
    }
}

#[derive(Clone, Copy)]
struct NanoBot {
    pos: Position,
    rad: usize,
}

impl NanoBot {
    fn can_reach_bot(&self, bot: &NanoBot) -> bool {
        self.pos.dist(&bot.pos) <= self.rad
    }

    fn within_range(&self, pos: &Position) -> bool {
        self.pos.dist(pos) <= self.rad
    }
}

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new(2018, 23);
    let input: String = aoc.get_input();

    let mut nanobots: Vec<NanoBot> = input
        .lines()
        .filter_map(|l| {
            let (x, y, z, r) = scan_fmt!(l, "pos=<{},{},{}>, r={}", isize, isize, isize, usize);
            Some(NanoBot {
                pos: Position {
                    x: x?,
                    y: y?,
                    z: z?,
                },
                rad: r?,
            })
        })
        .collect();

    nanobots.sort_unstable_by_key(|n| n.rad);

    let strongest = nanobots.last().unwrap();
    let in_range = nanobots
        .iter()
        .filter(|b| strongest.can_reach_bot(b))
        .count();
    aoc.submit_p1(in_range);

    // Okay, time to do some dynamic programming:
    //
    // For each nanobot, we can choose to be in range of it or to not be in
    // range of it. These are the two paths to explore. Regardless of what we
    // choose we now have a 'range' of locations where we can be. We start with
    // a range signifying any location.
    //
    // If we choose to be in range of a nanobot, our range becomes the
    // intersection of our existing range and the range of the nanobot. If we
    // choose to be out of range of a nanobot, it's the intersection of our
    // existing range and the inverted range of the nanobot. If at any point
    // this range becomes zero, we stop.
    //
    // We memoize on (which nanobot we're on) and (current range). Memoization
    // (and, by extension, DP) probably won't buy us anything here (at least not
    // without some clever way to equate ranges) so we're probably going to be
    // O(2 ^ n) where n is the number of nanobots. Still better than the
    // absolute brute force solution but not great.
    //
    // The tricky bit, of course, is finding the intersections of ranges.
    //
    // Let's take a cube as an example. You can define a cube as a range of x,
    // y, and z values and the reason you can do this is that for any x value in
    // the range exactly the same y and z values are valid (i.e. the y values in
    // the y range, z vals in the z range). This is in contrast with a sphere
    // where, for example, an x value (where x is distance from the sphere's
    // centre along the x-axis) equal to the sphere's radius will have exactly
    // one valid y/z value while an x value smaller than the radius will have
    // multiple.
    //
    // This means that finding the intersection between two nanobot ranges is as
    // simple as finding the intersections between their respective x/y/z
    // ranges! Right?
    //
    // Well, no. Defining a 3D figure (or, a 3-ball) with a Manhattan distance
    // radius actually gets you a a double sided pyramid, or an _octahedron_
    // where the base 'square' is rotated by 45 degrees. Here's some 2D art that
    // probably isn't very helpful:
    //
    //  #########  #########  #########  #########  21 + 24 = 25
    //  #...3...#  #...1...#  #...7...#  #...d...#  a -> (1, 1)
    //  #..323..#  #..123..#  #..567..#  #..cgk..#  b -> (2, 1)
    //  #.32123.#  #.12345.#  #.34567.#  #.bfjnr.#  c -> (3, 1)
    //  #321@123#  #1234567#  #1234567#  #aeimquy#  d -> (4, 1)
    //  #.32123.#  #.34567.#  #.12345.#  #.hlptx.#  e -> (1.5, 2)
    //  #..323..#  #..567..#  #..123..#  #..osw..#  f -> (2.5, 2)
    //  #...3...#  #...7...#  #...1...#  #...v...#  g -> (3.5, 2)
    //  #########  #########  #########  #########  ...
    //
    // It's a little hard to intuit but because the Manhattan distance is
    // involved, this weird looking octahedron shaped range for each nanobot
    // will be pointing the same way. That is, the angled bases will all be
    // facing the same way and the pointy tops too. I find it helpful to think
    // about spheres first: you can make a sphere by spinning a stick that's
    // tethered on one end (i.e. the sphere's center) every-which-way about two
    // planes (doing one gets you a circle). Octahedrons are the same except now
    // your stick is this weird kind of floppy thing that's made of links. Each
    // link has to be aligned with an axis and that's what gets you a diamond
    // looking thing (above) when you try to spin the stick. Ultimately, because
    // Manhattan distance snaps you to a grid and because the nanobots are all
    // in this same shared grid, all their ranges are going to point the same
    // way.
    //
    // Other possibly non-obvious thing is that the intersection of any of two
    // octahedrons is an octahedron.
    //
    // ##############  ##############
    // #........a...#  #...a........#
    // #.......aaa..#  #..aaa.......#
    // #......aaaaa.#  #.aaaaa......#
    // #.....aaaaaaa#  #aaaaaaa.....#
    // #......aaaaa.#  #.aaaaa......#
    // #...b...aaa..#  #..aca.......#
    // #..bbb...a...#  #..bcb.......#
    // #.bbbbb......#  #.bbbbb......#
    // #bbbbbbb.....#  #bbbbbbb.....#
    // #.bbbbb......#  #.bbbbb......#
    // #..bbb.......#  #..bbb.......#
    // #...b........#  #...b........#
    // ##############  ##############
    //
    //

    let p2 = 0;
    println!("{}", p2);
    // aoc.submit_p2(p2);
}
