#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};
use std::collections::HashSet;

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new(2015, 3);
    let input: String = aoc.get_input();

    fn do_the_steps(hs: &mut HashSet<(i16, i16)>, s: &str) {
        let (mut x, mut y) = (0, 0);
        hs.insert((x, y));

        s.chars().for_each(|c| {
            match c {
                '>' => x += 1,
                'v' => y -= 1,
                '<' => x -= 1,
                '^' => y += 1,
                 _  => (),
            };

            hs.insert((x, y));
        });
    }

    let houses: usize = input.lines().map(|l|{
        let mut hs = HashSet::<(i16, i16)>::new();
        do_the_steps(&mut hs, l);

        hs.iter().count() as usize
    }).sum();

    aoc.submit_p1(houses);

    let houses_p2: usize = input.lines().map(|l|{
        let mut hs = HashSet::<(i16, i16)>::new();
        do_the_steps(&mut hs, &l.chars().step_by(2).collect::<String>());

        let mut robo = l.chars();
        robo.next();
        do_the_steps(&mut hs, &robo.step_by(2).collect::<String>());

        hs.iter().count() as usize
    }).sum();

    aoc.submit_p2(houses_p2);
}
