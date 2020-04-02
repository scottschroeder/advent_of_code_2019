use crate::challenges::day13::game::{Board, Screen, Tile};
use crate::display::ImageNormal;
use crate::intcode::intcode_io::{Input, VecIO};
use crate::intcode::{run_intcode, IntCode};
use crate::util::parse_intcode;
use anyhow::Result;
use itertools::Itertools;
use std::io::Read;

pub fn part1(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let (_, out) = run_intcode(intcode, vec![])?;
    let mut board = Board::default();
    for chunk in out.as_slice().chunks_exact(3) {
        let tile = Tile::from(chunk[2]);
        board.add(chunk[0] as i32, chunk[1] as i32, tile);
    }
    let blocks = board.inner.values().filter(|t| **t == Tile::Block).count();
    Ok(format!("{}", blocks))
}

pub fn part2(input: &str) -> Result<String> {
    let mut intcode = parse_intcode(input)?;
    let (_, out) = run_intcode(intcode.clone(), vec![])?;
    let mut board = Board::default();
    for chunk in out.as_slice().chunks_exact(3) {
        let tile = Tile::from(chunk[2]);
        board.add(chunk[0] as i32, chunk[1] as i32, tile);
    }

    let mut img = ImageNormal::create(&board.inner);
    let screen = Screen::new(board, img);

    // insert 2 quarters
    intcode[0] = 2;
    let mut ic = IntCode::new_from_device(intcode, screen);
    ic.run_till_end()?;
    let (_, mut screen) = ic.emit();

    Ok(format!("{}", screen.score))
}

mod game {
    use crate::display::ImageNormal;
    use crate::intcode::intcode_io::{Input, Output};
    use anyhow::Result;
    use std::collections::HashMap;
    use std::fmt;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Tile {
        Empty,
        Wall,
        Block,
        Paddle,
        Ball,
    }

    impl fmt::Display for Tile {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Tile::Empty => write!(f, " "),
                Tile::Wall => write!(f, "|"),
                Tile::Block => write!(f, "X"),
                Tile::Paddle => write!(f, "_"),
                Tile::Ball => write!(f, "*"),
            }
        }
    }

    pub struct Screen {
        pub board: Board,
        image: ImageNormal<Tile>,
        pub score: i64,
        instruction: Vec<i64>,
    }

    impl Screen {
        pub(crate) fn new(board: Board, image: ImageNormal<Tile>) -> Screen {
            Screen {
                board,
                image,
                score: 0,
                instruction: Vec::new(),
            }
        }
        fn disp(&mut self) {
            self.image.update(&self.board.inner);
            println!("Score: {}", self.score);
            println!("{}", self.image);
        }
        fn update(&mut self, instr: i64) {
            self.instruction.push(instr);
            if self.instruction.len() == 3 {
                let x = self.instruction[0];
                let y = self.instruction[1];
                let d = self.instruction[2];
                if x == -1 && y == 0 {
                    self.score = d;
                } else {
                    self.board.add(x as i32, y as i32, Tile::from(d));
                }
                self.instruction.clear();
            }
        }
    }

    impl Input for Screen {
        fn input(&mut self) -> Result<i64> {
            let d = self.board.off_by().unwrap();
            Ok(if d > 0 {
                1
            } else if d < 0 {
                -1
            } else {
                0
            })
        }
    }

    impl Output for Screen {
        fn output(&mut self, out: i64) -> Result<()> {
            self.update(out);
            Ok(())
        }
    }

    #[derive(Debug, Default)]
    pub struct Board {
        pub inner: HashMap<(i32, i32), Tile>,
    }

    impl Board {
        pub fn add(&mut self, x: i32, y: i32, tile: Tile) {
            self.inner.insert((x, y), tile);
        }
        pub fn off_by(&self) -> Option<i64> {
            let mut paddle = None;
            let mut ball = None;
            for ((x, y), t) in self.inner.iter() {
                match t {
                    Tile::Paddle => paddle = Some(*x),
                    Tile::Ball => ball = Some(*x),
                    _ => {}
                }
                if let Some(d) = paddle.and_then(|p| ball.map(|b| b - p)) {
                    return Some(d as i64);
                }
            }
            None
        }
    }

    impl From<i64> for Tile {
        fn from(i: i64) -> Self {
            match i {
                0 => Tile::Empty,
                1 => Tile::Wall,
                2 => Tile::Block,
                3 => Tile::Paddle,
                4 => Tile::Ball,
                _ => unreachable!("unhandled tile type: {}", i),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn day13part1() {
        assert_eq!(part1(DAY13_INPUT).unwrap().as_str(), "239")
    }

    #[test]
    fn day13part2() {
        assert_eq!(part2(DAY13_INPUT).unwrap().as_str(), "12099")
    }
}
