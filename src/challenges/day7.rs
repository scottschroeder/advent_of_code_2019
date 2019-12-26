use crate::challenges::day7::amplifier::AmplifierCircut;
use crate::util::parse_intcode;
use anyhow::{anyhow as ah, Result};
use itertools::Itertools;

pub fn day7_part1(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let m = (0..5)
        .permutations(5)
        .map(|phases| {
            let c = AmplifierCircut::new(&phases);
            c.run(&intcode, 0).unwrap()
        })
        .max()
        .unwrap();
    Ok(format!("{}", m))
}

pub fn day7_part2(input: &str) -> Result<String> {
    Ok(format!("{}", 2))
}

mod amplifier {
    use crate::intcode::run_intcode;
    use anyhow::Result;

    #[derive(Debug)]
    pub struct Amplifier {
        phase: i64,
    }

    impl Amplifier {
        pub fn new(phase: i64) -> Amplifier {
            Amplifier { phase }
        }

        pub fn run(&self, intcode: Vec<i64>, input: i64) -> Result<i64> {
            let (mem, out) = run_intcode(intcode, vec![self.phase, input])?;
            Ok(out[0])
        }
    }

    pub struct AmplifierCircut {
        amps: Vec<Amplifier>,
    }

    impl AmplifierCircut {
        pub fn new(phases: &[i64]) -> AmplifierCircut {
            AmplifierCircut {
                amps: phases.iter().map(|p| Amplifier::new(*p)).collect(),
            }
        }
        pub fn run(&self, intcode: &[i64], input: i64) -> Result<i64> {
            let mut prev = input;
            for amp in &self.amps {
                prev = amp.run(intcode.to_vec(), prev)?;
            }
            Ok(prev)
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        fn test_amp_circut(intcode: &[i64], phases: &[i64], signal: i64) {
            let c = AmplifierCircut::new(phases);
            let out = c.run(intcode, 0).unwrap();
            assert_eq!(out, signal);
        }

        #[test]
        fn example_amp_circuts() {
            test_amp_circut(
                &[
                    3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0,
                ],
                &[4, 3, 2, 1, 0],
                43210,
            );
            test_amp_circut(
                &[
                    3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23,
                    23, 4, 23, 99, 0, 0,
                ],
                &[0, 1, 2, 3, 4],
                54321,
            );
            test_amp_circut(
                &[
                    3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7,
                    33, 1, 33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0,
                ],
                &[1, 0, 4, 3, 2],
                65210,
            );
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn day7part1() {
        assert_eq!(day7_part1(DAY7_INPUT).unwrap().as_str(), "11828")
    }

    #[test]
    fn day7part2() {
        assert_eq!(day7_part2(DAY7_INPUT).unwrap().as_str(), "2")
    }
}
