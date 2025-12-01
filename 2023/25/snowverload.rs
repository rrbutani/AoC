use std::collections::HashMap;

use aoc::{iterator_map_ext::IterMapExt, AdventOfCode, Display, FromStr};
use smallvec::SmallVec;
use smol_str::SmolStr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Component {
    name: SmolStr,
    connected: SmallVec<[SmolStr; 5]>,
}

impl FromStr for Component {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, connected) = s.split_once(": ").unwrap();
        Ok(Self {
            name: name.into(),
            connected: connected.split_whitespace().map(Into::into).collect(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Components {
    map: HashMap<SmolStr, Component>,
}

impl Display for Components {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            // graphviz!
            writeln!(f, "graph {{")?;

            for c in self.map.values() {
                writeln!(f, "  {};", c.name)?;
                for n in &c.connected {
                    writeln!(f, "  {} -- {};", c.name, n)?;
                }
            }

            writeln!(f, "}}")
        } else {
            unimplemented!()
        }
    }
}

impl FromStr for Components {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let it = s
            .lines()
            .map_parse::<Component>()
            .map(|c| (c.name.clone(), c));

        Ok(Self { map: it.collect() })
    }
}

const INP: &str = "jqt: rhn xhk nvd
rsh: frs pzl lsr
xhk: hfx
cmg: qnr nvd lhk bvb
rhn: xhk bvb hfx
bvb: xhk hfx
pzl: lsr hfx nvd
qnr: nvd
ntq: jqt hfx bvb xhk
nvd: lhk
lsr: lhk
rzs: qnr cmg lsr rsh
frs: qnr lhk lsr";

fn main() {
    let mut aoc = AdventOfCode::new(2023, 25);
    let inp = aoc.get_input();
    let inp = INP;
    let map: Components = inp.parse().unwrap();

    println!("{map:#}");
}
