use anyhow::{anyhow as ah, Context, Result};
use std::fmt::Display;
use std::io::Read;
use std::str::FromStr;
use std::{fs, path};
pub use digits::{digits, parse_digits};

pub fn parse_str<T>(s: &str) -> Result<T>
    where
        T: FromStr,
        <T as FromStr>::Err: Display,
{
    T::from_str(s).map_err(|e| ah!("{}", e))
}

pub fn read_to_string<P: AsRef<path::Path>>(path: P) -> Result<String> {
    slog_scope::trace!("Reading content of file: {}", path.as_ref().display());
    let mut f = fs::File::open(&path)
        .with_context(|| format!("Unable to open path: {}", path.as_ref().display()))?;

    let mut result = String::new();

    f.read_to_string(&mut result)?;
    Ok(result)
}

pub fn parse_int_lines(input: &str) -> Result<Vec<u64>> {
    input.lines().map(|l| parse_str::<u64>(l)).collect()
}

pub fn parse_intcode(input: &str) -> Result<Vec<i64>> {
    input
        .lines()
        .flat_map(|l| l.split(','))
        .filter_map(|ns| {
            let s: &str = ns.trim();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        })
        .map(|ns| parse_str::<i64>(ns))
        .collect()
}

mod digits {
    use anyhow::{anyhow as ah, Result};

    pub fn digits(mut x: u64) -> Vec<u8> {
        if x == 0 {
            return vec![0];
        }
        const BASE: u64 = 10;
        let mut result = vec![];
        while x != 0 {
            result.push((x % BASE) as u8);
            x /= BASE;
        }
        result.reverse();
        result
    }

    pub fn parse_digits(input: &str) -> Result<Vec<u8>> {
        input
            .trim()
            .chars()
            .map(|c| {
                c
                    .to_digit(10)
                    .ok_or_else(|| ah!("could not parse '{}' as digit", c))
                    .map(|d| d as u8)
            })
            .collect()
    }

    #[cfg(test)]
    mod tests {
        use super::*;


        #[test]
        fn digits_in() {
            assert_eq!(digits(1234), vec![1, 2, 3, 4]);
            assert_eq!(digits(0), vec![0]);
            assert_eq!(digits(99845), vec![9, 9, 8, 4, 5]);
        }

        #[test]
        fn parse_empty_digits() {
            assert_eq!(parse_digits("").unwrap(), vec![])
        }

        #[test]
        fn parse_one_digit() {
            assert_eq!(parse_digits("3").unwrap(), vec![3])
        }

        #[test]
        fn parse_leading_zero() {
            assert_eq!(parse_digits("03").unwrap(), vec![0, 3])
        }

        #[test]
        fn parse_all_digits() {
            assert_eq!(
                parse_digits("0123456789101112").unwrap(),
                vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 0, 1, 1, 1, 2]
            )
        }
    }
}
