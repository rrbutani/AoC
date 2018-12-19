#!/usr/bin/env rustr
#![feature(nll)]
#![feature(if_while_or_patterns)]

extern crate aoc;

#[allow(unused_imports)]
use aoc::{AdventOfCode, friends::*};
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display, Formatter};

#[derive(Copy, Clone, Debug)]
enum TurnHistory {
    Left,
    Straight,
    Right,
}

impl TurnHistory {
    fn new() -> Self {
        TurnHistory::Right
    }

    fn next(&mut self) -> &Self {
        use self::TurnHistory::*;
        *self = match *self {
            Left => Straight,
            Straight => Right,
            Right => Left,
        };

        self
    }
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    /// '^'
    Up,
    /// '>'
    Right,
    /// 'v'
    Down,
    /// '<'
    Left,
}

impl Direction {
    fn intersection_turn(self, turn_history: &mut TurnHistory) -> Self {
        use self::TurnHistory::*;
        // use self::TurnHistory as T;
        // use self::Direction as D;
        match (self, turn_history.next()) {
            (dir, Straight) => dir,
            (dir, Right) => dir.clockwise(),
            (dir, Left) => dir.counterclockwise(),
            // (D::Up, T::Right) => D::Right, // cw
            // (D::Up, T::Left) => D::Left, // ccw
            // (D::Down, T::Right) => D::Left, // cw
            // (D::Down, T::Left) => D::Right, // ccw
            // (D::Right, T::Right) => D::Down, // cw
            // (D::Right, T::Left) => D::Up, // ccw
            // (D::Left, T::Right) => D::Up, // cw
            // (D::Left, T::Left) => D::Down, // ccw
        }
    }

    fn clockwise(self) -> Self {
        use self::Direction::*;
        match self {
            Up => Right,
            Right => Down,
            Down => Left,
            Left => Up,
        }
    }

    fn counterclockwise(self) -> Self {
        use self::Direction::*;
        match self {
            Up => Left,
            Right => Up,
            Down => Right,
            Left => Down,
        }
    }
}

type TrackId = usize;

#[derive(Copy, Clone, Debug)]
struct Cart {
    valid: bool,
    track_id: TrackId,
    dir: Direction,
    turn_history: TurnHistory,
}

impl Cart {
    fn new(track_id: TrackId, dir: Direction) -> Self {
        Cart {
            valid: true,
            track_id,
            dir,
            turn_history: TurnHistory::new(),
        }
    }

