use crate::util::parse_int_lines;
use anyhow::Result;
type MassUnit = u64;

pub fn day1_part1(input: &str) -> Result<String> {
    let modules = parse_int_lines(input)?;
    let fuel = crate::challenges::day1::total_fuel(modules.into_iter());
    Ok(format!("{}", fuel))
}

pub fn day1_part2(input: &str) -> Result<String> {
    let modules = parse_int_lines(input)?;
    let fuel = crate::challenges::day1::total_fuel_recursive(modules.into_iter());
    Ok(format!("{}", fuel))
}

fn fuel_from_mass(mass: MassUnit) -> u64 {
    if mass < 7 {
        return 0;
    }
    (mass - 6) / 3
}

fn recursive_fuel_from_mass(mass: MassUnit) -> u64 {
    let mut total = 0u64;
    let mut new_mass = mass;
    loop {
        new_mass = fuel_from_mass(new_mass);
        total += new_mass;
        if new_mass == 0 {
            return total;
        }
    }
}

pub fn total_fuel(modules: impl Iterator<Item = MassUnit>) -> u64 {
    modules.map(fuel_from_mass).sum()
}

pub fn total_fuel_recursive(modules: impl Iterator<Item = MassUnit>) -> u64 {
    modules.map(recursive_fuel_from_mass).sum()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn fuel_from_mass_single() {
        assert_eq!(fuel_from_mass(12), 2);
        assert_eq!(fuel_from_mass(14), 2);
        assert_eq!(fuel_from_mass(1969), 654);
        assert_eq!(fuel_from_mass(100_756), 33583);
    }
    #[test]
    fn fuel_from_mass_recursive() {
        assert_eq!(recursive_fuel_from_mass(12), 2);
        assert_eq!(recursive_fuel_from_mass(14), 2);
        assert_eq!(recursive_fuel_from_mass(1969), 966);
        assert_eq!(recursive_fuel_from_mass(100_756), 50346);
    }

    #[test]
    fn day1part1() {
        assert_eq!(day1_part1(DAY1_INPUT).unwrap().as_str(), "3402634")
    }
    #[test]
    fn day1part2() {
        assert_eq!(day1_part2(DAY1_INPUT).unwrap().as_str(), "5101069")
    }
}
