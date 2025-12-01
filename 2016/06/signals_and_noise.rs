use aoc::AdventOfCode;

use itertools::Itertools;

fn main() {
    let mut aoc = AdventOfCode::new(2016, 6);
    let inp = aoc.get_input();
    let rows = inp.lines().map(|l| l.chars().collect_vec()).collect_vec();

    let select_char_in_col = |most_frequent| {
        (0..rows[0].len())
            .map(|col| {
                let mut by_freq = (0..rows.len())
                    .map(|row| rows[row][col])
                    .sorted()
                    .dedup_with_count()
                    .sorted()
                    .map(|(_f, c)| c)
                    .rev();

                let char = if most_frequent { by_freq.next() } else { by_freq.last() };

                char.unwrap()
            })
            .collect::<String>()
    };

    let p1 = select_char_in_col(true);
    _ = aoc.submit_p1(p1);

    let p2 = select_char_in_col(false);
    _ = aoc.submit_p2(p2);
}
