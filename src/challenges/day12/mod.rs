use anyhow::{Result, Error};
use self::d3::parse;

pub(crate) mod d3;

mod simulation {
    use super::d3::D3;
    use std::path::Iter;
    use crate::challenges::day12::d3::D3Dimm;
    use num::Integer;

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

    pub(crate) struct SystemCycle {
        initial: Vec<Moon>,
        current: System,
        iteration: usize,
        x_period: Option<usize>,
        y_period: Option<usize>,
        z_period: Option<usize>,
    }

    impl SystemCycle {
        pub(crate) fn new(system: System) -> SystemCycle {
            SystemCycle {
                initial: system.moons.clone(),
                current: system,
                iteration: 0,
                x_period: None,
                y_period: None,
                z_period: None,
            }
        }
        pub(crate) fn search(&mut self) {
            while !self.is_complete() {
                self.current.step(1);
                self.iteration += 1;
                if self.x_period.is_none() {
                    self.check(D3Dimm::X);
                }
                if self.y_period.is_none() {
                    self.check(D3Dimm::Y);
                }
                if self.z_period.is_none() {
                    self.check(D3Dimm::Z);
                }
            }
        }
        fn is_complete(&self) -> bool {
            self.x_period.is_some() && self.y_period.is_some() && self.z_period.is_some()
        }
        fn check(&mut self, d: D3Dimm) {
            if self.compare(d) {
                match d {
                    D3Dimm::X => self.x_period = Some(self.iteration),
                    D3Dimm::Y => self.y_period = Some(self.iteration),
                    D3Dimm::Z => self.z_period = Some(self.iteration),
                }
            }
        }
        fn compare(&self, d: D3Dimm) -> bool {
            self.initial
                .iter()
                .zip(self.current.moons.iter())
                .all(|(a, b)| {
                    a.pos.get(d) == b.pos.get(d) &&
                        a.vel.get(d) == b.vel.get(d)
                })
        }
        pub(crate) fn cycle(&self) -> Option<usize> {
            self.x_period
                .and_then(|x| {
                    self.y_period
                        .and_then(|y| {
                            self.z_period
                                .map(|z| (x, y, z))
                        })
                })
                .map(|(x, y, z)| {
                    x.lcm(&y).lcm(&z)
                })
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

        #[test]
        fn part2_ex1() {
            let moons = parse(DAY12_EX1).unwrap();
            let sys = System::new(moons);
            let mut cycle_searcher = SystemCycle::new(sys);
            cycle_searcher.search();
            let cycle = cycle_searcher.cycle().unwrap();
            assert_eq!(cycle, 2772);
        }

        #[test]
        fn part2_ex2() {
            let moons = parse(DAY12_EX2).unwrap();
            let sys = System::new(moons);
            let mut cycle_searcher = SystemCycle::new(sys);
            cycle_searcher.search();
            let cycle = cycle_searcher.cycle().unwrap();
            assert_eq!(cycle, 4686774924);
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
    let moons = parse(input)?;
    let sim = simulation::System::new(moons);
    let mut cycle_searcher = simulation::SystemCycle::new(sim);
    cycle_searcher.search();
    let cycle = cycle_searcher.cycle().unwrap();
    Ok(format!("{}", cycle))
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
        assert_eq!(part2(DAY12_INPUT).unwrap().as_str(), "420788524631496")
    }
}
