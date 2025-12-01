use aoc::{iterator_map_ext::IterMapExt, three_dimensional::CubeFromPairOfTriples, AdventOfCode};

fn main() {
    let mut aoc = AdventOfCode::new(2023, 22);
    let inp = aoc.get_input();
    let bricks = inp
        .lines()
        .map_parse()
        .map(CubeFromPairOfTriples::<'~', true, usize>::get);

    // let map = HashMap::with_capacity(300 * 10 * 10);

    // give each brick a number
    //
    // populate a hashmap with all the top face coords of each brick
    //
    // for each brick, take the bottom face
    //   - subtract (0, 0, 1) from each coord
    //   - record all the other bricks this touches
    //   - this yields a [brick idx -> [list of supporting brick idxes]] list
    //
    // search the previous list for bricks that are solely dependent on a single
    // brick; subtract from the total number of bricks
}
