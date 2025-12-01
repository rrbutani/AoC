use aoc::*;
use grid::{
    Direction::{self, self as D},
    TwoDimensionalGrid,
};

//------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::Display)]
enum Cell {
    #[strum(to_string = ".")]
    Empty,
    #[strum(to_string = "#")]
    Wall,
    #[strum(to_string = "O")]
    Box,
    #[strum(serialize = "@", to_string = "\u{001b}[34m@\u{001b}[0m")]
    Robot,
}

type Map = TwoDimensionalGrid<Cell>;

fn try_move_box(map: &mut Map, coord: Coord, dir: Direction) -> Option<()> {
    use Cell::*;
    debug_assert_eq!(map[coord], Cell::Box);

    let further = dir.apply(coord)?;
    let curr_contents = map.get(further)?;

    match curr_contents {
        Empty => {} // great, can move
        Wall => return None,
        Box => try_move_box(map, further, dir)?, // need to recurse..
        Robot => unreachable!(),
    }

    // okay, we can move!
    debug_assert_eq!(map[further], Cell::Empty);
    (map[further], map[coord]) = (Cell::Box, Cell::Empty);

    Some(())
}

//------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::Display)]
enum WideCell {
    #[strum(to_string = ".")]
    Empty,
    #[strum(to_string = "#")]
    Wall,
    #[strum(to_string = "[")]
    BoxLeft,
    #[strum(to_string = "]")]
    BoxRight,
    #[strum(serialize = "@", to_string = "\u{001b}[34m@\u{001b}[0m")]
    Robot,
}

type WideMap = TwoDimensionalGrid<WideCell>;

impl From<WideCell> for Cell {
    fn from(value: WideCell) -> Self {
        use {Cell as C, WideCell as W};
        match value {
            W::Empty => C::Empty,
            W::Wall => C::Wall,
            W::BoxLeft | W::BoxRight => C::Box,
            W::Robot => C::Robot,
        }
    }
}

fn try_move_wide_box(map: &mut WideMap, coord: Coord, dir: D) -> Option<()> {
    use WideCell::*;

    let (left, right) = match map[coord] {
        BoxLeft => (coord, D::Right.apply(coord).unwrap()),
        BoxRight => (D::Left.apply(coord).unwrap(), coord),
        other => panic!("got {other:?}"),
    };
    debug_assert_eq!(map[left], BoxLeft);
    debug_assert_eq!(map[right], BoxRight);

    // expand outwards from our left and right box coords to find all boxes that
    // we'd need to move:
    let (mut viable, mut affected) = (true, Vec::new());
    map.floodfill([left, right], |ctx, _| {
        // NOTE: we are relying on the map having a walls on the whole perimeter
        match ctx.cell() {
            Empty => None, // all is well, can stop expanding
            Wall => {
                // hit an obstacle! this isn't going to work
                viable = false;
                None
            }
            this @ (BoxLeft | BoxRight) => {
                // record coord for rewriting, keep expanding
                affected.push((ctx.coord(), *this));

                // in addition to expanding in the direction we're being pushed,
                // we also need to expand to include our other box half:
                let other = match this {
                    BoxLeft => D::Right,
                    BoxRight => D::Left,
                    _ => unreachable!(),
                };
                Some(SmallVec::from_iter([(dir, ()), (other, ())]))
            }
            Robot => unreachable!("can't wrap back around to the robot"),
        }
    });

    if !viable {
        return None;
    }

    // sort the movements w.r.t. the direction we're moving so that we don't
    // trample previous moves:
    affected.sort_by(|(a, _), (b, _)| {
        let (Coord { row: ay, col: ax }, Coord { row: by, col: bx }) = (a, b);
        match dir {
            D::North => (ay, ax).cmp(&(by, bx)), // do upper coords first
            D::South => (ay, ax).cmp(&(by, bx)).reverse(), // lower first
            D::East => (ax, ay).cmp(&(bx, by)).reverse(), // right first
            D::West => (ax, ay).cmp(&(bx, by)),  // left first
        }
    });

    for &(old_coord, val) in affected.iter() {
        let new = dir.apply(old_coord).unwrap();
        debug_assert_eq!(map[new], Empty);
        debug_assert_eq!(map[old_coord], val);

        (map[new], map[old_coord]) = (map[old_coord], Empty);
    }

    // // do all the movements; important that we apply these in reverse order
    // // so the "set to empty"s don't trample other moves
    // for &(old_coord, val) in affected.iter().rev() {
    //     let new = dir.apply(old_coord).unwrap();
    //     debug_assert_eq!(map[new], Empty);
    //     debug_assert_eq!(map[old_coord], val);
    //     (map[new], map[old_coord]) = (val, Empty);
    // }
    // TODO: why doesn't this work?

    Some(())
}

