#!/usr/bin/env rustr

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::iter;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Tile {
    data: [[char; 10]; 10],
    id: usize,
}

impl Display for Tile {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(fmt, "Tile {}:", self.id)?;

        for row in self.data.iter() {
            for c in row.iter() {
                write!(fmt, "{}", c)?
            }
            writeln!(fmt)?;
        }

        Ok(())
    }
}

impl FromStr for Tile {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        let id = s
            .lines()
            .next()
            .and_then(|s| s.strip_prefix("Tile "))
            .and_then(|s| s.strip_suffix(':'))
            .ok_or(())
            .and_then(|s| s.parse().map_err(|_| ()))?;

        let data = s
            .lines()
            .skip(1)
            .map(|l| l.chars().collect::<Vec<_>>().to())
            .collect::<Vec<_>>()
            .to();

        Ok(Tile { data, id })
    }
}

// type Edge = [char; 10];

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Side {
    Top = 0,
    Right = 1,
    Bottom = 2,
    Left = 3,
}

impl Side {
    const ALL: [Side; 4] = Self::all();

    const fn all() -> [Side; 4] {
        use Side::*;
        [Top, Right, Bottom, Left]
    }

    // Returns how many times you'd need to rotate `self` right to get to
    // `side`.
    fn offset(&self, other: Self) -> u8 {
        ((other as i8 - *self as i8 + 4) % 4) as u8
    }
}

#[derive(Copy, Clone, Debug)]
struct Edge {
    // lower when sorted
    lower: [char; 10],
    // higher when sorted
    higher: [char; 10],

    lower_is_actual: bool,
}

// impl Hash for Edge {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         let reversed: [char; 10] = self.0.iter().rev().copied().collect::<Vec<_>>().to();
//         let mut v = [self.0, reversed];
//         v.sort();

//         v.hash(state)
//     }
// }

// Specifically do not use `lower_is_actual` for comparison or Hashing.

impl Hash for Edge {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.lower.hash(h);
        self.higher.hash(h);
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Edge) -> bool {
        // don't even need to look at higher; we keep higher around so we don't
        // have to recompute it again and again
        self.lower == other.lower
    }
}

impl Eq for Edge {}

impl From<[char; 10]> for Edge {
    fn from(e: [char; 10]) -> Edge {
        Edge::new(e)
    }
}

impl Edge {
    fn new(inner: [char; 10]) -> Self {
        let reversed: [char; 10] = inner.iter().rev().copied().collect::<Vec<_>>().to();

        if reversed > dbg!(inner) {
            Edge {
                lower: inner,
                higher: reversed,
                lower_is_actual: true,
            }
        } else {
            Edge {
                lower: reversed,
                higher: inner,
                lower_is_actual: false,
            }
        }
    }

    fn actual(self) -> [char; 10] {
        if self.lower_is_actual {
            self.lower
        } else {
            self.higher
        }
    }
}

impl Tile {
    const LEN: usize = 10;

    // top, right, bottom, left
    fn edges(&self) -> [Edge; 4] {
        [
            Edge::new(self.data[0]),
            Edge::new(self.data.iter().map(|r| r[9]).collect::<Vec<_>>().to()),
            Edge::new(self.data[9]),
            Edge::new(self.data.iter().map(|r| r[0]).collect::<Vec<_>>().to()),
        ]
        // [
        //     Edge::new(self.data[0]),
        //     Edge::new(self.data.iter().map(|r| r[9]).collect::<Vec<_>>().to()),
        //     Edge::new(self.data[9].iter().rev().copied().collect::<Vec<_>>().to()),
        //     Edge::new(
        //         self.data
        //             .iter()
        //             .rev()
        //             .map(|r| r[0])
        //             .collect::<Vec<_>>()
        //             .to(),
        //     ),
        // ]
    }

    fn edge(&self, dir: Side) -> Edge {
        use Side::*;
        let idx = match dir {
            Top => 0,
            Right => 1,
            Bottom => 2,
            Left => 3,
        };

        *self.edges().iter().nth(idx).unwrap()
    }

    fn actual_edge(&self, dir: Side) -> [char; 10] {
        self.edge(dir).actual()
    }

    // Give this a (row, col) iterator that tells this function what indexes of
    // `data` to use to create a new copy of `data`.
    #[inline]
    fn transform(&mut self, idxes: impl Iterator<Item = (usize, usize)>) {
        // This is hilariously inefficient but it's okay; we're just having fun
        // here.
        self.data = idxes
            .collect::<Vec<_>>()
            .chunks(Self::LEN)
            .map(|r| {
                r.iter()
                    .map(|(r, c)| self.data[*r][*c])
                    .collect::<Vec<_>>()
                    .to()
            })
            .collect::<Vec<_>>()
            .to();
    }

    // [
    //     [0123456789],
    //     [0123456789],
    //     [0123456789],
    //     [0123456789],
    //     [0123456789],
    //     [0123456789],
    //     [0123456789],
    //     [0123456789],
    //     [0123456789],
    //     [0123456789],
    // ]

    fn rotate_right(&mut self) {
        self.transform(
            (0..Self::LEN)
                .cartesian_product(0..Self::LEN)
                .map(|(c, r)| (Self::LEN - 1 - r, c)),
        )
    }

