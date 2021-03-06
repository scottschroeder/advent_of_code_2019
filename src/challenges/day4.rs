use crate::util::{digits, parse_str};
use anyhow::Result;

pub fn part1(input: &str) -> Result<String> {
    let (low, high) = parse_range(input)?;
    let count = (low..=high).filter(|pw| is_valid_password(*pw)).count();
    Ok(format!("{}", count))
}

pub fn part2(input: &str) -> Result<String> {
    let (low, high) = parse_range(input)?;
    let count = (low..=high)
        .filter(|pw| is_valid_password_part2(*pw))
        .count();
    Ok(format!("{}", count))
}

fn parse_range(input: &str) -> Result<(u64, u64)> {
    let mut chunks = input.trim().split('-');
    let low = chunks.next().unwrap();
    let high = chunks.next().unwrap();
    Ok((parse_str(low)?, parse_str(high)?))
}

fn is_valid_password(x: u64) -> bool {
    let digits = digits(x);
    let mut double = false;
    if digits.len() != 6 {
        return false;
    }
    let mut prev = 0;
    for d in digits {
        if d == prev {
            double = true
        } else if d < prev {
            return false;
        }
        prev = d;
    }
    double
}

fn is_valid_password_part2(x: u64) -> bool {
    let digits = digits(x);
    let mut double = false;
    if digits.len() != 6 {
        return false;
    }
    let mut prev = 0;
    let mut streak = 0;
    for d in digits {
        if d == prev {
            streak += 1
        } else if d < prev {
            return false;
        } else {
            if streak == 1 {
                double = true;
            }
            streak = 0;
        }
        prev = d;
    }
    if streak == 1 {
        double = true;
    }
    double
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn parse_ranges() {
        assert_eq!(parse_range("23-57").unwrap(), (23, 57));
        assert_eq!(parse_range("265275-781584").unwrap(), (265_275, 781_584));
    }

    #[test]
    fn valid_pw_pt2() {
        assert_eq!(is_valid_password_part2(112_233), true);
        assert_eq!(is_valid_password_part2(123_444), false);
        assert_eq!(is_valid_password_part2(111_122), true);
        assert_eq!(is_valid_password_part2(123_455), true);
    }

    #[test]
    fn day4part1() {
        assert_eq!(part1(DAY4_INPUT).unwrap().as_str(), "960")
    }

    #[test]
    fn day4part2() {
        assert_eq!(part2(DAY4_INPUT).unwrap().as_str(), "626")
    }
}
