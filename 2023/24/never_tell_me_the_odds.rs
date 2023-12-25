use aoc::{iterator_map_ext::IterMapExt, AdventOfCode, FromStr, Itertools, Triple};
use z3::{
    ast::{Ast, Int},
    Config, Context, SatResult, Solver,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Hailstone {
    position: Triple<isize>,
    velocity: Triple<isize>,
}

impl FromStr for Hailstone {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (pos, vel) = s.split_once(" @ ").unwrap();
        Ok(Self { position: pos.parse().unwrap(), velocity: vel.parse().unwrap() })
    }
}

impl Hailstone {
    // fn xy_collision_point(&self, other: &Hailstone) -> Option<Triple<f64>> {
    fn xy_intercept_point(&self, other: &Hailstone) -> Option<((f64, f64), f64, f64)> {
        // h1_x(t) = x1 + t * v_x1
        // h1_y(t) = y1 + t * v_y1
        let Self { position: Triple(x1, y1, _), velocity: Triple(v_x1, v_y1, _) } = *self;
        // h2_x(t) = x2 + t * v_x2
        // h2_y(t) = y2 + t * v_y2
        let Self { position: Triple(x2, y2, _), velocity: Triple(v_x2, v_y2, _) } = *other;
        // trying to find (x, y) such that:
        // `h1_x(t) == h2_x(t)` and `h1_y(t) == h2_y(t)` for some t
        //
        // `x1 + t * v_x1 - (x2 + t * v_x2) == 0`
        // `y1 + t * v_y1 - (y2 + t * v_y2) == 0`
        //
        // ```
        // x1 + t * v_x1 - (x2 + t * v_x2) == y1 + t * v_y1 - (y2 + t * v_y2)
        // x1 + t * v_x1 - x2 - t * v_x2   == y1 + t * v_y1 - y2 - t * v_y2
        // x1 - x2 + t * (v_x1 - v_x2)     == y1 - y2 + t * (v_y1 - v_y2)
        // ```
        //
        // wait no no the hailstones themselves don't have to literally collide;
        // just the paths
        //
        // i.e. t1 and t2 can be different!

        // h1_x(t1) = x1 + t1 * v_x1
        // h1_y(t1) = y1 + t1 * v_y1
        //
        // h2_x(t2) = x2 + t2 * v_x2
        // h2_y(t2) = y2 + t2 * v_y2
        //
        // trying to find (x, y) such that:
        // `h1_x(t1) == h2_x(t2)` and `h1_y(t1) == h2_y(t2)` for some t1, t2
        //
        // let's rewrite into `y(x) = m * x + b` form:
        // ```
        // x(t) = x_c + t * v_x
        // t(x) = (x - x_c) / v_x
        //
        // y(t) = y_c + t * v_y
        // sub in `t(x)`:
        // y(x) = y_c + (x - x_c) / v_x * v_y
        //
        // b = y_c
        // m = v_y / v_x
        // x is ... offset
        // ```
        //
        // the equations for `self` and `other` now look like:
        // ```
        // p1_y(x) = y1 + (x - x1) * (v_y1 / v_x1)
        // p2_y(x) = y2 + (x - x2) * (v_y2 / v_x2)
        // ```
        //
        // We can set them equal to each other and solve for `x`:
        // ```
        // y1 + (x - x1) * (v_y1 / v_x1) = y2 + (x - x2) * (v_y2 / v_x2)
        // y1 - y2 - x1 * (v_y1 / v_x1) + x2 * (v_y2 / v_x2) = - x * (v_y1 / v_x1) + x * (v_y2 / v_x2)
        // y1 - y2 - x1 * (v_y1 / v_x1) + x2 * (v_y2 / v_x2) = x * ((v_y2 / v_x2) - (v_y1 / v_x1))
        //
        // m1 := v_y1 / v_x1
        // m2 := v_y2 / v_x2
        //
        //
        // y1 - y2 - x1 * m1 + x2 * m2 = x * (m2 - m1)
        // (y1 - y2 - x1 * m1 + x2 * m2) / (m2 - m1) = x
        // ```
        //
        // And then plug back into either equation to get `y`.

        let (y1, y2, x1, x2) = (y1 as f64, y2 as f64, x1 as f64, x2 as f64);
        let (v_y1, v_x1, v_y2, v_x2) = (v_y1 as f64, v_x1 as f64, v_y2 as f64, v_x2 as f64);
        let m1 = v_y1 / v_x1;
        let m2 = v_y2 / v_x2;
        if m1 == m2 {
            return None;
        }

        let x = (y1 - y2 - x1 * m1 + x2 * m2) / (m2 - m1);

        let y_res_1 = y1 + (x - x1) * (v_y1 / v_x1);
        let y_res_2 = y2 + (x - x2) * (v_y2 / v_x2);
        // assert!((y_res_1 - y_res_2).abs() < 1f64); // need a tolerance
        // ^ fails!
        let y = if (y_res_1 - y_res_2).abs() > 1f64 {
            (y_res_1 + y_res_2) / 2f64
        } else {
            y_res_1
        };

        // We also need to solve for `t1` and `t2` to know if the intercept
        // point was "in the past":
        // ```
        // t(x) = (x - x_c) / v_x
        // ```

        let t1 = (x - x1) / v_x1;
        let t2 = (x - x2) / v_x2;

        Some(((x, y), t1, t2))
    }
}