    fn flip_vertically(&mut self) {
        self.transform(
            (0..Self::LEN)
                .cartesian_product(0..Self::LEN)
                .map(|(r, c)| (Self::LEN - 1 - r, c)),
        )
    }

    fn flip_horizontally(&mut self) {
        self.transform(
            (0..Self::LEN)
                .cartesian_product(0..Self::LEN)
                .map(|(r, c)| (r, Self::LEN - 1 - c)),
        )
    }

    /*     fn rotate_right(&mut self) {
        let mut staging = [['_'; Self::LEN]; Self::LEN];

        for (r, row) in self.data.iter().enumerate() {
            for (c, d) in row.iter().enumerate() {
                staging[(Self::LEN - 1) - c][r] = *d;
            }
        }

        self.data = staging;
    }

    fn flip_vertically(&mut self) {
        let mut staging = [['_'; Self::LEN]; Self::LEN];

        for (r, row) in self.data.iter().enumerate() {
            staging[(Self::LEN - 1) - r] = *row;
        }

        self.data = staging;
    }

    fn flip_horizontally(&mut self) {
        let mut staging = [['_'; Self::LEN]; Self::LEN];

        for (r, row) in self.data.iter().enumerate() {
            for (c, d) in row.iter().enumerate() {
                staging[r][(Self::LEN - 1) - c] = *d;
            }
        }

        self.data = staging;
    } */

    // Rotates and flips so that `edge` is on `side`.
    //
    // Returns `Err(())` if the edge does not match any of our edges.
    fn adjust_to<E: Into<Edge>>(&mut self, edge: E, side: Side) -> Result<(), ()> {
        let edge = edge.into();
        // println!("================================================================");
        // println!("ADJUSTING so that {:?} is on {:?}", edge, side);

        // println!("\nBEFORE: {}", self);

        // First we check if we need to rotate:
        // let edge = edge.into();
        let (_, current_side) = self
            .edges()
            .iter()
            .zip(Side::ALL.iter())
            .find(|(e, _)| **e == edge)
            .ok_or(())?;

        let rotate_times = current_side.offset(side);
        for _ in 0..rotate_times {
            self.rotate_right();
        }

        // Then we check if we need to flip:
        let edges = self.edges();
        let (matching_edge, current_side) = edges
            .iter()
            .zip(Side::ALL.iter())
            .find(|(e, _)| **e == edge)
            .unwrap();

        let flip = dbg!(edge.actual()) != dbg!(matching_edge.actual());
        if flip {
            match current_side {
                Side::Right | Side::Left => self.flip_vertically(),
                Side::Top | Side::Bottom => self.flip_horizontally(),
            }
        }

        // println!("\nAFTER: {}", self);
        // println!("================================================================");

        Ok(())
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2020, 20);
    let input: String = aoc.get_input();

    let input2 = "
Tile 2311:
..##.#..#.
##..#.....
#...##..#.
####.#...#
##.##.###.
##...#.###
.#.#.#..##
..#....#..
###...#.#.
..###..###

Tile 1951:
#.##...##.
#.####...#
.....#..##
#...######
.##.#....#
.###.#####
###.##.##.
.###....#.
..#.#..#.#
#...##.#..

Tile 1171:
####...##.
#..##.#..#
##.#..#.#.
.###.####.
..###.####
.##....##.
.#...####.
#.##.####.
####..#...
.....##...

Tile 1427:
###.##.#..
.#..#.##..
.#.##.#..#
#.#.#.##.#
....#...##
...##..##.
...#.#####
.#.####.#.
..#..###.#
..##.#..#.

Tile 1489:
##.#.#....
..##...#..
.##..##...
..#...#...
#####...#.
#..#.#.#.#
...#.#.#..
##.#...##.
..##.##.##
###.##.#..

Tile 2473:
#....####.
#..#.##...
#.##..#...
######.#.#
.#...#.#.#
.#########
.###.#..#.
########.#
##...##.#.
..###.#.#.

Tile 2971:
..#.#....#
#...###...
#.#.###...
##.##..#..
.#####..##
.#..####.#
#..#.#..#.
..####.###
..#.#.###.
...#.#.#.#

Tile 2729:
...#.#.#.#
####.#....
..#.#.....
....#..#.#
.##..##.#.
.#.####...
####.#.#..
##.####...
##..#.##..
#.##...##.

Tile 3079:
#.#.#####.
.#..######
..#.......
######....
####.#..#.
.#...#.##.
#.#####.##
..#.###...
..#.......
..#.###...";

    let tiles: Vec<Tile> = input
        .trim()
        .split("\n\n")
        .map(|l| l.parse().unwrap())
        .collect();

    let mut edge_map: HashMap<Edge, Vec<usize>> = HashMap::new();
    for t in &tiles {
        for edge in t.edges().iter() {
            edge_map.entry(*edge).or_default().push(t.id);
        }
    }

    // Find the edges that only show up once and then find the tiles that have
    // two such edges; these are our corners:
    let mut id_to_lonely_edge_counts = HashMap::<usize, Vec<Edge>>::new();
    for (e, v) in edge_map.iter().filter(|(_, v)| v.len() == 1) {
        id_to_lonely_edge_counts.entry(v[0]).or_default().push(*e);
    }

