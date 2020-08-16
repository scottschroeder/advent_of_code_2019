use anyhow::{anyhow as ah, Result};
use self::grid::Grid;
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
    Ok(format!("{:?}", 0))
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
        assert_eq!(part2(DAY24_INPUT).unwrap().as_str(), "0")
    }
}
