use std::{
    collections::{HashMap, VecDeque},
    sync::atomic::{AtomicBool, Ordering},
};

use aoc::{AdventOfCode, Display, FromStr, Itertools};
use smallvec::SmallVec;
use smol_str::SmolStr;

// #[repr(bool)]
// enum Pulse {
//     On,
//     Off,
// }

#[derive(Debug, Clone, PartialEq, Eq)]
enum Module {
    FlipFlop { last: bool }, // default off
    // i.e. Not And
    Conjunction { prevs: HashMap<SmolStr, bool> },
    Callback(fn(&SmolStr, bool)),
}

impl Module {
    fn update(&mut self, from: &SmolStr, pulse: bool) -> Option<bool> {
        use Module::*;
        match self {
            FlipFlop { last } => {
                if pulse {
                    None
                } else {
                    *last = !*last;
                    Some(*last)
                }
            }
            Conjunction { prevs } => {
                *prevs.get_mut(from).unwrap() = pulse;
                Some(!prevs.values().all(|&x| x))
            }
            Callback(func) => {
                func(from, pulse);
                None
            }
        }
    }
}

type ModuleList<const N: usize = 2> = SmallVec<[SmolStr; N]>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Configuration {
    inp: ModuleList<8>,
    map: HashMap<SmolStr, (Module, ModuleList<2>)>,
    // next_button_press: bool,
}

impl FromStr for Configuration {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut inp = None;
        let mut map = HashMap::with_capacity(80);
        for (module, nexts) in s.lines().map(|l| l.split_once(" -> ").unwrap()) {
            let nexts = nexts
                .split(", ")
                .filter(|n| !n.is_empty())
                .map(SmolStr::new);
            let (name, module) = if let Some(name) = module.strip_prefix('%') {
                (name, Module::FlipFlop { last: false })
            } else if let Some(name) = module.strip_prefix('&') {
                (name, Module::Conjunction { prevs: HashMap::new() })
            } else if module == "broadcaster" {
                if inp.is_some() {
                    panic!("multiple broadcasters!")
                } else {
                    inp = Some(nexts.collect());
                }
                continue;
            } else {
                panic!("invalid input: {module}");
            };

            let nexts = nexts.collect::<SmallVec<_>>();
            map.insert(SmolStr::new(name), (module, nexts));
        }

        // register prev modules for conjunctions:
        //
        // note: falls over if the broadcaster feed a conjunction but... this
        // doesn't seem to happen in the given input
        let mut conjunction_prevs = map
            .iter()
            .filter(|(_, (m, _))| matches!(m, Module::Conjunction { .. }))
            .map(|(k, _)| (k.clone(), HashMap::new()))
            .collect::<HashMap<_, _>>();
        for (m, (_, nexts)) in &map {
            for next in nexts {
                if let Some(prev_map) = conjunction_prevs.get_mut(next) {
                    prev_map.insert(m.clone(), false);
                }
            }
        }
        for (m, prevs) in conjunction_prevs {
            match map.get_mut(&m).unwrap().0 {
                Module::Conjunction { prevs: ref mut old_prevs } => {
                    *old_prevs = prevs;
                }
                _ => unreachable!(),
            }
        }

        Ok(Self { inp: inp.unwrap(), map /* next_button_press: false */ })
    }
}

const BROADCASTER: SmolStr = SmolStr::new_inline("broadcaster");

impl Configuration {
    // returns pulse counts: low, high
    fn pulse(&mut self) -> (usize, usize) {
        // let initial_pulse = self.next_button_press;
        // self.next_button_press = !self.next_button_press;
        let initial_pulse = false;

        let (mut low_count, mut high_count) =
            ((initial_pulse == false) as _, (initial_pulse == true) as _);

        let mut queue = VecDeque::with_capacity(self.map.len());
        queue.extend(
            self.inp
                .iter()
                .map(|m| (BROADCASTER, m.clone(), initial_pulse)),
        );

        while let Some((src_mod, curr_mod, pulse)) = queue.pop_front() {
            let count = if pulse { &mut high_count } else { &mut low_count };
            *count += 1;

            if DEBUG {
                eprintln!(
                    "{src_mod:>13} {val} —→ {curr_mod}",
                    val = if pulse {
                        "\u{001b}[33mhi\u{001b}[0m"
                    } else {
                        "\u{001b}[35mlo\u{001b}[0m"
                    }
                );
            }
            let Some((module, nexts)) = self.map.get_mut(&curr_mod) else {
                if DEBUG {
                    eprintln!("warning: missing next module {curr_mod}; referenced by {src_mod}");
                }
                continue;
            };
            if let Some(output) = module.update(&src_mod, pulse) {
                for dst in nexts {
                    queue.push_back((curr_mod.clone(), dst.clone(), output));
                }
            }
        }

        (low_count, high_count)
    }
}