    let corners = id_to_lonely_edge_counts
        .iter()
        .filter(|(_, e)| e.len() == 2)
        .collect::<Vec<_>>();

    assert_eq!(corners.len(), 4);
    let p1: usize = corners.iter().map(|(i, _)| *i).product();
    let _ = aoc.submit_p1(p1);

    // They were kind to us and made it so that nothing matches more than 2
    // edges:
    // for (edge, count) in edge_map.iter() {
    //     println!("{:?} → {}", edge, count.len())
    // }

    let mut displaced: HashMap<usize, Tile> = tiles.iter().map(|t| (t.id, t.clone())).collect();

    // Remove the corner tiles from the displaced map *and* remove the corner
    // tiles' edges from the edgemap.
    let mut corners_with_border_edges: Vec<(Tile, Vec<Edge>)> = vec![];
    for (tile_id, border_edges) in &corners {
        // let tile = displaced.remove(tile_id).unwrap();
        let tile = displaced.get(tile_id).unwrap();
        for edge in border_edges.iter() {
            // This is gross and is equivalent to `remove_item`.
            let tile_idx_vec = edge_map.get_mut(edge).unwrap();
            tile_idx_vec.swap_remove(tile_idx_vec.iter().position(|val| val == *tile_id).unwrap());
        }
        corners_with_border_edges.push((tile.clone(), (*border_edges).clone()));
    }

    // X----------X
    // |          | We'll place the corners first; to figure out which corner is
    // |          | which we need to try construct a path from each corner to
    // |          | two other corners.
    // |          |
    // |          | This will fail for some pairs of corners which is what tells
    // |          | us which corner is which.
    // |          |
    // |          |
    // |          |
    // |          |
    // X----------X

    // We return Tiles leading from e1 to e2 that are rotated to match `dir`.
    //
    // `dir` indicates what side `e2` is thought to be on.
    fn find_border_path(
        mut e1: Edge,
        e1_tile_id: usize,
        e2: Edge,
        dir: Side,
        available_tiles: &mut HashMap<usize, Tile>,
        edge_map: &HashMap<Edge, Vec<usize>>,
    ) -> Option<Vec<Tile>> {
        // We really should check that we didn't add anything to the sequence
        // in the cases where we return "no path exists" but for now we'll
        // rely on the fact that we never have more than 2 matching edges (i.e.
        // no "false" matches).
        //
        // If we didn't want to rely on this fact we'd want to put the things in
        // `seq` back into `available_tiles` when we end up not having a match.
        let mut seq = vec![];
        let mut previous_id = e1_tile_id;

        // println!("\n\n------------------------------------------------------------\n\nmatching until {:?}\n", e2);
        let mut func = || -> Option<()> {
            while e1 != e2 {
                let v = edge_map.get(&e1)?;

                // let tile_id = if let Some(prev_id) = previous_id {
                // println!("matches for ({}) {:?}: {:?}", previous_id, e1, v);
                let mut filt = v.iter().filter(|i| **i != previous_id);

                // This can be None in the event that we're trying to find a
                // path to the wrong corner and have hit the border.
                let tile_id = *filt.next()?;
                assert!(filt.next().is_none());
                // } else {
                //     // Not totally sure how this happens; I guess if a corner is
                //     // flipped the wrong way it actually doesn't work? Doesn't seem
                //     // right.
                //     if v.is_empty() {
                //         return None;
                //     }
                //     dbg!(e1);
                //     dbg!(v);
                //     assert_eq!(v.len(), 1);
                //     v[0]
                // };

                previous_id = tile_id;

                let mut next = available_tiles.remove(&tile_id)?;

                // If we find a matching tile, rotate/flip it to match:
                next.adjust_to(e1, dir).unwrap();
                seq.push(next.clone());

                // Note: we should check that this is a border tile; i.e. that one
                // of it's edges has no neighbouring tiles.

                // Now we get the opposite edge:
                let opp_edge = iter::repeat(next.edges().iter().copied())
                    .flatten()
                    .take(8)
                    .skip_while(|e| *e != e1)
                    .nth(2)
                    .unwrap();

                // println!("Matched {:?} to {}\nNext edge: {:?}", e1, next, opp_edge);
                e1 = opp_edge;
            }

            Some(())
        };

        if let Some(()) = func() {
            Some(seq)
        } else {
            for t in seq {
                available_tiles.insert(t.id, t);
            }

            None
        }
    }

    // We store an (Edge, Side) pair so that we know how to rotate the tile
    // later.
    let mut placed: HashMap<(usize, usize), Tile> = HashMap::new();

    // Completely arbitarily we'll decree our first corner to be the one at
    // (0, 0).
    //
    // Once we find the two other corners that have a path to our corner, we'll
    // know the relative placement of all four of our corners *and* our height
    // and width.
    let (mut top_left, mut top_left_borders) = corners_with_border_edges.pop().unwrap();
    let tile = displaced.remove(&top_left.id).unwrap();