const INP: &str = "19, 13, 30 @ -2,  1, -2
18, 19, 22 @ -1, -1, -2
20, 25, 34 @ -2, -2, -4
12, 31, 28 @ -1, -2, -1
20, 19, 15 @  1, -5, -3";

fn p1(hailstones: &[Hailstone], lower_bound: f64, upper_bound: f64) -> usize {
    hailstones
        .iter()
        .tuple_combinations()
        .filter_map(|(h1, h2)| h1.xy_intercept_point(h2))
        .filter(|&((x, y), t1, t2)| {
            // Ignore intercepts in the past:
            if t1 < 0f64 || t2 < 0f64 {
                return false;
            }

            if !(x >= lower_bound && x <= upper_bound) {
                return false;
            }
            if !(y >= lower_bound && y <= upper_bound) {
                return false;
            }

            true
        })
        .count()
}

// Part 2:
//
// We're trying to find: `rx, ry, rz; rv_x, rv_y, rv_z`
//
// such that:
// ```
// for i in { number of hailstones }:
//     h1_x(t{i}) = x{i} + t{i} * v_x{i} == rx(t{i}) = rx + t{i} * rv_x
//     h1_y(t{i}) = y{i} + t{i} * v_y{i} == ry(t{i}) = ry + t{i} * rv_y
//     h1_z(t{i}) = z{i} + t{i} * v_z{i} == rz(t{i}) = rz + t{i} * rv_z
//
//    (for some `t{i}`)
// ```
//
// Do we need to consider all the hailstones?
//   - if we were given points + times for the hailstones:
//     + in 2D, two points would be sufficient to interpolate and uniquely
//       identify the line pass through them
//     + in 3D I think two points are still sufficient
//   - but, we are given lines instead of points...
//
// we have 6 unknowns for each set of three equations we add on (we get three
// equations for each hailstone) we get one more unknown: `t{i}`
//
// So it's:
//   - 3 equations, 7 unknowns
//   - 6 equations, 8 unknowns
//   - 9 equations, 9 unknowns
//   - ...
//
// I think this means we should be able to only consider three hailstones? Not
// sure.
//
// We can try it for the example input:
// (variables with underscores in them are unknowns, the rest are constants)
// ```
// x1 + t_1 * vx1 - r_x - t_1 * rv_x = 0
// y1 + t_1 * vy1 - r_y - t_1 * rv_y = 0
// z1 + t_1 * vz1 - r_z - t_1 * rv_z = 0
// x2 + t_2 * vx2 - r_x - t_2 * rv_x = 0
// y2 + t_2 * vy2 - r_y - t_2 * rv_y = 0
// z2 + t_2 * vz2 - r_z - t_2 * rv_z = 0
// x3 + t_3 * vx3 - r_x - t_3 * rv_x = 0
// y3 + t_3 * vy3 - r_y - t_3 * rv_y = 0
// z3 + t_3 * vz3 - r_z - t_3 * rv_z = 0
//
// 19 + t_1 * -2 - r_x - t_1 * rv_x = 0
// 13 + t_1 *  1 - r_y - t_1 * rv_y = 0
// 30 + t_1 * -2 - r_z - t_1 * rv_z = 0
// 18 + t_2 * -1 - r_x - t_2 * rv_x = 0
// 19 + t_2 * -1 - r_y - t_2 * rv_y = 0
// 22 + t_2 * -2 - r_z - t_2 * rv_z = 0
// 20 + t_3 * -2 - r_x - t_3 * rv_x = 0
// 25 + t_3 * -2 - r_y - t_3 * rv_y = 0
// 34 + t_3 * -4 - r_z - t_3 * rv_z = 0
//
// t_1 * -2 - t_1 * rv_x - r_x           = -19
// t_1 *  1 - t_1 * rv_y      - r_y      = -13
// t_1 * -2 - t_1 * rv_z           - r_z = -30
// t_2 * -1 - t_2 * rv_x - r_x           = -18
// t_2 * -1 - t_2 * rv_y      - r_y      = -19
// t_2 * -2 - t_2 * rv_z           - r_z = -22
// t_3 * -2 - t_3 * rv_x - r_x           = -20
// t_3 * -2 - t_3 * rv_y      - r_y      = -25
// t_3 * -4 - t_3 * rv_z           - r_z = -34
// ```
//
// uhhh... two unknowns are multiplied by each other (t{n} and rv..) so we can't
// use system of linear equations solving techniques (Gauss-Jordan elimination)
// directly
//
// we need to do rewriting...
// ```
// x + t_ * vx - r_x - t_ * rv_x = 0
// y + t_ * vy - r_y - t_ * rv_y = 0
// z + t_ * vz - r_z - t_ * rv_z = 0
//
// rv_x in terms of t:
// t * rv_x = X + t * V_X - r_x
// rv_x = X/t + V_X - r_x/t
// ```
// and now we have a new problem: `r_x/t`
//
// this is clearly possible to do but... I think I'd rather play with Z3 than do
// algebra, write/use a gauss-jordan solver, and debug finnicky numerical
// accuracy issues
#[rustfmt::skip]
fn p2(hailstones: &[Hailstone]) -> (Triple<isize>, Triple<isize>) {
    // pos, vel
    // ```
    // x1 + t_1 * vx1 = r_x + t_1 * rv_x
    // y1 + t_1 * vy1 = r_y + t_1 * rv_y
    // z1 + t_1 * vz1 = r_z + t_1 * rv_z
    // x2 + t_2 * vx2 = r_x + t_2 * rv_x
    // y2 + t_2 * vy2 = r_y + t_2 * rv_y
    // z2 + t_2 * vz2 = r_z + t_2 * rv_z
    // x3 + t_3 * vx3 = r_x + t_3 * rv_x
    // y3 + t_3 * vy3 = r_y + t_3 * rv_y
    // z3 + t_3 * vz3 = r_z + t_3 * rv_z
    // ```
    //
    // Unknowns:
    //   - t_1, t_2, t_3
    //   - r_x, r_y, r_z
    //   - rv_x, rv_y, rv_z
    /*
    #[rustfmt::skip]
    let [
        Hailstone { position: Triple(x1, y1, z1), velocity: Triple(vx1, vy1, vz1) },
        Hailstone { position: Triple(x2, y2, z2), velocity: Triple(vx2, vy2, vz2) },
        Hailstone { position: Triple(x3, y3, z3), velocity: Triple(vx3, vy3, vz3) },
    ] = hailstones[0..3] else { unreachable!() };
    let [t1, t2, t3] = ["t1", "t2", "t3"].map(|n| Int::new_const(&ctx, n));
    */

    let ctx = Context::new(&Config::new());
    let solver = Solver::new(&ctx);

    let [rx, ry, rz] = ["rx", "ry", "rz"].map(|n| Int::new_const(&ctx, n));
    let [rv_x, rv_y, rv_z] = ["rv_x", "rv_y", "rv_z"].map(|n| Int::new_const(&ctx, n));

    // #[rustfmt::skip]
    for (n, &Hailstone {
        position: Triple(x, y, z),
        velocity: Triple(v_x, v_y, v_z),
    }) in
        // doesn't actually matter what you put here as long as its at least 3
        // (over-constrained)
        //
        // interestingly, solving seems to go way faster with more inputs?
        hailstones[0..(30.min(hailstones.len()))].iter().enumerate()
    {
        let n = n + 1;
        let t = Int::new_const(&ctx, format!("t_{n}"));

        for (vel, pos, r_vel, r_pos) in [
            (v_x, x, &rv_x, &rx),
            (v_y, y, &rv_y, &ry),
            (v_z, z, &rv_z, &rz)
        ] {
            let lhs = &t * (vel as i64) + (pos as i64);
            let rhs = &t * r_vel + r_pos;

            let eq = lhs._safe_eq(&rhs).unwrap();
            solver.assert(&eq);
        }
    }

    let res = solver.check();
    if let SatResult::Unknown = res {
        eprintln!("unknown res: {:?}", solver.get_reason_unknown());
    }

    assert_eq!(res, SatResult::Sat);
    let model = solver.get_model().unwrap();

    eprintln!("stats:");
    for stat in solver.get_statistics().entries() {
        eprintln!("  - {}: {:?}", stat.key, stat.value);
    }

    let resolve_triple = |a: Int<'_>, b, c| {
        let [a, b, c] = [a, b, c]
            .map(|v| model.get_const_interp(&v).unwrap())
            .map(|v| v.as_i64().unwrap())
            .map(|i| i.try_into().unwrap());

        Triple(a, b, c)
    };

    let pos = resolve_triple(rx, ry, rz);
    let vel = resolve_triple(rv_x, rv_y, rv_z);

    (pos, vel)
}

fn main() {
    let mut aoc = AdventOfCode::new(2023, 24);
    let inp = aoc.get_input();
    // let inp = INP;
    let hailstones = inp.lines().map_parse::<Hailstone>().collect_vec();

    // let p1 = p1(&hailstones, 7., 27.);
    let p1 = p1(&hailstones, 200_000_000_000_000., 400_000_000_000_000.);
    _ = aoc.submit_p1(p1);

    let p2 = {
        let (pos, _) = p2(&hailstones);
        let Triple(x, y, z) = pos;
        x + y + z
    };
    _ = aoc.submit_p2(p2);
}

// stuff I don't understand:
// https://en.wikipedia.org/wiki/Gr%C3%B6bner_basis
