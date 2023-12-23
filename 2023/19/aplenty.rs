use std::{collections::HashMap, mem};

use aoc::{iterator_map_ext::IterMapExt, AdventOfCode, FromStr, Itertools};
use cranelift::{
    codegen::ir::{types, ArgumentPurpose, Function, UserExternalName, UserFuncName},
    prelude::{AbiParam, Block, FunctionBuilder, FunctionBuilderContext, InstBuilder, IntCC},
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
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
    X,
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
        Ok(Self { map, module: None })
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
    ) {
        let target_as_block = |target: &_| match target {
            Target::Rejected => (accepted_block, false),
            Target::Accepted => (rejected_block, false),
            Target::SentToOtherWorkflow(other) => (block_map[other], true),
        };

        let mut block = block_map[&self.name];

        // Lower each cond as a block:
        for cond in &self.conditions {
            fb.switch_to_block(block);
            fb.append_block_params_for_function_params(block);
            let &[x, m, a, s] = fb.block_params(block) else {
                unreachable!()
            };

            let lhs = match cond.var {
                Var::X => x,
                Var::M => m,
                Var::A => a,
                Var::S => s,
            };
            let rhs = fb.ins().iconst(types::I16, cond.rhs as i64);
            let cmp = fb.ins().icmp(cond.op, lhs, rhs);
            let (if_true, forward_args) = target_as_block(&cond.target);

            // If the condition isn't met we want to just fall through; we do
            // this by creating a block for the next condition:
            block = fb.create_block();

            let all_args = [x, m, a, s];
            fb.ins().brif(
                cmp,
                if_true,
                if forward_args { &all_args } else { &[] },
                block,
                &all_args,
            );
            fb.seal_block(block); // this will be the only branch to this block
        }

        // Finally, lower the default condition as the final block:
        let (last, forward_args) = target_as_block(&self.default);
        fb.switch_to_block(block);
        fb.append_block_params_for_function_params(block);
        let &[x, m, a, s] = fb.block_params(block) else {
            unreachable!()
        };

        let all_args = [x, m, a, s];
        fb.ins()
            .jump(last, if forward_args { &all_args } else { &[] });

        if !self.conditions.is_empty() {
            fb.seal_block(block); // only predecessor will be a previous cond's block
        }
    }
}

impl Workflows {
    fn jit<'w>(&'w mut self) -> impl Fn(Part) -> bool + 'w {
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

        let func_sig = {
            // Note: this should match the `Part` type.
            let mut sig = module.make_signature();
            let x = AbiParam::special(types::I16, ArgumentPurpose::StructArgument(0));
            let m = AbiParam::special(types::I16, ArgumentPurpose::StructArgument(16));
            let a = AbiParam::special(types::I16, ArgumentPurpose::StructArgument(32));
            let s = AbiParam::special(types::I16, ArgumentPurpose::StructArgument(48));
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
            fb.ins().jump(blocks[&SmolStr::new("in")], &all_args);

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
                workflow.lower(&blocks, &mut fb, rejected, accepted);
            }

            // Finish up:
            fb.seal_all_blocks();
            fb.finalize();

            eprintln!("func: {}", ctx.func.display());
        }

        // Finish up:
        module.define_function(func, &mut ctx).unwrap();
        module.clear_context(&mut ctx);

        module.finalize_definitions().unwrap();

        // Place this module into storage so it outlives this function:
        self.module = Some(module);
        let module = self.module.as_ref().unwrap();

        // Get a pointer to the jitted function:
        let ptr = module.get_finalized_function(func);
        let func_raw = unsafe { mem::transmute::<_, extern "C" fn(Part) -> i8>(ptr) };
        let func = move |p: Part| -> bool { func_raw(p) != 0 };

        func
    }
}

// -----------------------------------------------------------------------------
// Main:

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

fn main() {
    let mut aoc = AdventOfCode::new(2023, 19);
    let inp = aoc.get_input();
    let inp = INP;
    let (workflows, parts) = inp.split_once("\n\n").unwrap();
    let (mut workflows, parts) = (
        workflows.parse::<Workflows>().unwrap(),
        parts.lines().map_parse::<Part>().collect_vec(),
    );

    let p1: u64 = {
        let filter = workflows.jit();
        parts
            .par_iter()
            .filter(|&&p| filter(p))
            .map(|p| p.rating_sum())
            .sum()
    };
    dbg!(p1);
}