    assert_eq!(top_left_borders.len(), 2);
    let border1 = top_left_borders.pop().unwrap();
    let border2 = top_left_borders.pop().unwrap();

    // println!(
    //     "top left:\n{}\n\n borders: \n{:?}\n{:?}",
    //     top_left, border1, border2
    // );

    // We won't worry about flipping now; the entire picture might be flipped
    // horizontally and/or veritically but that's okay.
    //
    // For now we just need to flip and rotate our corner so that it's two
    // border edges are on it's left and top sides.
    //
    // Let's just try both:
    top_left.adjust_to(border1, Side::Left).unwrap();

    if top_left.edge(Side::Bottom) == border2 {
        top_left.flip_vertically();
    }

    // println!("Post-Adj:\n{}", top_left);
    // println!("{:?} is left", border1);

    // If that worked, the top edge should now be `border2`:
    if top_left.edge(Side::Top) != border2 {
        // Otherwise, let's make `border2` the left edge:
        top_left.adjust_to(border2, Side::Left).unwrap();

        if top_left.edge(Side::Bottom) == border1 {
            top_left.flip_vertically();
        }

        // println!("Post-Adj (again):\n{}", top_left);
        // println!("{:?} is left", border2);

        assert_eq!(top_left.edge(Side::Top), border1);
    }

    // Okay!
    placed.insert((0, 0), top_left.clone());

    // Now we can search for the top border of the picture.
    //
    // To do this we rotate/flip each of the remaining corners into the
    // orientation they'd be in if they were the top right corner and then try
    // to find a path between our top left's right edge and the potential
    // top right's left side.
    //
    // Once we find a match, we've got our top right corner.
    let mut top_right_with_path: Option<(usize, Tile, Vec<Tile>)> = None;
    for (idx, (corner, borders)) in corners_with_border_edges.iter().enumerate() {
        // There are 8 total configurations a tile can be in.
        //
        // t_ |+ --t _._| |_._ t-- +| _t
        //
        // Only two of these are suitable for being a top right corner.
        // --t +|
        //
        // A horizontal flip and a rotate right lets you convert between these.
        let mut top_right = corner.clone();

        assert_eq!(borders.len(), 2);
        let border1 = borders[0];
        let border2 = borders[1];

        // First we do what roughly what we did for the top left corner:
        top_right.adjust_to(border1, Side::Top).unwrap();
        if top_right.edge(Side::Left) == border2 {
            top_right.flip_horizontally();
        }
        if top_right.edge(Side::Right) != border2 {
            top_right.adjust_to(border2, Side::Top).unwrap();

            if top_right.edge(Side::Left) == border1 {
                top_right.flip_horizontally();
            }
            assert_eq!(top_right.edge(Side::Right), border1);
        }

        println!("BP from TL to TR. {} → {}", top_left.id, top_right.id);
        // Next we try to see if we get a path from the top left's right side
        // to this tile's left side:
        if let Some(path) = find_border_path(
            top_left.edge(Side::Right),
            top_left.id,
            top_right.edge(Side::Left),
            Side::Left,
            &mut displaced,
            &edge_map,
        ) {
            top_right_with_path = Some((idx, top_right, path));
            break;
        }

        // If that didn't work, try to flip horizontally and rotate right once
        // and try again:
        top_right.flip_horizontally();
        top_right.rotate_right();

        println!("BP from TL to TR, after TR rot.");
        // Again:
        if let Some(path) = find_border_path(
            top_left.edge(Side::Right),
            top_left.id,
            top_right.edge(Side::Left),
            Side::Left,
            &mut displaced,
            &edge_map,
        ) {
            top_right_with_path = Some((idx, top_right, path));
            break;
        }
    }

    // Now that we have our top right corner (and the top border) we also have
    // our width:
    let (top_right_idx, top_right, mut top_border) = top_right_with_path.unwrap();
    let width = top_border.len() + 2;

    // Which means we can place the top right corner and the top border:
    displaced.remove(&top_right.id);
    placed.insert((width - 1, 0), top_right);
    for (idx, tile) in top_border.drain(..).enumerate() {
        let res = placed.insert((idx + 1, 0), tile);
        assert!(res.is_none());
    }

    // And remove it from our list of remaining corners:
    corners_with_border_edges.remove(top_right_idx);

    // Next up we've got the bottom left corner; we've got to do the same steps
    // for this one:
    let mut bottom_left_with_path: Option<(usize, Tile, Vec<Tile>)> = None;
    for (idx, (corner, borders)) in corners_with_border_edges.iter().enumerate() {
        let mut bottom_left = corner.clone();

        assert_eq!(borders.len(), 2);
        let border1 = borders[0];
        let border2 = borders[1];

        bottom_left.adjust_to(border1, Side::Bottom).unwrap();
        if bottom_left.edge(Side::Right) == border2 {
            bottom_left.flip_horizontally();
        }
        if bottom_left.edge(Side::Left) != border2 {
            bottom_left.adjust_to(border2, Side::Bottom).unwrap();

            if bottom_left.edge(Side::Right) == border1 {
                bottom_left.flip_horizontally();
            }
            assert_eq!(bottom_left.edge(Side::Left), border1);
        }

        println!("BP from TL to BL. {} → {}", top_left.id, bottom_left.id);
        // Next we try to see if we get a path from the top left's bottom side
        // to this tile's top side:
        if let Some(path) = find_border_path(
            top_left.edge(Side::Bottom),
            top_left.id,
            bottom_left.edge(Side::Top),
            Side::Top,
            &mut displaced,
            &edge_map,
        ) {
            bottom_left_with_path = Some((idx, bottom_left, path));
            break;
        }

        // If that didn't work, try to flip horizontally and rotate right once
        // and try again:
        bottom_left.flip_horizontally();
        bottom_left.rotate_right();

        println!("BP from TL to BL, after BL rot.");
        // Again:
        if let Some(path) = find_border_path(
            top_left.edge(Side::Bottom),
            top_left.id,
            bottom_left.edge(Side::Top),
            Side::Top,
            &mut displaced,
            &edge_map,
        ) {
            bottom_left_with_path = Some((idx, bottom_left, path));
            break;
        }
    }