    /// None if there is no collision
    /// Some(Position) if there is
    fn step(&mut self, tracks: &mut std::vec::Vec<TrackSegment>) -> Option<Position> {
        let current_track = &mut tracks[self.track_id];
        
        if !self.valid {
            current_track.occupied = false;
            return None
        }

        // Get direction to move in (based on our _current_ track)
        use self::Direction::*;
        use self::TrackType::*;

        // Get the next track based on our direction:
        let id = if let Some(TrackLink::Id(id)) = match self.dir {
            Up => current_track.above,
            Right => current_track.right,
            Down => current_track.below,
            Left => current_track.left,
        } { id }
        else { println!("{:?} -> {:?}", self, current_track); panic!("ahh!") };

        current_track.occupied = false;
        self.track_id = id;

        let next_track = &mut tracks[id];
        if next_track.occupied { next_track.occupied = false; return Some(next_track.pos) }
        else { next_track.occupied = true }


        self.dir = match (next_track.track_type, self.dir) {
            (Vertical, dir) | (Horizontal, dir) => { dir },
            (TopRight, dir @ Right) | (TopLeft, dir @ Up) | (BottomLeft, dir @ Left) | (BottomRight, dir @ Down) => { dir.clockwise() },
            (TopRight, dir @ Up) | (TopLeft, dir @ Left) | (BottomLeft, dir @ Down) | (BottomRight, dir @ Right) => { dir.counterclockwise() },
            (Intersection, dir) => { dir.intersection_turn(&mut self.turn_history) },
            // (Vertical, Left) | (Vertical, Right) | (Horizontal, Up) | (Horizontal, Down) => unreachable!(),
            // (TopRight, Left) | (TopRight, Down) | (TopLeft, Right) | (TopLeft, Down) => unreachable!(),
            // (BottomRight, Left) | (BottomRight, Up) | (BottomLeft, Right) | (BottomLeft, Up) => unreachable!(),
            _ => unreachable!(),
        };

        // match (current_track.track_type, cart_dir) {
        //     (Vertical, Up) => {
        //         current_track.occupied = false;
        //         let id = if let Some(TrackLink::Id(id)) = current_track.above {
        //             id
        //         } else { panic!("ahh!") };

        //         let next_track = &mut tracks[id];
        //         next_track.occupied = true;
        //         self.track_id = id;
        //         self.dir = 
        //     },
        //     (Vertical, Down) => {},
        //     (Horizontal, Left) => {},
        //     (Horizontal, Right) => {},
        //     (TopRight, Right) => {},
        //     (TopRight, Up) => {},
        //     (TopLeft, Left) => {},
        //     (TopLeft, Up) => {},
        //     (BottomRight, Right) => {},
        //     (BottomRight, Down) => {},
        //     (BottomLeft, Left) => {},
        //     (BottomLeft, Down) => {},
        //     (Intersection, dir) => {},
        //     (Vertical, Left) | (Vertical, Right) | (Horizontal, Up) | (Horizontal, Down) => unreachable!(),
        //     (TopRight, Left) | (TopRight, Down) | (TopLeft, Right) | (TopLeft, Down) => unreachable!(),
        //     (BottomRight, Left) | (BottomRight, Up) | (BottomLeft, Right) | (BottomLeft, Up) => unreachable!(),
        // }

        None
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
struct Position {
    x: usize,
    y: usize,
}

impl Display for Position {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{},{}", self.x, self.y)
    }
}

impl Position {
    fn with_offset(&self, x_adj: i8, y_adj: i8) -> Option<Self> {
        // This is lazy but good enough for now..

        if (self.x as isize + x_adj as isize) < 0 ||
                (self.y as isize + y_adj as isize) < 0 {
            None
        } else {
            Some(Position {
                x: (self.x as isize + x_adj as isize) as usize,
                y: (self.y as isize + y_adj as isize) as usize,
            })
        }
    }

    fn shift(&mut self, x_adj: i8, y_adj: i8) {
        self.x = ((self.x as isize) + (x_adj as isize)) as usize;
        self.y = ((self.y as isize) + (y_adj as isize)) as usize;
    }
}

#[derive(Copy, Clone, Debug)]
enum TrackLink {
    Position(Position),
    Id(TrackId)
}

impl TrackLink {
    fn into_id(&self, hm: &HashMap<Position, (TrackId, TrackType)>) -> Option<Self> {
        match self {
            TrackLink::Position(pos) => {
                hm.get(&pos).map(|(i, _)| TrackLink::Id(*i))
            },
            a => Some((*a).clone())
        }
    }

    fn from_pos(base: &Position, x_adj: i8, y_adj: i8) -> Option<Self> {
        base.with_offset(x_adj, y_adj).map(|p| TrackLink::Position(p))
    }
}

#[derive(Copy, Clone, Debug)]
enum TrackType {
    /// '|'
    Vertical,
    /// '-'
    Horizontal,
    /// '/'
    TopRight,
    /// '\'
    TopLeft,
    /// '/'
    BottomRight,
    /// '\'
    BottomLeft,
    /// '+'
    Intersection,
}

#[derive(Copy, Clone, Debug)]
struct TrackSegment {
    above: Option<TrackLink>,
    right: Option<TrackLink>,
    below: Option<TrackLink>,
    left: Option<TrackLink>,
    track_type: TrackType,
    pos: Position,
    id: TrackId,
    occupied: bool,
}

/// If tl is None -> Some(())
/// If tl is Some that maps to an id -> Some(())
/// If tl is Some that doesn't map -> None
fn option_to_id(tl: &mut Option<TrackLink>, hm: &HashMap<Position, (TrackId, TrackType)>) -> Option<()> {
    match tl {
        Some(p) => {
            *tl = Some(p.into_id(hm)?);
            Some(())
        },
        None => Some(()),
    }
}

impl TrackSegment {
    fn new(track_type: TrackType, id: TrackId, pos: Position, occupied: bool) -> Self {
        let above = TrackLink::from_pos(&pos, 0, -1);
        let right = TrackLink::from_pos(&pos, 1,  0);
        let below = TrackLink::from_pos(&pos, 0,  1);
        let left = TrackLink::from_pos(&pos, -1,  0);

        let s = |o: Option<TrackLink>| Some(o.unwrap());

        use self::TrackType::*; // All of the 4 C 2 + Intersection:
        let (above, right, below, left) = match track_type {
            Vertical => (s(above), None, s(below), None),
            Horizontal => (None, s(right), None, s(left)),
            TopRight => (None, None, s(below), s(left)),
            TopLeft => (None, s(right), s(below), None),
            BottomRight => (s(above), None, None, s(left)),
            BottomLeft => (s(above), s(right), None, None),
            Intersection => (s(above), s(right), s(below), s(left)),
        };

        TrackSegment {
            track_type,
            pos,
            id: id,
            occupied,
            above,
            right,
            below,
            left,
        }
    }

