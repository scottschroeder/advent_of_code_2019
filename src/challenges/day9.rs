use crate::util::parse_intcode;
use anyhow::Result;

pub fn day9_part1(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    Ok(format!("{}", 0))
}

pub fn day9_part2(input: &str) -> Result<String> {
    Ok(format!("{}", 0))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn day9part1() {
        assert_eq!(day9_part1(DAY9_INPUT).unwrap().as_str(), "0")
    }

    #[test]
    fn day9part2() {
        assert_eq!(day9_part2(DAY9_INPUT).unwrap().as_str(), "0")
    }
}