    // Now that we have our bottom left corner (and the top border) we also have
    // our height:
    let (bottom_left_idx, bottom_left, mut left_border) = bottom_left_with_path.unwrap();
    let height = left_border.len() + 2;

    // Which means we can place the bottom left corner and the left border:
    displaced.remove(&bottom_left.id);
    placed.insert((0, height - 1), bottom_left);
    for (idx, tile) in left_border.drain(..).enumerate() {
        let res = placed.insert((0, idx + 1), tile);
        assert!(res.is_none());
    }

    // And remove it from our list of remaining corners:
    corners_with_border_edges.remove(bottom_left_idx);

    // Okay! With three of our 4 corners placed, we can now just toss the
    // remaining corner back into the displaced list and map into the edge map
    // and just let _the "algorithm"_ do it's thing.
    assert_eq!(corners_with_border_edges.len(), 1);
    let (bottom_right, _) = corners_with_border_edges.pop().unwrap();

    // println!("\n\nPlaced:");
    // for (c, t) in placed.iter() {
    //     println!("({}, {}) ⇒ {}", c.0, c.1, t);
    // }

    let print = |g| {
        println!("Placed:\n{}", grid_to_string(g, height, width));
        3
    };
    print(&placed);

    displaced.insert(bottom_right.id, bottom_right.clone());
    for e in bottom_right.edges().iter() {
        edge_map.entry(*e).or_default().push(bottom_right.id);
    }

    // Now we just go and fill in the missing spaces. Since we've been assured
    // there are no "false" edge matches we can go in whatever direction we
    // want and we don't even have to check that _all_ the edges for a new
    // tile that we insert match.
    //
    // Note that up to this point we have not _really_ relied on this property;
    // if we were to continue to try to not rely on it here, we'd only place the
    // spots that have 1 tile match and continue to do so (while updating the
    // edge map) until we've exhausted the displaced tiles.
    //
    // Instead we'll fill in rows or columns while checking (for example, if we
    // went with rows) the left and top edges of each new placed tile.
    for x in 1..width {
        for y in 1..height {
            let left = placed.get(&(x - 1, y)).unwrap();
            let top = placed.get(&(x, y - 1)).unwrap();
            let expected_left_edge = left.edge(Side::Right);
            let expected_top_edge = top.edge(Side::Bottom);

            let left_matches: HashSet<_> =
                edge_map.get(&expected_left_edge).unwrap().iter().collect();
            let top_matches: HashSet<_> =
                edge_map.get(&expected_top_edge).unwrap().iter().collect();

            // println!(
            //     "for ({}, {}) (l: {}, t: {}), want:\n  - left: {:?}\n  - top: {:?}",
            //     x, y, left.id, top.id, expected_left_edge, expected_top_edge
            // );
            // println!(
            //     "left matches: {:?}\ntop matches: {:?}",
            //     left_matches, top_matches
            // );

            let mut matches = left_matches.intersection(&top_matches);
            let tile_idx = **matches.next().unwrap();
            assert!(matches.next().is_none());

            let mut matched = displaced.remove(&tile_idx).unwrap();

            // Should only need to do one of the two edges?
            // println!("\nMatched (pre-adj): {}", matched);
            matched.adjust_to(expected_left_edge, Side::Left).unwrap();
            matched.adjust_to(expected_top_edge, Side::Top).unwrap();

            if matched.edge(Side::Left) != expected_left_edge {
                matched.flip_horizontally();
            }

            // println!("\nMatched: {}", matched);

            let res = placed.insert((x, y), matched);
            // println!("Placed:\n{}", grid_to_string(&placed, height, width));
            assert!(res.is_none());
        }
    }

    // Finally, we'll just double check that we actually used all the tiles:
    assert!(displaced.is_empty());

    // Okay!
    let sea = Sea::new(&placed, height, width);

    #[rustfmt::skip] // No touch the monster!
    let monster = Monster::new(
        concat!(
            "                  # \n",
            "#    ##    ##    ###\n",
            " #  #  #  #  #  #   \n",
        )
    );

    // #[rustfmt::skip] // No touch the monster!
    // let monster = Monster::new(
    //     concat!(
    //         ".####",
    //         "#####",
    //     )
    // );
    println!("Sea:\n{}", sea);
    println!("Monster:\n{:?}", monster);

