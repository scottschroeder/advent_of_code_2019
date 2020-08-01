use crate::util::{digits_to_int, parse_digits};
use anyhow::Result;
use std::fmt;
use std::fmt::{Error, Formatter};
use std::hint::unreachable_unchecked;
use std::iter;


pub fn part1(input: &str) -> Result<String> {
    Ok(format!("{}", 0))
}

pub fn part2(input: &str) -> Result<String> {
    Ok(format!("{}", 0))
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn day18part1() {
        assert_eq!(part1(DAY18_INPUT).unwrap().as_str(), "0")
    }

    #[test]
    fn day18part2() {
        assert_eq!(part2(DAY18_INPUT).unwrap().as_str(), "0")
    }

}
