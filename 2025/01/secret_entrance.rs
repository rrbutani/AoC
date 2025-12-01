use aoc::*;
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
enum Rotation {
    Left(u32),
    Right(u32),
}

impl FromStr for Rotation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (dir, num) = s.split_at(1);
        let num = num.parse::<_>().unwrap();
        let dir = match dir {
            "R" => Rotation::Right,
            "L" => Rotation::Left,
            _ => panic!("{s}"),
        };
        Ok(dir(num))
    }
}

impl Rotation {
    // rewritten as rightward rotation
    fn as_clockwise_offs(self) -> u32 {
        match self {
            Rotation::Left(n) => Dial::LEN - (n % Dial::LEN),
            Rotation::Right(n) => n,
        }
    }

    // rewritten so offset is in 0..Dial::LEN; returns extra lap count
    fn normalized(self) -> (Self, usize) {
        use Rotation::*;
        let (Left(offs) | Right(offs)) = self;
        let extra_lap_count = (offs / 100).to();

        let offs = offs % 100;
        let new = match self {
            Left(_) => Left(offs),
            Right(_) => Right(offs),
        };
        (new, extra_lap_count)
    }
}

#[derive(Debug, Clone, Copy)]
struct Dial {
    position: u32, // always in 0..Dial::LEN
}

impl Dial {
    const LEN: u32 = 100;
}

impl Default for Dial {
    fn default() -> Self {
        Self { position: Self::LEN / 2 }
    }
}

impl Dial {
    fn rotate(mut self, rot: Rotation) -> Self {
        self.position = (self.position + rot.as_clockwise_offs()) % Dial::LEN;
        self
    }

    fn rotate_counting_zeros(self, rot: Rotation) -> (Self, usize) {
        let (norm, mut zero_count) = rot.normalized();
        let extra_zero = match norm {
            Rotation::Left(n) => self.position != 0 && n >= self.position, // rotate left past 0
            Rotation::Right(n) => n + self.position >= Dial::LEN,          // right
        };
        zero_count += extra_zero as usize;

        let position = match norm {
            Rotation::Left(n) => (self.position + Dial::LEN)
                .checked_add_signed(-n.to::<i32>())
                .unwrap(),
            Rotation::Right(n) => self.position + n,
        };
        (Dial { position: position % Dial::LEN }, zero_count)
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2025, 1);
    let inp = aoc.get_input();
    let rots = inp.lines().map_parse::<Rotation>().collect_vec();

    let p1 = rots
        .iter()
        .scan(Dial::default(), |dial, &rot| {
            *dial = dial.rotate(rot);
            Some(*dial)
        })
        // .inspect(|dial| eprintln!("{dial:?}"))
        .filter(|&dial| dial.position == 0)
        .count();
    _ = aoc.submit_p1(p1);

    let p2 = rots
        .iter()
        .scan(Dial::default(), |dial, &rot| {
            let count;
            (*dial, count) = dial.rotate_counting_zeros(rot);
            Some(count)
        })
        .sum::<usize>();
    _ = aoc.submit_p2(p2);
}