    let mut roughness = None;
    // We'll try each of the eight arrangements:
    for sea in [true, false]
        .iter()
        .cartesian_product(0..=3)
        .map(|(flip, rot)| {
            let mut sea = sea.clone();
            if *flip {
                sea.flip_vertically();
            }
            for _ in 0..rot {
                sea.rotate_right();
            }
            sea
        })
    {
        println!("Searching:\n{}\n\n", sea);
        if let Some((monsters, rough, sea)) = sea.search_for_monster(&monster) {
            println!(
                "Found {} monsters (roughness of {}) in this sea:\n{}",
                monsters, rough, sea
            );
            roughness = Some(rough);
            break;
        }
    }

    let p2: usize = roughness.unwrap();
    println!("{}", p2);
    let _ = aoc.submit_p2(p2);
}

fn grid_to_string(grid: &HashMap<(usize, usize), Tile>, height: usize, width: usize) -> String {
    let mut chars: Vec<Vec<char>> = vec![];

    for y in 0..height {
        for _ in 0..Tile::LEN {
            chars.push(vec![]);
        }
        let len = chars.len();
        let output_row = &mut chars[(len - Tile::LEN)..];
        for x in 0..width {
            if let Some(t) = grid.get(&(x, y)) {
                for (r, tile_row) in t.data.iter().enumerate() {
                    output_row[r].extend(tile_row);
                    output_row[r].push(' ');
                }
            } else {
                for i in 0..Tile::LEN {
                    output_row[i].extend([' '; Tile::LEN].iter());
                    output_row[i].push(' ');
                }
            }
        }
        chars.push(vec![]);
    }

    chars
        .iter()
        .flat_map(|l| l.iter().copied().chain(iter::once('\n')))
        .collect()
}

fn grid_to_string_degapped(
    grid: &HashMap<(usize, usize), Tile>,
    height: usize,
    width: usize,
) -> String {
    // let Sea { inner } = Sea::new(grid, height, width);

    format!("{}", Sea::new(grid, height, width))

    // inner
    //     .iter()
    //     .flat_map(|l| l.iter().copied().chain(iter::once('\n')))
    //     .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Monster {
    inner: Vec<Vec<char>>,
    height: usize,
    width: usize,
}

impl Monster {
    fn new(s: &str) -> Self {
        let mut inner = vec![];
        for line in s.lines() {
            inner.push(vec![]);
            let row = inner.last_mut().unwrap();
            row.extend(line.chars());
        }

        let width = inner.iter().map(|l| l.len()).max().unwrap();

        Self {
            height: inner.len(),
            width,
            inner,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Sea {
    inner: Vec<Vec<char>>,
    height: usize,
    width: usize,
}

impl Display for Sea {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{}",
            self.inner
                .iter()
                .flat_map(|l| l.iter().copied().chain(iter::once('\n')))
                .collect::<String>()
        )
    }
}

impl Sea {
    fn new(grid: &HashMap<(usize, usize), Tile>, height: usize, width: usize) -> Self {
        let mut chars: Vec<Vec<char>> = vec![];

        for y in 0..height {
            for _ in 0..(Tile::LEN - 2) {
                chars.push(vec![]);
            }
            let len = chars.len();
            let output_row = &mut chars[(len - (Tile::LEN - 2))..];
            for x in 0..width {
                if let Some(t) = grid.get(&(x, y)) {
                    for (r, tile_row) in t.data.iter().skip(1).take(Tile::LEN - 2).enumerate() {
                        output_row[r].extend(tile_row.iter().skip(1).take(Tile::LEN - 2));
                        // output_row[r].push(' ');
                    }
                } else {
                    for i in 0..Tile::LEN {
                        output_row[i].extend(['X'; Tile::LEN].iter());
                        // output_row[i].push(' ');
                    }
                }
            }
            // chars.push(vec![]);
        }

        Self {
            inner: chars,
            height: height * (Tile::LEN - 2),
            width: width * (Tile::LEN - 2),
        }
    }

    #[inline]
    fn transform(&mut self, idxes: impl Iterator<Item = (usize, usize)>) {
        // Also inefficient but it's 6AM and I do not care
        self.inner = idxes
            .collect::<Vec<_>>()
            .chunks(self.width)
            .map(|r| {
                r.iter()
                    .map(|(r, c)| self.inner[*r][*c])
                    .collect::<Vec<_>>()
            })
            .collect();
    }

    fn rotate_right(&mut self) {
        let Self { height, width, .. } = *self;

        self.height = width;
        self.width = height;

        self.transform(
            (0..width)
                .cartesian_product(0..height)
                .map(|(c, r)| (width - 1 - r, c)),
        );
    }

    fn flip_vertically(&mut self) {
        let Self { height, width, .. } = *self;

        self.transform(
            (0..width)
                .cartesian_product(0..height)
                .map(|(r, c)| (width - 1 - r, c)),
        );
    }

