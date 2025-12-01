use std::{
    borrow::Cow,
    cmp::{Eq, Ordering, Reverse},
    collections::{BTreeSet, HashMap, HashSet},
    hash::Hash,
    mem,
};

use aoc::*;
use typed_arena::Arena;

type WireId = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Connection<Of = WireId> {
    Immediate(bool),
    And(Of, Of),
    Xor(Of, Of),
    Or(Of, Of),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Circuit {
    ids: Vec<String>,
    wires: Vec<Connection>,
    name_to_id: HashMap<String, WireId>,
}

impl FromStr for Circuit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Connection::*;

        let (initial_vals, gates) = s.split_once("\n\n").unwrap();
        let initial_vals = initial_vals
            .lines()
            .map(|s| s.split_once(": ").unwrap())
            .map(|(n, v)| {
                (
                    n,
                    match v {
                        "1" => true,
                        "0" => false,
                        other => panic!("{other}"),
                    },
                )
            });
        let gates = gates.lines().map(|s| {
            let (gate, out) = s.split_once(" -> ").unwrap();
            let (lhs, op, rhs) = gate.split(' ').collect_tuple().unwrap();

            (lhs, op, rhs, out)
        });

        let wire_names = initial_vals
            .clone()
            .map(|(n, _)| n)
            .chain(gates.clone().flat_map(|(l, _, r, o)| [l, r, o]));
        let mut ids = vec![];
        let mut name_to_id = HashMap::new();
        for w in wire_names {
            if !name_to_id.contains_key(w) {
                let idx = ids.len() as u16;
                ids.push(w.to_string());
                name_to_id.insert(w.to_string(), idx);
            }
        }

        let mut wires = vec![None; ids.len()];
        for (n, v) in initial_vals {
            let id = name_to_id[n] as usize;
            debug_assert_eq!(wires[id], None);
            wires[id] = Some(Immediate(v));
        }
        for (lhs, op, rhs, out) in gates {
            let [lhs, rhs, out] = [lhs, rhs, out].map(|n| name_to_id[n]);
            let op = match op {
                "AND" => And,
                "OR" => Or,
                "XOR" => Xor,
                other => panic!("unknown op: {other}"),
            };

            debug_assert_eq!(wires[out as usize], None);
            wires[out as usize] = Some(op(lhs, rhs));
        }

        Ok(Self { ids, name_to_id, wires: wires.into_iter().collect::<Option<_>>().unwrap() })
    }
}

impl Connection {
    fn with_names<'c>(self, circuit: &'c Circuit) -> Connection<&'c str> {
        use Connection::*;
        let conv = |l: WireId, r: WireId, func: fn(&'c str, &'c str) -> Connection<&'c str>| {
            func(&circuit.ids[l as usize], &circuit.ids[r as usize])
        };

        match self {
            Immediate(v) => Immediate(v),
            And(l, r) => conv(l, r, And),
            Xor(l, r) => conv(l, r, Xor),
            Or(l, r) => conv(l, r, Or),
        }
    }
}

impl Circuit {
    pub fn resolve(&mut self, wire: &str) -> bool {
        self.resolve_by_id(self.name_to_id[wire])
    }

    fn resolve_by_id(&mut self, id: WireId) -> bool {
        use Connection::*;
        let val = match self.wires[id as usize] {
            Immediate(v) => v,
            And(a, b) => self.resolve_by_id(a) & self.resolve_by_id(b),
            Or(a, b) => self.resolve_by_id(a) | self.resolve_by_id(b),
            Xor(a, b) => self.resolve_by_id(a) ^ self.resolve_by_id(b),
        };
        if self.wires[id as usize] != Immediate(val) {
            self.wires[id as usize] = Immediate(val);
            #[cfg(debug_assertions)]
            #[cfg(any())]
            eprintln!("{} = {}", self.ids[id as usize], if val { 1 } else { 0 });
        }
        val
    }