    fn to_id(&mut self, hm: &HashMap<Position, (TrackId, TrackType)>) -> Option<()> {
        option_to_id(&mut self.above, hm)
            .and_then(|_| option_to_id(&mut self.right, hm))
            .and_then(|_| option_to_id(&mut self.below, hm))
            .and_then(|_| option_to_id(&mut self.left, hm))
    }
}

fn parse_map(map: Vec<Vec<char>>) -> Option<(Vec<TrackSegment>, Vec<Cart>)> {
    let mut tracks = Vec::new();
    let mut carts = Vec::new();

    let mut hm = HashMap::<Position, (TrackId, TrackType)>::new();

    // Make the track segments:
    for (y, row) in map.iter().enumerate() {
        for (x, c) in row.iter().enumerate() {
            let pos = Position { x, y };

            use self::TrackType::*;
            let track_type = match *c {
                '|' | '^' | 'v' => Vertical,
                '-' | '<' | '>' => Horizontal,
                '+' => Intersection,
                '\\' | '/' => {
                    // We need some context; if there's a vertical track or an intersection above
                    // us we're a Bottom* track, otherwise Top*.
                    if let Some((_, Vertical)) | Some((_, Intersection)) = pos.with_offset(0, -1).as_ref().and_then(|p| hm.get(p)) {
                        match *c { '\\' => BottomLeft, '/' => BottomRight, _ => unreachable!() }
                    } else {
                        match *c { '\\' => TopRight, '/' => TopLeft, _ => unreachable!() }
                    }
                },
                _ => continue
            };

            let id = tracks.len();
            let occupied = match *c {
                '^' | '>' | 'v' | '<' => {

                    use self::Direction::*;
                    let dir = match *c {
                        '^' => Up,
                        '>' => Right,
                        'v' => Down,
                        '<' => Left,
                        _ => unreachable!()
                    };

                    carts.push(Cart::new(id, dir));
                    true
                },
                _ => false
            };

            tracks.push(TrackSegment::new(track_type, id, pos, occupied));
            hm.insert(pos, (id, track_type));
        }
    }


    // And now link them up:
    for ts in tracks.iter_mut() {
        ts.to_id(&hm)?
        // let res = ts.to_id(&hm);

        // if let None = res {
        //     println!("{:?}", ts);
        //     return None;
        // }
    }

    Some((tracks, carts))
}

fn print_map(tracks: &Vec<TrackSegment>, carts: &Vec<Cart>) {
    let (mut row, mut col) = (0, 0);

    for track in tracks {
        let pos = track.pos;

        while pos.y > row { println!(""); row += 1; col = 0; }
        while pos.x > col { print!(" "); col += 1 }

        if track.occupied {
            // Find the cart:
            use self::Direction::*;
            print!("{}", match
                carts.iter()
                    .filter(|c| c.track_id == track.id)
                    .next()
                    .unwrap()
                    .dir
            {
                Up => '^',
                Right => '>',
                Down => 'v',
                Left => '<',
            });
        }
        else {
            use self::TrackType::*;
            print!("{}", match track.track_type {
                Vertical => '|',
                Horizontal => '-',
                TopLeft | BottomRight => '/',
                TopRight | BottomLeft => '\\',
                Intersection => '+',
            })
        }
        col += 1;
    }

    println!("\n");
}

fn sort_carts<'a>(tracks: &Vec<TrackSegment>, carts: &'a mut Vec<Cart>) -> Vec<&'a mut Cart> {
    let mut carts_sorted: Vec<&mut Cart> = carts.iter_mut().collect();

    carts_sorted.sort_by(|c1, c2| {
            let p1 = tracks[c1.track_id].pos;
            let p2 = tracks[c2.track_id].pos;

            (p1.y, p1.x).cmp(&(p2.y, p2.x))
    });

    carts_sorted
}

