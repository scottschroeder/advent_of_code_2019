use crate::util::{digits_to_int, parse_digits};
use anyhow::Result;
use std::fmt;
use std::fmt::{Error, Formatter};
use std::hint::unreachable_unchecked;
use std::iter;

const SIGNAL_BASE: [i8; 4] = [0, 1, 0, -1];
const DECODE_ROUNDS: usize = 100;
const DECODE_REPEAT: usize = 10000;
const DECODE_INDEX: usize = 7;
const DECODE_OUTPUT: usize = 8;

pub fn part1(input: &str) -> Result<String> {
    let input_signal = Signal::parse(input)?;
    let mut signal = process_signal(&input_signal, DECODE_ROUNDS);
    signal.truncate(8);
    Ok(format!("{}", signal))
}

pub fn part2(input: &str) -> Result<String> {
    let input_signal = Signal::parse(input)?;
    Ok(format!("{}", large_signal_decode(&input_signal)))
}

fn large_signal_decode(input: &Signal) -> Signal {
    let input_signal = input.repeat(DECODE_REPEAT);
    let idx = input_signal.0[..DECODE_INDEX]
        .iter()
        .map(|i| *i as u8)
        .collect::<Vec<u8>>();
    let idx = digits_to_int(idx.as_slice()) as usize;
    assert!(idx > ((input_signal.0.len() + 1) / 2));
    let interesting = &input_signal.0[idx..];
    let mut s = signal_back_half(interesting, DECODE_ROUNDS);
    s.truncate(DECODE_OUTPUT);
    s
}

fn signal_back_half(s: &[i8], phases: usize) -> Signal {
    let mut prev = s.to_vec();
    prev.reverse();
    for _ in 0..phases {
        prev = signal_reverse_fill(prev.as_slice());
    }
    prev.reverse();
    Signal(prev)
}

fn signal_reverse_fill(previous: &[i8]) -> Vec<i8> {
    let mut phase = Vec::with_capacity(previous.len() + 1);
    phase.push(previous[0]);
    for idx in 1..previous.len() {
        phase.push((phase[idx - 1] + previous[idx]) % 10);
    }
    phase
}

fn process_signal(signal: &Signal, rounds: usize) -> Signal {
    let mut signal = signal.clone();
    for r in 0..rounds {
        log::trace!("Round: {}", r);
        signal.0 = signal_round(signal.0.as_slice());
        // not actually fast, I expect the iterator version is
        // getting avx instructions
        //signal.0 = signal_round_fast(signal.0.as_slice());
    }
    signal
}

#[derive(Debug, Clone, PartialEq)]
struct Signal(Vec<i8>);

impl Signal {
    fn parse(input: &str) -> Result<Signal> {
        let mut signal = parse_digits(input)?
            .iter()
            .map(|x| *x as i8)
            .collect::<Vec<i8>>();
        Ok(Signal(signal))
    }
    fn truncate(&mut self, size: usize) {
        self.0.truncate(size)
    }
    fn repeat(&self, repeat: usize) -> Signal {
        let total = self.0.len() * repeat;
        Signal(self.0.iter().cycle().take(total).cloned().collect())
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for d in &self.0 {
            write!(f, "{}", d)?
        }
        Ok(())
    }
}

fn signal_round(input: &[i8]) -> Vec<i8> {
    input
        .iter()
        .enumerate()
        .map(|(idx, _)| {
            input
                .iter()
                .zip(generate_pattern(idx))
                .skip(idx) // all zeros anyway
                .map(|(x, p)| (*x * p) as i64)
                .sum::<i64>()
        })
        .map(|s| (s.abs() % 10) as i8)
        .collect()
}

fn signal_round_calc(input: &[i8]) -> Vec<i8> {
    input
        .iter()
        .enumerate()
        .map(|(idx, _)| {
            input
                .iter()
                .enumerate()
                .map(|(inner_idx, x)| (*x * pattern_at(idx, inner_idx)) as i64)
                .sum::<i64>()
        })
        .map(|s| (s.abs() % 10) as i8)
        .collect()
}

fn generate_pattern(pos: usize) -> impl Iterator<Item = i8> {
    SIGNAL_BASE
        .into_iter()
        .cloned()
        .flat_map(move |x| iter::repeat(x).take(pos + 1))
        .cycle()
        .skip(1)
}

#[inline]
fn pattern_at(pos: usize, idx: usize) -> i8 {
    let x0 = (((idx + 1) / (pos + 1)) & 0b11) as i8;
    ((2 - x0) * x0) % 2
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn day16part1() {
        assert_eq!(part1(DAY16_INPUT).unwrap().as_str(), "59281788")
    }

    #[test]
    fn day16part2() {
        assert_eq!(part2(DAY16_INPUT).unwrap().as_str(), "96062868")
    }

    #[test]
    fn pattern() {
        for pos in 0..1000 {
            for (idx, expected) in generate_pattern(pos).take(1000).enumerate() {
                assert_eq!(pattern_at(pos, idx), expected);
            }
        }
    }

    #[test]
    fn ex1() {
        let signal = Signal::parse("12345678").unwrap();
        let expected = vec!["48226158", "34040438", "03415518", "01029498"];
        for (rounds, e) in expected.iter().enumerate() {
            let actual = format!("{}", process_signal(&signal, rounds + 1));
            assert_eq!(actual.as_str(), *e);
        }
    }

    #[test]
    fn ex2() {
        let cases = vec![
            ("80871224585914546619083218645595", "24176176"),
            ("19617804207202209144916044189917", "73745418"),
            ("69317163492948606335995924319873", "52432133"),
        ];
        for (input, expected) in cases {
            let s = Signal::parse(input).unwrap();
            let mut output = process_signal(&s, DECODE_ROUNDS);
            output.truncate(8);
            let actual = format!("{}", output);
            assert_eq!(actual, expected)
        }
    }

    #[test]
    fn part2_ex() {
        let cases = vec![
            ("03036732577212944063491565474664", "84462026"),
            ("02935109699940807407585447034323", "78725270"),
            ("03081770884921959731165446850517", "53553731"),
        ];
        for (input, expected) in cases {
            let s = Signal::parse(input).unwrap();
            let output = large_signal_decode(&s);
            let actual = format!("{}", output);
            assert_eq!(actual, expected)
        }
    }
}
