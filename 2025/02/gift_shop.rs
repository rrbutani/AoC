use aoc::{AdventOfCode, Itertools};

fn main() {
    let mut aoc = AdventOfCode::new(2025, 2);
    let inp = aoc.get_input();
    let ranges = inp
        .trim()
        .split(',')
        .map(|r| {
            let (start, end) = r.split_once('-').unwrap();
            let [start, end] = [start, end].map(|n| n.parse::<u64>().unwrap());
            debug_assert!(start < end);
            start..=end
        })
        .collect_vec();

    let mut sum = 0;
    for n in ranges.iter().cloned().flat_map(|r| r.into_iter()) {
        let log10 = n.ilog10();
        //    1 -    9 -> 0
        //   10 -   99 -> 1
        //  100 -  999 -> 2
        // 1000 - 9999 -> 3
        if log10 % 2 == 1 {
            // if odd
            let mask = 10u64.pow(log10 / 2 + 1); // i.e. 3 -> 10 ^ 2 -> 100
            let lower = n % mask;
            let upper = n / mask;
            if lower == upper {
                sum += n;
            }
        }
        // can do better than scanning of course but.. let's see what p2 is
    }
    _ = aoc.submit_p1(sum);
}
