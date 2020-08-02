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
    //debug!(slog_scope::logger(), "{:#?}", m);
    let g = graph::CaveGraph::from_map(m);
    //debug!(slog_scope::logger(), "{}", g.dot());
    //return Ok(format!("{}", g.dot()));
    let shortest = g.shortest_path();
    Ok(format!("{}", shortest))
}

pub fn part2(input: &str) -> Result<String> {
    let e = KeySet::new();
    debug!(slog_scope::logger(), "{:?}", e);
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

    pub(crate) const EXAMPLES: [&str; 6] = [
        DAY18_EX1,
        DAY18_EX2,
        DAY18_EX3,
        DAY18_EX4,
        DAY18_EX5,
        DAY18_INPUT,
    ];

    #[test]
    fn day18part1() {
        assert_eq!(part1(DAY18_INPUT).unwrap().as_str(), "0")
    }

    #[test]
    fn day18part2() {
        assert_eq!(part2(DAY18_INPUT).unwrap().as_str(), "0")
    }
}
