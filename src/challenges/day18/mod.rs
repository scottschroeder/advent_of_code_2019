use self::keys::{Key, KeySet};
use crate::util::{digits_to_int, parse_digits};
use anyhow::Result;
use std::fmt;
use std::fmt::{Error, Formatter};
use std::hint::unreachable_unchecked;
use std::iter;

mod graph;
mod keys;
mod map_reader;

pub fn part1(input: &str) -> Result<String> {
    let m = map_reader::Map::parse(input);
    //log::debug!( "{:#?}", m);
    let g = graph::CaveGraph::from_map(m);
    //log::debug!( "{}", g.dot());
    //return Ok(format!("{}", g.dot()));
    let shortest = g.shortest_path();
    Ok(format!("{}", shortest))
}

pub fn part2(input: &str) -> Result<String> {
    let e = KeySet::new();
    log::debug!("{:?}", e);
    Ok(format!("{}", 0))
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

    pub(crate) const EXAMPLES: [&str; 5] = [
        DAY18_EX1,
        DAY18_EX2,
        DAY18_EX3,
        DAY18_EX4,
        DAY18_EX5,
    ];

    pub(crate) const ANSWERS: [u32; 5] = [
        8,
        86,
        132,
        136,
        81
    ];

    #[test]
    fn day18part1() {
        assert_eq!(part1(DAY18_INPUT).unwrap().as_str(), "0")
    }

    #[test]
    fn day18part2() {
        assert_eq!(part2(DAY18_INPUT).unwrap().as_str(), "0")
    }

    #[test]
    fn examples() {
        for (input, output) in EXAMPLES.iter().zip(ANSWERS.iter()) {
            assert_eq!(part1(input).unwrap(), format!("{}", output));
        }
    }
}
