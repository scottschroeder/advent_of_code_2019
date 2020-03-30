use anyhow::{Result, Error};
use self::d3::parse;

pub(crate) mod d3;

mod simulation {
    use super::d3::D3;
    use std::path::Iter;

    #[derive(Debug, Clone, Copy)]
    struct Moon {
        pos: D3,
        vel: D3,
    }

    impl Moon {
        fn energy(&self) -> i64 {
            self.pos.abs().total() * self.vel.abs().total()
        }
    }

    #[derive(Debug, Clone)]
    pub(crate) struct System {
        moons: Vec<Moon>,
    }

    impl System {
        pub(crate) fn new(pos: Vec<D3>) -> Self {
            System {
                moons: pos.into_iter().map(|d3| Moon { pos: d3, vel: D3::default() }).collect()
            }
        }
        pub(crate) fn energy(&self) -> i64 {
            self.moons.iter().map(|m| m.energy()).sum()
        }

        pub(crate) fn step(&mut self, count: usize) {
            for _ in 0..count {
                self.gravity();
                self.velocity();
            }
        }

        fn gravity(&mut self) {
            for idx in 1..self.moons.len() {
                let (head, tail) = self.moons.split_at_mut(idx);
                let m1 = &mut head[idx - 1];
                for m2 in tail {
                    let d = m1.pos.compare(&m2.pos);
                    m1.vel -= d;
                    m2.vel += d;
                }
            }
        }
        fn velocity(&mut self) {
            for m in self.moons.iter_mut() {
                m.pos += m.vel;
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::challenges::day12::d3::parse;
        use crate::challenges::test::*;

        #[test]
        fn ex1() {
            let moons = parse(DAY12_EX1).unwrap();
            let mut sys = System::new(moons);
            sys.step(10);
            assert_eq!(sys.energy(), 179);
        }

        #[test]
        fn ex2() {
            let moons = parse(DAY12_EX2).unwrap();
            let mut sys = System::new(moons);
            sys.step(100);
            assert_eq!(sys.energy(), 1940);
        }
    }
}

pub fn part1(input: &str) -> Result<String> {
    let moons = parse(input)?;
    let mut sim = simulation::System::new(moons);
    sim.step(1000);
    Ok(format!("{:#?}", sim.energy()))
}

pub fn part2(input: &str) -> Result<String> {
    Ok(format!("{}", 0))
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn check_part1() {
        assert_eq!(part1(DAY12_INPUT).unwrap().as_str(), "9139")
    }

    #[test]
    fn check_part2() {
        assert_eq!(part2(DAY12_INPUT).unwrap().as_str(), "0")
    }
}
