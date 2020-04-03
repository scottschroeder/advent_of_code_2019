use crate::challenges::day14::spacechem::{fuel_from_ore, ore_search};
use anyhow::Result;

pub fn part1(input: &str) -> Result<String> {
    Ok(format!("{}", ore_search(input)))
}

pub fn part2(input: &str) -> Result<String> {
    Ok(format!("{}", fuel_from_ore(input, 1_000_000_000_000)))
}

mod spacechem;

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn day14part1() {
        assert_eq!(part1(DAY14_INPUT).unwrap().as_str(), "907302")
    }

    #[test]
    fn day14part2() {
        assert_eq!(part2(DAY14_INPUT).unwrap().as_str(), "1670299")
    }
}
