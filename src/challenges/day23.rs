use anyhow::{anyhow as ah, Result};
use crate::intcode::run_intcode;
use crate::util::parse_intcode;


pub fn part1(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    // let (_, out) = run_intcode(intcode, vec![1])?;
    Ok(format!("{:?}", 0))
}

pub fn part2(input: &str) -> Result<String> {
    Ok(format!("{:?}", 0))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn verify_part1() {
        assert_eq!(part1(DAY23_INPUT).unwrap().as_str(), "0")
    }

    #[test]
    fn verify_part2() {
        assert_eq!(part2(DAY23_INPUT).unwrap().as_str(), "0")
    }
}
