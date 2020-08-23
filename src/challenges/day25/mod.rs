use crate::intcode::IntCode;
use crate::util::parse_intcode;
use anyhow::{Result};

pub fn part1(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let mut cpu = IntCode::new(
        intcode,
        self::interactive::BufferedStdin::default(),
        self::interactive::IntCodeStdout,
    );
    cpu.run_till_end()?;
    Ok(format!("{}", 0))
}

mod interactive {
    use crate::intcode::intcode_io::{Input, Output};
    use anyhow::Result;
    use std::io;

    pub struct IntCodeStdout;

    impl Output for IntCodeStdout {
        fn output(&mut self, out: crate::intcode::Int) -> anyhow::Result<()> {
            print!("{}", out as u8 as char);
            Ok(())
        }
    }

    #[derive(Default)]
    pub struct BufferedStdin {
        buf: Vec<i64>,
    }
    impl BufferedStdin {
        fn read_line(&mut self) -> Result<()> {
            let mut buf = String::new();
            io::stdin().read_line(&mut buf)?;
            self.buf = buf.as_bytes().iter().rev().map(|b| *b as i64).collect();
            Ok(())
        }
    }

    impl Input for BufferedStdin {
        fn input(&mut self) -> Result<crate::intcode::Int> {
            loop {
                if let Some(c) = self.buf.pop() {
                    return Ok(c);
                }
                self.read_line()?;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    // #[test]
    // fn verify_part1() {
    //     assert_eq!(part1(DAY25_INPUT).unwrap().as_str(), "262848")
    // }
}