    pub fn z_value(&mut self) -> usize {
        let mut z_ids = self
            .ids
            .iter()
            .enumerate()
            .filter(|(_id, n)| n.starts_with("z"))
            .collect_vec();
        z_ids.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
        let z_ids = z_ids.into_iter().map(|(id, _)| id).collect_vec();

        let mut out = 0;
        for z_id in z_ids {
            out = (out << 1) | self.resolve_by_id(z_id as _) as usize;
        }

        out
    }

    pub fn calc(&self, x: usize, y: usize) -> usize {
        let mut this = self.clone();
        for (id, name) in self.ids.iter().enumerate() {
            let (pos, num) = if let Some(num) = name.strip_prefix("x") {
                (num, x)
            } else if let Some(num) = name.strip_prefix("y") {
                (num, y)
            } else {
                continue;
            };
            let pos = pos.parse::<u32>().unwrap();
            this.wires[id] = Connection::Immediate((num & (1 << pos)) != 0);
        }

        this.z_value()
    }

    pub fn swap(&mut self, a: &str, b: &str) {
        let (a, b) = (self.name_to_id[a], self.name_to_id[b]);
        self.wires.swap(a as _, b as _);
    }
}

impl fmt::Display for Circuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Connection::*;
        const GROUP_Z: bool = true;

        if f.alternate() {
            // graphviz
            let mut x_subgraph = HashMap::new();
            let mut y_subgraph = HashMap::new();
            let mut z_subgraph = HashMap::new();
            let mut rest = HashMap::new();
            let mut edges = vec![];

            for (out, conn) in self.wires.iter().enumerate() {
                let out = &self.ids[out];
                let slot = match out {
                    x if x.starts_with("x") => &mut x_subgraph,
                    y if y.starts_with("y") => &mut y_subgraph,
                    z if z.starts_with("z") => &mut z_subgraph,
                    _ => &mut rest,
                };

                let conn = conn.with_names(self);
                let tooltip = format!("{conn:?}").replace("\"", "\\\""); // todo: add symbolic to tooltip?
                let label = out;
                let (shape, color) = match conn {
                    Immediate(_) => ("box", "grey"),
                    And(_, _) => ("invtrapezium", "blue"),
                    Xor(_, _) => ("invhouse", "red"),
                    Or(_, _) => ("invtriangle", "green"),
                };

                match conn {
                    Immediate(_) => {}
                    And(l, r) | Xor(l, r) | Or(l, r) => {
                        edges.push((l, out));
                        edges.push((r, out));
                    }
                }

                slot.insert(out, format!(
                    "{out} [label=\"{label}\" shape=\"{shape}\" tooltip=\"{tooltip}\" color={color}]"
                ));
            }

            writeln!(f, "digraph {{")?;

            for (n, cluster) in [("x", x_subgraph), ("y", y_subgraph), ("z", z_subgraph)] {
                let mut sorted = cluster.into_iter().collect_vec();
                sorted.sort_by_key(|(n, _str)| n.as_str());
                sorted.reverse();

                if n == "x" || n == "y" || (GROUP_Z && n == "z") {
                    // lower as a struct:
                    let mut label = format!("[{n}]");
                    for (out, _) in sorted {
                        let num = out.strip_prefix(n).unwrap();
                        label += &format!("| <{num}> {num}");
                    }

                    writeln!(f, "    {n} [label=\"{label}\" shape=record color=grey];")?;
                } else {
                    writeln!(f, "    subgraph cluster_{n} {{")?;
                    for (_, s) in sorted {
                        writeln!(f, "        {s};")?;
                    }
                    writeln!(f, "    }}")?;
                }
            }

            for (_, s) in rest {
                writeln!(f, "    {s};")?;
            }
            writeln!(f, "\n\n")?;

            let mut x_edges = vec![];
            let mut y_edges = vec![];
            let mut z_dst_edges = vec![];
            let mut other_edges = vec![];
            edges.into_iter().for_each(|_edge @ (s, d)| {
                let s = s.replace("x", "x:").replace("y", "y:");
                let d = if GROUP_Z { d.replace("z", "z:") } else { d.to_string() };

                let list = if d.starts_with("z") {
                    &mut z_dst_edges
                } else if s.starts_with("x") {
                    &mut x_edges
                } else if s.starts_with("y") {
                    &mut y_edges
                } else {
                    &mut other_edges
                };
                list.push((s, d));
            });
            x_edges.sort();
            x_edges.reverse();
            y_edges.sort();
            y_edges.reverse();
            z_dst_edges.sort_by_key(|(s, d)| (Reverse(d.to_string()), s.to_string()));
            // other_edges.sort(); // todo

            for (a, b) in x_edges
                .into_iter()
                .chain(y_edges)
                .chain(other_edges)
                .chain(z_dst_edges)
            {
                writeln!(f, "    {a} -> {b};")?;
            }

            writeln!(f, "}}")?;
            Ok(())
        } else {
            unimplemented!()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Symbolic<'s, Of: Clone = String> {
    Input(Of),
    And(&'s Self, &'s Self),
    Or(&'s Self, &'s Self),
    Xor(&'s Self, &'s Self),
}
impl fmt::Display for Symbolic<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Symbolic::*;
        let (l, op, r) = match self {
            Input(inp) => return f.write_str(inp),
            And(l, r) => (l, "&", r),
            Or(l, r) => (l, "|", r),
            Xor(l, r) => (l, "^", r),
        };

        write!(f, "({l} {op} {r})")
    }
}
impl<Of: Hash + Ord + Eq + Clone> Symbolic<'_, Of> {
    fn inputs(&self) -> BTreeSet<Of> {
        let mut out = BTreeSet::new();

        // note: not bothering to cache already visited...
        fn visitor<Of: Hash + Ord + Eq + Clone>(sym: &Symbolic<'_, Of>, record: &mut BTreeSet<Of>) {
            use Symbolic::*;
            match sym {
                Input(inp) => _ = record.insert(inp.clone()),
                And(l, r) | Or(l, r) | Xor(l, r) => {
                    visitor(l, record);
                    visitor(r, record);
                }
            }
        }
        visitor(self, &mut out);
        out
    }

    fn depth(&self) -> usize {
        use Symbolic::*;
        match self {
            Input(_) => 0,
            And(l, r) | Or(l, r) | Xor(l, r) => l.depth().max(r.depth()) + 1,
        }
    }
}

