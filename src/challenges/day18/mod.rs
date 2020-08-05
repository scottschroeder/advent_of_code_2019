use self::keys::{Key, KeySet};
use anyhow::Result;

mod graph;
mod keys;
mod map_reader;

pub fn part1(input: &str) -> Result<String> {
    let m = map_reader::Map::parse(input);
    let g = graph::CaveGraph::from_map(m);
    log::trace!("\n{}", g.dot());
    let start = graph::SingleState {
        pos: g.start().nth(0).unwrap(),
        length: 0,
        keys: KeySet::new(),
    };
    let shortest = g.shortest_path(start).unwrap();
    Ok(format!("{}", shortest))
}

pub fn part2(input: &str) -> Result<String> {
    let mut m = map_reader::Map::parse(input);
    m.split_map()?;
    log::trace!("\n{}", m);
    let g = graph::CaveGraph::from_map(m);
    log::trace!("\n{}", g.dot());
    let mut start = graph::QuadState::default();
    for (idx, pos) in g.start().enumerate().take(4) {
        start.pos[idx] = pos;
    }
    let shortest = g.shortest_path(start).unwrap();
    Ok(format!("{}", shortest))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Tile {
    Wall,
    Space,
    Start,
    Door(Key),
    Key(Key),
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    pub(crate) const EXAMPLES: [&str; 5] = [DAY18_EX1, DAY18_EX2, DAY18_EX3, DAY18_EX4, DAY18_EX5];
    pub(crate) const EXAMPLES2: [&str; 4] = [DAY18_EX6, DAY18_EX7, DAY18_EX8, DAY18_EX9];

    pub(crate) const ANSWERS: [u32; 5] = [8, 86, 132, 136, 81];
    pub(crate) const ANSWERS2: [u32; 4] = [8, 24, 32, 72];

    #[test]
    fn day18part1() {
        assert_eq!(part1(DAY18_INPUT).unwrap().as_str(), "2684")
    }

    #[test]
    fn day18part2() {
        assert_eq!(part2(DAY18_INPUT).unwrap().as_str(), "1886")
    }

    #[test]
    fn examples() {
        for (input, output) in EXAMPLES.iter().zip(ANSWERS.iter()) {
            assert_eq!(part1(input).unwrap(), format!("{}", output));
        }
    }
    #[test]
    fn examples_pt2() {
        for (input, output) in EXAMPLES2.iter().zip(ANSWERS2.iter()) {
            assert_eq!(part2(input).unwrap(), format!("{}", output));
        }
    }
}
