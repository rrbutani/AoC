use aoc::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ButtonBehavior {
    a: Coord,
    b: Coord,
    prize: Coord,
}

#[rustfmt::skip]
impl FromStr for ButtonBehavior {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.split("\n");
        let mut coord = |pfx, delim| {
            let (x, y) = lines.next()?.strip_prefix(pfx)?.split_once(", ")?;
            let [x, y]: [Option<usize>; 2] = [("X", x), ("Y", y)].map(|(p, s)| {
                s.strip_prefix(p)?.strip_prefix(delim)?.parse().ok()
            });
            Some((x?, y?).into())
        };
        let a = coord("Button A: ", "+").ok_or(())?;
        let b = coord("Button B: ", "+").ok_or(())?;
        let prize = coord("Prize: ", "=").ok_or(())?;
        assert_eq!(lines.next(), None);
        Ok(ButtonBehavior { a, b, prize })
    }
}

impl ButtonBehavior {
    /// system of equations:
    /// ```python
    /// a * ax + b * bx = px
    /// a * ay + b * by = py
    /// ```
    /// ```python
    /// a * 3 + b * 1 = tokens
    /// ```
    ///
    /// unless a is a multiple of b or b is a multiple of a, there's only 1
    /// possible answer?
    ///
    /// ```python
    /// a * ay + b * by = py
    /// (py - b * by) = a * ay
    /// (py - b * by) / ay = a
    ///
    /// a * ax + b * bx = px
    /// ((py - b * by) / ay) * ax + b * bx = px
    /// ((py - b * by) / ay) * ax + b * bx = px
    /// ((py / ay) - ((b * by) / ay)) * ax + b * bx = px
    /// ((py * ax) / ay) - (b * ax * by / ay) + b * bx = px
    /// ((py * ax) / ay) - px = ((ax * by / ay) - bx) * b
    /// (((py * ax) / ay) - px) / ((ax * by / ay) - bx) = b
    /// ((py * ax - px * ay) / ay) / ((ax * by - bx * ay / ay)) = b
    /// (py * ax - px * ay) / (ax * by - bx * ay) = b
    /// ```
    ///
    /// ```python
    /// a * ax + b * bx = px
    /// a = (px - b * bx) / ax
    /// ```
    pub fn solve(&self) -> Option<(usize, usize)> {
        let Coord { row: ax, col: ay } = self.a.as_signed();
        let Coord { row: bx, col: by } = self.b.as_signed();
        let Coord { row: px, col: py } = self.prize.as_signed();

        let top = py * ax - px * ay;
        let bot = ax * by - bx * ay;
        if bot == 0 {
            // uhhhh
            // a is a multiple of b or vice versa?
            //
            // need to: take the one that's cheaper, in tokens, and use it
            // exclusively
            //
            // hopefully this doesn't actually come up in the input...
            unimplemented!();
        }

        if top % bot != 0 {
            return None; // no solution
        }
        let b = top / bot;

        let top_a = px - b * bx;
        if top_a % ax != 0 {
            return None; // not sure if this comes up? update: yes (part 2)
        }
        let a = top_a / ax;

        debug_assert!(a >= 0 && b >= 0);
        Some((a as usize, b as usize))
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2024, 13);
    let inp = aoc.get_input();
    let bbs: Vec<ButtonBehavior> = inp.split("\n\n").map_parse().collect();

    let sum = |offs: (usize, usize)| {
        let o: Coord = offs.into();
        bbs.iter()
            .filter_map(|&b| ButtonBehavior { prize: b.prize + o, ..b }.solve())
            .map(|(a, b)| a * 3 + b)
            .sum::<usize>()
    };

    _ = aoc.submit_p1(sum((0, 0)));
    _ = aoc.submit_p2(sum((10usize.pow(13), 10usize.pow(13))));
}
