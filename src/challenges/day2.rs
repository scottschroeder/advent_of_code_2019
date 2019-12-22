use crate::intcode::run_intcode;
use crate::util::parse_intcode;
use anyhow::{anyhow, Context, Result};

pub fn day2_part1(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let n = crate::challenges::day2::gravity_assit_calc(intcode, 12, 02);
    Ok(format!("{}", n))
}

pub fn day2_part2(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let (a1, a2) = crate::challenges::day2::scan_args(&intcode, 19690720).expect("no valid inputs");
    Ok(format!("{:02}{:02}", a1, a2))
}

pub fn gravity_assit_calc(mut intcode: Vec<u64>, arg1: u64, arg2: u64) -> u64 {
    intcode[1] = arg1;
    intcode[2] = arg2;
    let finished = run_intcode(intcode);
    finished[0]
}

pub fn scan_args(intcode: &Vec<u64>, expected: u64) -> Option<(u64, u64)> {
    for arg1 in 0..100u64 {
        for arg2 in 0..100u64 {
            let actual = gravity_assit_calc(intcode.clone(), arg1, arg2);
            if expected == actual {
                return Some((arg1, arg2));
            }
        }
    }
    None
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn day2part1() {
        assert_eq!(day2_part1(DAY2_INPUT).unwrap().as_str(), "3101878")
    }

    #[test]
    fn day2part2() {
        assert_eq!(day2_part2(DAY2_INPUT).unwrap().as_str(), "8444")
    }
}
