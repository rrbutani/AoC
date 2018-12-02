#!/usr/bin/env rustr
extern crate aoc;

#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};

fn letter_counts(s: &str) -> (bool, bool) {
    let mut chars = [0u8; 256]; // We are making an assumption here..
    s.chars().for_each(|c| chars[c as usize] += 1);

    let f = |n| chars.iter().any(|v| *v == n);
    (f(2), f(3))
}

fn compare_allowing_one(a: &[&str]) -> Option<String> {
    let v = a[0].chars().zip(a[1].chars());

    if v.clone().filter(|(a, b)| a != b).count() == 1 {
        Some(v.filter(|(a, b)| a == b).map(|(a, _)| a).collect())
    } else { None }
}

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new_with_year(2018, 02);
    let input: String = aoc.get_input();

    let counts = input.lines()
        .map(letter_counts)
        .fold((0, 0), |a, x| (a.0 + x.0 as u32, a.1 + x.1 as u32));

    let p1 = counts.0 * counts.1;
    aoc.submit_p1(p1);

    // This is wrong, but did indeed work for my input.
    // consider abcde, abcbe, and abccc; sorted order is: abcbe, abccc, and
    // abcde meaning that abcde and abcbe wouldn't be compared.
    let mut p2 = input.lines().collect::<Vec<&str>>();
    p2.sort();
    let p2 = p2.as_slice()
        .windows(2)
        .find_map(compare_allowing_one)
        .unwrap();
    aoc.submit_p2(p2);

    // So, here's another way. We can use the sorted trick so long as our
    // prefixes are the same. Let's put our strings in a tree:
    //
    // Say we have:
    //   - abcde
    //   - abccc
    //   - abcbe
    //   - abddd
    //   - acabb
    //   - babab
    //   - bbcde
    //
    // We can then make this: (a prefix tree or a trie)
    //         a ---- root ----- b
    //        / \               / \
    //       b   c             a   b
    //      / \   \           /     \
    //     c   d   a         b       c
    //    /|\   \   \       /         \
    //   d b c   d   b     a           d
    //  / /   \   \   \   /             \
    // e e     c   d   b b               e
    //
    // Start at the bottom with a full string, say with abcde.
    //
    // To begin, say you're okay with your last letter (e) being different.
    // Go to your parent (abcd) and check if it has other children. Since it
    // doesn't, we know that there are no strings that only differ from us in
    // their last letter.
    // 
    // Next let's say we're okay with our 2nd to last letter (d) being
    // different. Go to it's parent (abc) and check if it has other children.
    // Indeed, it does! For each child it has, we must compare our suffix (e)
    // with it's (e) (note: suffix being the characters beyond the character
    // we're ignoring). Since they're the same (e == e) we've got a match!
    //
    // For completions sake, here's the full list of comparisons we'd do:
    //  - abcd_ -> abcd: has no other children
    //  - abc_e -> abc: has two other children!
    //    * abc_e == abc_e: match!
    //    * abc_e != abc_c: no match
    //  - abcb_ -> abcb: has no other childen // breadth first, jump to children so we don't repeat
    //  - abc_e -> abc: has two other children, one of which is unprocessed:
    //    * abc_e != abc_c: no match
    //  - abcc_ -> abcc: has no other children
    //  - abc_c -> abc: has no *unprocessed* children:
    //  - ab_de -> ab: has one other child!
    //    * ab_c != ab_d: no match
    //  ...
    //  - abdd_ -> abdd: has no other children
    //  - abd_d -> abd: has no other children
    //  - ab_dd -> ab: has one other child!
    //    * ab_d == ab_b: no match! // Note that we add one character back in
    //    * ab_d == ab_d: match!    // at a time. We can possibly use a HashMap
    //      * ab_dd == ab_de: no match! // to speed this up.
    //  - a_ddd -> a: has one other child
    //
    // In writing the above it became clear that the worst case is still ~n^2:
    //               a
    //              / \
    //             a   b
    //            / \ / \
    //           a  b a  b
    //          /\ /\ /\ /\
    //          ab ab ab ab
    // contains:
    //  - aaaa
    //  - aaab
    //  - aaba
    //  - aabb
    //  - abaa
    //  - abab
    //  - ABBA // <3
    //  - abbb
    // 
    // There's only a performance win when you're able to eliminate multiple
    // strings as match candidates because the prefixes don't match. If the
    // string set is optimal:
    // 
    //               a
    //              / \
    //             b   c
    //            / \ / \
    //           d  e f  g
    //          /\ /\ /\ /\
    //          hi jk lm no
    // contains:
    //  - abdh
    //  - abdi
    //  - abej
    //  - abek
    //  - acfl
    //  - acfm
    //  - acgn
    //  - acgo
    //
    // We get to skip comparing abd{h,i} against acf{l,m} and acg{n,o} (which
    // we need to do two comparisons to establish). As the strings get longer,
    // the benefits from this become more pronounced but the inherent
    // assumption is that strings differ early on.
    //
    // In more detail:
    //  - abdh vs. abdh, abej, abek, acf, acg,
    //  - abdi vs. abej, abek, // We can skip acf and acg since we haven't gone up a level and abdi's sibling ended matching with them early
    //  - abej vs. abek, acf, acg // Note that abej can't skip acf and acg; "what if abdi ended early because of the 'd'? I have an 'e'! I might match!"
    //  - abek vs. // We can skip acf and acg for the same reasons abdi can
    //  - acfl vs. acfm, acgn, acgo
    //  - acfm vs. acgn, acgo
    //  - acgn vs. acgo
    //  
    // All told, that's 16 comparisons to the 28 that the naive combinations
    // approach would have needed.
    //  
    // If we had 16 strings in the set, we'd have needed 32 comparisons ~(2*n)
    // compared to the 120 the naive approach would have taken ((n * n-1) / 2).
    //
    // For the 250 strings: 500 to 31,125, *assuming* a perfectly optimal input
    // set.
    //
    // We also haven't even begun to consider the cost of assembling this
    // structure or the (potential) additional memory cost. Yes, we do use less
    // space storing characters because it's a trie, but pointers are much
    // bigger than characters and if we use 4 or 8 character chunks to compensate
    // we lose the benefits of having a tree.
    //
    // On the flipside, we do save a few comparisons by not having to compare
    // every two strings from their beginnings. On the other hand, processors
    // are really very good at things that are easily vectorizable like that.
    //
    // Prefix + Suffix trees also seems like a dead end.
    //  
    // So, I don't know.

    // We still need a working solution though and I don't want to do it the
    // combinations way, so let's try something completely different: a
    // solution that's proportional to the number of characters we have in
    // time complexity and even worse with space!
    // For m = num strings and n = num chars in each string, this is:
    // time: O(n*m); space: O(n*n*m)
    //
    // About 300K of additional memory for about a fifth as many operations...
    // except that HashSets aren't magic and that this is actually probably
    // slower.
    
    use std::collections::HashSet;
    use std::iter::FromIterator;

    let mut hs = HashSet::new();

    let p2: String = input.lines().map(|inp| {
        Vec::<String>::from_iter((0..inp.len())
            .into_iter()
            .map(|i| {
                let mut s = String::new();
                inp.chars()
                    .enumerate()
                    .for_each(|(k, c)| {
                        s.push(if k != i {c} else {'_'})
                    });
                    s
            })
        )
    }).flat_map(|v| v.into_iter())
        .filter(|s| !hs.insert(s.clone()))
        .next()
        .unwrap()
        .chars()
        .filter(|c| *c != '_')
        .collect();

    aoc.submit_p2(p2);

    // Or, as an (almost) one line monstrosity:
    // let mut hs = HashSet::new(); aoc.submit_p2::<String>(AdventOfCode::new_with_year(2018, 02).get_input().lines().map(|inp| { Vec::<String>::from_iter((0..inp.len()).into_iter().map(|i| { let mut s = String::new(); inp.chars().enumerate().for_each(|(k, c)| { s.push(if k != i {c} else {'_'})}); s}))}).flat_map(|v| v.into_iter()).filter(|s| !hs.insert(s.clone())).next().unwrap().chars().filter(|c| *c != '_').collect());
}
