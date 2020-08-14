use self::shuf::{Shuffle, Technique};
use anyhow::{anyhow as ah, Result};

pub(crate) mod shuf;

const PT1_DECK: usize = 10007;
const PT1_INDEX: usize = 2019;
const PT2_DECK: usize = 119315717514047;
const PT2_INDEX: usize = 2020;
const PT2_REPEAT: u64 = 101741582076661;

pub fn part1(input: &str) -> Result<String> {
    let procedures = parse(input)?;
    log::debug!("Procedures: {:#?}", procedures);
    let shuffle = Shuffle::new(PT1_DECK, procedures.as_slice())?;
    log::debug!("Shuffle: {:?}", shuffle);
    let pos = shuffle
        .full()
        .enumerate()
        .find(|&(_, c)| c == PT1_INDEX)
        .map(|(idx, _)| idx);
    Ok(format!("{:?}", pos.unwrap()))
}

pub fn part2(input: &str) -> Result<String> {
    let procedures = parse(input)?;
    log::debug!("Procedures: {:#?}", procedures);
    let shuffle = Shuffle::new(PT2_DECK, procedures.as_slice())?.repeat(PT2_REPEAT);
    log::debug!("Shuffle: {:?}", shuffle);
    let c = shuffle.index(PT2_INDEX);
    Ok(format!("{}", c))
}

fn parse(input: &str) -> Result<Vec<Technique>> {
    input
        .lines()
        .map(|s| {
            let last = s
                .split(" ")
                .last()
                .ok_or_else(|| ah!("command did not have argument: {:?}", s))?;
            if s.starts_with("cut") {
                let cut = str::parse::<i64>(last)?;
                Ok(Technique::Cut(cut))
            } else if s.contains("deal with increment") {
                let inc = str::parse::<i64>(last)?;
                if inc < 1 {
                    Err(ah!("increment argument <1: {:?}", s))
                } else {
                    Ok(Technique::Increment(inc))
                }
            } else if s.contains("deal into new stack") {
                Ok(Technique::Stack)
            } else {
                Err(ah!("unknown command: {:?}", s))
            }
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn verify_part1() {
        assert_eq!(part1(DAY22_INPUT).unwrap().as_str(), "3939")
    }

    #[test]
    fn verify_part2() {
        assert_eq!(part2(DAY22_INPUT).unwrap().as_str(), "55574110161534")
    }
}
