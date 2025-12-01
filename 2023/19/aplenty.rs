use std::{
    collections::HashMap,
    // hint::unreachable_unchecked,
    mem,
    // ops::Range,
    sync::Mutex,
    time::Instant,
};

use aoc::{iterator_map_ext::IterMapExt, AdventOfCode, FromStr, Itertools};
use cranelift::{
    codegen::ir::{types, Function, UserExternalName, UserFuncName},
    prelude::{
        AbiParam, Block, FunctionBuilder, FunctionBuilderContext, InstBuilder, IntCC, Value,
    },
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module};
// use iced_x86::{Decoder, DecoderOptions, Formatter, Instruction, IntelFormatter};
use rayon::{
    prelude::{IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSlice,
};
use smallvec::SmallVec;
use smol_str::SmolStr;

// -----------------------------------------------------------------------------
// Parsing:

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Part {
    x: u16,
    m: u16,
    a: u16,
    s: u16,
}

impl FromStr for Part {
    type Err = ();

    fn from_str(inp: &str) -> Result<Self, Self::Err> {
        let inp = inp
            .strip_prefix('{')
            .and_then(|s| s.strip_suffix('}'))
            .unwrap();

        let (mut x, mut m, mut a, mut s) = (None, None, None, None);
        for (k, v) in inp.split(',').map(|kv| kv.split_once('=').unwrap()) {
            match k {
                "x" => x = Some(v.parse().unwrap()),
                "m" => m = Some(v.parse().unwrap()),
                "a" => a = Some(v.parse().unwrap()),
                "s" => s = Some(v.parse().unwrap()),
                _ => panic!("bad key: {k}"),
            }
        }

        Ok(Self {
            x: x.unwrap(),
            m: m.unwrap(),
            a: a.unwrap(),
            s: s.unwrap(),
        })
    }
}

impl Part {
    fn rating_sum(&self) -> u64 {
        self.x as u64 + self.m as u64 + self.a as u64 + self.s as u64
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumString)]
enum Var {
    #[strum(serialize = "x")]
    X = 0,
    #[strum(serialize = "m")]
    M,
    #[strum(serialize = "a")]
    A,
    #[strum(serialize = "s")]
    S,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumString)]
enum BinOp {
    #[strum(serialize = "<")]
    LessThan,
    #[strum(serialize = ">")]
    GreaterThan,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumString)]
enum Target {
    #[strum(serialize = "R")]
    Rejected,
    #[strum(serialize = "A")]
    Accepted,
    #[strum(default)]
    SentToOtherWorkflow(SmolStr),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Cond {
    var: Var,
    op: BinOp,
    rhs: u16,
    target: Target,
}

impl FromStr for Cond {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (var, rest) = s.split_at(1);
        let (op, rest) = rest.split_at(1);
        let (rhs, target) = rest.split_once(':').unwrap();

