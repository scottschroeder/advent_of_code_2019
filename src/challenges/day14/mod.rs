use anyhow::Result;
use crate::challenges::day14::spacechem::ore_search;

pub fn part1(input: &str) -> Result<String> {
    Ok(format!("{}", ore_search(input)))
}

pub fn part2(input: &str) -> Result<String> {
    Ok(format!("{}", 0))
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
        //assert_eq!(part2(DAY14_INPUT).unwrap().as_str(), "0")
    }
}


