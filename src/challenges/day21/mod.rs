use crate::intcode::run_intcode;
use crate::util::parse_intcode;
use std::fmt;
use anyhow::{anyhow as ah, Result};

const PROG_PT1: &str = include_str!("part1");

pub fn part1(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let program = compile(PROG_PT1);
    let (_, out) = run_intcode(intcode, program)?;
    let last = *out.last().unwrap();
    if last > 127 {
        Ok(format!("{}", last))
    } else {
        log::error!("{}", SpringError(out.as_slice()));
        Err(ah!("intcode program failed"))
    }
}

pub fn part2(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let (_, out) = run_intcode(intcode, vec![2])?;
    Ok(format!("{}", out[0]))
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
        assert_eq!(part2(DAY21_INPUT).unwrap().as_str(), "0")
    }
}
