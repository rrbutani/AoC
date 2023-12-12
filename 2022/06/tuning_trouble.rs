use aoc::*;

use std::collections::HashSet;

fn find_start(seq: &[char], len: usize) -> Option<usize> {
    seq
        // .array_windows::<4>()
        .windows(len)
        .enumerate()
        .find(|(_, chars)| {
            // inefficient but w/e
            chars.iter().collect::<HashSet<_>>().len() == len
        })
        .map(|(i, _)| i + len)
}

fn main() {
    let mut aoc = AdventOfCode::new(2022, 6);
    let inp = aoc.get_input();
    let inp = inp.chars().collect_vec();
    let find_start = |len| find_start(&inp, len).unwrap();

    aoc.submit_p1(find_start(4)).unwrap();
    aoc.submit_p2(find_start(14)).unwrap();
}

#[cfg(test)]
mod tests {
    use super::{find_start, Itertools};

    fn test(inp: &str, len: usize, exp: usize) {
        let chars = inp.chars().collect_vec();
        assert_eq!(find_start(&chars, len), Some(exp));
    }

    #[test]
    fn p1() {
        let t = |exp, inp| test(inp, 4, exp);

        t(5, "bvwbjplbgvbhsrlpgdmjqwftvncz");
        t(6, "nppdvjthqldpwncqszvftbrmjlhg");
        t(10, "nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg");
        t(11, "zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw");
    }

    #[test]
    fn p2() {
        let t = |exp, inp| test(inp, 14, exp);

        t(19, "mjqjpqmgbljsphdztnvjfqwrcgsmlb");
        t(23, "bvwbjplbgvbhsrlpgdmjqwftvncz");
        t(23, "nppdvjthqldpwncqszvftbrmjlhg");
        t(29, "nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg");
        t(26, "zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw");
    }
}
