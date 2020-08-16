use anyhow::{anyhow as ah, Result};


pub fn part1(input: &str) -> Result<String> {
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
        assert_eq!(part1(DAY24_INPUT).unwrap().as_str(), "0")
    }

    #[test]
    fn verify_part2() {
        assert_eq!(part2(DAY24_INPUT).unwrap().as_str(), "0")
    }
}
