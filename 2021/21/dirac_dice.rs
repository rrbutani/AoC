#!/usr/bin/env rustr

use aoc::*;

fn main() {
    let mut aoc = AdventOfCode::new(2021, 21);
    let [p1, p2]: [u8; 2] = aoc
        .get_input()
        .lines()
        .map(|l| l.split_once(":").unwrap().1.trim().parse().unwrap())
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap();

    // let p1 = 4;
    // let p2 = 8;

    // p1:
    // (6 * x + 1, 6 * x + 2, 6 * x + 3) for x in ..
    // sum ^ % 10 → 6 4 2 0 8 6 ... (cycles of 5)
    // 6 4 2 0 8 = 10; cycle time of 5 under mod 10
    //
    // 1: 1 7 1 3 3 | 1 7 1 3 3
    // 2: 2 8 2 4 4 | 2 8 2 4 4
    // 3: 3 9 3 5 5 | 3 9 3 5 5
    // 4: 4 X 4 6 6 | 4 X 4 6 6
    // 5: 5 1 5 7 7 | 5 1 5 7 7
    // 6: 6 2 6 8 8 | 6 2 6 8 8
    // 7: 7 3 7 9 9 | 7 3 7 9 9
    // 8: 8 4 8 X X | 8 4 8 X X
    // 9: 9 5 9 1 1 | 9 5 9 1 1
    // X: X 6 X 2 2 | X 6 X 2 2
    //
    // Checked with:
    // ```python
    // list(accumulate([S] + [sum(x) % 10 for x in [(6*x+1, 6*x+2, 6*x+3) for x in range(10)]], lambda a, b: ((a + b - 1) % 10) + 1))
    // ```

    // p2:
    // (6 * x + 4, 6 * x + 5, 6 * x + 6) for x in ..
    // sum ^ % 10 → 5 3 1 9 7 5 ... (cycles of 5)
    // 5 3 1 9 7 5 = 30, but 6 is not divisible by 5..
    // 5 3 1 9 7 5 3 1 9 7 = 50; cycle time of 10 under mod 10
    //
    // 1: 1 6 9 X 9 6 1 4 5 4 | 1 6 9 X 9 6 1 4 5 4
    // 2: 2 7 X 1 X 7 2 5 6 5 | 2 7 X 1 X 7 2 5 6 5
    // 3: 3 8 1 2 1 8 3 6 7 6 | 3 8 1 2 1 8 3 6 7 6
    // 4: 4 9 2 3 2 9 4 7 8 7 | 4 9 2 3 2 9 4 7 8 7
    // 5: 5 X 3 4 3 X 5 8 9 8 | 5 X 3 4 3 X 5 8 9 8
    // 6: 6 1 4 5 4 1 6 9 X 9 | 6 1 4 5 4 1 6 9 X 9
    // 7: 7 2 5 6 5 2 7 X 1 X | 7 2 5 6 5 2 7 X 1 X
    // 8: 8 3 6 7 6 3 8 1 2 1 | 8 3 6 7 6 3 8 1 2 1
    // 9: 9 4 7 8 7 4 9 2 3 2 | 9 4 7 8 7 4 9 2 3 2
    // X: X 5 8 9 8 5 X 3 4 3 | X 5 8 9 8 5 X 3 4 3
    //
    // Checked with:
    // ```python
    // list(accumulate([S] + [sum(x) % 10 for x in [(6*x+4, 6*x+5, 6*x+6) for x in range(20)]], lambda a, b: ((a + b - 1) % 10) + 1))
    // ```

    const P1_INCREMENT_PATTERN: [u8; 5] = [6, 4, 2, 0, 8];
    const P2_INCREMENT_PATTERN: [u8; 5] = [5, 3, 1, 9, 7];

    let p1_score_sequence: [u8; 5] = P1_INCREMENT_PATTERN
        .iter()
        .scan(p1, |sum, inc| {
            *sum = ((*sum + inc - 1) % 10) + 1;
            Some(*sum)
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let p2_score_sequence: [u8; 10] = P2_INCREMENT_PATTERN
        .iter()
        .cycle()
        .take(P2_INCREMENT_PATTERN.len() * 2)
        .scan(p2, |sum, inc| {
            *sum = ((*sum + inc - 1) % 10) + 1;
            Some(*sum)
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let (p1_steps, p2_steps) = solve(1000, &p1_score_sequence, &p2_score_sequence);
    let steps = (p1_steps + p2_steps + 2) * 3;
    let part1 = if p1_steps > p2_steps {
        steps * score_after_steps(p2_steps + 1, &p2_score_sequence)
    } else {
        steps * score_after_steps(p1_steps + 1, &p1_score_sequence)
    };

    // why did I write a constant-ish time p1 solution. gdi.
    aoc.submit_p1(part1).unwrap();

    // [1, 2, 3] x [1, 2, 3] x [1, 2, 3] | sum
    // ^:
    // 3: 1x
    // 4: 3x
    // 5: 6x
    // 6: 7x
    // 7: 6x
    // 8: 3x
    // 9: 1x
    let mut p1_wins = 0;
    let mut p2_wins = 0;
    fn sim(
        p1_score: usize,
        p2_score: usize,
        p1_pos: u8,
        p2_pos: u8,
        p1_turn: bool,
    ) -> (usize, usize) {
        if p1_score >= 21 {
            return (1, 0);
        }
        if p2_score >= 21 {
            return (0, 1);
        }

        #[rustfmt::skip]
        const TABLE: [(usize, u8); 7] = [
            (1, 3),
            (3, 4),
            (6, 5),
            (7, 6),
            (6, 7),
            (3, 8),
            (1, 9),
        ];

        if p1_turn {}
    }
}

fn score_after_steps(steps: usize, seq: &[u8]) -> usize {
    let s = seq.iter().map(|s| *s as usize);
    s.clone().sum::<usize>() * (steps / seq.len()) + s.take(steps % seq.len()).sum::<usize>()
}

// steps for p1, steps for p2
fn solve(target_score: usize, p1: &[u8], p2: &[u8]) -> (usize, usize) {
    // we're really after the LCM here but we've been given nice numbers so we
    // don't need to write or use an actual impl that uses Stein's algorithm to
    // find the GCD to find the LCM, etc.
    assert!(p1.len() * 2 == p2.len());
    let lcm = p2.len();

    let p1_seq_sum = p1.iter().map(|i| *i as usize).cycle().take(lcm).sum();
    let p2_seq_sum = p2.iter().map(|i| *i as usize).cycle().take(lcm).sum();

    if target_score < p1_seq_sum || target_score < p2_seq_sum {
        panic!("can't handle small scores! the non-linear nature means we can make mistakes; step instead");
    }

    let calculate_steps = |sum, seq: &[u8]| {
        let steps = (target_score / sum) * lcm;
        let mut remaining = (target_score % sum) as isize;
        steps
            + seq
                .iter()
                .take_while(|n| {
                    remaining -= **n as isize;
                    remaining > 0
                })
                .count()
    };

    if p1_seq_sum > p2_seq_sum {
        let steps = calculate_steps(p1_seq_sum, p1);
        (steps, steps - 1)
    } else {
        let steps = calculate_steps(p2_seq_sum, p2);
        (steps, steps)
    }
}
