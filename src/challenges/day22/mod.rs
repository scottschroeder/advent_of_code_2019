use anyhow::{anyhow as ah, Result};
use self::shuf::{ShuffleMethod, Deck, Shuffle};

pub(crate) mod parse;
pub(crate) mod shuf;

pub fn part1(input: &str) -> Result<String> {
    let procedures = parse::parse(input)?;
    log::debug!("Procedures: {:#?}", procedures);
    let shuffle = Shuffle::new(10007, procedures.as_slice())?;
    let pos = shuffle.full().enumerate().find(|&(idx, c)| c == 2019).map(|(idx, c)| idx);
    Ok(format!("{:?}", pos.unwrap()))
}

const PT2_DECK: usize = 119315717514047;
const PT2_REPEAT: usize = 101741582076661;
const PT2_INDEX: usize = 2020;
const LOG_LOOP: usize = 1_000_000;


pub fn part2(input: &str) -> Result<String> {
    let procedures = parse::parse(input)?;
    log::debug!("Procedures: {:#?}", procedures);
    let shuffle = Shuffle::new(PT2_DECK, procedures.as_slice())?;
    let mut idx = PT2_INDEX;
    let mut seen = std::collections::HashMap::new();
    let mut c = 0;
    let prev = loop {
        if let Some(prev) = seen.insert(idx, c) {
            break prev;
        }
        idx = shuffle.index(idx);
        // if idx == PT2_INDEX {
        //     break
        // }
        if c % LOG_LOOP == 0{
            log::trace!("idx: {} {}", c / LOG_LOOP, seen.len());
        }
        c+=1;
    };
    let loop_size = c - prev;
    let offset = (PT2_REPEAT - prev) % loop_size;
    let fin = (0..offset)
        .fold(idx, |idx, _| {shuffle.index(idx)});
    //Ok(format!("{} -> {} ({}) offset:{} start:{:?}", prev, c, loop_size, offset, idx))
    Ok(format!("{}", fin))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn verify_part1() {
        assert_eq!(part1(DAY22_INPUT).unwrap().as_str(), "0")
    }

    #[test]
    fn verify_part2() {
        assert_eq!(part2(DAY22_INPUT).unwrap().as_str(), "0")
    }

    #[test]
    fn verify_part1_ex1() {
        assert_eq!(part2(DAY22_EX1).unwrap().as_str(), "0")
    }
    #[test]
    fn verify_part1_ex2() {
        assert_eq!(part2(DAY22_EX2).unwrap().as_str(), "0")
    }
    #[test]
    fn verify_part1_ex3() {
        assert_eq!(part2(DAY22_EX3).unwrap().as_str(), "0")
    }
    #[test]
    fn verify_part1_ex4() {
        assert_eq!(part2(DAY22_EX4).unwrap().as_str(), "0")
    }
}
