use anyhow::{anyhow as ah, Result};
use self::shuf::{ShuffleMethod, Deck, Shuffle};
use self::parse::parse;

pub(crate) mod parse;
pub(crate) mod shuf;

pub fn part1(input: &str) -> Result<String> {
    let procedures = parse(input)?;
    log::debug!("Procedures: {:#?}", procedures);
    let shuffle = Shuffle::new(10007, procedures.as_slice())?;
    log::debug!("Shuffle: {:?}", shuffle);
    let pos = shuffle.full().enumerate().find(|&(idx, c)| c == 2019).map(|(idx, c)| idx);
    Ok(format!("{:?}", pos.unwrap()))
}

const PT2_DECK: usize = 119315717514047;
const PT2_REPEAT: usize = 101741582076661;
const PT2_INDEX: usize = 2020;

// const PT2_DECK: usize = 107;
// const PT2_REPEAT: usize = 3087;
// const PT2_INDEX: usize = 3;

/*

deal with increment 7
deal into new stack
deal into new stack
*/

/*
0:  0 1 2 3 4 5 6 7 8 9
1:  3 4 5 6 7 8 9 0 1 2 
2:  6 7 8 9 0 1 2 3 4 5 
3:  9 0 1 2 3 4 5 6 7 8 
4:  2 3 4 5 6 7 8 9 0 1 
5:  5 6 7 8 9 0 1 2 3 4 
6:  8 9 0 1 2 3 4 5 6 7 
7:  1 2 3 4 5 6 7 8 9 0 
8:  4 5 6 7 8 9 0 1 2 3 
9:  7 8 9 0 1 2 3 4 5 6 
10: 0 1 2 3 4 5 6 7 8 9 
*/

/*
0:  0 1 2 3 4 5 6 7 8 9
1:  2 1 0 9 8 7 6 5 4 3
2:  0 1 2 3 4 5 6 7 8 9
*/

pub fn part2(input: &str) -> Result<String> {
    // return Ok("0".to_string());
    let procedures = parse(input)?;
    log::debug!("Procedures: {:#?}", procedures);
    let shuffle = Shuffle::new(PT2_DECK, procedures.as_slice())?;
    log::debug!("Shuffle: {:?}", shuffle);
    // log::trace!("{:?}", iter_shuf(shuffle, PT2_DECK, PT2_REPEAT));
    let shuffle = shuffle.repeat(PT2_REPEAT as u64);
    log::debug!("Shuffle: {:?}", shuffle);
    // log::trace!("{:?}", shuffle.full().collect::<Vec<_>>());
    let c = shuffle.index(PT2_INDEX);
    Ok(format!("{}", c))
}

fn iter_shuf(shuffle: Shuffle, size: usize, rep: usize) -> Vec<usize> {
    let mut iter_shuf = Vec::with_capacity(size);
    for master_idx in 0..size {
        let mut idx = master_idx;
        for n in 0..rep {
            idx = shuffle.index(idx);
        }
        iter_shuf.push(idx);
    }
    iter_shuf
}

pub fn part2loop(input: &str) -> Result<String> {
    // return Ok("0".to_string());
    let procedures = parse(input)?;
    log::debug!("Procedures: {:#?}", procedures);
    let shuffle = Shuffle::new(PT2_DECK, procedures.as_slice())?;
    log::debug!("Shuffle: {:?}", shuffle);
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
        c+=1;
    };
    let loop_size = c - prev;
    let offset = (PT2_REPEAT - prev) % loop_size;
    log::info!("{} -> {} ({}) offset:{} start:{:?}", prev, c, loop_size, offset, idx);
    let fin = (0..offset)
        .fold(idx, |idx, _| {shuffle.index(idx)});
    Ok(format!("{}", fin))
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
