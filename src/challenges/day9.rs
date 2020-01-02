use crate::intcode::run_intcode;
use crate::util::parse_intcode;
use anyhow::Result;

pub fn part1(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let (_, out) = run_intcode(intcode, vec![1])?;
    Ok(format!("{}", out[0]))
}

pub fn part2(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let (_, out) = run_intcode(intcode, vec![2])?;
    Ok(format!("{}", out[0]))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn day9part1() {
        assert_eq!(part1(DAY9_INPUT).unwrap().as_str(), "2752191671")
    }

    #[test]
    fn day9part2() {
        assert_eq!(part2(DAY9_INPUT).unwrap().as_str(), "87571")
    }
}
