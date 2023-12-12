#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::collections::{HashMap, HashSet};
use std::mem;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Ingredient<'a>(&'a str);

impl<'a> From<&'a str> for Ingredient<'a> {
    fn from(s: &'a str) -> Self {
        if s.is_empty() {
            panic!()
        }
        Ingredient(s)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Allergen<'a>(&'a str);

impl<'a> From<&'a str> for Allergen<'a> {
    fn from(s: &'a str) -> Self {
        if s.is_empty() {
            panic!()
        }
        Allergen(s)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Food<'a> {
    ingredients: HashSet<Ingredient<'a>>,
    allergens: HashSet<Allergen<'a>>,
}

impl<'a> TryFrom<&'a str> for Food<'a> {
    type Error = ();

    fn try_from(s: &'a str) -> Result<Self, ()> {
        let mut s = s.split("(contains ");
        let ingredients = s.next().unwrap();
        let allergens = s.next().unwrap().strip_suffix(")").unwrap();

        Ok(Self {
            ingredients: ingredients
                .split(' ')
                .filter(|s| !s.is_empty())
                .map(Into::into)
                .collect(),
            allergens: allergens.split(", ").map(Into::into).collect(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AllergenSolveState<'a> {
    Pinned(Ingredient<'a>),
    OneOf(HashSet<Ingredient<'a>>),
}

impl<'a> AllergenSolveState<'a> {
    fn new(possible_ingredients: impl Iterator<Item = Ingredient<'a>>) -> Self {
        let mut possible_ingredients: HashSet<_> = possible_ingredients.collect();

        if possible_ingredients.len() == 1 {
            Self::Pinned(possible_ingredients.drain().next().unwrap())
        } else {
            Self::OneOf(possible_ingredients)
        }
    }

    /// Returns Some() if eliminating this ingredient caused this to become
    /// pinned.
    fn eliminate(&mut self, ingredient: &Ingredient<'a>) -> Option<Ingredient<'a>> {
        match self {
            Self::Pinned(_) => None,
            Self::OneOf(ings) => {
                if ings.remove(ingredient) && ings.len() == 1 {
                    let ing = ings.drain().next().unwrap();
                    self.pin(ing);
                    return Some(ing);
                }

                None
            }
        }
    }

    fn pin(&mut self, ingredient: Ingredient<'a>) {
        *self = AllergenSolveState::Pinned(ingredient)
    }

    fn is_pinned(&self) -> bool {
        matches!(self, AllergenSolveState::Pinned(_))
    }

    fn pinned_ingredient(&self) -> Option<Ingredient<'a>> {
        if let AllergenSolveState::Pinned(p) = self {
            Some(*p)
        } else {
            None
        }
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2020, 21);
    let input: String = aoc.get_input();

    let foods: Vec<Food> = input.lines().map(TryConvert::to).collect();
    let mut allergen_map: HashMap<Allergen, Vec<&Food>> = HashMap::new();
    let mut ingredient_map: HashMap<Ingredient, Vec<&Food>> = HashMap::new();

    for f in &foods {
        for ing in &f.ingredients {
            ingredient_map.entry(*ing).or_default().push(f);
        }

        for alg in &f.allergens {
            allergen_map.entry(*alg).or_default().push(f);
        }
    }

    let mut solve_states: HashMap<Allergen, AllergenSolveState> = allergen_map
        .keys()
        .map(|k| {
            // To start with we grab only the ingredients that show up in all
            // the foods that contain the allergen:
            (
                *k,
                AllergenSolveState::new(
                    ingredient_map
                        .keys()
                        .filter(|ing| allergen_map[k].iter().all(|f| f.ingredients.contains(ing)))
                        .copied(),
                ),
            )
        })
        .collect();

    // Multiple facts we can use to make progress:
    //   - single allergen in a food:
    //       + if the allergen is in other foods, the matching ingredient must
    //         be in all the foods (note that we can't use this to narrow things
    //         down if there's more than 1 unpinned allergen in a food — i.e. we
    //         can't limit the potential ingredient matches for an allergen to
    //         just the foods that show up in )

    // let mut unsolved = solve_states
    //     .values()
    //     .filter(|v| matches!(v, AllergenSolveState::OneOf(_)))
    //     .count();

    // // it'd be more efficient to keep a running count that we remember to
    // // subtract from in the right places but this is okay for now
    // let unsolved = |ss: &HashMap<_, _>| {
    //     ss.values()
    //         .any(|v| matches!(v, AllergenSolveState::OneOf(_)))
    // };

    let mut unsolved = solve_states.len();
    let mut previously_solved: Vec<(Allergen, Ingredient)> = solve_states
        .iter()
        .filter_map(|(a, s)| match s {
            AllergenSolveState::Pinned(ing) => Some((*a, *ing)),
            _ => None,
        })
        .collect();

    while unsolved != 0 {
        // // it'd be more efficient to only do this for the newly solved but eh
        // for a in solve_states.iter().filter()

        if previously_solved.is_empty() {
            panic!(
                "We seem to be stuck! (unsolved = {})\n{:?}",
                unsolved, solve_states
            );
        }

        // First apply the things we just solved:
        let mut just_solved: Vec<(Allergen, Ingredient)> = vec![];
        for (a, ing) in previously_solved.drain(..) {
            println!("{:?} is solved to be {:?}", a, ing);
            unsolved -= 1;
            for (a, ss) in solve_states.iter_mut().filter(|(_, ss)| !ss.is_pinned()) {
                if let Some(pinned) = ss.eliminate(&ing) {
                    just_solved.push((*a, pinned));
                }
            }
        }
        mem::swap(&mut just_solved, &mut previously_solved);
    }

    // If we manage to solve everything just like that, we can just move on...
    let unsafe_ingredients: HashSet<_> = solve_states
        .values()
        .map(|ss| ss.pinned_ingredient().unwrap())
        .collect();
    assert_eq!(
        unsafe_ingredients.len(),
        solve_states.len(),
        "Expecting as many allergens as unsafe ingredients."
    );
    let safe_ingredients = ingredient_map.keys().copied().collect::<HashSet<_>>();
    let safe_ingredients = safe_ingredients
        .difference(&unsafe_ingredients)
        .collect::<HashSet<_>>();

    let p1: usize = foods
        .iter()
        .map(|f| {
            f.ingredients
                .iter()
                .filter(|f| safe_ingredients.contains(f))
                .count()
        })
        .sum();
    let _ = aoc.submit_p1(p1);

    let mut unsafe_ingredients: Vec<_> = solve_states
        .iter()
        .map(|(a, ss)| (a, ss.pinned_ingredient().unwrap()))
        .collect();
    unsafe_ingredients.sort();
    let list: Vec<_> = unsafe_ingredients.iter().map(|(_, ing)| ing.0).collect();
    let p2 = list.join(",");

    println!("{}", p2);
    let _ = aoc.submit_p2(p2);
}
