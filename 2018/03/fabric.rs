#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};

// Tuple of Options to Option of Tuple
fn too_to_oot<A, B, C, D, E>((a, b, c, d, e): (Option<A>, Option<B>, Option<C>, Option<D>, Option<E>)) -> Result<(A, B, C, D, E), ()>
{
    Ok((a.ok_or(())?, b.ok_or(())?, c.ok_or(())?, d.ok_or(())?, e.ok_or(())?))
}

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new(2018, 03);
    let input: String = aoc.get_input();

    let input = input.lines().filter_map(|l|
        too_to_oot(scan_fmt!(l, "#{d} @ {d},{d}: {d}x{d}", u16, u16, u16, u16, u16)).ok()
    );
    /*.filter_map(|(i, x, y, w, h)|
        i.and_then(|i| x.map(|x| (i, x)))
            .and_then(|(i, x)| y.map(|y| (i, x, y)))
                .and_then(|(i, x, y)| w.map(|w| (i, x, y, w)))
                    .and_then(|(i, x, y, w)| h.map(|h| (i, x, y, w, h)))
    );*/

    // input.for_each(|(_, x, y, w, h)| println!("{},{}: {}x{}", x, y, h, w));

    // HashSet of (u16, u16) coordinates: 32 * 1000 * 1000 = ~32 MB; O(1000 * 1000 * N)
    //  -> Possibly 1 bit per, depending on how hash sets are implemented here?
    //  -> But also hashing is silly here since we have a clear and good indexing
    //     scheme.
    // Grid of bits: 1 * 1000 * 1000 = ~1 MB; O(1000 * 1000 * N)
    //  -> runtime worstcase assumes all the slices are the maximum size.
    // 2 BitVectors per Slice + comparisons between every two claims:
    //  -> 1000 * 2 * N (here, N > 1000 so this is actually _worse_!!)
    //  -> O(1 * N ^ 2) (again since N > 1000, this is probably worse!)
    //  -> Also if you're doing the comparisons for every combination of 2,
    //     you don't need the bitvectors; they're not a win. It's probably
    //     actually faster to do it in place.
    // The right way to do this is to use a segment tree and sort by coordinates
    // but I'm pressed for time, so let's just take the naive approach.

    // This is both wrong *and* slow. But it is fun.
    // Update: with release mode it's pretty fast, but it's still wrong
    // (we ignore multiple overlaps; this solved a different problem).
    // let p1 = input.clone().map(|(_, x, y, w, h)| {
    //     let mut bv_row = BitVec::with_capacity(1000);
    //     let mut bv_col = BitVec::with_capacity(1000);

    //     // row: <-- x --><-- w --><-- 1000 - (w + x) -->
    //     bv_row.grow(1000 - (w + x) as usize, false);
    //     bv_row.grow(w as usize, true);
    //     bv_row.grow(x as usize, true);

    //     // col: <-- y --><-- h --><-- 1000 - (h + y) -->
    //     bv_col.grow(1000 - (h + y) as usize, false);
    //     bv_col.grow(h as usize, true);
    //     bv_col.grow(y as usize, true);

    //     (bv_col, bv_row)
    // }).combinations(2).map(|v| {
    //     let ((c1, r1), (c2, r2)) = (&v[0], &v[1]);

    //     let (mut c1, mut r1) = (c1.clone(), r1.clone());
    //     if c1.union(c2) && r1.union(r2) {
    //         c1.iter().filter(|x| *x).count() * r1.iter().filter(|x| *x).count()
    //     } else {
    //         0
    //     }

    // });

    let mut grid = [0u8; 1000 * 1000];

    input.clone().for_each(|(_, x, y, w, h)| {
        let (x, y, w, h) = (x as usize, y as usize, w as usize, h as usize);
        (x..x+w).for_each(|x| (y..y+h).for_each(|y| {
            grid[1000 * y + x] = grid[1000 * y + x].saturating_add(1);
        }))
    });

    let p1 = grid.iter().filter(|x| **x > 1).count();

    aoc.submit_p1(p1);

    let p2 = input.clone().filter(|(_, x, y, w, h)| {
        let (x, y, w, h) = (*x as usize, *y as usize, *w as usize, *h as usize);
        (x..x+w).all(|x| (y..y+h).all(|y| grid[1000 * y + x] <= 1))
    }).map(|(i, _, _, _, _)| i).next().unwrap();

    aoc.submit_p2(p2);
}