        Ok(Self {
            var: var.parse().unwrap(),
            op: op.parse().unwrap(),
            rhs: rhs.parse().unwrap(),
            target: target.parse().unwrap(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Workflow {
    name: SmolStr,
    conditions: SmallVec<[Cond; 4]>,
    default: Target,
}

impl FromStr for Workflow {
    type Err = ();

    fn from_str(rest: &str) -> Result<Self, Self::Err> {
        let (name, rest) = rest.split_once('{').unwrap();
        let mut conditions = rest.strip_suffix('}').unwrap().split(',');
        let default = conditions.next_back().unwrap();

        Ok(Self {
            name: name.into(),
            conditions: conditions.map_parse().collect(),
            default: default.parse().unwrap(),
        })
    }
}

// #[derive(Debug, Clone, PartialEq, Eq)]
struct Workflows {
    map: HashMap<SmolStr, Workflow>,
    module: Option<JITModule>,
    func_id: Option<FuncId>,
}

impl FromStr for Workflows {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let map: HashMap<_, _> = s
            .lines()
            .map_parse()
            .map(|w: Workflow| (w.name.clone(), w))
            .collect();

        assert!(map.contains_key("in"));
        Ok(Self {
            map,
            module: None,
            func_id: None,
        })
    }
}

// -----------------------------------------------------------------------------
// Codegen:

impl From<BinOp> for IntCC {
    fn from(value: BinOp) -> Self {
        use BinOp::*;
        match value {
            LessThan => IntCC::UnsignedLessThan,
            GreaterThan => IntCC::UnsignedGreaterThan,
        }
    }
}

impl Workflow {
    fn lower(
        &self,
        block_map: &HashMap<&SmolStr, Block>,
        fb: &mut FunctionBuilder,
        rejected_block: Block,
        accepted_block: Block,
        [x, m, a, s]: [Value; 4],
    ) {
        let target_as_block = |target: &_| match target {
            Target::Rejected => rejected_block,
            Target::Accepted => accepted_block,
            Target::SentToOtherWorkflow(other) => block_map[other],
        };

        let mut block = block_map[&self.name];

        // Lower each cond as a block:
        for cond in &self.conditions {
            fb.switch_to_block(block);

            let lhs = match cond.var {
                Var::X => x,
                Var::M => m,
                Var::A => a,
                Var::S => s,
            };
            let cmp = fb.ins().icmp_imm(cond.op, lhs, cond.rhs as i64);
            let if_true = target_as_block(&cond.target);

            // If the condition isn't met we want to just fall through; we do
            // this by creating a block for the next condition:
            block = fb.create_block();

            // note: operands appear to be backwards? rhs, lhs
            fb.ins().brif(cmp, if_true, &[], block, &[]);
            fb.seal_block(block); // this will be the only branch to this block
        }

        // Finally, lower the default condition as the final block:
        let last = target_as_block(&self.default);
        fb.switch_to_block(block);
        fb.ins().jump(last, &[]);

        if !self.conditions.is_empty() {
            fb.seal_block(block); // only predecessor will be a previous cond's block
        }
    }
}

impl Workflows {
    // separate function so that we can have get at the function via an
    // immutable ref, if already jitted
    fn get_filter_func<'w>(&self) -> Option<impl Fn(Part) -> bool + Sync + 'w> {
        if let Some((module, func_id)) = self.module.as_ref().zip(self.func_id) {
            // Get a pointer to the jitted function:
            let ptr = module.get_finalized_function(func_id);

            // let func_raw = unsafe { mem::transmute::<_, extern "C" fn(Part) -> i8>(ptr) };
            let func_raw =
                unsafe { mem::transmute::<_, extern "C" fn(u16, u16, u16, u16) -> i8>(ptr) };
            let func = move |Part { x, m, a, s }| -> bool { func_raw(x, m, a, s) == 1 };

            Some(func)
        } else {
            None
        }
    }

    fn jit<'w>(&'w mut self) -> impl Fn(Part) -> bool + Sync + 'w {
        if let Some(func) = self.get_filter_func() {
            return func;
        }

        let mut module = {
            // We don't actually need any of these libcalls:
            // https://docs.rs/cranelift-module/latest/src/cranelift_module/lib.rs.html#41-63
            // let libcall_names = Box::new(|libcall| panic!("{libcall:?}"));

            // nevermind, we do use probestack...
            let libcall_names = cranelift_module::default_libcall_names();
            let builder = JITBuilder::new(libcall_names).unwrap();
            JITModule::new(builder)
        };
        let mut ctx = module.make_context();
        ctx.want_disasm = DEBUG;

        let func_sig = {
            // Note: this should match the `Part` type.
            let mut sig = module.make_signature();
            // let x = AbiParam::special(types::I16, ArgumentPurpose::StructArgument(0));
            // let m = AbiParam::special(types::I16, ArgumentPurpose::StructArgument(16));
            // let a = AbiParam::special(types::I16, ArgumentPurpose::StructArgument(32));
            // let s = AbiParam::special(types::I16, ArgumentPurpose::StructArgument(48));
            let x = AbiParam::new(types::I16);
            let m = AbiParam::new(types::I16);
            let a = AbiParam::new(types::I16);
            let s = AbiParam::new(types::I16);
            sig.params.extend([x, m, a, s]);
            sig.returns.push(AbiParam::new(types::I8)); // stand-in for bool
            sig
        };
        let func = module
            .declare_function("matches", Linkage::Export, &func_sig)
            .unwrap();

        // Rather than jit each workflow as a separate function and rely on
        // tail call optimization to elide the extra overhead, we just JIT one
        // big function with each workflow as its own label (and use jumps
        // instead of calls to go to other workflows).
        // module.
        {
            ctx.func = Function::with_name_signature(
                UserFuncName::User(UserExternalName::new(0, func.as_u32())),
                func_sig,
            );

            let mut func_ctx = FunctionBuilderContext::new();
            let mut fb = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);

            let entry = fb.create_block();

            // Create blocks for all the workflows:
            let blocks: HashMap<_, _> = self.map.keys().map(|k| (k, fb.create_block())).collect();

            // Fill in the entrypoint:
            fb.switch_to_block(entry);
            fb.append_block_params_for_function_params(entry);
            let &[x, m, a, s] = fb.block_params(entry) else {
                unreachable!()
            };
            let all_args = [x, m, a, s];
            fb.ins().jump(blocks[&SmolStr::new("in")], &[]);

            // And make accepted/rejected blocks:
            let rejected = {
                let block = fb.create_block();
                fb.switch_to_block(block);
                let zero = fb.ins().iconst(types::I8, 0);
                fb.ins().return_(&[zero]);
                block
            };
            let accepted = {
                let block = fb.create_block();
                fb.switch_to_block(block);
                let one = fb.ins().iconst(types::I8, 1);
                fb.ins().return_(&[one]);
                block
            };

            // Lower each workflow:
            for workflow in self.map.values() {
                workflow.lower(&blocks, &mut fb, rejected, accepted, all_args);
            }

            // Finish up:
            fb.seal_all_blocks();
            fb.finalize();

            // Print IR with the block names substituted:
            if DEBUG {
                let mut blocks = blocks;
                let [e, a, r] = ["entry", "accepted", "rejected"].map(SmolStr::new);
                blocks.insert(&e, entry);
                blocks.insert(&a, accepted);
                blocks.insert(&r, rejected);

                print_ir(&ctx.func, blocks);
            }
        }

        // JIT:
        module.define_function(func, &mut ctx).unwrap();

        // Print disasm:
        #[cfg(target_arch = "x86_64")]
        if DEBUG {
            eprintln!(
                "disasm: {}",
                ctx.compiled_code().unwrap().vcode.as_ref().unwrap()
            );

            // eprintln!("\n\nDisasm:");
            // let code = ctx.compiled_code().unwrap().code_buffer();
            // let mut decoder = Decoder::new(64, code, DecoderOptions::NONE);
            // let mut formatter = IntelFormatter::new();
            // let mut output = String::new();

            // let mut insn = Instruction::default();
            // while decoder.can_decode() {
            //     decoder.decode_out(&mut insn);
            //     output.clear();
            //     formatter.format(&insn, &mut output);
            //     eprintln!("  {output}");
            // }
            // eprintln!("\n\n");
        }
        // TODO: handle aarch64 using capstone... nvm!

        // Finish Up:
        module.clear_context(&mut ctx);
        module.finalize_definitions().unwrap();

        // Place this module into storage so it outlives this function:
        self.module = Some(module);
        self.func_id = Some(func);

        // future invocations should use the already-jitted function:
        self.jit()
    }
}

fn print_ir<'s, 'f>(func: &'f Function, block_map: HashMap<&'s SmolStr, Block>) {
    let mut ir = func.display().to_string();
    for (name, block) in block_map {
        ir = ir.replace(
            &format!("block{}", block.as_u32()),
            &format!("b{}_{name}", block.as_u32()),
        );
    }

    eprintln!("func: {}", ir);
}

// -----------------------------------------------------------------------------
// Part Two:

#[allow(unused)]
fn p2_brute_force(filter: &impl Fn(Part) -> bool) -> u64 {
    // This is, of course, not how the problem is intended to be solved but we
    // _did_ just waste a bunch of time getting Cranelift set up so it'd be
    // silly _not_ to make use of it, a little :P

    // 256,000,000,000,000
    // 256 quadrillion options
    //
    // 8 cores, say a thousand clock cycles for each invocation of `filter`
    // at (say) 4 GHz that's ~4 million invocations a second
    //
    // 32Q / 4M => 8 billion seconds... i.e. about 250 years

    // nevermind

    unimplemented!()
}

/////

/*
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct State {
    x: Range<u16>,
    m: Range<u16>,
    a: Range<u16>,
    s: Range<u16>,
    dest: Target,
}

impl State {
    fn split(self, cond: Cond) -> (State, State) {

    }

    fn overlap(&self, other: &Self) -> State {
        let range_overlap = |a: Range<u16>, b: Range<u16>| {
            let (mut a_min, mut a_max) = (a.start, a.end);
            let (mut b_min, mut b_max) = (b.start, b.end);

            // normalize so that a starts before b:
            if a_min >= b_min {
                mem::swap(&mut a_min, &mut b_min);
                mem::swap(&mut a_max, &mut b_max);
            }

            // actually nevermind; we don't need to do all this
        };
    }
}

fn p2_range_splitting(workflows: &Workflows) -> u64 {
    todo!()
}
*/

/////

// TODO: link 2020 aoc day 23 video?

// 540 conditions
//
// 246 x conditions
// 255 m conditions
// 289 a conditions
// 290 s conditions
//
// that makes: a ~5 billion element search space
//
// at a hundred (average) clock cycles per `filter` invocation, on a 5GHz CPU
// that's 100 seconds (before parallelization)
//
// the range splitting approach is likely to be more efficient (for every xmas
// condition we go (needlessly) split the cube in the other three dimensions as
// well under this coordinate compression based approach)
//
// but we'll call this acceptable, for now

#[rustfmt::skip]
fn p2_coordinate_compression(workflows: &Workflows) -> u64 {
    let filter_func = workflows.get_filter_func().unwrap();
    let compressed_coordinates = {
        let mut pivot_points = [(); 4].map(|()| {
            let mut v = Vec::with_capacity(300);
            v.extend([1, 4001]);
            v
        });
        workflows
            .map
            .values()
            .flat_map(|w| w.conditions.iter())
            .for_each(|c| {
                let pivot_point = match c.op {
                    BinOp::LessThan => c.rhs, // < rhs; rhs doesn't match cond
                    BinOp::GreaterThan => c.rhs + 1, // > rhs; rhs + 1 doesn't match cond
                };
                pivot_points[c.var as usize].push(pivot_point);
            });

        for v in &mut pivot_points {
            v.sort_unstable();

            for pair in v.windows(2) {
                // the input has this property
                debug_assert_ne!(pair[0], pair[1]);
            }
        }

        pivot_points
    };

    // we'd like to use `multi_cartesian_product` here but we also really want
    // to use rayon (and we care about perf...)
    //
    // so: we give rayon the first two levels of the loop
    let [x, m, a, s] = compressed_coordinates;
    x.par_windows(2)
        .flat_map(|x| m.par_windows(2).map(move |m| (x, m)))
        .map(|(x, m)| {
            let it = a
                .windows(2)
                .flat_map(|a| s.windows(2).map(move |s| (x, m, a, s)));

            it.map(|(x, m, a, s)| {
                // let &[x, x2] = x else { unsafe { unreachable_unchecked() } };
                // let &[m, m2] = m else { unsafe { unreachable_unchecked() } };
                // let &[a, a2] = a else { unsafe { unreachable_unchecked() } };
                // let &[s, s2] = s else { unsafe { unreachable_unchecked() } };
                let &[x, x2] = x else { unreachable!() };
                let &[m, m2] = m else { unreachable!() };
                let &[a, a2] = a else { unreachable!() };
                let &[s, s2] = s else { unreachable!() };


                // let accepted = filter_func(Part { x, m, a, s });
                // if DEBUG {
                //     eprintln!("cell: x:{x:4}..{x2:4} m:{m:4}..{m2:4} a:{a:4}..{a2:4} s:{s:4}..{s2:4} => {accepted}");
                // }
                if filter_func(Part { x, m, a, s }) {
                    (x2 - x) as u64 * (m2 - m) as u64 * (a2 - a) as u64 * (s2 - s) as u64
                } else {
                    0
                }
            })
            .sum::<u64>()
        })
        .sum()
}

// -----------------------------------------------------------------------------
// Main:

const DEBUG: bool = false;

const INP: &str = "px{a<2006:qkq,m>2090:A,rfg}
pv{a>1716:R,A}
lnx{m>1548:A,A}
rfg{s<537:gd,x>2440:R,A}
qs{s>3448:A,lnx}
qkq{x<1416:A,crn}
crn{x>2662:A,R}
in{s<1351:px,qqz}
qqz{s>2770:qs,m<1801:hdj,R}
gd{a>3333:R,R}
hdj{m>838:A,pv}

{x=787,m=2655,a=1222,s=2876}
{x=1679,m=44,a=2067,s=496}
{x=2036,m=264,a=79,s=2244}
{x=2461,m=1339,a=466,s=291}
{x=2127,m=1623,a=2188,s=1013}";

// in{x>0:R,m<4000:R,a<4000:R,s<4000:R,A}
const INP_SMALL: &str = "in{x<4000:R,m<4000:R,a<4000:R,s<4000:R,A}

{x=0,m=0,a=0,s=0}
";

static LAST_EVENT: Mutex<Option<Instant>> = Mutex::new(None);
fn event(name: &str) {
    let mut last_ev = LAST_EVENT.lock().unwrap();
    let last = *last_ev;
    *last_ev = Some(Instant::now());

    let Some(last) = last else {
        return;
    };
    eprintln!("[timing] {:?} for {name}", last.elapsed());
}

fn main() {
    let mut aoc = AdventOfCode::new(2023, 19);
    let inp = aoc.get_input();
    event("start");
    // let inp = INP;
    // let inp = INP_SMALL;
    let (workflows, parts) = inp.split_once("\n\n").unwrap();
    let (mut workflows, parts) = (
        workflows.parse::<Workflows>().unwrap(),
        parts.lines().map_parse::<Part>().collect_vec(),
    );
    event("parse");

    let filter = workflows.jit();
    event("jit");

    let p1: u64 = {
        parts
            .par_iter()
            .filter(|&&p| filter(p))
            .map(|p| p.rating_sum())
            .sum()
    };
    event("p1-exec");
    _ = aoc.submit_p1(p1);

    drop(filter);
    let p2: u64 = p2_coordinate_compression(&workflows);
    _ = aoc.submit_p2(p2);
}
