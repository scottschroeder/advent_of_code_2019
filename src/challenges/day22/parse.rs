use super::shuf::ShuffleMethod;
use anyhow::{anyhow as ah, Result};

pub(crate) fn parse(input: &str) -> Result<Vec<ShuffleMethod>> {
    input.lines().map(|s| {
        let last = s
            .split(" ")
            .last()
            .ok_or_else(|| ah!("command did not have argument: {:?}", s))?;
        if s.starts_with("cut") {
            let cut = str::parse::<i64>(last)?;
            Ok(ShuffleMethod::Cut(cut))
        } else if s.contains("deal with increment") {
            let inc = str::parse::<i64>(last)?;
            if inc < 1 {
                Err(ah!("increment argument <1: {:?}", s))
            } else {
                Ok(ShuffleMethod::Increment(inc as usize))
            }
        } else if s.contains("deal into new stack") {
            Ok(ShuffleMethod::Stack)
        } else {
            Err(ah!("unknown command: {:?}", s))
        }
    })
    .collect()
}