impl Configuration {
    fn as_dot(&self) -> impl Display + '_ {
        struct DotFormatter<'c>(&'c Configuration);
        impl Display for DotFormatter<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                writeln!(f, "digraph {{")?;

                writeln!(f, "  broadcaster;")?;
                for (n, (m, _)) in &self.0.map {
                    writeln!(f, "  {n} [")?;

                    match m {
                        Module::FlipFlop { .. } => {
                            writeln!(f, "    color=\"#0c9c78\"")?;
                            writeln!(f, "    shape=\"box\"")?;
                        }
                        Module::Conjunction { .. } => {
                            writeln!(f, "    color=\"#f00000\"")?;
                            writeln!(f, "    shape=\"box\"")?;
                        }
                        _ => {}
                    }

                    writeln!(f, "  ];")?;
                }

                writeln!(f, "\n\n")?;
                for inp in &self.0.inp {
                    writeln!(f, "  broadcaster -> {inp};")?;
                }
                for (n, (_, next)) in &self.0.map {
                    for next in next {
                        writeln!(f, "  {n} -> {next};")?;
                    }
                }

                writeln!(f, "}}")
            }
        }

        DotFormatter(self)
    }
}

// -----------------------------------------------------------------------------

const DEBUG: bool = false;

const INP1: &str = "broadcaster -> a, b, c
%a -> b
%b -> c
%c -> inv
&inv -> a";

const INP2: &str = "broadcaster -> a
%a -> inv, con
&inv -> b
%b -> con
&con -> output
%output -> \n";