    // If monsters are found, returns the number of monsters and the roughness
    // and the (updated) Sea.
    fn search_for_monster(mut self, monster: &Monster) -> Option<(usize, usize, Self)> {
        if self.height < monster.height || self.width < monster.width {
            return None;
        }

        let mut monster_count = 0;

        for y in 0..(self.height - monster.height) {
            for x in 0..(self.width - monster.width) {
                let mut potentially_a_monster = true;
                for dy in 0..monster.height {
                    for dx in 0..monster.width {
                        if monster.inner[dy][dx] == '#' && self.inner[y + dy][x + dx] != '#' {
                            potentially_a_monster = false;
                        }
                    }
                }

                let actually_a_monster = potentially_a_monster;
                if actually_a_monster {
                    monster_count += 1;

                    for dy in 0..monster.height {
                        for dx in 0..monster.width {
                            if monster.inner[dy][dx] == '#' {
                                self.inner[y + dy][x + dx] = 'O';
                            }
                        }
                    }
                }
            }
        }

        if monster_count != 0 {
            let roughness = self
                .inner
                .iter()
                .map(|r| r.iter().filter(|s| **s == '#').count())
                .sum();

            Some((monster_count, roughness, self))
        } else {
            None
        }
    }
}

// A less dumb way to go about all of the above is just to pin down a corner and
// then just keep going right until you hit something with no border, same for
// below, etc.

#[cfg(test)]
mod tile_tests {
    use super::*;

    // Note the small cluster of 9s in the bottom left so that RIGHT and VERT
    // aren't the same.
    const TILE: Tile = Tile {
        data: [
            ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'],
            ['9', '0', '1', '2', '3', '4', '5', '6', '7', '8'],
            ['8', '9', '0', '1', '2', '3', '4', '5', '6', '7'],
            ['7', '8', '9', '0', '1', '2', '3', '4', '5', '6'],
            ['6', '7', '8', '9', '0', '1', '2', '3', '4', '5'],
            ['5', '6', '7', '8', '9', '0', '1', '2', '3', '4'],
            ['4', '5', '6', '7', '8', '9', '0', '1', '2', '3'],
            ['3', '4', '5', '6', '7', '8', '9', '0', '1', '2'],
            ['2', '3', '4', '5', '6', '7', '8', '9', '9', '9'],
            ['1', '2', '3', '4', '5', '6', '7', '8', '9', '9'],
        ],
        id: 0,
    };

    const RIGHT: Tile = Tile {
        data: [
            ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'],
            ['2', '3', '4', '5', '6', '7', '8', '9', '0', '1'],
            ['3', '4', '5', '6', '7', '8', '9', '0', '1', '2'],
            ['4', '5', '6', '7', '8', '9', '0', '1', '2', '3'],
            ['5', '6', '7', '8', '9', '0', '1', '2', '3', '4'],
            ['6', '7', '8', '9', '0', '1', '2', '3', '4', '5'],
            ['7', '8', '9', '0', '1', '2', '3', '4', '5', '6'],
            ['8', '9', '0', '1', '2', '3', '4', '5', '6', '7'],
            ['9', '9', '1', '2', '3', '4', '5', '6', '7', '8'],
            ['9', '9', '2', '3', '4', '5', '6', '7', '8', '9'],
        ],
        id: 0,
    };

    const HORIZ: Tile = Tile {
        data: [
            ['9', '8', '7', '6', '5', '4', '3', '2', '1', '0'],
            ['8', '7', '6', '5', '4', '3', '2', '1', '0', '9'],
            ['7', '6', '5', '4', '3', '2', '1', '0', '9', '8'],
            ['6', '5', '4', '3', '2', '1', '0', '9', '8', '7'],
            ['5', '4', '3', '2', '1', '0', '9', '8', '7', '6'],
            ['4', '3', '2', '1', '0', '9', '8', '7', '6', '5'],
            ['3', '2', '1', '0', '9', '8', '7', '6', '5', '4'],
            ['2', '1', '0', '9', '8', '7', '6', '5', '4', '3'],
            ['9', '9', '9', '8', '7', '6', '5', '4', '3', '2'],
            ['9', '9', '8', '7', '6', '5', '4', '3', '2', '1'],
        ],
        id: 0,
    };

    const VERT: Tile = Tile {
        data: [
            ['1', '2', '3', '4', '5', '6', '7', '8', '9', '9'],
            ['2', '3', '4', '5', '6', '7', '8', '9', '9', '9'],
            ['3', '4', '5', '6', '7', '8', '9', '0', '1', '2'],
            ['4', '5', '6', '7', '8', '9', '0', '1', '2', '3'],
            ['5', '6', '7', '8', '9', '0', '1', '2', '3', '4'],
            ['6', '7', '8', '9', '0', '1', '2', '3', '4', '5'],
            ['7', '8', '9', '0', '1', '2', '3', '4', '5', '6'],
            ['8', '9', '0', '1', '2', '3', '4', '5', '6', '7'],
            ['9', '0', '1', '2', '3', '4', '5', '6', '7', '8'],
            ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'],
        ],
        id: 0,
    };

    #[test]
    fn rotate_right() {
        let mut t = TILE;
        t.rotate_right();
        assert_eq!(t, RIGHT);
    }

    #[test]
    fn flip_horizontally() {
        let mut t = TILE;
        t.flip_horizontally();
        assert_eq!(t, HORIZ);
    }