//------------------------------------------------------------------------------

fn apply_moves_and_calculate_score<C: Copy + Into<Cell> + Display + Eq>(
    map: &mut TwoDimensionalGrid<C>,
    handle_box: fn(&mut TwoDimensionalGrid<C>, Coord, Direction) -> Option<()>,
    moves: impl Iterator<Item = Direction>,
    cell_for_gps_score: C,
) -> usize {
    use Cell::*;
    let mut robot_pos = map.find(|&c| c.into() == Cell::Robot).next().unwrap();

    for m in moves {
        // check if coord is in bounds (left/top), first
        let Some(new) = m.apply(robot_pos) else {
            continue;
        };

        // check if coord is actually in bounds/what's currently there:
        let Some(&contents) = map.get(new) else {
            continue;
        };

        match contents.into() {
            Empty => {}       // all good, can move here
            Wall => continue, // can't move into a wall
            Box => {
                let Some(()) = handle_box(map, new, m) else {
                    continue;
                };
            }
            Robot => unreachable!(),
        }

        debug_assert_eq!(map[robot_pos].into(), Robot);
        debug_assert_eq!(map[new].into(), Empty);

        (map[new], map[robot_pos]) = (map[robot_pos], map[new]);
        robot_pos = new;

        #[cfg(debug_assertions)]
        eprintln!("{map}");
    }

    #[cfg(debug_assertions)]
    eprintln!("{map}");

    // calculate GPS score
    map.find(|&c| c == cell_for_gps_score)
        .map(|Coord { row, col }| row * 100 + col)
        .sum()
}

//------------------------------------------------------------------------------

fn p1(mut map: Map, moves: impl Iterator<Item = Direction>) -> usize {
    apply_moves_and_calculate_score(&mut map, try_move_box, moves, Cell::Box)
}

fn p2(map: &Map, moves: impl Iterator<Item = Direction>) -> usize {
    use {Cell as C, WideCell::*};

    // map to wide map
    let mut map = WideMap::new_from_iter(map.iter().map(|r| {
        r.iter().flat_map(|&c| match c {
            C::Empty => [Empty, Empty],
            C::Wall => [Wall, Wall],
            C::Box => [BoxLeft, BoxRight],
            C::Robot => [Robot, Empty],
        })
    }));

    apply_moves_and_calculate_score(&mut map, try_move_wide_box, moves, BoxLeft)
}

fn main() {
    let mut aoc = AdventOfCode::new(2024, 15);
    let inp = aoc.get_input();

    let (map, moves) = inp.split_once("\n\n").unwrap();
    let map = Map::from_str(map).unwrap();
    let moves = moves.chars().filter_map(|c| {
        Some(match c {
            '^' => D::Up,
            '>' => D::Right,
            'v' => D::Down,
            '<' => D::Left,
            '\n' => return None,
            other => panic!("unexpected: '{other}'"),
        })
    });

    _ = aoc.submit_p1(p1(map.clone(), moves.clone()));
    _ = aoc.submit_p2(p2(&map, moves));
}
