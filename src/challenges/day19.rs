use crate::intcode::run_intcode;
use crate::util::parse_intcode;
use anyhow::Result;

const GRID_SIZE: i64 = 100 - 1;

pub fn part1(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let grid = 50;
    let mut points = 0;
    for y in 0..grid {
        for x in 0..grid {
            let (_, out) = run_intcode(intcode.clone(), vec![x, y])?;
            points += out[0];
        }
    }
    Ok(format!("{}", points))
}

pub fn part2(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let check = move |x, y| {
        let (_, out) = run_intcode(intcode.clone(), vec![x, y]).unwrap();
        log::debug!("({}, {}) => {}", x, y, out[0]);
        out[0] == 1
    };
    let mut x = 0;
    let mut y = GRID_SIZE;
    loop {
        if check(x, y) {
            if check(x + GRID_SIZE, y - GRID_SIZE) {
                break;
            } else {
                y += 1;
            }
        } else {
            x += 1;
        }
    }
    let score = 10000 * x + (y - GRID_SIZE);
    Ok(format!("{}", score))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn verify_part1() {
        assert_eq!(part1(DAY19_INPUT).unwrap().as_str(), "199")
    }

    #[test]
    fn verify_part2() {
        assert_eq!(part2(DAY19_INPUT).unwrap().as_str(), "10180726")
    }
}
