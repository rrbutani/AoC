#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new(2015, 5);
    let input: String = aoc.get_input();

    let num_nice_strings = input.lines().filter(|s| {
        let v = s.chars().collect::<Vec<char>>();
        s.chars().filter(|c| "aeiou".contains(*c)).count() >= 3 &&
        v.windows(2).any(|v| v[0] == v[1]) &&
        ! s.contains("ab") &&
        ! s.contains("cd") &&
        ! s.contains("pq") &&
        ! s.contains("xy")
    }).count();

    aoc.submit_p1(num_nice_strings);

    let num_nice_strings = input.lines().filter(|s| {
        let v = s.chars().collect::<Vec<char>>();
        v.windows(2).enumerate().any(|(i, v)|{
            let pair = format!("{}{}", v[0], v[1]);
            let idx  = i + 2;

            idx < s.len() && (s[idx..]).contains(&pair)
        }) &&
        v.windows(3).any(|v| v[0] == v[2])
    }).count();

    aoc.submit_p2(num_nice_strings);
}
