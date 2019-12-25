use crate::orbital_data::OrbitalMap;
use anyhow::{anyhow as ah, Result};

pub fn day6_part1(input: &str) -> Result<String> {
    let bodies = OrbitalMap::from_orbital_data(input);
    let tc = bodies.transitive_closure();
    Ok(format!("{}", tc.connections()))
}

pub fn day6_part2(input: &str) -> Result<String> {
    let bodies = OrbitalMap::from_orbital_data(input);
    let dist = bodies
        .shortest_path("YOU", "SAN")
        .ok_or_else(|| ah!("could not find path from YOU -> SAN"))?;
    Ok(format!("{}", dist))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn day6part1_example() {
        assert_eq!(day6_part1(DAY6_EXAMPLE_INPUT).unwrap().as_str(), "42")
    }

    #[test]
    fn day6part1() {
        assert_eq!(day6_part1(DAY6_INPUT).unwrap().as_str(), "186597")
    }

    #[test]
    fn day6part2() {
        assert_eq!(day6_part2(DAY6_INPUT).unwrap().as_str(), "412")
    }
}
