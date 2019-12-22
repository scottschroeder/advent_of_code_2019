type MassUnit = u64;

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
    use crate::challenges::day1::{fuel_from_mass, recursive_fuel_from_mass};

    #[test]
    fn fuel_from_mass_single() {
        assert_eq!(fuel_from_mass(12), 2);
        assert_eq!(fuel_from_mass(14), 2);
        assert_eq!(fuel_from_mass(1969), 654);
        assert_eq!(fuel_from_mass(100756), 33583);
    }
    #[test]
    fn fuel_from_mass_recursive() {
        assert_eq!(recursive_fuel_from_mass(12), 2);
        assert_eq!(recursive_fuel_from_mass(14), 2);
        assert_eq!(recursive_fuel_from_mass(1969), 966);
        assert_eq!(recursive_fuel_from_mass(100756), 50346);
    }
}
