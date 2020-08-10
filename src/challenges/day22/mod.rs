use anyhow::{anyhow as ah, Result};
use self::shuf::{ShuffleMethod, Deck, Shuffle};

pub(crate) mod parse;
pub(crate) mod shuf;

pub fn part1(input: &str) -> Result<String> {
    let procedures = parse::parse(input)?;
    log::debug!("Procedures: {:#?}", procedures);
    Ok(format!("{}", 0))
}

pub fn part2(input: &str) -> Result<String> {
    // let inc = ShuffleMethod::Increment(3);
    // let v: Vec<usize> = (0..10).map(|idx| {
    //     inc.index(idx, 10)
    // }).collect();
    shuf::test_inc(100);
    Ok(format!("{:?}", 0))
}

fn shuffle(deck_size: u32, instructions: Vec<ShuffleMethod>) -> Deck {
    let deck = Deck::new(deck_size);
    let mut shuffle = Shuffle::new(deck);
    shuffle.do_sequence(instructions);
    shuffle.finalize()
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