fn run_until_collision(tracks: &mut Vec<TrackSegment>, carts: &mut Vec<Cart>) -> Position {
    loop {

        let mut carts_sorted = sort_carts(tracks, carts);

        for cart in carts_sorted.iter_mut() {
            if let Some(position) = cart.step(tracks) {
                // print_map(tracks, carts);

                return position;
            }
        }
        // print_map(tracks, carts);
        // println!("{:#?}", carts);
    }
}

fn til_the_end(tracks: &mut Vec<TrackSegment>, carts: &mut Vec<Cart>) -> Position {
    while carts.len() > 1 {
        // println!("{} carts left", carts.len());
        // for c in carts.iter() { let pos = tracks[c.track_id].pos; println!("cart at {}, {}", pos.x, pos.y);}
        // print_map(tracks, carts);
        let mut carts_sorted = sort_carts(tracks, carts);

        let mut collision_positions = HashSet::new();

        for cart in carts_sorted.iter_mut() {
            let position = tracks[cart.track_id].pos;

            if collision_positions.contains(&position) {
                cart.valid = false;
            }

            if let Some(position) = cart.step(tracks) {
                // // This is a bad way:
                // carts.retain(|c| tracks[c.track_id].pos != position)
                cart.valid = false;
                collision_positions.insert(position);
                // // Find the cart it collided with:
                // carts.iter_mut().filter(|c| tracks[c.track_id].pos == position).next().unwrap().valid = false;
            }
        }

        for cart in carts_sorted.iter_mut() {
            let position = tracks[cart.track_id].pos;

            if collision_positions.contains(&position) {
                cart.valid = false;
            }
        }

        // Cull the carts:
        carts.retain(|c| c.valid);
    }

    tracks[carts[0].track_id].pos
}

#[allow(unused_must_use)]
fn main() {
    let mut aoc = AdventOfCode::new_with_year(2018, 13);
    let input: String = aoc.get_input();

    let input: Vec<Vec<char>> = input.lines().map(|l| l.chars().collect()).collect();

    let (mut tracks, mut carts) = parse_map(input).unwrap();

    let collision = run_until_collision(&mut tracks.clone(), &mut carts.clone());

    aoc.submit_p1(collision);

    let last_one_standing = til_the_end(&mut tracks, &mut carts);

    aoc.submit_p2(last_one_standing);
}


