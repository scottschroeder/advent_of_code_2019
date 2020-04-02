use crate::intcode::{run_intcode, IntCode};
use crate::util::parse_intcode;
use anyhow::Result;
use itertools::Itertools;
use crate::challenges::day13::game::{Board, Tile, JoyStick, Screen};
use crate::display::ImageNormal;
use crate::intcode::intcode_io::VecIO;

pub fn part1(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let (_, out) = run_intcode(intcode, vec![])?;
    let mut board = Board::default();
    for chunk in out.as_slice().chunks_exact(3) {
        let tile = Tile::from(chunk[2]);
        board.add(chunk[0] as i32, chunk[1] as i32, tile);
    }
    let blocks = board
        .inner
        .values()
        .filter(|t| **t == Tile::Block)
        .count();
    Ok(format!("{}", blocks))
}

pub fn part2(input: &str) -> Result<String> {
    let mut intcode = parse_intcode(input)?;

    // Setup Board
    let (_, out) = run_intcode(intcode.clone(), vec![])?;
    let mut board = Board::default();
    for chunk in out.as_slice().chunks_exact(3) {
        let tile = Tile::from(chunk[2]);
        board.add(chunk[0] as i32, chunk[1] as i32, tile);
    }
    let mut img = ImageNormal::create(&board.inner);

    // insert 2 quarters
    intcode[0] = 2;

    let screen = Screen::new(board, img);
    let mut ic = IntCode::new(intcode, JoyStick, screen);
    ic.run_till_end()?;

    Ok(format!("{}", 0))
}

mod game {
    use std::collections::HashMap;
    use std::fmt;
    pub(crate) use joystick::JoyStick;
    use crate::display::ImageNormal;
    use crate::intcode::intcode_io::Output;
    use anyhow::Result;

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
        board: Board,
        image: ImageNormal<Tile>,
        score: i64,
        instruction: Vec<i64>,
    }

    impl Screen {
        pub fn new(board: Board, image: ImageNormal<Tile>) -> Screen {
            Screen { board, image, score: 0, instruction: Vec::new() }
        }
        fn disp(&mut self) {
            self.image.update(&self.board.inner);
            println!("Score: {}", self.score);
            println!("{}", self.image);
        }
        fn update(&mut self, instr: i64) -> bool {
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
                true
            } else {
                false
            }
        }
    }

    impl Output for Screen {
        fn output(&mut self, out: i64) -> Result<()> {
            if self.update(out) {
                self.disp();
            }
            Ok(())
        }
    }

    #[derive(Debug, Default)]
    pub struct Board {
        pub inner: HashMap<(i32, i32), Tile>
    }

    impl Board {
        pub fn add(&mut self, x: i32, y: i32, tile: Tile) {
            self.inner.insert((x, y), tile);
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

    mod joystick {
        use crate::intcode::intcode_io::Input;
        use anyhow::Result;
        use std::io;

        enum Movement {
            None,
            Left,
            Right,
        }

        impl From<Movement> for i64 {
            fn from(m: Movement) -> Self {
                match m {
                    Movement::None => 0,
                    Movement::Left => -1,
                    Movement::Right => 1,
                }
            }
        }

        pub(crate) struct JoyStick;

        impl Input for JoyStick {
            fn input(&mut self) -> Result<i64> {
                let mut ret = String::new();
                io::stdin().read_line(&mut ret)?;
                let step = match ret.as_str() {
                    "a\n" => Movement::Left,
                    "d\n" => Movement::Right,
                    _ => Movement::None,
                };
                Ok(step.into())
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
        assert_eq!(part2(DAY13_INPUT).unwrap().as_str(), "0")
    }
}

