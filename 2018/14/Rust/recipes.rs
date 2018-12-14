#!/usr/bin/env rustr
extern crate aoc;

#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};

fn recipe_sequence(len: usize) -> Vec<u8> {
    let mut recipes = Vec::with_capacity(len);
    recipes.push(3u8);
    recipes.push(7u8);

    let (mut elf1, mut elf2) = (0, 1);

    while recipes.len() < len {
        let (r1, r2) = (recipes[elf1], recipes[elf2]);
        let new = r1 + r2;

        if new >= 10 {
            recipes.push(1);
            recipes.push(new - 10);
        } else {
            recipes.push(new);
        }

        elf1 = (elf1 + r1 as usize + 1) % recipes.len();
        elf2 = (elf2 + r2 as usize + 1) % recipes.len();

        // for (idx, v) in recipes.iter().enumerate() {
        //     if idx == elf1 {
        //         print!("({})", v)
        //     } else if idx == elf2 {
        //         print!("[{}]", v)
        //     } else {
        //         print!(" {} ", v)
        //     }
        // }
        // println!("");
    }

    recipes
}

fn scores(skip: usize, count: usize) -> String {
    recipe_sequence(skip + count).iter()
        .skip(skip)
        .take(count)
        .map(|r| format!("{}", r)).collect()
}

fn position(search: &str, mut guess: usize) -> usize {
    let mut recipes: String;
    let mut last_guess: usize = 0;

    loop {
        println!("Trying with {}", guess);
        recipes = recipe_sequence(guess).iter()
            .skip(last_guess.saturating_sub(search.len()))
            .map(|r| format!("{}", r)).collect();

        if let Some(l) = recipes.find(search) {
            return l + last_guess.saturating_sub(search.len());
        }

        last_guess = guess;
        guess += guess * 5 + 1;
    }
}

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new_with_year(2018, 14);
    let input: String = aoc.get_input();

    let num_recipes: usize = input.lines().next().unwrap().parse().unwrap();

    aoc.submit_p1(scores(num_recipes, 10));
    aoc.submit_p2(position(&format!("{}", num_recipes), 1000000));
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recipe_sequence_tests() {
        assert_eq!(recipe_sequence(2), vec![3, 7]);
        assert_eq!(recipe_sequence(5), vec![3, 7, 1, 0, 1, 0]);
        assert_eq!(recipe_sequence(6), vec![3, 7, 1, 0, 1, 0]);
    }

    #[test]
    fn scores_tests() {
        assert_eq!(scores(9, 10), "5158916779");
        assert_eq!(scores(5, 10), "0124515891");
        assert_eq!(scores(18, 10), "9251071085");
        assert_eq!(scores(2018, 10), "5941429882");
    }

    #[test]
    fn position_tests() {
        assert_eq!(position("51589", 0), 9);
        assert_eq!(position("01245", 0), 5);
        assert_eq!(position("92510", 0), 18);
        assert_eq!(position("59414", 100000), 2018);
        assert_eq!(position("5992684592", 10), 165061);
        assert_eq!(position("165061", 20000000), 20181148);
    }
}