impl Circuit {
    fn with_symbolics<R>(&self, func: impl FnOnce(Vec<&Symbolic<'_>>) -> R) -> R {
        let symbolics_arena = Arena::new();
        let mut symbolics = vec![None; self.wires.len()];

        fn symbolic_for_id<'a>(
            symbolics: &mut Vec<Option<&'a Symbolic<'a>>>,
            arena: &'a Arena<Symbolic<'a>>,
            connections: &Vec<Connection>,
            id_to_name: &Vec<String>,
            id: WireId,
        ) -> &'a Symbolic<'a> {
            let id = id as usize;
            if let Some(sym) = symbolics[id] {
                return sym;
            }

            use {Connection::*, Symbolic as S};
            macro_rules! s {
                ($id:ident) => {
                    symbolic_for_id(symbolics, arena, connections, id_to_name, $id)
                };
            }
            let mut sym = match connections[id] {
                Immediate(_) => S::Input(id_to_name[id].clone()),
                And(l, r) => S::And(s!(l), s!(r)),
                Xor(l, r) => S::Xor(s!(l), s!(r)),
                Or(l, r) => S::Or(s!(l), s!(r)),
            };
            match &mut sym {
                S::Input(_) => {}
                S::And(l, r) | S::Or(l, r) | S::Xor(l, r) => {
                    // normalize:
                    const FLIP: bool = false;
                    let f = |c: Ordering| if FLIP { c.reverse() } else { c };

                    match f(l.depth().cmp(&r.depth())) {
                        Ordering::Greater => mem::swap(l, r),
                        Ordering::Less => {}
                        Ordering::Equal => {
                            if f(l.to_string().cmp(&r.to_string())) == Ordering::Greater {
                                mem::swap(l, r)
                            }
                        }
                    }
                }
            }
            let sym = arena.alloc(sym);
            symbolics[id] = Some(sym);

            sym
        }

        for i in 0..self.wires.len() {
            symbolic_for_id(&mut symbolics, &symbolics_arena, &self.wires, &self.ids, i as WireId);
        }

        let symbolics = symbolics.into_iter().map(Option::unwrap).collect_vec();
        func(symbolics)
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2024, 24);
    let mut circuit = Circuit::from_str(&aoc.get_input()).unwrap();

