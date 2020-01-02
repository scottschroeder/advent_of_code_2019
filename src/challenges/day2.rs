use crate::intcode::run_intcode;
use crate::util::parse_intcode;
use anyhow::Result;

pub fn part1(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let n = crate::challenges::day2::gravity_assit_calc(intcode, 12, 2)?;
    Ok(format!("{}", n))
}

pub fn part2(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let (a1, a2) =
        crate::challenges::day2::scan_args(&intcode, 19_690_720).expect("no valid inputs");
    Ok(format!("{:02}{:02}", a1, a2))
}

pub fn gravity_assit_calc(mut intcode: Vec<i64>, arg1: i64, arg2: i64) -> Result<i64> {
    intcode[1] = arg1;
    intcode[2] = arg2;
    let (mem, _output) = run_intcode(intcode, vec![])?;
    Ok(mem[0])
}

pub fn scan_args(intcode: &[i64], expected: i64) -> Option<(i64, i64)> {
    for arg1 in 0..100i64 {
        for arg2 in 0..100i64 {
            if let Ok(actual) = gravity_assit_calc(intcode.to_vec(), arg1, arg2) {
                if expected == actual {
                    return Some((arg1, arg2));
                }
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
        assert_eq!(part1(DAY2_INPUT).unwrap().as_str(), "3101878")
    }

    #[test]
    fn day2part2() {
        assert_eq!(part2(DAY2_INPUT).unwrap().as_str(), "8444")
    }
}
