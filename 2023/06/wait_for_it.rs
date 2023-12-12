use aoc::AdventOfCode;

use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Record {
    time: usize,
    distance: usize,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct Records(Vec<Record>);

impl FromStr for Records {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let mut line = |prefix| {
            lines
                .next()
                .unwrap()
                .strip_prefix(prefix)
                .unwrap()
                .split_whitespace()
                .map(|t| t.parse::<usize>().unwrap())
        };
        let times = line("Time: ");
        let distances = line("Distance: ");
        assert!(lines.next().is_none());

        Ok(Records(
            times
                .zip(distances)
                .map(|(time, distance)| Record { time, distance })
                .collect(),
        ))
    }
}

const INP: &str = "Time:      7  15   30
Distance:  9  40  200";
const INP2: &str = "Time:      71530
Distance:  940200";
const INP3: &str = "Time:        60808676
Distance:   601116315591300";

fn quadratic_equation(a: f64, b: f64, c: f64) -> (f64, Option<f64>) {
    let discriminant: f64 = b.powi(2) - 4. * a * c;
    if discriminant < 0. {
        panic!("complex nums not impl'd");
    }
    if discriminant == 0. {
        return (-b / 2., None);
    }

    let plus = (-b + discriminant.sqrt()) / 2.;
    let minus = (-b - discriminant.sqrt()) / 2.;
    (plus, Some(minus))
}

fn find_number_of_ways_to_win(Record { time, distance }: Record) -> usize {
    let (l, r) = quadratic_equation(-1., time as _, -(distance as f64));
    let (l, r) = (-l, -r.unwrap());
    let (mut l, mut r) = (l.ceil() as usize, r.floor() as usize);

    // If we're exactly equal to distance (or if float error caused us
    // to drift), adjust:
    if l * (time - l) <= distance {
        l += 1;
    }
    if r * (time - r) <= distance {
        r -= 1;
    }

    r - l + 1
}

fn main() {
    let mut aoc = AdventOfCode::new(2023, 6);
    let inp = aoc.get_input();
    // let inp = INP3;
    let records = inp.parse::<Records>().unwrap();

    // score(time, accel) = accel * (time - accel)
    //
    // solve for `target = score(time, ?)`:
    //
    // accel * (time - accel) - target = 0
    // `-1 * accel^2 + time * accel - target`
    //
    // quadratic equation: `ax^2 + bx + c = 0`
    //   - x = accel
    //   - a = -1
    //   - b = time
    //   - c = -target
    //
    // `-b ± sqrt(b² - 4ac)/2`
    //
    // we expect two roots; we want the distance between the roots (after
    // rounding)
    let p1 = records
        .0
        .iter()
        .cloned()
        .map(find_number_of_ways_to_win)
        .reduce(|a, b| a * b)
        .unwrap();
    // dbg!(p1);
    aoc.submit_p1(p1).unwrap();

    let p2 = {
        let mut lines = inp.lines();
        let digits = |l: &str| {
            l.chars()
                .filter(|x| x.is_ascii_digit())
                .collect::<String>()
                .parse::<usize>()
                .unwrap()
        };
        let time = digits(lines.next().unwrap());
        let distance = digits(lines.next().unwrap());

        find_number_of_ways_to_win(Record { time, distance })
    };
    // dbg!(p2);
    aoc.submit_p2(p2).unwrap();
}