fn main() {
    let mut aoc = AdventOfCode::new(2023, 20);
    let inp = aoc.get_input();
    // let inp = INP1;
    // let inp = INP2;
    let conf: Configuration = inp.parse().unwrap();

    let p1: usize = {
        let mut conf = conf.clone();
        let (mut lo, mut hi) = (0, 0);
        for _ in 0..1000 {
            let (l, h) = conf.pulse();
            if DEBUG {
                eprintln!("lo: {l}, hi: {h}\n");
            }
            lo += l;
            hi += h;
        }

        dbg!(lo, hi);
        lo * hi
    };
    _ = aoc.submit_p1(p1);

    let p2: usize = {
        /*
        static GOT_LOW: AtomicBool = AtomicBool::new(false);
        let watcher_module = Module::Callback(|_, p| {
            if !p {
                GOT_LOW.store(true, Ordering::SeqCst);
            }
        });

        conf.map
            .insert(SmolStr::new("rx"), (watcher_module.clone(), SmallVec::new()));

        let mut count = 0;
        while !GOT_LOW.load(Ordering::SeqCst) {
            conf.pulse();
            count += 1;
            if count % 100_000 == 0 {
                eprint!(".");
            }
        }
        count
        */

        if DEBUG {
            eprintln!("{}", conf.as_dot());
        }

        // Staring at the graph for the configuration (see ./config.svg) reveals
        // that the module of interest (rx) is fed by a conjunction (kc) that is
        // fed by four networks, each of which is passed through an intermediary
        // inverter (conjunction with a single input: vn, kt, ph, hn) before
        // arriving at the final conjunction (kc).
        //
        // Each of these networks has a similar structure:
        //   - 12 flip flops chained
        //   - each flip flop EITHER:
        //     + *feeds* into the network's conjunction
        //     + is *fed* by the network's conjunction
        //   - with the exception of the first flip-flip which is both fed &
        //     feeds
        //
        // If the network were "fully connected" (i.e. every flip flop were fed
        // by and were feeding into the conjunction) then this would resemble a
        // *binary counter*. The hypothetical edge from the conjunction back to
        // each flip flop would ensure that the network rolls over to 0 once it
        // maxes out instead of getting stuck at 0b111111111111.
        //
        // However, our input — as a result of having some flip-flops feed and
        // some be fed — is more complex. I _think_ each network describes an
        // LFSR (Linear Feedback Shift Register)? Maybe not.
        //
        // https://en.wikipedia.org/wiki/Linear-feedback_shift_register
        //
        // ------
        //
        // We'll call the flip-flops that feed into the conjunction "taps" and
        // the flip-flips that are fed "resets".
        //
        // The conjunction is triggered when the flip-flop chain counts up to a
        // number for which all the "taps" are `1`. Until this happens the
        // conjunction yields `1` (which flip-flops) ignore meaning that it has
        // no effect on the reset flip-flops.
        //
        // Once the conjunction is triggered, all the reset flip-flops are set
        // to `1` (toggled). These produce no updates of their own (all the
        // output 1s for the reset nodes are fed to flip-flops which ignore 1s)
        // so the state is now: all 1s. The conjunction's output is still 0.
        //
        // The next button input propagates down the flip-flop network resetting
        // everything to 0, restarting the cycle.
        //
        // ... or rather: this _would_ be true if not for the first flip-flop
        // being both a tap and a reset.
        //
        // How about we just simulate:

        const T: bool = true; // so we get a different syntax highlighting color
        let f = false;
        let mf = [T, f, T, f, T, f, T, T, f, T, T, T];

        fn find_cycle_counts_for_taps<const N: usize, const R: usize>(taps: [bool; N]) -> [usize; R]
        where
            [usize; R]: smallvec::Array<Item = usize>,
        {
            let mut resets = taps.clone().map(|x| !x);
            resets[0] = T;

            let idx_to_string = |i| String::from(((i as u8) + b'a') as char);
            let mut inp = vec![String::from("broadcaster -> a")];
            for (i, &t) in taps.iter().enumerate() {
                let name = idx_to_string(i);
                let next_flop = if i < taps.len() {
                    format!("{}, ", idx_to_string(i + 1))
                } else {
                    String::new()
                };
                let to_con = if t { "con, " } else { "" };
                inp.push(format!("%{} -> {}{}", name, next_flop, to_con));
            }
            let con_nexts = resets
                .iter()
                .enumerate()
                .filter(|(_, r)| **r)
                .map(|(i, _)| idx_to_string(i))
                .join(", ");
            inp.push(format!("&con -> output, {}", con_nexts));

            let inp = inp.join("\n");
            let mut conf = Configuration::from_str(&inp).unwrap();

            static GOT_LOW: AtomicBool = AtomicBool::new(false);
            let watcher_module = Module::Callback(|_, p| {
                if !p {
                    GOT_LOW.store(true, Ordering::SeqCst);
                }
            });

            conf.map
                .insert(SmolStr::from("output"), (watcher_module, SmallVec::new()));

            if DEBUG {
                eprintln!("{}", conf.as_dot());
            }

            let mut count = 0usize;
            let mut low_cycles = SmallVec::<[usize; R]>::new();
            loop {
                if GOT_LOW.load(Ordering::SeqCst) {
                    if DEBUG {
                        eprintln!("low at cycle {count}!");
                    }
                    low_cycles.push(count);
                    GOT_LOW.store(false, Ordering::SeqCst);
                    if low_cycles.len() == R {
                        break low_cycles.into_inner().unwrap();
                    }
                }

                count += 1;
                conf.pulse();
            }
        }

        let cyc = find_cycle_counts_for_taps::<12, 3>(mf);
        if DEBUG {
            eprintln!("{}, {}, {}", cyc[0], cyc[1] - cyc[0], cyc[2] - cyc[1]);
        }
        assert_eq!(cyc[0], 3797);
        assert_eq!(cyc[1] - cyc[0], 3797);
        assert_eq!(cyc[2] - cyc[1], 3797);

        // Yeah okay, the extra edge from the first node to the conjunction
        // doesn't seem to break anything...
        //
        // yields: 0b111011010101 (3797)
        // tps.rev:  TTTfTTfTfTfT (matches)

        // To find the taps for each network programmatically:
        let taps = {
            let mut taps = Vec::with_capacity(conf.inp.len());
            for i in &conf.inp {
                let mut conjunction = None;
                for next_of_start in &conf.map[i].1 {
                    if matches!(conf.map[next_of_start].0, Module::Conjunction { .. }) {
                        conjunction = Some(next_of_start);
                        break;
                    }
                }

                // follow each input's next nodes, recording which nodes are
                // connected to the conjunction
                //
                // stop once we hit a node that has only the conjunction as its
                // next node
                let mut curr = i;
                let mut bits = Vec::with_capacity(12);
                loop {
                    let mut next = None;
                    let mut is_tap = false;

                    if !matches!(conf.map[curr].0, Module::FlipFlop { .. }) {
                        panic!("expected flip flop at `{curr}");
                    }
                    for next_of_curr in &conf.map[curr].1 {
                        match &conf.map[next_of_curr].0 {
                            Module::FlipFlop { .. } => {
                                assert!(next.is_none());
                                next = Some(next_of_curr);
                            }
                            Module::Conjunction { .. } => {
                                assert_eq!(conjunction.unwrap(), next_of_curr);
                                is_tap = true;
                            }
                            Module::Callback(_) => unreachable!(),
                        }
                    }

                    bits.push(is_tap);

                    if let Some(next) = next {
                        curr = next
                    } else {
                        break;
                    }
                }
                taps.push(bits);
            }

            taps
        };

        let cycle_counts = taps.iter().map(|bits| {
            bits.iter()
                .enumerate()
                .map(|(i, &bit)| (bit as usize) << i)
                .sum::<usize>()
        });

        // conveniently these are all prime numbers...
        //
        // even so:
        fn euclid(a: usize, b: usize) -> usize {
            if a == 0 {
                b
            } else {
                euclid(b % a, a)
            }
        }
        fn lcm(a: usize, b: usize) -> usize {
            (a * b) / euclid(a, b)
        }

        cycle_counts.into_iter().fold(1, lcm)
    };
    _ = aoc.submit_p2(p2);
}
