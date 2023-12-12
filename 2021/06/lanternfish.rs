#!/usr/bin/env rustr

use aoc::*;

fn main() {
    let mut aoc = AdventOfCode::new(2021, 6);
    let inp = aoc.get_input().split(',').dbg().map_parse().collect_vec();
    let inp = inp.into_iter().fold([0; 9], |mut arr, i: usize| {
        arr[i] += 1;
        arr
    });

    // routes:
    //   - memoization: <starting state> N days later will always yield the same
    //     number of children; can add up the mappings for the states on day 0
    //   - "brute force": efficient simulation isn't hard; it's just a rotate
    //   - curve fitting?
    //     + if we didn't have the "9 days to reproduce the first time only"
    //       requirement it'd be very easy to come up with an equation; it's
    //       just `2 ** (N // 7)`
    //     + the 9 days thing ruins this a bit; it's not obvious to me how to
    //       model this

    fn simulate_step(school: &mut [usize; 9]) {
        school.rotate_left(1);
        school[6] += school.last().copied().unwrap();
    }

    // let's stare at some graphs for a bit
    #[cfg(any(feature = "python", feature = "plot"))]
    let points = (0..=500)
        .scan([0, 0, 0, 0, 0, 0, 0, 0, 1], |s, i| {
            simulate_step(s);
            Some((i, s.iter().sum::<usize>()))
        })
        .collect_vec();

    #[cfg(feature = "python")]
    inline_python::python! {
        import matplotlib.pyplot as plt
        plt.plot('points)
        plt.show()
    }
    #[cfg(feature = "plot")]
    {
        use plotters::prelude::*;

        let root =
            BitMapBackend::new("lanternfish_population.png", (1920, 1080)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let mut chart = ChartBuilder::on(&root)
            .build_cartesian_2d(0..points.len(), 0..points.last().unwrap().1 + 1)
            .unwrap();
        chart.configure_mesh().draw().unwrap();

        chart.draw_series(LineSeries::new(points, &RED)).unwrap();
        // .label("y = x^2")
        // .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    }

    //  0 | (1 0) | 8
    //  1 | (1 0) | 7
    //  2 | (1 0) | 6
    //  3 | (1 0) | 5
    //  4 | (1 0) | 4
    //  5 | (1 0) | 3
    //  6 | (1 0) | 2
    //  7 | (1 0) | 1
    //  8 | (1 0) | 0
    //  9 | (1 1) | 6 8
    // 10 | () | 5 7
    // 11 | () | 4 6
    // 12 | () | 3 5
    // 13 | () | 2 4
    // 14 | () | 1 3
    // 15 | () | 0 2
    // 16 | () | 6 1 8
    // 17 | () | 5 0 7
    // 18 | () | 4 6 6 8
    // 19 | () | 3 5 5 7
    // 20 | () | 2 4 3 6

    /*     const STARTING_VAL: usize = 8;

    // 0.6983371574⋅1.0909299047^x;
    // let eq = |x| (0.6983371574 * 1.0909299047f64.powf(x as f64)) as usize;
    // y=0.00052970846317244355x5−0.19541134471412432792x4+25.64692335621842463972x3−1416.01342319004732931150x2+29695.28208247545179842661x−147640.27385001214498893374
    let eq = |x| {
        let x = x as f64;
        let res = 0.00052970846317244355 * x.powi(5) - 0.19541134471412432792 * x.powi(5)
            + 25.64692335621842463972 * x.powi(3)
            - 1416.01342319004732931150 * x.powi(2)
            + 29695.28208247545179842661 * x
            - 147640.27385001214498893374;
        res as usize
    };

    let mut v = vec![STARTING_VAL];
    for day in 0..200 {
        // println!("{:2} | {}", day, v.iter().join(" "));
        println!("{:2} | {} vs {}", day, v.len(), eq(day as usize));

        let new = v
            .iter_mut()
            .filter_map(|i| {
                if *i == 0 {
                    *i = 6;
                    Some(STARTING_VAL)
                } else {
                    *i -= 1;
                    None
                }
            })
            .collect_vec();
        v.extend(new);
    } */

    let p1: usize = (0..80)
        .fold(inp, |mut arr, _| {
            simulate_step(&mut arr);
            arr
        })
        .iter()
        .sum();
    aoc.submit_p1(p1).unwrap();

    let p2: usize = (0..256)
        .fold(inp, |mut arr, _| {
            simulate_step(&mut arr);
            arr
        })
        .iter()
        .sum();
    aoc.submit_p2(p2).unwrap();
}