/*
enum TurnHistory {
    Left,
    StraightFirst,
    Right,
    StraightSecond,
}

impl TurnHistory {
    fn new() -> Self {
        TurnHistory::Left
    }

    fn next(&mut self) {
        use self::TurnHistory::*;
        *self = match *self {
            Left => StraightFirst,
            StraightFirst => Right,
            Right => StraightSecond,
            StraightSecond => Left,
        }
    }
}

enum Direction {
    Clockwise,
    Counterclockwise,
    Unknown(char),
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct Position {
    x: usize,
    y: usize,
}

struct Cart {
    dir: Direction,
    turn_history: TurnHistory,
    current_loc_id: usize,
}

impl Cart {
    fn new(current_loc_id: usize, dir: char) -> Self {
        Cart {
            dir: Direction::Unknown(dir),
            turn_history: TurnHistory::new(),
            current_loc_id,
        }
    }

    fn set_direction(&mut self, dir: Direction) -> &Self {
        self.dir = dir;
        self
    }
}

enum TrackLink {
    Position(Position),
    Id(usize),
}

impl Default for TrackLink {
    fn default() -> Self {
        TrackLink::Position(Position { x: 0, y: 0 })
    }
}

impl TrackLink {
    fn from_id(id: usize) -> Self {
        TrackLink::Id(id)
    }

    fn from_position(x: usize, y: usize) -> Self {
        TrackLink::Position(Position { x, y })
    }

    fn convert_to_id(self, hm: HashMap<Position, (usize, Direction)>) -> Option<Self> {
        match self {
            TrackLink::Position(p) => {
                hm.get(&p).and_then(|(id, _)| Some(TrackLink::Id(*id)))
            },
            _ => Some(self)
        }
    }
}

#[derive(Default)]
struct Intersection {
    ccw: TrackLink,
    cw: TrackLink,
    linked_cw: TrackLink,
    linked_ccw: TrackLink,
}

impl Intersection {
    fn new(ccw: TrackLink, cw: TrackLink, linked_cw: TrackLink, linked_ccw: TrackLink) -> Self {
        Intersection {
            ccw,
            cw,
            linked_cw,
            linked_ccw,
        }
    }
}

#[derive(Default)]
struct Regular {
    ccw: TrackLink,
    cw: TrackLink,
}

impl Regular {
    fn new(ccw: TrackLink, cw: TrackLink) -> Self {
        Regular {
            ccw,
            cw,
        }
    }
}

enum TrackType {
    Regular(Regular),
    Intersection(Intersection),
}

impl TrackType {
    fn new_regular_track() -> Self {
        TrackType::Regular(Regular::default())
    }
    
    fn default_regular_track() -> Self {
        TrackType::Regular(Regular::default())
    }

    fn new_intersection_track() -> Self {
        TrackType::Intersection(Intersection::default())
    }
}

struct TrackSegment {
    pos: Position,
    id: usize,
    track_type: TrackType,
    occupied: bool,
}

fn parse_map(map: Vec<Vec<char>>) -> (Vec<TrackSegment>, Vec<Cart>) {
    let mut tracks = Vec::new();
    let mut carts = Vec::new();

    let mut hm = HashMap::<Position, (usize, Direction, char)>::new();

    for (y, row) in map.iter().enumerate() {
        for (x, c) in row.iter().enumerate() {
            if *c == ' ' { continue }

            let pos = Position { x, y };
            let id = tracks.len();

            let track_type = match *c {
                // Just a regular track:
                '/' | '\\' | '|' | '-' |
                // Actually a cart, but has a regular track underneath it!
                '>' | 'v' | '<' | '^' =>  {
                    TrackType::new_regular_track()
                },

                // A fancy track:
                '+' => TrackType::new_intersection_track(),

                c => panic!(format!("Unexpected character: {}", c)),
            };

            let direction = match *c {
                '/' | '\\' => {
                    let ((ccw_x, ccw_y), (cw_x, cw_y)) = match
                        (*c, if y == 0 { None } else {
                            hm.get(&Position { x, y: y - 1 })
                        })
                    {
                        // Bottom right:
                        ('/', Some((_, _, '|'))) => ((x, y - 1), (x - 1, y)),
                        // Top left:
                        ('/', _) => ((x, y + 1), (x + 1, y)),
                        // Bottom left:
                        ('\\', Some((_, _, '|'))) => ((x + 1, y), (x, y - 1)),
                        // Top right:
                        ('\\', _) => ((x - 1, y), (x, y + 1)),
                        _ => unreachable!()
                    };

                    TrackType::Regular(Regular::new(
                        TrackLink::from_position(ccw_x, ccw_y),
                        TrackLink::from_position(cw_x, cw_y),
                    ))
                },

            };

            let occupied = match *c {
                '>' | 'v' | '<' | '^' => true,
                _ => false,
            };

            tracks.push(TrackSegment { pos, id, track_type, occupied });
            hm.insert(pos, id);
        }
    }

    // Now we get to link the track segments:
    for ts in tracks.iter_mut() {
        match ts.track_type {
            TrackType::Regular(ref mut r) => {

            },
            TrackType::Intersection(ref i) => {

            }
        }
    }

    (tracks, carts)
}
*/