    // // graphviz output:
    // println!("{circuit:#}");

    _ = aoc.submit_p1(circuit.clone().z_value());

    let p2 = {
        let swaps = [("z06", "dhg"), ("brk", "dpd"), ("z23", "bhd"), ("z38", "nbf")];

        for (s1, s2) in &swaps {
            circuit.swap(s1, s2);
        }
        // graphviz output:
        println!("{circuit:#}");

        let nodes_with_prefix = |prefix| {
            let mut nodes = circuit
                .ids
                .iter()
                .enumerate()
                .filter(|(_, n)| n.starts_with(prefix))
                .collect_vec();
            nodes.sort_by_key(|(_, n)| n.as_str());
            nodes
        };
        let [xs, ys, zs] = ["x", "y", "z"].map(nodes_with_prefix);

        circuit.with_symbolics(|syms| {
            // matrix output showing sensitivity:
            let p = |inps: BTreeSet<_>| {
                eprint!("  ");
                for (_, x_name) in &xs {
                    if inps.contains(x_name.as_str()) {
                        eprint!(" X ")
                    } else {
                        eprint!("   ")
                    }
                }
                eprint!(" |  ");
                for (_, y_name) in &ys {
                    if inps.contains(y_name.as_str()) {
                        eprint!(" X ")
                    } else {
                        eprint!("   ")
                    }
                }
                eprintln!();
            };

            eprint!("  X:");
            for (_, x_name) in &xs {
                eprint!(" {}", x_name.strip_prefix("x").unwrap());
            }
            eprint!(" |Y:");
            for (_, y_name) in &ys {
                eprint!(" {}", y_name.strip_prefix("y").unwrap());
            }
            eprintln!();

            for (id, name) in zs.iter().copied() {
                let s = syms[id];
                eprint!("{name}");
                p(s.inputs());
            }

            ////////////////////

            // flattened equation output
            // note: with normalization, it's very apparent where the mismatches
            // are...
            eprintln!();
            for (id, name) in zs.iter().copied() {
                let s = syms[id];
                eprintln!("{name} = {s}");
            }
        });

        let mut swapped_wires = swaps
            .iter()
            .flat_map(|(a, b)| [a, b])
            .copied()
            .collect_vec();
        swapped_wires.sort();
        swapped_wires.join(",")
    };
    _ = aoc.submit_p2(p2);
}

// done: generate symbolic
// done: check sensitivity list
//  - as a heuristic, see which outputs are oversensitive/undersensitive

// todo: use z3 or similar to ask whether we can prove that an output is
// correct? (symbolic form)
//  - this gets into, like, real stuff — formal verification w/jasper, ACL2, etc
// todo: quickcheck-style random testing + shrinking? fuzzing?
// todo: do some light binary op transforms to sus out equivalence?
//  - assumption is that the puzzle input generator isn't doing anything wild..

// 45-bit adder → 2^46 vals to test; technically small enough to brute force..
//  - ... but not if you factor in all the possible swaps (`x (220 ^ 4)`)
// todo: what kind? kogge-stone? ripple carry? carry-lookahead?

// ripple carry adder? just find all the outputs whose equations don't start
// with an xor as required?
