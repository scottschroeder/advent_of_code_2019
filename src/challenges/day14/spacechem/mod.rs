use crate::util::parse_str;
use anyhow::{anyhow as ah, Result};
use num::Integer;
use reaction_parser::parse_reaction_manifest;
use std::collections::{HashMap, HashSet, VecDeque};

mod reaction_parser;

pub(crate) fn ore_search(manifest: &str) -> i64 {
    let factory = NanoFactory::parse_reactions(manifest).unwrap();
    let supply = factory.make_n(&("FUEL".into()), 1);
    -supply.get(&Molecule::from("ORE"))
}

pub(crate) fn fuel_from_ore(manifest: &str, amount: i64) -> i64 {
    let factory = NanoFactory::parse_reactions(manifest).unwrap();
    factory.search(
        &("FUEL".into()),
        Quantity {
            amount,
            molecule: "ORE".into(),
        },
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Molecule {
    name: String,
}

impl<T: Into<String>> From<T> for Molecule {
    fn from(s: T) -> Self {
        Molecule { name: s.into() }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Quantity {
    amount: i64,
    molecule: Molecule,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Reaction {
    inputs: Vec<Quantity>,
    output: Quantity,
}

pub(crate) struct NanoFactory {
    manifest: HashMap<Molecule, Reaction>,
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Supply {
    inner: HashMap<Molecule, i64>,
}

impl Supply {
    fn set(&mut self, m: Molecule, amount: i64) {
        self.inner.insert(m, amount);
    }
    fn adjust(&mut self, m: Molecule) -> &mut i64 {
        self.inner.entry(m).or_insert(0)
    }
    fn get(&self, m: &Molecule) -> i64 {
        self.inner.get(m).cloned().unwrap_or(0)
    }
    fn deficit(&self) -> impl Iterator<Item = Molecule> + '_ {
        self.inner
            .iter()
            .filter_map(|(m, q)| if *q < 0 { Some(m.clone()) } else { None })
    }
}

impl NanoFactory {
    fn parse_reactions(reactions: &str) -> Result<NanoFactory> {
        NanoFactory::from_reactions(parse_reaction_manifest(reactions)?)
    }
    fn from_reactions(reactions: Vec<Reaction>) -> Result<NanoFactory> {
        let mut manifest = HashMap::with_capacity(reactions.len());
        for r in reactions {
            if let Some(prev) = manifest.insert(r.output.molecule.clone(), r) {
                return Err(ah!("multiple ways to make: {:?}", prev.output.molecule));
            }
        }
        Ok(NanoFactory { manifest })
    }
    fn resolve(&self, supply: &mut Supply) {
        let mut deficit: VecDeque<Molecule> = supply.deficit().collect();
        while let Some(m) = deficit.pop_back() {
            if let Some(r) = self.manifest.get(&m) {
                let m_supply = supply.adjust(m);
                if *m_supply >= 0 {
                    continue;
                }
                let needed = -*m_supply;
                let iterations_required = needed.div_ceil(&r.output.amount);
                *m_supply += iterations_required * r.output.amount;

                for input in &r.inputs {
                    let i_supply = supply.adjust(input.molecule.clone());
                    *i_supply -= input.amount * iterations_required;
                    deficit.push_front(input.molecule.clone());
                }
            }
        }
    }
    fn make_n(&self, desired: &Molecule, n: i64) -> Supply {
        let mut supply = Supply::default();
        supply.set(desired.clone(), -n);
        self.resolve(&mut supply);
        supply
    }

    fn search(&self, desired: &Molecule, limit: Quantity) -> i64 {
        let f_prime = |x: i64| {
            let mut supply = self.make_n(desired, x);
            let x0 = supply.get(&desired) + x;
            let y0 = -supply.get(&limit.molecule);
            let slope = y0 as f64 / x0 as f64;
            log::trace!("input: {} x: {} y: {} slope: {}", x, x0, y0, slope);
            slope
        };
        let mut guess = 1;
        loop {
            let old_guess = guess;
            let r = f_prime(old_guess);
            guess = (limit.amount as f64 / r) as i64;
            if old_guess == guess {
                break guess;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PT2_SUPPLY: i64 = 1_000_000_000_000;

    const EX1: &str = r##"
        10 ORE => 10 A
        1 ORE => 1 B
        7 A, 1 B => 1 C
        7 A, 1 C => 1 D
        7 A, 1 D => 1 E
        7 A, 1 E => 1 FUEL
    "##;

    const EX2: &str = r##"
        9 ORE => 2 A
        8 ORE => 3 B
        7 ORE => 5 C
        3 A, 4 B => 1 AB
        5 B, 7 C => 1 BC
        4 C, 1 A => 1 CA
        2 AB, 3 BC, 4 CA => 1 FUEL
    "##;

    const EX3: &str = r##"
        157 ORE => 5 NZVS
        165 ORE => 6 DCFZ
        44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
        12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
        179 ORE => 7 PSHF
        177 ORE => 5 HKGWZ
        7 DCFZ, 7 PSHF => 2 XJWVT
        165 ORE => 2 GPVTF
        3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT
    "##;

    const EX4: &str = r##"
        2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
        17 NVRVD, 3 JNWZP => 8 VPVL
        53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
        22 VJHF, 37 MNCFX => 5 FWMGM
        139 ORE => 4 NVRVD
        144 ORE => 7 JNWZP
        5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
        5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
        145 ORE => 6 MNCFX
        1 NVRVD => 8 CXFTF
        1 VJHF, 6 MNCFX => 4 RFSQX
        176 ORE => 6 VJHF
    "##;

    const EX5: &str = r##"
        171 ORE => 8 CNZTR
        7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
        114 ORE => 4 BHXH
        14 VRPVC => 6 BMBT
        6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
        6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
        15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
        13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
        5 BMBT => 4 WPTQ
        189 ORE => 9 KTJDG
        1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
        12 VRPVC, 27 CNZTR => 2 XDBXC
        15 KTJDG, 12 BHXH => 5 XCVML
        3 BHXH, 2 VRPVC => 7 MZWV
        121 ORE => 7 VRPVC
        7 XCVML => 6 RJRHP
        5 BHXH, 4 VRPVC => 5 LTCX
    "##;

    fn check_supply(manifest: &str, expected_supply: Vec<(&str, i64)>) {
        let factory = NanoFactory::parse_reactions(manifest).unwrap();
        let supply = factory.make_n(&("FUEL".into()), 1);
        let expected = Supply {
            inner: expected_supply
                .into_iter()
                .map(|(s, c)| (Molecule::from(s), c))
                .collect::<HashMap<Molecule, i64>>(),
        };

        assert_eq!(supply, expected);
    }

    #[test]
    fn test_super_simple_manifest() {
        check_supply("2 ORE => 1 FUEL", vec![("FUEL", 0), ("ORE", -2)])
    }

    #[test]
    fn test_two_stage_reaction() {
        check_supply(
            r#"
                2 ORE => 2 A
                3 A => 2 FUEL
            "#,
            vec![("FUEL", 1), ("A", 1), ("ORE", -4)],
        )
    }

    #[test]
    fn ex1() {
        assert_eq!(ore_search(EX1), 31);
    }

    #[test]
    fn ex2() {
        assert_eq!(ore_search(EX2), 165);
    }

    #[test]
    fn ex3() {
        assert_eq!(ore_search(EX3), 13312);
    }

    #[test]
    fn ex4() {
        assert_eq!(ore_search(EX4), 180697);
    }

    #[test]
    fn ex5() {
        assert_eq!(ore_search(EX5), 2210736);
    }

    #[test]
    fn ex3_p2() {
        assert_eq!(fuel_from_ore(EX3, PT2_SUPPLY), 82892753)
    }

    #[test]
    fn ex4_p2() {
        assert_eq!(fuel_from_ore(EX4, PT2_SUPPLY), 5586022)
    }

    #[test]
    fn ex5_p2() {
        assert_eq!(fuel_from_ore(EX5, PT2_SUPPLY), 460664)
    }
}
