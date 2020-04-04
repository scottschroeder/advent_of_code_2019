use anyhow::Result;
use std::iter;

const SIGNAL_BASE: [i64; 4] = [0, 1, 0, -1];

pub fn part1(input: &str) -> Result<String> {
    demo(0);
    demo(1);
    demo(2);
    demo(3);
    Ok(format!("{}", 0))
}

pub fn part2(input: &str) -> Result<String> {
    Ok(format!("{}", 0))
}

fn demo(pos: usize) {
    println!("e{}: {:?}",
             pos,
             generate_pattern(pos).take(10).collect::<Vec<_>>()
    )
}

fn generate_pattern(pos: usize) -> impl Iterator<Item=i64> {
    SIGNAL_BASE
        .into_iter()
        .cloned()
        .flat_map(move |x| iter::repeat(x).take(pos + 1))
        .cycle()
        .skip(1)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn day16part1() {
        //assert_eq!(part1(DAY16_INPUT).unwrap().as_str(), "0")
    }

    #[test]
    fn day16part2() {
        //assert_eq!(part2(DAY16_INPUT).unwrap().as_str(), "0")
    }
}
