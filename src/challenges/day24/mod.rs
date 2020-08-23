use anyhow::{anyhow as ah, Result};
use self::grid::{Grid, RecursiveGrid};
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
    // self::grid::demo();
    let mut g = RecursiveGrid::from_map(input)?;
    for _ in 0..200 {
        g.update();
    }
    log::trace!("{}", g);
    Ok(format!("{:?}", g.count()))
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
}
