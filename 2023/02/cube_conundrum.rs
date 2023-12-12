use aoc::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct Reveal {
    red: usize,
    green: usize,
    blue: usize,
}

impl Reveal {
    fn can_fit(&self, other: Reveal) -> bool {
        self.red >= other.red && self.green >= other.green && self.blue >= other.blue
    }

    fn power(&self) -> usize {
        self.red * self.green * self.blue
    }
}

impl FromStr for Reveal {
    type Err = ();
    // comma separated list of `<n> <color>`
    fn from_str(s: &str) -> Result<Self, ()> {
        let mut out = Reveal::default();
        for s in s.split(", ") {
            let (n, color) = s.split_once(' ').unwrap();
            let n = n.parse().unwrap();
            let count = match color {
                "red" => &mut out.red,
                "blue" => &mut out.blue,
                "green" => &mut out.green,
                x => panic!("invalid color: {x}"),
            };

            *count = n;
        }

        Ok(out)
    }
}

#[derive(Debug, Clone, Hash)]
struct Game {
    id: usize,
    reveals: Vec<Reveal>,
}

impl FromStr for Game {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (game, reveals) = s.split_once(": ").unwrap();
        let id = game.strip_prefix("Game ").unwrap();
        let id = id.parse().unwrap();

        let reveals = reveals.split("; ").map(|r| r.parse().unwrap()).collect();

        Ok(Game { id, reveals })
    }
}

impl Game {
    fn min_required_cube_counts(&self) -> Reveal {
        let min = |func: fn(&Reveal) -> usize| self.reveals.iter().map(func).max().unwrap();

        Reveal {
            red: min(|r| r.red),
            green: min(|r| r.green),
            blue: min(|r| r.blue),
        }
    }
}

const INP: &str = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green
";

fn main() {
    let mut aoc = AdventOfCode::new(2023, 2);
    let inp = aoc.get_input();
    let games = inp.lines().map_parse::<Game>().collect_vec();

    let p1_threshold = Reveal {
        red: 12,
        green: 13,
        blue: 14,
    };
    let p1: usize = games
        .iter()
        .filter(|g| p1_threshold.can_fit(g.min_required_cube_counts()))
        .map(|g| g.id)
        .sum();
    aoc.submit_p1(p1).unwrap();

    let p2: usize = games
        .iter()
        .map(|g| g.min_required_cube_counts().power())
        .sum();
    aoc.submit_p2(p2).unwrap();
}