    #[test]
    fn flip_vertically() {
        let mut t = TILE;
        t.flip_vertically();
        assert_eq!(t, VERT);
    }

    #[test]
    fn no_adj() {
        let mut t = TILE;
        t.adjust_to(
            Edge::new(['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']),
            Side::Top,
        )
        .unwrap();
        assert_eq!(t, TILE);
    }

    #[test]
    fn adj_rot_right() {
        let mut t = TILE;
        t.adjust_to(
            Edge::new(['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']),
            Side::Right,
        )
        .unwrap();
        assert_eq!(t, RIGHT);
    }

    #[test]
    fn adj_top_flip() {
        let mut t = Tile {
            data: [
                ['.', '.', '.', '#', '.', '#', '.', '#', '.', '#'],
                ['#', '#', '#', '#', '.', '#', '.', '.', '.', '.'],
                ['.', '.', '#', '.', '#', '.', '.', '.', '.', '.'],
                ['.', '.', '.', '.', '#', '.', '.', '#', '.', '#'],
                ['.', '#', '#', '.', '.', '#', '#', '.', '#', '.'],
                ['.', '#', '.', '#', '#', '#', '#', '.', '.', '.'],
                ['#', '#', '#', '#', '.', '#', '.', '#', '.', '.'],
                ['#', '#', '.', '#', '#', '#', '#', '.', '.', '.'],
                ['#', '#', '.', '.', '#', '.', '#', '#', '.', '.'],
                ['#', '.', '#', '#', '.', '.', '.', '#', '#', '.'],
            ],
            id: 0,
        };
        t.adjust_to(
            Edge::new(['#', '.', '#', '#', '.', '.', '.', '#', '#', '.']),
            Side::Top,
        )
        .unwrap();

        assert_eq!(
            t,
            Tile {
                data: [
                    ['#', '.', '#', '#', '.', '.', '.', '#', '#', '.'],
                    ['#', '#', '.', '.', '#', '.', '#', '#', '.', '.'],
                    ['#', '#', '.', '#', '#', '#', '#', '.', '.', '.'],
                    ['#', '#', '#', '#', '.', '#', '.', '#', '.', '.'],
                    ['.', '#', '.', '#', '#', '#', '#', '.', '.', '.'],
                    ['.', '#', '#', '.', '.', '#', '#', '.', '#', '.'],
                    ['.', '.', '.', '.', '#', '.', '.', '#', '.', '#'],
                    ['.', '.', '#', '.', '#', '.', '.', '.', '.', '.'],
                    ['#', '#', '#', '#', '.', '#', '.', '.', '.', '.'],
                    ['.', '.', '.', '#', '.', '#', '.', '#', '.', '#'],
                ],
                id: 0
            }
        )
    }

    #[test]
    fn right_edge() {
        let mut t = Tile {
            data: [
                ['.', '#', '#', '.', '.', '.', '.', '#', '#', '#'],
                ['.', '#', '.', '#', '#', '.', '#', '#', '.', '#'],
                ['.', '#', '#', '.', '#', '#', '#', '.', '.', '#'],
                ['.', '#', '#', '.', '.', '#', '#', '#', '#', '#'],
                ['.', '.', '.', '.', '.', '#', '.', '.', '#', '.'],
                ['#', '.', '#', '#', '.', '.', '#', '.', '.', '.'],
                ['#', '#', '#', '#', '.', '#', '#', '#', '#', '.'],
                ['.', '.', '#', '#', '#', '#', '#', '.', '.', '#'],
                ['.', '.', '#', '#', '#', '#', '#', '#', '.', '#'],
                ['.', '.', '.', '.', '.', '#', '.', '.', '#', '.'],
            ],
            id: 0,
        };

        assert_eq!(
            t.edge(Side::Right).actual(),
            ['#', '#', '#', '#', '.', '.', '.', '#', '#', '.']
        );
    }

    #[test]
    fn right_edge_after_adj() {
        let mut t = Tile {
            data: [
                ['.', '#', '#', '.', '.', '.', '#', '#', '#', '#'],
                ['#', '.', '.', '#', '.', '#', '#', '.', '.', '#'],
                ['.', '#', '.', '#', '.', '.', '#', '.', '#', '#'],
                ['.', '#', '#', '#', '#', '.', '#', '#', '#', '.'],
                ['#', '#', '#', '#', '.', '#', '#', '#', '.', '.'],
                ['.', '#', '#', '.', '.', '.', '.', '#', '#', '.'],
                ['.', '#', '#', '#', '#', '.', '.', '.', '#', '.'],
                ['.', '#', '#', '#', '#', '.', '#', '#', '.', '#'],
                ['.', '.', '.', '#', '.', '.', '#', '#', '#', '#'],
                ['.', '.', '.', '#', '#', '.', '.', '.', '.', '.'],
            ],
            id: 0,
        };

        t.adjust_to(
            ['.', '.', '.', '#', '#', '.', '.', '.', '.', '.'],
            Side::Left,
        )
        .unwrap();

        assert_eq!(
            t.edge(Side::Right).actual(),
            ['.', '#', '#', '.', '.', '.', '#', '#', '#', '#']
        );
    }
}
