use aoc::AdventOfCode;
use itertools::Itertools;

fn has_abba(s: &str) -> bool {
    s.chars()
        .tuple_windows()
        .any(|(a, b, c, d)| a == d && b == c && a != b)
}

fn main() {
    let mut aoc = AdventOfCode::new(2016, 7);
    let inp = aoc.get_input();
    let ips = inp
        .lines()
        .map(|ip| {
            let mut supernets = vec![String::new()];
            let mut hypernets = vec![];
            let mut curr = &mut supernets[0];

            for c in ip.chars() {
                match c {
                    '[' => {
                        hypernets.push(String::new());
                        curr = hypernets.last_mut().unwrap();
                    }
                    ']' => {
                        supernets.push(String::new());
                        curr = supernets.last_mut().unwrap();
                    }
                    o => curr.push(o),
                }
            }

            (supernets, hypernets)
        })
        .collect_vec();

    let p1 = ips
        .iter()
        .filter(|(sup, hyper)| {
            sup.iter().any(|s| has_abba(s)) && hyper.iter().all(|s| !has_abba(s))
        })
        .count();
    _ = aoc.submit_p1(p1);
}
