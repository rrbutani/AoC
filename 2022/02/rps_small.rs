// R < P < S < R; wrap-around
//
// # |  R  |  P  |  S
// R |  0  | -1  | -2*
// P |  1* |  0  | -1
// S |  2  |  1* |  0
//
// with `+3 % 3`:
// # |  R  |  P  |  S
// R |  0  |  2  |  1*
// P |  1* |  0  |  2
// S |  2  |  1* |  0
//
// 0 => draw => 3
// 1 => win  => 6
// 2 => loss => 0
//
// this is just a rotate and then a multiply:
// 0 -> 1 * 3 -> 3
// 1 -> 2 * 3 -> 6
// 2 -> 0 * 3 -> 0
//
// i.e.: +1 % 3 * 3
//
// altogether that's:
// (((((you - opp) + 3) % 3) + 1) % 3) * 3
//
// which we can simplify to:
// ((you - opp + 4) % 3) * 3

// part 2 is essentially asking us to index into the table above
// 1 -> lose -> 2 (i.e. - 1 (aka +2 % 3) to opp's move)
// 2 -> draw -> 0 (i.e. + 0 to opp's move)
// 3 -> win  -> 1 (i.e. + 1 to opp's move)
//
// taking advantage of the circular nature of mod, we get this nice flat
// mapping of what number to add:
// 1 -> lose -> +2 % 3
// 2 -> draw -> +3 % 3
// 3 -> win  -> +4 % 3
//
// which collapses into: `(outcome + (opp - 1) + 1) % 3` for the move to play
// to which we add 1 to get it's score
// to which we add `(outcome - 1) * 3` for the game score
//
// all together that's: `(opp + outcome) % 3 + 1 + (outcome - 1) * 3`
// or: `(o + r) % 3 + 1 + 3*r - 3`
// ->  `(o + r) % 3 + 3*r - 2`

use aoc::*;
#[rustfmt::skip]
fn main(){sub!(|i|{
    let g=|f:fn(_)->u64|i.line_bytes_try_map(|[a,_,b]:[_;3]|(a-b'@',b-b'W')).map(f).sum::<u64>();
    (g(|(o, y)|(y+((y+4-o)%3)*3)as _), g(|(o,r)|((o+r)%3+3*r-2)as _))
});}

// let mut a = AdventOfCode::new(2022, 2);
// let i = a.get_input();
// let g = || i.lines().map(|l| l.as_bytes()).map_try(|[a, _, b]: [u8; 3]| (a - b'@', b - b'W'));

// a.submit_p1::<u64>(g().map(|(o, y)| (y + ((y + 4 - o) % 3) * 3) as u64).sum()).unwrap();
// a.submit_p2::<u64>(g().map(|(o, r)| ((o + r) % 3 + 3 * r - 2) as u64).sum()).unwrap();
