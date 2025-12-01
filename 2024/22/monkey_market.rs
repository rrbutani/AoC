use std::{
    collections::{HashMap, HashSet},
    iter::successors,
};

use aoc::*;

fn next(mut curr: usize) -> usize {
    curr = ((curr * 64) ^ curr) % 16_777_216;
    curr = ((curr / 32) ^ curr) % 16_777_216;
    curr = ((curr * 2048) ^ curr) % 16_777_216;
    curr
}

#[test]
fn secret_sequence() {
    use std::iter::successors;
    let it = successors(Some(123), |&n| Some(next(n))).skip(1).take(10);
    assert_eq!(
        it.collect_vec(),
        [
            15887950, 16495136, 527345, 704524, 1553684, 12683156, 11100544, 12249484, 7753432,
            5908254,
        ]
    );
}

fn main() {
    let mut aoc = AdventOfCode::new(2024, 22);
    let inp = aoc.get_input();
    let secret_nums = inp.lines().map_parse::<usize>();

    let p1 = secret_nums
        .clone()
        .map(|mut s| {
            for _ in 0..2000 {
                s = next(s);
            }
            s
        })
        // .inspect(|n| eprintln!("{n}"))
        .sum::<usize>();
    _ = aoc.submit_p1(p1);

    let p2: usize = {
        /*
        for (idx, mut num) in secret_nums.clone().enumerate() {
            let mut seen = HashSet::new();
            for s in 0..2000 {
                if !seen.insert(num) {
                    eprintln!("cyc! {idx} at {s}");
                    break;
                }
                num = next(num);
            }
        }
        */
        // no cycles

        let buyer_info = secret_nums
            .map(|s| {
                successors(Some(s), |&s| Some(next(s)))
                    .take(2001)
                    .map(|n| (n % 10) as u8)
                    .collect_vec()
            })
            .map(|prices| {
                let deltas = prices
                    .windows(2)
                    .map(|w| w[1] as i8 - w[0] as i8)
                    .collect_vec();
                (prices, deltas)
            })
            .collect_vec();

        #[cfg(any())]
        {
            for (buyer_idx, (prices, deltas)) in buyer_info.iter().enumerate() {
                eprintln!("----------------------------------------------------");
                eprintln!("buyer {buyer_idx}");
                eprintln!("{}", prices[0]);
                eprintln!("{} ({:2})", prices[1], deltas[0]);
                eprintln!("{} ({:2})", prices[2], deltas[1]);
                eprintln!("{} ({:2})", prices[3], deltas[2]);
                for (idx, delta_seq) in deltas.windows(4).enumerate() {
                    eprintln!("{} ({:2}) — {:?}", prices[idx + 4], deltas[idx + 3], delta_seq);
                }
                eprintln!();
            }
        }

        // just gonna do dumb brute force...
        //
        // produce a map; scores for each delta of size 4
        let mut delta_seq_scores = HashMap::new();
        for (_buyer_idx, (prices, deltas)) in buyer_info.iter().enumerate() {
            let mut seen = HashSet::<&[i8]>::new();
            for (idx, delta_seq) in deltas.windows(4).enumerate() {
                if seen.contains(delta_seq) {
                    continue; // buyers only buy once
                }
                seen.insert(delta_seq);
                *delta_seq_scores.entry(delta_seq).or_default() += prices[idx + 4] as usize;
            }
        }

        #[cfg(any())]
        {
            for (d, score) in &delta_seq_scores {
                eprintln!("{d:?} = {score}");
            }
        }

        delta_seq_scores.values().max().copied().unwrap()
        // let delta_seqs = info.iter().flat_map(|(_, deltas)| deltas.windows(4)).collect::<HashSet>
    };
    _ = aoc.submit_p2(p2);
}
