use crate::intcode::run_intcode;
use crate::util::parse_intcode;
use anyhow::{anyhow as ah, Result};
use std::fmt;

const PROG_PT1: &str = include_str!("part1");
const PROG_PT2: &str = include_str!("part2");

pub fn part1(input: &str) -> Result<String> {
    let score = run_program(input, PROG_PT1)?;
    Ok(format!("{}", score))
}

pub fn part2(input: &str) -> Result<String> {
    let score = run_program(input, PROG_PT2)?;
    Ok(format!("{}", score))
}

pub fn run_program(intcode: &str, program: &str) -> Result<i64> {
    let intcode = parse_intcode(intcode)?;
    let program = compile(program);

    let (_, out) = run_intcode(intcode, program)?;
    let last = *out.last().unwrap();
    if last > 127 {
        Ok(last)
    } else {
        log::error!("{}", SpringError(out.as_slice()));
        Err(ah!("intcode program failed"))
    }
}

struct SpringError<'a>(&'a [i64]);

impl<'a> fmt::Display for SpringError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for x in self.0 {
            write!(f, "{}", char::from(*x as u8))?;
        }
        Ok(())
    }
}

fn compile(code: &str) -> Vec<i64> {
    code.chars().map(|c| c as i64).collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn verify_part1() {
        assert_eq!(part1(DAY21_INPUT).unwrap().as_str(), "19355645")
    }

    #[test]
    fn verify_part2() {
        assert_eq!(part2(DAY21_INPUT).unwrap().as_str(), "1137899149")
    }
}
