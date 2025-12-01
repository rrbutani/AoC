use std::collections::{HashSet, VecDeque};

use aoc::*;
use grid::{Direction, SingleChar, TwoDimensionalGrid};
use rand::Rng;

type Map = TwoDimensionalGrid<SingleChar>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Region {
    char: SingleChar,
    coords: Vec<Coord>,
}

impl Region {
    pub fn area(&self) -> usize {
        self.coords.len()
    }

    fn perimeter_edges(&self, map: &Map) -> HashSet<(Coord, Direction)> {
        let mut perimeter_edges = HashSet::new();

        // gather edges touching the perimeter
        //
        // any edge for any coord in `coords` that's not touching a `self.char`
        // is on the perimeter:
        for &c in &self.coords {
            for d in Direction::ALL {
                if d.apply(c).and_then(|c| map.get(c)) == Some(&self.char) {
                    continue;
                } else {
                    perimeter_edges.insert((c, d));
                }
            }
        }
        perimeter_edges
    }

    pub fn perimeter(&self, map: &Map) -> usize {
        self.perimeter_edges(map).len()
    }

    pub fn num_sides(&self, map: &Map) -> usize {
        let mut perimeter_edges = self.perimeter_edges(map);

        // now form groups out of the edges
        //
        // any two edges are on the same side if they have the same direction
        // and share an axis corresponding to that direction (orthogonal)
        let mut sides = Vec::<Vec<(Coord, Direction)>>::new();
        while let Some(&next) = perimeter_edges.iter().next() {
            let mut side = Vec::new();

            let (coord, dir) = next;
            let diff: Coord<isize> = match dir {
                // must share same column
                Direction::East | Direction::West => (1isize, 0).into(),
                // must share same row
                Direction::North | Direction::South => (0, 1isize).into(),
            };

            let mut queue = VecDeque::from([coord]);
            while let Some(coord) = queue.pop_front() {
                assert!(perimeter_edges.remove(&(coord, dir)));
                side.push((coord, dir));

                // try both directions:
                for d in [-diff, diff] {
                    if let Some(next) = coord.checked_add_signed(d) {
                        if perimeter_edges.contains(&(next, dir)) {
                            queue.push_back(next);
                        }
                    }
                }
            }

            sides.push(side);
        }

        sides.len()
    }
}

fn find_regions(map: &Map) -> Vec<Region> {
    let mut unvisited: HashSet<_> = map.cell_iter().map(|(c, _)| c).collect();
    let mut regions = vec![];

    // pick a cell
    while let Some(&starting_coord) = unvisited.iter().next() {
        let mut coords = vec![];
        let char = map[starting_coord];

        // expanding outwards in cardinal dirs until the cell doesn't match:
        map.floodfill([starting_coord], |ctx, _| {
            if ctx.cell() == &char {
                coords.push(ctx.coord());
                assert!(unvisited.remove(&ctx.coord()));

                Some(SmallVec::from_buf(Direction::ALL.map(|d| (d, ()))))
            } else {
                None
            }
        });

        // all coords that we reached define this region
        regions.push(Region { char, coords });
    }

    regions
}

// just for fun
fn color_print(map: &Map, regions: &[Region]) {
    struct ColoredChar {
        char: char,
        color: owo_colors::Rgb,
    }
    impl Display for ColoredChar {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            use owo_colors::OwoColorize;
            Display::fmt(&self.char.color(self.color), f)
        }
    }
    impl From<SingleChar> for ColoredChar {
        fn from(SingleChar(char): SingleChar) -> Self {
            ColoredChar { char, color: owo_colors::Rgb(0, 0, 0) }
        }
    }

    let mut map = map.map(|_, &s| ColoredChar::from(s));
    let mut rand = rand::thread_rng();
    for Region { coords, .. } in regions {
        let color = owo_colors::Rgb(rand.gen(), rand.gen(), rand.gen());
        for &c in coords {
            map[c].color = color;
        }
    }

    eprintln!("{map}");
}

fn main() {
    let mut aoc = AdventOfCode::new(2024, 12);
    let map = Map::from_str(&aoc.get_input()).unwrap();
    let regions = find_regions(&map);

    let p1: usize = regions.iter().map(|r| r.area() * r.perimeter(&map)).sum();
    _ = aoc.submit_p1(p1);

    let p2: usize = regions.iter().map(|r| r.area() * r.num_sides(&map)).sum();
    _ = aoc.submit_p2(p2);

    color_print(&map, &regions);
}
