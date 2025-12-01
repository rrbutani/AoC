use aoc::*;
use grid::TwoDimensionalGrid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Schematic {
    // required pin heights (gaps)
    Lock(SmallVec<[u8; 5]>),
    // protuding pin heights (teeth)
    Key(SmallVec<[u8; 5]>),
}

impl FromStr for Schematic {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::Display)]
        enum Cell {
            #[strum(to_string = "#")]
            Filled,
            #[strum(to_string = ".")]
            Empty,
        }
        let grid = TwoDimensionalGrid::<Cell>::from_str(s).unwrap();

        let all = |row: &Vec<_>, val| row.iter().all(|&c| c == val);
        let top = |v| all(grid.first().unwrap(), v);
        let bot = |v| all(grid.last().unwrap(), v);

        let (kind, look_for): (fn(_) -> Schematic, _) = if top(Cell::Filled) && bot(Cell::Empty) {
            (Schematic::Lock, Cell::Empty)
        } else if top(Cell::Empty) && bot(Cell::Filled) {
            (Schematic::Key, Cell::Filled)
        } else {
            panic!("not a key or a lock:\n{grid}");
        };

        let mut measurements = SmallVec::new();
        for col in 0..grid.width() {
            let count = (0..grid.height())
                .filter(|&row| grid[Coord { row, col }] == look_for)
                .count() as u8;
            measurements.push(count);
        }

        Ok(kind(measurements))
    }
}

impl Schematic {
    fn key_matches(&self, key: &Self) -> bool {
        use Schematic::*;
        match (self, key) {
            (Lock(lock), Key(key)) => {
                if lock.len() != key.len() {
                    return false;
                }

                lock.iter().zip(key.iter()).all(|(gap, tooth)| gap >= tooth)
            }
            _ => panic!(),
        }
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2024, 25);
    let inp = aoc.get_input();

    let (mut locks, mut keys) = (vec![], vec![]);
    inp.split("\n\n")
        .map_parse::<Schematic>()
        .for_each(|s| match s {
            Schematic::Lock(_) => locks.push(s),
            Schematic::Key(_) => keys.push(s),
        });

    let p1 = locks
        .iter()
        .cartesian_product(keys.iter())
        .filter(|(l, k)| l.key_matches(k))
        .count();
    _ = aoc.submit_p1(p1);

    _ = aoc.submit_p2(2024);
}
