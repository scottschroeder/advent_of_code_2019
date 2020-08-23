use self::grid::{Grid, RecursiveGrid};
use anyhow::Result;
use std::collections::HashSet;

mod grid;

pub fn part1(input: &str) -> Result<String> {
    let mut g = Grid::from_map(input)?;
    let mut seen = HashSet::new();

    while seen.insert(g.clone()) {
        g.update()
    }
    log::debug!("\n{}", g);
    Ok(format!("{}", g.biodiversity()))
}

pub fn part2(input: &str) -> Result<String> {
    Ok(format!("{:?}", recurse_n(input, 200)?))
}

fn recurse_n(input: &str, rounds: usize) -> Result<usize> {
    let mut g = RecursiveGrid::from_map(input)?;
    for _ in 0..rounds {
        g.update();
    }
    log::trace!("{}", g);
    Ok(g.count())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn verify_part1() {
        assert_eq!(part1(DAY24_INPUT).unwrap().as_str(), "28903899")
    }

    #[test]
    fn verify_part2() {
        assert_eq!(part2(DAY24_INPUT).unwrap().as_str(), "1896")
    }

    #[test]
    fn verify_p1ex1() {
        assert_eq!(part1(DAY24_EX1).unwrap().as_str(), "2129920")
    }
    #[test]
    fn verify_p2ex1() {
        assert_eq!(recurse_n(DAY24_EX1, 10).unwrap(), 99)
    }
}
