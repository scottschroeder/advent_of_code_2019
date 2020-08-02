use crate::intcode::run_intcode;
use crate::util::parse_intcode;
use anyhow::Result;

pub fn part1(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let code = test_diagnostic(intcode, 1)?;
    Ok(format!("{}", code))
}

pub fn part2(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let code = test_diagnostic(intcode, 5)?;
    Ok(format!("{}", code))
}

pub fn test_diagnostic(intcode: Vec<i64>, system_code: i64) -> Result<i64> {
    let input = vec![system_code];
    let (_mem, output) = run_intcode(intcode, input)?;
    log::debug!("output: {:?}", output);
    let diag = output[output.len() - 1];
    Ok(diag)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn day5part1() {
        assert_eq!(part1(DAY5_INPUT).unwrap().as_str(), "5346030")
    }

    #[test]
    fn day5part2() {
        assert_eq!(part2(DAY5_INPUT).unwrap().as_str(), "513116")
    }
}
